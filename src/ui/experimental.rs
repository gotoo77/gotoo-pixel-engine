use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::RangeInclusive;

use crate::{ActionId, Framebuffer, Pixel, Rect, Size, TextRenderer};

use super::{
    UiComponentStyle, UiStyleOverride, UiStyleSheet, UiTheme, UiVisualState,
    kernel::{
        UiInteractionOutput, UiInteractionState, UiNavigationPolicy, UiResolvedTarget,
        run_interaction_pass,
    },
    layout::{layout_responsive_grid, layout_vertical_children},
    style::{UiResolvedStyle, resolve_style},
};
pub use super::{
    kernel::{UiInput, UiPointerInput},
    layout::UiGridSpec,
};

const ROOT_ID: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UiId(u64);

impl UiId {
    pub const ROOT: Self = Self(ROOT_ID);

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    Root,
    Column,
    Panel,
    Grid,
    Text,
    Button,
    ToggleBool,
    SliderF32,
}

impl WidgetKind {
    fn is_interactive(self) -> bool {
        matches!(self, Self::Button | Self::ToggleBool | Self::SliderF32)
    }

    fn accepts_activation(self) -> bool {
        matches!(self, Self::Button | Self::ToggleBool)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constraints {
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}

impl Constraints {
    pub const fn loose(max: Size) -> Self {
        Self {
            min_width: 0,
            max_width: max.width,
            min_height: 0,
            max_height: max.height,
        }
    }

    pub const fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    fn normalized(self) -> Self {
        Self {
            min_width: self.min_width.min(self.max_width),
            max_width: self.max_width,
            min_height: self.min_height.min(self.max_height),
            max_height: self.max_height,
        }
    }

    fn constrain(self, size: Size) -> Size {
        let normalized = self.normalized();
        Size {
            width: size.width.clamp(normalized.min_width, normalized.max_width),
            height: size
                .height
                .clamp(normalized.min_height, normalized.max_height),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiNavInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub confirm: bool,
    pub cancel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetRef<T> {
    id: UiId,
    generation: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> WidgetRef<T> {
    fn new(id: UiId, generation: u64) -> Self {
        Self {
            id,
            generation,
            _marker: PhantomData,
        }
    }

    pub const fn id(self) -> UiId {
        self.id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiDiagnostic {
    DuplicateExplicitKey {
        scope: UiId,
        key: String,
        occurrence: u32,
    },
    WidgetKindChanged {
        id: UiId,
        previous: WidgetKind,
        current: WidgetKind,
    },
}

#[derive(Debug, Clone, Copy)]
enum UiValue {
    Bool(bool),
    F32(f32),
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for bool {}
    impl Sealed for f32 {}
}

pub trait UiValueType: sealed::Sealed + Copy + 'static {
    fn read(output: &UiOutput, id: UiId) -> Option<Self>;
}

impl UiValueType for bool {
    fn read(output: &UiOutput, id: UiId) -> Option<Self> {
        match output.changed_values.get(&id).copied() {
            Some(UiValue::Bool(value)) => Some(value),
            _ => None,
        }
    }
}

impl UiValueType for f32 {
    fn read(output: &UiOutput, id: UiId) -> Option<Self> {
        match output.changed_values.get(&id).copied() {
            Some(UiValue::F32(value)) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiOutput {
    generation: u64,
    interaction: UiInteractionOutput,
    changed_values: HashMap<UiId, UiValue>,
    diagnostics: Vec<UiDiagnostic>,
    dump: String,
    metrics: UiMetrics,
}

impl UiOutput {
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn focused_id(&self) -> Option<UiId> {
        self.interaction.focused_id()
    }

    pub const fn hovered_id(&self) -> Option<UiId> {
        self.interaction.hovered_id()
    }

    pub fn activated<T>(&self, handle: WidgetRef<T>) -> bool {
        handle.generation == self.generation && self.interaction.activated(handle.id)
    }

    pub fn action_pressed(&self, action: ActionId) -> bool {
        self.interaction.action_pressed(action)
    }

    pub fn changed<T: UiValueType>(&self, handle: WidgetRef<T>) -> Option<T> {
        (handle.generation == self.generation)
            .then(|| T::read(self, handle.id))
            .flatten()
    }

    pub const fn cancel_requested(&self) -> bool {
        self.interaction.cancel_requested()
    }

    pub fn diagnostics(&self) -> &[UiDiagnostic] {
        &self.diagnostics
    }

    pub fn dump(&self) -> &str {
        &self.dump
    }

    pub const fn metrics(&self) -> UiMetrics {
        self.metrics
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiMetrics {
    pub node_count: usize,
    pub interactive_count: usize,
    pub persistent_state_entries: usize,
    pub value_change_count: usize,
    pub activation_count: usize,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct StateEntry {
    kind: WidgetKind,
    last_seen_generation: u64,
}

#[derive(Debug, Default)]
pub struct UiStateStore {
    generation: u64,
    interaction: UiInteractionState,
    entries: HashMap<UiId, StateEntry>,
}

impl UiStateStore {
    pub const fn focused_id(&self) -> Option<UiId> {
        self.interaction.focused_id()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

#[derive(Debug)]
enum NodeContent<'a> {
    Root,
    Column,
    Panel,
    Grid {
        spec: UiGridSpec,
    },
    Text {
        text: Cow<'a, str>,
    },
    Button {
        label: Cow<'a, str>,
        action: Option<ActionId>,
    },
    ToggleBool {
        label: Cow<'a, str>,
        input: bool,
        effective: bool,
    },
    SliderF32 {
        label: Cow<'a, str>,
        input: f32,
        effective: f32,
        min: f32,
        max: f32,
        step: f32,
    },
}

#[derive(Debug)]
struct Node<'a> {
    id: UiId,
    kind: WidgetKind,
    parent: Option<usize>,
    children: Vec<usize>,
    content: NodeContent<'a>,
    style: UiStyleOverride,
    measured: Size,
    rect: Rect,
}

impl<'a> Node<'a> {
    fn new(
        id: UiId,
        kind: WidgetKind,
        parent: Option<usize>,
        content: NodeContent<'a>,
        style: UiStyleOverride,
    ) -> Self {
        Self {
            id,
            kind,
            parent,
            children: Vec::new(),
            content,
            style,
            measured: Size {
                width: 0,
                height: 0,
            },
            rect: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct InteractionTarget {
    id: UiId,
    rect: Rect,
    activatable: bool,
    action: Option<ActionId>,
}

impl UiResolvedTarget for InteractionTarget {
    fn ui_id(&self) -> UiId {
        self.id
    }

    fn ui_rect(&self) -> Rect {
        self.rect
    }

    fn ui_accepts_activation(&self) -> bool {
        self.activatable
    }
}

#[derive(Debug)]
struct ScopeFrame {
    id: UiId,
    next_slot: u32,
    explicit_key_counts: HashMap<String, u32>,
}

impl ScopeFrame {
    fn new(id: UiId) -> Self {
        Self {
            id,
            next_slot: 0,
            explicit_key_counts: HashMap::new(),
        }
    }
}

pub struct UiBuilder<'a> {
    generation: u64,
    nodes: Vec<Node<'a>>,
    parent_stack: Vec<usize>,
    scope_stack: Vec<ScopeFrame>,
    diagnostics: Vec<UiDiagnostic>,
}

impl<'a> UiBuilder<'a> {
    fn new(generation: u64) -> Self {
        let root = Node::new(
            UiId::ROOT,
            WidgetKind::Root,
            None,
            NodeContent::Root,
            UiStyleOverride::default(),
        );
        Self {
            generation,
            nodes: vec![root],
            parent_stack: vec![0],
            scope_stack: vec![ScopeFrame::new(UiId::ROOT)],
            diagnostics: Vec::new(),
        }
    }

    pub fn keyed<R>(&mut self, key: &str, build: impl FnOnce(&mut Self) -> R) -> R {
        let scope = self.scope_stack.last_mut().expect("root scope");
        let occurrence = scope.explicit_key_counts.entry(key.to_owned()).or_insert(0);
        let current_occurrence = *occurrence;
        *occurrence = occurrence.saturating_add(1);

        if current_occurrence > 0 {
            self.diagnostics.push(UiDiagnostic::DuplicateExplicitKey {
                scope: scope.id,
                key: key.to_owned(),
                occurrence: current_occurrence.saturating_add(1),
            });
        }

        let keyed_id = derive_keyed_id(scope.id, key, current_occurrence);
        self.scope_stack.push(ScopeFrame::new(keyed_id));
        let result = build(self);
        self.scope_stack.pop();
        result
    }

    pub fn column<R>(&mut self, build: impl FnOnce(&mut Self) -> R) -> R {
        self.container(
            WidgetKind::Column,
            NodeContent::Column,
            UiStyleOverride::default(),
            build,
        )
    }

    pub fn panel<R>(&mut self, build: impl FnOnce(&mut Self) -> R) -> R {
        self.panel_styled(UiStyleOverride::default(), build)
    }

    pub fn panel_styled<R>(
        &mut self,
        style: UiStyleOverride,
        build: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.container(WidgetKind::Panel, NodeContent::Panel, style, build)
    }

    pub fn grid<R>(&mut self, spec: UiGridSpec, build: impl FnOnce(&mut Self) -> R) -> R {
        self.container(
            WidgetKind::Grid,
            NodeContent::Grid { spec },
            UiStyleOverride::default(),
            build,
        )
    }

    pub fn text(&mut self, text: impl Into<Cow<'a, str>>) -> UiId {
        self.text_styled(text, UiStyleOverride::default())
    }

    pub fn text_styled(&mut self, text: impl Into<Cow<'a, str>>, style: UiStyleOverride) -> UiId {
        let id = self.allocate_implicit_id();
        self.push_leaf(
            id,
            WidgetKind::Text,
            NodeContent::Text { text: text.into() },
            style,
        );
        id
    }

    pub fn button(&mut self, label: impl Into<Cow<'a, str>>) -> WidgetRef<()> {
        self.button_internal(label.into(), None, UiStyleOverride::default())
    }

    pub fn button_action(
        &mut self,
        label: impl Into<Cow<'a, str>>,
        action: ActionId,
    ) -> WidgetRef<()> {
        self.button_internal(label.into(), Some(action), UiStyleOverride::default())
    }

    pub fn button_styled(
        &mut self,
        label: impl Into<Cow<'a, str>>,
        style: UiStyleOverride,
    ) -> WidgetRef<()> {
        self.button_internal(label.into(), None, style)
    }

    pub fn button_action_styled(
        &mut self,
        label: impl Into<Cow<'a, str>>,
        action: ActionId,
        style: UiStyleOverride,
    ) -> WidgetRef<()> {
        self.button_internal(label.into(), Some(action), style)
    }

    pub fn toggle(&mut self, label: impl Into<Cow<'a, str>>, value: bool) -> WidgetRef<bool> {
        self.toggle_styled(label, value, UiStyleOverride::default())
    }

    pub fn toggle_styled(
        &mut self,
        label: impl Into<Cow<'a, str>>,
        value: bool,
        style: UiStyleOverride,
    ) -> WidgetRef<bool> {
        let id = self.allocate_implicit_id();
        self.push_leaf(
            id,
            WidgetKind::ToggleBool,
            NodeContent::ToggleBool {
                label: label.into(),
                input: value,
                effective: value,
            },
            style,
        );
        WidgetRef::new(id, self.generation)
    }

    pub fn slider_f32(
        &mut self,
        label: impl Into<Cow<'a, str>>,
        value: f32,
        range: RangeInclusive<f32>,
        step: f32,
    ) -> WidgetRef<f32> {
        self.slider_f32_styled(label, value, range, step, UiStyleOverride::default())
    }

    pub fn slider_f32_styled(
        &mut self,
        label: impl Into<Cow<'a, str>>,
        value: f32,
        range: RangeInclusive<f32>,
        step: f32,
        style: UiStyleOverride,
    ) -> WidgetRef<f32> {
        let (min, max) = ordered_range(&range);
        let effective = snap_to_step(value, min, max, step);
        let id = self.allocate_implicit_id();
        self.push_leaf(
            id,
            WidgetKind::SliderF32,
            NodeContent::SliderF32 {
                label: label.into(),
                input: value,
                effective,
                min,
                max,
                step: step.abs(),
            },
            style,
        );
        WidgetRef::new(id, self.generation)
    }

    fn button_internal(
        &mut self,
        label: Cow<'a, str>,
        action: Option<ActionId>,
        style: UiStyleOverride,
    ) -> WidgetRef<()> {
        let id = self.allocate_implicit_id();
        self.push_leaf(
            id,
            WidgetKind::Button,
            NodeContent::Button { label, action },
            style,
        );
        WidgetRef::new(id, self.generation)
    }

    fn container<R>(
        &mut self,
        kind: WidgetKind,
        content: NodeContent<'a>,
        style: UiStyleOverride,
        build: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let id = self.allocate_implicit_id();
        let parent = *self.parent_stack.last().expect("root parent");
        let index = self.nodes.len();
        self.nodes
            .push(Node::new(id, kind, Some(parent), content, style));
        self.nodes[parent].children.push(index);

        self.parent_stack.push(index);
        self.scope_stack.push(ScopeFrame::new(id));
        let result = build(self);
        self.scope_stack.pop();
        self.parent_stack.pop();
        result
    }

    fn push_leaf(
        &mut self,
        id: UiId,
        kind: WidgetKind,
        content: NodeContent<'a>,
        style: UiStyleOverride,
    ) {
        let parent = *self.parent_stack.last().expect("root parent");
        let index = self.nodes.len();
        self.nodes
            .push(Node::new(id, kind, Some(parent), content, style));
        self.nodes[parent].children.push(index);
    }

    fn allocate_implicit_id(&mut self) -> UiId {
        let scope = self.scope_stack.last_mut().expect("root scope");
        let slot = scope.next_slot;
        scope.next_slot = scope.next_slot.saturating_add(1);
        derive_implicit_id(scope.id, slot)
    }
}

pub fn run<'a, R>(
    framebuffer: &mut Framebuffer,
    state: &mut UiStateStore,
    nav: UiNavInput,
    theme: UiTheme,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_with_input_styled(
        framebuffer,
        state,
        UiInput {
            nav,
            ..UiInput::default()
        },
        theme,
        UiStyleSheet::default(),
        build,
    )
}

pub fn run_with_input<'a, R>(
    framebuffer: &mut Framebuffer,
    state: &mut UiStateStore,
    input: UiInput<'_>,
    theme: UiTheme,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_with_input_styled(
        framebuffer,
        state,
        input,
        theme,
        UiStyleSheet::default(),
        build,
    )
}

pub fn run_styled<'a, R>(
    framebuffer: &mut Framebuffer,
    state: &mut UiStateStore,
    nav: UiNavInput,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_with_input_styled(
        framebuffer,
        state,
        UiInput {
            nav,
            ..UiInput::default()
        },
        theme,
        stylesheet,
        build,
    )
}

pub fn run_with_input_styled<'a, R>(
    framebuffer: &mut Framebuffer,
    state: &mut UiStateStore,
    input: UiInput<'_>,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    let surface = Size {
        width: framebuffer.width(),
        height: framebuffer.height(),
    };
    run_impl(
        surface,
        Some(framebuffer),
        state,
        input,
        theme,
        stylesheet,
        build,
    )
}

pub fn run_headless<'a, R>(
    surface: Size,
    state: &mut UiStateStore,
    nav: UiNavInput,
    theme: UiTheme,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_headless_with_input_styled(
        surface,
        state,
        UiInput {
            nav,
            ..UiInput::default()
        },
        theme,
        UiStyleSheet::default(),
        build,
    )
}

pub fn run_headless_with_input<'a, R>(
    surface: Size,
    state: &mut UiStateStore,
    input: UiInput<'_>,
    theme: UiTheme,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_headless_with_input_styled(surface, state, input, theme, UiStyleSheet::default(), build)
}

pub fn run_headless_styled<'a, R>(
    surface: Size,
    state: &mut UiStateStore,
    nav: UiNavInput,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_headless_with_input_styled(
        surface,
        state,
        UiInput {
            nav,
            ..UiInput::default()
        },
        theme,
        stylesheet,
        build,
    )
}

pub fn run_headless_with_input_styled<'a, R>(
    surface: Size,
    state: &mut UiStateStore,
    input: UiInput<'_>,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    run_impl(surface, None, state, input, theme, stylesheet, build)
}

fn run_impl<'a, R>(
    surface: Size,
    framebuffer: Option<&mut Framebuffer>,
    state: &mut UiStateStore,
    input: UiInput<'_>,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    build: impl FnOnce(&mut UiBuilder<'a>) -> R,
) -> (UiOutput, R) {
    state.generation = state.generation.saturating_add(1).max(1);
    let generation = state.generation;

    let mut builder = UiBuilder::new(generation);
    let result = build(&mut builder);

    let mut diagnostics = builder.diagnostics;
    let mut nodes = builder.nodes;
    let text_renderer = TextRenderer::new(theme.font);

    let root_constraints = Constraints::tight(surface);
    measure_node(
        &mut nodes,
        0,
        root_constraints,
        theme,
        stylesheet,
        text_renderer,
    );
    arrange_node(
        &mut nodes,
        0,
        Rect {
            x: 0,
            y: 0,
            width: surface.width,
            height: surface.height,
        },
        theme,
        stylesheet,
    );

    let mut targets = Vec::new();
    let mut incompatible_ids = HashSet::new();
    for node in &nodes {
        if !node.kind.is_interactive() {
            continue;
        }

        let action = match &node.content {
            NodeContent::Button { action, .. } => *action,
            _ => None,
        };
        targets.push(InteractionTarget {
            id: node.id,
            rect: node.rect,
            activatable: node.kind.accepts_activation(),
            action,
        });

        match state.entries.get(&node.id).copied() {
            Some(entry) if entry.kind != node.kind => {
                diagnostics.push(UiDiagnostic::WidgetKindChanged {
                    id: node.id,
                    previous: entry.kind,
                    current: node.kind,
                });
                incompatible_ids.insert(node.id);
                state.entries.insert(
                    node.id,
                    StateEntry {
                        kind: node.kind,
                        last_seen_generation: generation,
                    },
                );
            }
            Some(mut entry) => {
                entry.last_seen_generation = generation;
                state.entries.insert(node.id, entry);
            }
            None => {
                state.entries.insert(
                    node.id,
                    StateEntry {
                        kind: node.kind,
                        last_seen_generation: generation,
                    },
                );
            }
        }
    }

    let current_focus_order = targets.iter().map(|target| target.id).collect::<Vec<_>>();
    state
        .interaction
        .repair_for_current_order_excluding(&current_focus_order, &incompatible_ids);

    let previous_pointer_capture = state.interaction.pointer_capture_id();
    let interaction = run_interaction_pass(
        &mut state.interaction,
        input,
        &targets,
        UiNavigationPolicy::Linear,
        |id| {
            targets
                .iter()
                .find(|target| target.id == id)
                .and_then(|target| target.action)
        },
    );

    let focused = interaction.focused_id();
    let mut changed_values = HashMap::new();

    for activated in interaction.activated_ids().iter().copied() {
        if let Some(index) = nodes.iter().position(|node| node.id == activated)
            && let NodeContent::ToggleBool {
                input, effective, ..
            } = &mut nodes[index].content
        {
            *effective = !*input;
            changed_values.insert(activated, UiValue::Bool(*effective));
        }
    }

    if let Some(focused) = focused
        && let Some(index) = nodes.iter().position(|node| node.id == focused)
    {
        let slider_delta = i64::from(input.nav.right) - i64::from(input.nav.left);
        if slider_delta != 0
            && let NodeContent::SliderF32 {
                input,
                effective,
                min,
                max,
                step,
                ..
            } = &mut nodes[index].content
        {
            let before = *effective;
            *effective = stepped_slider_value(*effective, *min, *max, *step, slider_delta);
            if *effective != before || *effective != *input {
                changed_values.insert(focused, UiValue::F32(*effective));
            }
        }
    }

    let pointer_capture = state.interaction.pointer_capture_id().or_else(|| {
        (input.pointer.released && !input.pointer.pressed)
            .then_some(previous_pointer_capture)
            .flatten()
    });
    if let (Some(captured), Some((x, _))) = (pointer_capture, input.pointer.position)
        && let Some(node) = nodes.iter_mut().find(|node| node.id == captured)
    {
        let track = slider_track_rect(node.rect);
        if let NodeContent::SliderF32 {
            input,
            effective,
            min,
            max,
            step,
            ..
        } = &mut node.content
        {
            let ratio = ((i64::from(x) - i64::from(track.x)) as f32
                / track.width.saturating_sub(1).max(1) as f32)
                .clamp(0.0, 1.0);
            *effective = snap_to_step(*min + ratio * (*max - *min), *min, *max, *step);
            if *effective != *input {
                changed_values.insert(captured, UiValue::F32(*effective));
            } else {
                changed_values.remove(&captured);
            }
        }
    }

    if let Some(framebuffer) = framebuffer {
        let mut context = PaintContext {
            framebuffer,
            theme,
            stylesheet,
            text_renderer,
            interaction: &interaction,
            interaction_state: &state.interaction,
        };
        paint_node(&nodes, 0, &mut context);
    }

    let dump = dump_nodes(
        &nodes,
        focused,
        &changed_values,
        interaction.activated_ids(),
    );

    state
        .entries
        .retain(|_, entry| entry.last_seen_generation == generation);

    let metrics = UiMetrics {
        node_count: nodes.len(),
        interactive_count: targets.len(),
        persistent_state_entries: state.entries.len(),
        value_change_count: changed_values.len(),
        activation_count: interaction.activation_count(),
        diagnostic_count: diagnostics.len(),
    };

    (
        UiOutput {
            generation,
            interaction,
            changed_values,
            diagnostics,
            dump,
            metrics,
        },
        result,
    )
}

fn measure_node(
    nodes: &mut [Node<'_>],
    index: usize,
    constraints: Constraints,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    text_renderer: TextRenderer,
) -> Size {
    let kind = nodes[index].kind;
    let size = match kind {
        WidgetKind::Root => {
            let children = std::mem::take(&mut nodes[index].children);
            let mut content_height = 0_u32;
            let mut content_width = 0_u32;
            let inner_max_width = constraints
                .max_width
                .saturating_sub(theme.padding.saturating_mul(2));
            let inner_max_height = constraints
                .max_height
                .saturating_sub(theme.padding.saturating_mul(2));
            for (position, child) in children.iter().copied().enumerate() {
                let child_size = measure_node(
                    nodes,
                    child,
                    Constraints::loose(Size {
                        width: inner_max_width,
                        height: inner_max_height,
                    }),
                    theme,
                    stylesheet,
                    text_renderer,
                );
                content_width = content_width.max(child_size.width);
                content_height = content_height.saturating_add(child_size.height);
                if position + 1 < children.len() {
                    content_height = content_height.saturating_add(theme.row_spacing);
                }
            }
            nodes[index].children = children;
            constraints.constrain(Size {
                width: content_width.saturating_add(theme.padding.saturating_mul(2)),
                height: content_height.saturating_add(theme.padding.saturating_mul(2)),
            })
        }
        WidgetKind::Column => {
            measure_linear_container(nodes, index, constraints, theme, stylesheet, text_renderer)
        }
        WidgetKind::Panel => {
            measure_linear_container(nodes, index, constraints, theme, stylesheet, text_renderer)
        }
        WidgetKind::Grid => {
            let spec = match &nodes[index].content {
                NodeContent::Grid { spec } => *spec,
                _ => unreachable!(),
            };
            measure_grid_container(
                nodes,
                index,
                constraints,
                spec,
                theme,
                stylesheet,
                text_renderer,
            )
        }
        WidgetKind::Text => {
            let NodeContent::Text { text } = &nodes[index].content else {
                unreachable!();
            };
            let (width, height) = text_renderer.text_size(text.as_ref(), theme.text_scale.max(1));
            constraints.constrain(Size { width, height })
        }
        WidgetKind::Button | WidgetKind::ToggleBool | WidgetKind::SliderF32 => constraints
            .constrain(Size {
                width: constraints.max_width,
                height: theme.row_height,
            }),
    };
    nodes[index].measured = size;
    size
}

fn measure_linear_container(
    nodes: &mut [Node<'_>],
    index: usize,
    constraints: Constraints,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    text_renderer: TextRenderer,
) -> Size {
    let style = resolved_static_style(&nodes[index], theme, stylesheet);
    let (padding, gap) = match nodes[index].kind {
        WidgetKind::Column => (0, theme.row_spacing),
        _ => (style.padding, style.vertical_gap),
    };
    let children = std::mem::take(&mut nodes[index].children);
    let inner_max_width = constraints
        .max_width
        .saturating_sub(padding.saturating_mul(2));
    let inner_max_height = constraints
        .max_height
        .saturating_sub(padding.saturating_mul(2));
    let mut width = 0_u32;
    let mut height = 0_u32;

    for (position, child) in children.iter().copied().enumerate() {
        let child_size = measure_node(
            nodes,
            child,
            Constraints::loose(Size {
                width: inner_max_width,
                height: inner_max_height,
            }),
            theme,
            stylesheet,
            text_renderer,
        );
        width = width.max(child_size.width);
        height = height.saturating_add(child_size.height);
        if position + 1 < children.len() {
            height = height.saturating_add(gap);
        }
    }

    nodes[index].children = children;
    constraints.constrain(Size {
        width: width.saturating_add(padding.saturating_mul(2)),
        height: height.saturating_add(padding.saturating_mul(2)),
    })
}

fn measure_grid_container(
    nodes: &mut [Node<'_>],
    index: usize,
    constraints: Constraints,
    spec: UiGridSpec,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    text_renderer: TextRenderer,
) -> Size {
    let constraints = constraints.normalized();
    let children = std::mem::take(&mut nodes[index].children);
    let layout = layout_responsive_grid(
        Rect {
            x: 0,
            y: 0,
            width: constraints.max_width,
            height: constraints.max_height,
        },
        spec,
        children.len(),
    );

    for (position, child) in children.iter().copied().enumerate() {
        let cell = layout.rects.get(position).copied().unwrap_or(Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });
        measure_node(
            nodes,
            child,
            Constraints::loose(Size {
                width: cell.width,
                height: cell.height,
            }),
            theme,
            stylesheet,
            text_renderer,
        );
    }

    let height = layout
        .rects
        .iter()
        .map(|rect| {
            u32::try_from(rect.y.max(0))
                .unwrap_or(u32::MAX)
                .saturating_add(rect.height)
        })
        .max()
        .unwrap_or_else(|| spec.padding.saturating_mul(2))
        .saturating_add(if layout.rects.is_empty() {
            0
        } else {
            spec.padding
        });

    nodes[index].children = children;
    constraints.constrain(Size {
        width: constraints.max_width,
        height,
    })
}

fn arrange_node(
    nodes: &mut [Node<'_>],
    index: usize,
    rect: Rect,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
) {
    nodes[index].rect = rect;
    match nodes[index].kind {
        WidgetKind::Root => arrange_linear_children(
            nodes,
            index,
            rect,
            theme.padding,
            theme.row_spacing,
            theme,
            stylesheet,
        ),
        WidgetKind::Column => {
            arrange_linear_children(nodes, index, rect, 0, theme.row_spacing, theme, stylesheet)
        }
        WidgetKind::Panel => {
            let style = resolved_static_style(&nodes[index], theme, stylesheet);
            arrange_linear_children(
                nodes,
                index,
                rect,
                style.padding,
                style.vertical_gap,
                theme,
                stylesheet,
            )
        }
        WidgetKind::Grid => {
            let spec = match &nodes[index].content {
                NodeContent::Grid { spec } => *spec,
                _ => unreachable!(),
            };
            arrange_grid_children(nodes, index, rect, spec, theme, stylesheet);
        }
        _ => {}
    }
}

fn arrange_linear_children(
    nodes: &mut [Node<'_>],
    index: usize,
    rect: Rect,
    padding: u32,
    gap: u32,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
) {
    let children = std::mem::take(&mut nodes[index].children);
    let child_heights = children
        .iter()
        .map(|&child| nodes[child].measured.height)
        .collect::<Vec<_>>();
    let child_rects = layout_vertical_children(rect, padding, gap, &child_heights);

    for (child, child_rect) in children.iter().copied().zip(child_rects) {
        arrange_node(nodes, child, child_rect, theme, stylesheet);
    }

    nodes[index].children = children;
}

fn arrange_grid_children(
    nodes: &mut [Node<'_>],
    index: usize,
    rect: Rect,
    spec: UiGridSpec,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
) {
    let children = std::mem::take(&mut nodes[index].children);
    let layout = layout_responsive_grid(rect, spec, children.len());

    for (child, child_rect) in children.iter().copied().zip(layout.rects) {
        arrange_node(nodes, child, child_rect, theme, stylesheet);
    }

    nodes[index].children = children;
}

struct PaintContext<'a> {
    framebuffer: &'a mut Framebuffer,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    text_renderer: TextRenderer,
    interaction: &'a UiInteractionOutput,
    interaction_state: &'a UiInteractionState,
}

fn paint_node(nodes: &[Node<'_>], index: usize, context: &mut PaintContext<'_>) {
    let node = &nodes[index];
    let style = resolved_visual_style(
        node,
        context.theme,
        context.stylesheet,
        context.interaction,
        context.interaction_state,
    );

    match &node.content {
        NodeContent::Root | NodeContent::Column | NodeContent::Grid { .. } => {}
        NodeContent::Panel => {
            context.framebuffer.fill_rect(
                node.rect.x,
                node.rect.y,
                node.rect.width,
                node.rect.height,
                style.background,
            );
            draw_border(
                context.framebuffer,
                node.rect,
                style.border,
                style.border_width,
            );
        }
        NodeContent::Text { text } => {
            draw_text_left(
                context.framebuffer,
                context.text_renderer,
                node.rect,
                text.as_ref(),
                context.theme.text_scale,
                style.text,
            );
        }
        NodeContent::Button { label, .. } => {
            draw_control_frame(context.framebuffer, node.rect, style);
            draw_text_centered(
                context.framebuffer,
                context.text_renderer,
                node.rect,
                label.as_ref(),
                context.theme.text_scale,
                style.text,
            );
        }
        NodeContent::ToggleBool {
            label, effective, ..
        } => {
            draw_control_frame(context.framebuffer, node.rect, style);
            let suffix = if *effective { "ON" } else { "OFF" };
            let text = format!("{}: {}", label, suffix);
            let mut label_rect = node.rect;
            let inset = style.border_width.max(1).saturating_add(4);
            let (label_width, _) = context
                .text_renderer
                .text_size(&format!("{}: OFF", label), context.theme.text_scale.max(1));
            // Reserve the same space in both states; compact rows keep the text-only form.
            if node.rect.width
                >= label_width
                    .saturating_add(40)
                    .saturating_add(inset.saturating_mul(2))
                && node.rect.height >= 14_u32.saturating_add(inset.saturating_mul(2))
            {
                label_rect.width = label_rect.width.saturating_sub(40 + inset);
                let x = node
                    .rect
                    .x
                    .saturating_add(node.rect.width.saturating_sub(32 + inset) as i32);
                let y = centered_coordinate(node.rect.y, node.rect.height, 14);
                context.framebuffer.fill_rect(
                    x,
                    y,
                    32,
                    14,
                    if *effective {
                        style.accent
                    } else {
                        style.muted_text
                    },
                );
                context.framebuffer.fill_rect(
                    x.saturating_add(if *effective { 20 } else { 2 }),
                    y.saturating_add(2),
                    10,
                    10,
                    style.background,
                );
            }
            draw_text_centered(
                context.framebuffer,
                context.text_renderer,
                label_rect,
                &text,
                context.theme.text_scale,
                style.text,
            );
        }
        NodeContent::SliderF32 {
            label,
            effective,
            min,
            max,
            step,
            ..
        } => {
            draw_control_frame(context.framebuffer, node.rect, style);
            let label_rect = Rect {
                x: node.rect.x.saturating_add(4),
                y: node.rect.y,
                width: node.rect.width / 2,
                height: node.rect.height,
            };
            draw_text_left(
                context.framebuffer,
                context.text_renderer,
                label_rect,
                &format!(
                    "{}: {}",
                    label,
                    slider_value_text(*effective, *min, *max, *step)
                ),
                context.theme.text_scale,
                style.text,
            );
            let track = slider_track_rect(node.rect);
            let static_style = resolved_static_style(node, context.theme, context.stylesheet);
            context.framebuffer.fill_rect(
                track.x,
                track.y,
                track.width,
                track.height,
                static_style.border,
            );
            let ratio = if *max > *min {
                ((*effective - *min) / (*max - *min)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let fill = ((track.width as f32) * ratio).round() as u32;
            if fill > 0 {
                context.framebuffer.fill_rect(
                    track.x,
                    track.y,
                    fill.min(track.width),
                    track.height,
                    style.accent,
                );
            }
            let thumb = slider_thumb_rect(track, ratio);
            context.framebuffer.fill_rect(
                thumb.x,
                thumb.y,
                thumb.width,
                thumb.height,
                style.accent,
            );
        }
    }

    for child in &node.children {
        paint_node(nodes, *child, context);
    }
}

fn draw_control_frame(framebuffer: &mut Framebuffer, rect: Rect, style: UiResolvedStyle) {
    framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, style.background);
    draw_border(framebuffer, rect, style.border, style.border_width);
}

fn draw_border(framebuffer: &mut Framebuffer, rect: Rect, color: Pixel, width: u32) {
    for inset in 0..width.max(1) {
        let rect = super::layout::inset_rect(rect, inset);
        if rect.width == 0 || rect.height == 0 {
            break;
        }
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
    }
}

fn resolved_static_style(
    node: &Node<'_>,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
) -> UiResolvedStyle {
    resolve_style(
        theme,
        component_style(stylesheet, node.kind),
        node.style,
        UiVisualState::default(),
    )
}

fn resolved_visual_style(
    node: &Node<'_>,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    interaction: &UiInteractionOutput,
    interaction_state: &UiInteractionState,
) -> UiResolvedStyle {
    let component = component_style(stylesheet, node.kind);
    let visual = UiVisualState {
        focused: interaction.focused_id() == Some(node.id),
        hovered: interaction.hovered_id() == Some(node.id),
        active: interaction_state.is_active(node.id),
    };
    let mut resolved = resolve_style(theme, component, node.style, visual);

    let later_border = (visual.hovered && component.hovered.border.is_some())
        || (visual.active && component.active.border.is_some());
    if visual.focused && component.focused.border.is_none() && !later_border {
        resolved.border = resolved.accent;
    }

    resolved
}

fn component_style(stylesheet: UiStyleSheet, kind: WidgetKind) -> UiComponentStyle {
    match kind {
        WidgetKind::Panel => stylesheet.panel,
        WidgetKind::Text => stylesheet.text,
        WidgetKind::Button => stylesheet.button,
        WidgetKind::ToggleBool => stylesheet.toggle_bool,
        WidgetKind::SliderF32 => stylesheet.slider_f32,
        WidgetKind::Root | WidgetKind::Column | WidgetKind::Grid => UiComponentStyle::default(),
    }
}

fn draw_text_left(
    framebuffer: &mut Framebuffer,
    text_renderer: TextRenderer,
    rect: Rect,
    text: &str,
    scale: u32,
    color: Pixel,
) {
    let scale = scale.max(1);
    let (_, height) = text_renderer.text_size(text, scale);
    text_renderer.draw_scaled(
        framebuffer,
        rect.x,
        centered_coordinate(rect.y, rect.height, height),
        text,
        scale,
        color,
    );
}

fn draw_text_centered(
    framebuffer: &mut Framebuffer,
    text_renderer: TextRenderer,
    rect: Rect,
    text: &str,
    scale: u32,
    color: Pixel,
) {
    let scale = scale.max(1);
    let (width, height) = text_renderer.text_size(text, scale);
    text_renderer.draw_scaled(
        framebuffer,
        centered_coordinate(rect.x, rect.width, width),
        centered_coordinate(rect.y, rect.height, height),
        text,
        scale,
        color,
    );
}

fn dump_nodes(
    nodes: &[Node<'_>],
    focused: Option<UiId>,
    changed: &HashMap<UiId, UiValue>,
    activated: &HashSet<UiId>,
) -> String {
    let mut dump = String::new();
    dump_node(nodes, 0, 0, focused, changed, activated, &mut dump);
    dump
}

fn dump_node(
    nodes: &[Node<'_>],
    index: usize,
    depth: usize,
    focused: Option<UiId>,
    changed: &HashMap<UiId, UiValue>,
    activated: &HashSet<UiId>,
    dump: &mut String,
) {
    let node = &nodes[index];
    let indent = "  ".repeat(depth);
    let focus = if focused == Some(node.id) {
        " focused"
    } else {
        ""
    };
    let activation = if activated.contains(&node.id) {
        " activated"
    } else {
        ""
    };
    let change = match changed.get(&node.id) {
        Some(UiValue::Bool(value)) => format!(" changed={value}"),
        Some(UiValue::F32(value)) => format!(" changed={value:.3}"),
        None => String::new(),
    };

    let parent_detail = node
        .parent
        .map(|parent| format!(" parent={:016x}", nodes[parent].id.raw()))
        .unwrap_or_default();

    let detail = match &node.content {
        NodeContent::Root => String::new(),
        NodeContent::Column => String::new(),
        NodeContent::Panel => String::new(),
        NodeContent::Grid { spec } => format!(
            " min_cell_width={} preferred_cell_height={} gap={} padding={}",
            spec.min_cell_width, spec.preferred_cell_height, spec.gap, spec.padding
        ),
        NodeContent::Text { text } => format!(" text={:?}", text.as_ref()),
        NodeContent::Button { label, action } => {
            format!(" label={:?} action={:?}", label.as_ref(), action)
        }
        NodeContent::ToggleBool {
            label,
            input,
            effective,
        } => format!(
            " label={:?} input={} effective={}",
            label.as_ref(),
            input,
            effective
        ),
        NodeContent::SliderF32 {
            label,
            input,
            effective,
            min,
            max,
            step,
        } => format!(
            " label={:?} input={:.3} effective={:.3} range={:.3}..={:.3} step={:.3}",
            label.as_ref(),
            input,
            effective,
            min,
            max,
            step
        ),
    };

    use std::fmt::Write as _;
    let _ = writeln!(
        dump,
        "{indent}{:?} id={:016x}{parent_detail} rect=[{},{} {}x{}]{focus}{activation}{change}{detail}",
        node.kind,
        node.id.raw(),
        node.rect.x,
        node.rect.y,
        node.rect.width,
        node.rect.height,
    );

    for child in &node.children {
        dump_node(nodes, *child, depth + 1, focused, changed, activated, dump);
    }
}

fn derive_implicit_id(parent: UiId, slot: u32) -> UiId {
    let mut hash = hash_u64(ROOT_ID, parent.raw());
    hash = hash_u64(hash, 0);
    hash = hash_u64(hash, u64::from(slot));
    UiId(hash)
}

fn derive_keyed_id(parent: UiId, key: &str, occurrence: u32) -> UiId {
    let mut hash = hash_u64(ROOT_ID, parent.raw());
    hash = hash_u64(hash, 1);
    hash = hash_bytes(hash, key.as_bytes());
    hash = hash_u64(hash, u64::from(occurrence));
    UiId(hash)
}

fn hash_u64(seed: u64, value: u64) -> u64 {
    hash_bytes(seed, &value.to_le_bytes())
}

fn hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn ordered_range(range: &RangeInclusive<f32>) -> (f32, f32) {
    let start = *range.start();
    let end = *range.end();
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn snap_to_step(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let value = value.clamp(min, max);
    let step = step.abs();
    if step == 0.0 || max <= min {
        return value;
    }
    (min + ((value - min) / step).round() * step).clamp(min, max)
}

fn stepped_slider_value(value: f32, min: f32, max: f32, step: f32, direction: i64) -> f32 {
    let step = step.abs();
    if step == 0.0 {
        return value.clamp(min, max);
    }
    let value = snap_to_step(value, min, max, step);
    snap_to_step(value + step * direction as f32, min, max, step)
}

fn slider_track_rect(rect: Rect) -> Rect {
    let x_offset = rect.width / 2;
    let width = rect.width.saturating_sub(x_offset).saturating_sub(8).max(1);
    let height = 5_u32.min(rect.height.max(1));
    Rect {
        x: rect.x.saturating_add(u32_to_i32(x_offset)),
        y: centered_coordinate(rect.y, rect.height, height),
        width,
        height,
    }
}

fn slider_thumb_rect(track: Rect, ratio: f32) -> Rect {
    let width = 3_u32.min(track.width.max(1));
    let height = track.height.saturating_add(4);
    let center_offset = ((track.width.saturating_sub(1) as f32) * ratio.clamp(0.0, 1.0)).round()
        as u32;
    let center_x = track.x.saturating_add(u32_to_i32(center_offset));
    let min_x = track.x;
    let max_x = track
        .x
        .saturating_add(u32_to_i32(track.width.saturating_sub(width)));
    let x = center_x
        .saturating_sub(u32_to_i32(width / 2))
        .clamp(min_x, max_x);
    Rect {
        x,
        y: centered_coordinate(track.y, track.height, height),
        width,
        height,
    }
}

fn slider_value_text(value: f32, min: f32, max: f32, step: f32) -> String {
    let precision = if step > 0.0 {
        (0..=9)
            .find(|digits| {
                let scaled = f64::from(step) * 10_f64.powi(*digits);
                let origin = f64::from(min) * 10_f64.powi(*digits);
                let end = f64::from(max) * 10_f64.powi(*digits);
                scaled >= 1.0 - 0.00001
                    && (scaled - scaled.round()).abs() < 0.00001
                    && (origin - origin.round()).abs() < 0.00001
                    && (end - end.round()).abs() < 0.00001
            })
            .unwrap_or(9) as usize
    } else {
        return value.to_string();
    };
    format!("{value:.precision$}")
}

fn centered_coordinate(origin: i32, extent: u32, content_extent: u32) -> i32 {
    let coordinate = i64::from(origin) + (i64::from(extent) - i64::from(content_extent)) / 2;
    coordinate.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use crate::{Touch, TouchPhase};

    use super::*;

    const RESUME: ActionId = ActionId::new("mfe.resume");

    fn size() -> Size {
        Size {
            width: 240,
            height: 160,
        }
    }

    fn theme() -> UiTheme {
        UiTheme::default()
    }

    fn color(value: u8) -> Pixel {
        Pixel::rgb(value, value, value)
    }

    #[test]
    fn slider_mouse_click_drag_release_and_hover() {
        let mut state = UiStateStore::default();
        let (initial, _) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.slider_f32("VOLUME", 0.0, 0.0..=1.0, 0.05)
        });
        let track = slider_track_rect(rect_for_label(initial.dump(), "VOLUME"));
        let mut value = 0.0;
        for (x, pressed, released, expected) in [
            (track.x + (track.width / 2) as i32, true, false, Some(0.5)),
            (track.x + track.width as i32 + 50, false, false, Some(1.0)),
            (track.x - 50, false, true, Some(0.0)),
            (track.x + track.width as i32, false, false, None),
        ] {
            let (output, slider) = run_headless_with_input(
                size(),
                &mut state,
                UiInput {
                    pointer: UiPointerInput {
                        position: Some((x, track.y)),
                        pressed,
                        released,
                    },
                    ..UiInput::default()
                },
                theme(),
                |ui| ui.slider_f32("VOLUME", value, 0.0..=1.0, 0.05),
            );
            assert_eq!(output.changed(slider), expected);
            if let Some(next) = output.changed(slider) {
                value = next;
            }
        }
    }

    #[test]
    fn slider_value_display_respects_step_and_range_origin() {
        assert_eq!(slider_value_text(0.65000004, 0.0, 1.0, 0.05), "0.65");
        assert_eq!(slider_value_text(1.25, 0.25, 2.0, 1.0), "1.25");
        assert_eq!(slider_value_text(0.001, 0.0, 1.0, 0.001), "0.001");
        assert_eq!(slider_value_text(42.0, 0.0, 100.0, 1.0), "42");
        assert_eq!(slider_value_text(0.000001, 0.0, 1.0, 0.000001), "0.000001");
    }

    #[test]
    fn slider_focused_track_does_not_look_full_at_minimum_and_thumb_marks_position() {
        let theme = UiTheme {
            border: color(73),
            accent: color(211),
            row_height: 32,
            ..theme()
        };
        let mut state = UiStateStore::default();
        let mut framebuffer = Framebuffer::new(240, 80);
        let (output, slider) = run(
            &mut framebuffer,
            &mut state,
            UiNavInput::default(),
            theme,
            |ui| ui.slider_f32("SIZE", 16.0, 16.0..=56.0, 1.0),
        );

        assert_eq!(output.focused_id(), Some(slider.id()));
        let slider_rect = rect_for_label(output.dump(), "SIZE");
        let track = slider_track_rect(slider_rect);
        let thumb = slider_thumb_rect(track, 0.0);

        assert_eq!(
            framebuffer.pixel(track.x + track.width as i32 - 2, track.y + 1),
            Some(theme.border)
        );
        assert_eq!(
            framebuffer.pixel(thumb.x + 1, thumb.y + 1),
            Some(theme.accent)
        );
        assert_eq!(thumb.x, track.x);
    }

    #[test]
    fn slider_thumb_tracks_minimum_middle_and_maximum() {
        let track = Rect {
            x: 20,
            y: 10,
            width: 101,
            height: 5,
        };
        let min = slider_thumb_rect(track, 0.0);
        let mid = slider_thumb_rect(track, 0.5);
        let max = slider_thumb_rect(track, 1.0);

        assert_eq!(min.x, track.x);
        assert!(mid.x > min.x);
        assert!(max.x > mid.x);
        assert_eq!(
            max.x + max.width as i32,
            track.x + track.width as i32
        );
        assert!(min.height > track.height);
    }

    #[test]
    fn toggle_indicator_tracks_effective_value_in_the_activation_frame() {
        let theme = UiTheme {
            row_height: 32,
            ..theme()
        };
        for (input, confirm, expected) in [
            (false, false, false),
            (true, false, true),
            (false, true, true),
            (true, true, false),
        ] {
            let mut state = UiStateStore::default();
            let mut framebuffer = Framebuffer::new(240, 80);
            let (output, toggle) = run(
                &mut framebuffer,
                &mut state,
                UiNavInput {
                    confirm,
                    ..UiNavInput::default()
                },
                theme,
                |ui| ui.toggle("ENABLED", input),
            );
            assert_eq!(output.changed(toggle), confirm.then_some(expected));
            let rect = rect_for_label(output.dump(), "ENABLED");
            let x = rect.x + rect.width as i32 - 37;
            let y = centered_coordinate(rect.y, rect.height, 14) + 6;
            let track = if expected {
                theme.accent
            } else {
                theme.muted_text
            };
            assert_eq!(framebuffer.pixel(x, y), Some(track));
            assert_eq!(
                framebuffer.pixel(x + 6, y),
                Some(if expected {
                    track
                } else {
                    theme.control_background
                })
            );
            assert_eq!(
                framebuffer.pixel(x + 24, y),
                Some(if expected {
                    theme.control_background
                } else {
                    track
                })
            );
        }
    }

    fn first_control_position() -> (i32, i32) {
        (12, 12)
    }

    fn rect_for_label(dump: &str, label: &str) -> Rect {
        let needle = format!("label={label:?}");
        let line = dump
            .lines()
            .find(|line| line.contains(&needle))
            .expect("label in dump");
        let rect_text = line
            .split_once("rect=[")
            .expect("rect prefix")
            .1
            .split_once(']')
            .expect("rect suffix")
            .0;
        let (coordinates, size) = rect_text.split_once(' ').expect("rect separator");
        let (x, y) = coordinates.split_once(',').expect("coordinate separator");
        let (width, height) = size.split_once('x').expect("size separator");
        Rect {
            x: x.parse().expect("x"),
            y: y.parse().expect("y"),
            width: width.parse().expect("width"),
            height: height.parse().expect("height"),
        }
    }

    #[test]
    fn tiny_ui_builds_headlessly_with_small_concept_surface() {
        let mut state = UiStateStore::default();
        let (output, resume) =
            run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
                ui.panel(|ui| {
                    ui.text("PAUSED");
                    ui.button_action("RESUME", RESUME)
                })
            });

        assert_eq!(output.metrics().interactive_count, 1);
        assert_eq!(output.metrics().persistent_state_entries, 1);
        assert_eq!(state.focused_id(), Some(resume.id()));
        assert_eq!(state.interaction.focused_id(), Some(resume.id()));
        assert_eq!(output.focused_id(), Some(resume.id()));
        assert_eq!(output.hovered_id(), None);
        assert!(output.dump().contains("Button"));
        assert!(output.dump().contains("PAUSED"));
    }

    #[test]
    fn button_activation_is_semantic_and_transaction_local() {
        let mut state = UiStateStore::default();
        let (output, resume) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                confirm: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| ui.button_action("RESUME", RESUME),
        );

        assert!(output.activated(resume));
        assert!(output.action_pressed(RESUME));
        assert!(!output.cancel_requested());
    }

    #[test]
    fn pointer_focus_and_release_activate_transactional_button() {
        let mut state = UiStateStore::default();
        let position = first_control_position();

        let _ = run_headless_with_input(
            size(),
            &mut state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(position),
                    pressed: true,
                    released: false,
                },
                ..UiInput::default()
            },
            theme(),
            |ui| ui.button_action("RESUME", RESUME),
        );

        let (output, resume) = run_headless_with_input(
            size(),
            &mut state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(position),
                    pressed: false,
                    released: true,
                },
                ..UiInput::default()
            },
            theme(),
            |ui| ui.button_action("RESUME", RESUME),
        );

        assert_eq!(output.hovered_id(), Some(resume.id()));
        assert_eq!(output.focused_id(), Some(resume.id()));
        assert!(output.activated(resume));
        assert!(output.action_pressed(RESUME));
    }

    #[test]
    fn pointer_activation_proposes_transactional_toggle_value() {
        let mut state = UiStateStore::default();
        let position = first_control_position();

        let _ = run_headless_with_input(
            size(),
            &mut state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(position),
                    pressed: true,
                    released: false,
                },
                ..UiInput::default()
            },
            theme(),
            |ui| ui.toggle("ENABLED", false),
        );

        let (output, toggle) = run_headless_with_input(
            size(),
            &mut state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(position),
                    pressed: false,
                    released: true,
                },
                ..UiInput::default()
            },
            theme(),
            |ui| ui.toggle("ENABLED", false),
        );

        assert!(output.activated(toggle));
        assert_eq!(output.changed(toggle), Some(true));
    }

    #[test]
    fn transactional_touch_move_off_cancels_capture() {
        let mut state = UiStateStore::default();
        let position = first_control_position();
        let started = [Touch {
            id: 41,
            phase: TouchPhase::Started,
            position: Some(position),
        }];
        let moved = [Touch {
            id: 41,
            phase: TouchPhase::Moved,
            position: Some((230, 150)),
        }];

        let _ = run_headless_with_input(
            size(),
            &mut state,
            UiInput {
                touches: &started,
                ..UiInput::default()
            },
            theme(),
            |ui| ui.button_action("RESUME", RESUME),
        );
        assert_eq!(state.interaction.touch_capture_count(), 1);

        let (output, resume) = run_headless_with_input(
            size(),
            &mut state,
            UiInput {
                touches: &moved,
                ..UiInput::default()
            },
            theme(),
            |ui| ui.button_action("RESUME", RESUME),
        );
        assert_eq!(state.interaction.touch_capture_count(), 0);
        assert!(!output.activated(resume));
    }

    #[test]
    fn toggle_returns_typed_proposal_without_mutating_consumer_state() {
        let mut state = UiStateStore::default();
        let enabled = false;

        let (output, toggle) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                confirm: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| ui.toggle("ENABLED", enabled),
        );

        assert!(!enabled);
        assert_eq!(output.changed(toggle), Some(true));
        assert!(output.dump().contains("effective=true"));
    }

    #[test]
    fn slider_effective_value_updates_same_frame_but_dependent_text_uses_input_snapshot() {
        let mut state = UiStateStore::default();
        let volume = 0.65_f32;

        let (_, _) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.toggle("ENABLED", true);
            ui.slider_f32("VOLUME", volume, 0.0..=1.0, 0.05)
        });

        let (output, slider) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                down: true,
                right: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| {
                ui.toggle("ENABLED", true);
                let slider = ui.slider_f32("VOLUME", volume, 0.0..=1.0, 0.05);
                ui.text(format!("GLOBAL VOLUME = {volume:.2}"));
                slider
            },
        );

        assert_eq!(volume, 0.65);
        let proposed = output.changed(slider).expect("slider proposal");
        assert!((proposed - 0.70).abs() < f32::EPSILON);
        assert!(output.dump().contains("effective=0.700"));
        assert!(output.dump().contains("GLOBAL VOLUME = 0.65"));
    }

    #[test]
    fn consumer_description_closure_runs_once() {
        let mut state = UiStateStore::default();
        let mut calls = 0;

        let _ = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            calls += 1;
            ui.button("ONLY ONCE");
        });

        assert_eq!(calls, 1);
    }

    #[test]
    fn transactional_grid_composes_arbitrary_widgets_responsively() {
        let spec = UiGridSpec {
            min_cell_width: 100,
            preferred_cell_height: 70,
            gap: 8,
            padding: 8,
        };

        let mut wide_state = UiStateStore::default();
        let (wide, (button_a, _, _, _)) = run_headless(
            Size {
                width: 360,
                height: 220,
            },
            &mut wide_state,
            UiNavInput::default(),
            theme(),
            |ui| {
                ui.grid(spec, |ui| {
                    let button_a = ui.button("A");
                    let button_b = ui.button("B");
                    let toggle = ui.toggle("ENABLED", false);
                    let slider = ui.slider_f32("VOLUME", 0.5, 0.0..=1.0, 0.1);
                    ui.text("GRID LABEL");
                    (button_a, button_b, toggle, slider)
                })
            },
        );
        assert_eq!(wide.metrics().interactive_count, 4);
        assert_eq!(wide.focused_id(), Some(button_a.id()));
        assert!(wide.dump().contains("Grid"));
        assert!(wide.dump().contains("ToggleBool"));
        assert!(wide.dump().contains("SliderF32"));
        assert!(wide.dump().contains("GRID LABEL"));

        let wide_a = rect_for_label(wide.dump(), "A");
        let wide_b = rect_for_label(wide.dump(), "B");
        assert!(wide_b.x > wide_a.x);

        let mut small_state = UiStateStore::default();
        let (small, _) = run_headless(
            Size {
                width: 120,
                height: 220,
            },
            &mut small_state,
            UiNavInput::default(),
            theme(),
            |ui| {
                ui.grid(spec, |ui| {
                    ui.button("A");
                    ui.button("B");
                    ui.toggle("ENABLED", false);
                    ui.slider_f32("VOLUME", 0.5, 0.0..=1.0, 0.1);
                    ui.text("GRID LABEL");
                })
            },
        );
        let small_a = rect_for_label(small.dump(), "A");
        let small_b = rect_for_label(small.dump(), "B");
        assert_eq!(small_a.x, small_b.x);
    }

    #[test]
    fn keyed_identity_survives_sibling_insertion_and_reorder() {
        let mut state = UiStateStore::default();

        let (_, (_, b1)) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            let a = ui.keyed("a", |ui| ui.button("A"));
            let b = ui.keyed("b", |ui| ui.button("B"));
            (a, b)
        });

        let (_, (_, b2)) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| {
                let a = ui.keyed("a", |ui| ui.button("A"));
                let b = ui.keyed("b", |ui| ui.button("B"));
                (a, b)
            },
        );
        assert_eq!(b1.id(), b2.id());
        assert_eq!(state.focused_id(), Some(b2.id()));

        let (_, b3) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.keyed("x", |ui| ui.button("X"));
            let b = ui.keyed("b", |ui| ui.button("B"));
            ui.keyed("a", |ui| ui.button("A"));
            b
        });

        assert_eq!(b2.id(), b3.id());
        assert_eq!(state.focused_id(), Some(b3.id()));
    }

    #[test]
    fn removing_focused_widget_uses_deterministic_fallback_and_prunes_state() {
        let mut state = UiStateStore::default();

        let _ = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.keyed("a", |ui| ui.button("A"));
            ui.keyed("b", |ui| ui.button("B"));
        });
        let _ = run_headless(
            size(),
            &mut state,
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| {
                ui.keyed("a", |ui| ui.button("A"));
                ui.keyed("b", |ui| ui.button("B"));
            },
        );

        let (_, a) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.keyed("a", |ui| ui.button("A"))
        });

        assert_eq!(state.focused_id(), Some(a.id()));
        assert_eq!(state.entry_count(), 1);
    }

    #[test]
    fn duplicate_explicit_key_is_diagnosed_deterministically() {
        let mut state = UiStateStore::default();

        let (output, _) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.keyed("dup", |ui| ui.button("FIRST"));
            ui.keyed("dup", |ui| ui.button("SECOND"));
        });

        assert!(matches!(
            output.diagnostics(),
            [UiDiagnostic::DuplicateExplicitKey {
                key,
                occurrence: 2,
                ..
            }] if key == "dup"
        ));
    }

    #[test]
    fn widget_kind_reuse_resets_state_and_is_diagnosed() {
        let mut state = UiStateStore::default();

        let (_, old_button) =
            run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
                ui.keyed("same", |ui| ui.button("BUTTON"))
            });

        let (output, new_toggle) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                confirm: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| ui.keyed("same", |ui| ui.toggle("TOGGLE", false)),
        );

        assert_eq!(old_button.id(), new_toggle.id());
        assert!(output.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            UiDiagnostic::WidgetKindChanged {
                previous: WidgetKind::Button,
                current: WidgetKind::ToggleBool,
                ..
            }
        )));
        assert_eq!(output.changed(new_toggle), Some(true));
    }

    #[test]
    fn widget_ref_is_generation_scoped_for_output_queries() {
        let mut state = UiStateStore::default();

        let (_, first) = run_headless(size(), &mut state, UiNavInput::default(), theme(), |ui| {
            ui.slider_f32("VOLUME", 0.5, 0.0..=1.0, 0.1)
        });

        let (second_output, second) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                right: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| ui.slider_f32("VOLUME", 0.5, 0.0..=1.0, 0.1),
        );

        assert_eq!(first.id(), second.id());
        assert_ne!(first.generation(), second.generation());
        assert_eq!(second_output.changed(first), None);
        assert_eq!(second_output.changed(second), Some(0.6));
    }

    #[test]
    fn cancel_is_exposed_without_becoming_game_state() {
        let mut state = UiStateStore::default();
        let (output, _) = run_headless(
            size(),
            &mut state,
            UiNavInput {
                cancel: true,
                ..UiNavInput::default()
            },
            theme(),
            |ui| ui.text("TEST"),
        );
        assert!(output.cancel_requested());
    }

    #[test]
    fn render_path_uses_existing_framebuffer_primitives() {
        let mut state = UiStateStore::default();
        let mut framebuffer = Framebuffer::new(160, 96);

        let (output, _) = run(
            &mut framebuffer,
            &mut state,
            UiNavInput::default(),
            theme(),
            |ui| {
                ui.panel(|ui| {
                    ui.text("MFE");
                    ui.button("BUTTON");
                });
            },
        );

        assert_eq!(output.metrics().interactive_count, 1);
        assert!(
            framebuffer
                .as_rgba8()
                .chunks_exact(4)
                .any(|pixel| pixel.iter().any(|channel| *channel != 0))
        );
    }

    #[test]
    fn styled_run_applies_component_and_local_paint_without_changing_interaction() {
        let mut state = UiStateStore::default();
        let mut framebuffer = Framebuffer::new(220, 140);
        let local_button_background = color(61);
        let toggle_background = color(91);
        let slider_background = color(121);
        let slider_accent = color(151);
        let panel_background = color(181);
        let stylesheet = UiStyleSheet {
            panel: UiComponentStyle {
                base: UiStyleOverride {
                    background: Some(panel_background),
                    ..UiStyleOverride::default()
                },
                ..UiComponentStyle::default()
            },
            button: UiComponentStyle {
                base: UiStyleOverride {
                    background: Some(color(31)),
                    ..UiStyleOverride::default()
                },
                ..UiComponentStyle::default()
            },
            toggle_bool: UiComponentStyle {
                base: UiStyleOverride {
                    background: Some(toggle_background),
                    ..UiStyleOverride::default()
                },
                ..UiComponentStyle::default()
            },
            slider_f32: UiComponentStyle {
                base: UiStyleOverride {
                    background: Some(slider_background),
                    accent: Some(slider_accent),
                    ..UiStyleOverride::default()
                },
                ..UiComponentStyle::default()
            },
            ..UiStyleSheet::default()
        };

        let (output, button) = run_styled(
            &mut framebuffer,
            &mut state,
            UiNavInput::default(),
            theme(),
            stylesheet,
            |ui| {
                ui.panel(|ui| {
                    let button = ui.button_styled(
                        "ACTION",
                        UiStyleOverride {
                            background: Some(local_button_background),
                            ..UiStyleOverride::default()
                        },
                    );
                    ui.toggle("ENABLED", false);
                    ui.slider_f32("VOLUME", 0.5, 0.0..=1.0, 0.1);
                    button
                })
            },
        );

        assert_eq!(output.focused_id(), Some(button.id()));
        assert!(!output.activated(button));

        let panel_rect = rect_for_kind(output.dump(), "Panel");
        assert_eq!(
            framebuffer.pixel(panel_rect.x + 2, panel_rect.y + 2),
            Some(panel_background)
        );

        let button_rect = rect_for_label(output.dump(), "ACTION");
        assert_eq!(
            framebuffer.pixel(button_rect.x + 2, button_rect.y + 2),
            Some(local_button_background)
        );

        let toggle_rect = rect_for_label(output.dump(), "ENABLED");
        assert_eq!(
            framebuffer.pixel(toggle_rect.x + 2, toggle_rect.y + 2),
            Some(toggle_background)
        );

        let slider_rect = rect_for_label(output.dump(), "VOLUME");
        assert_eq!(
            framebuffer.pixel(slider_rect.x + 2, slider_rect.y + 2),
            Some(slider_background)
        );
        let track = slider_track_rect(slider_rect);
        assert_eq!(
            framebuffer.pixel(track.x + 1, track.y + 1),
            Some(slider_accent)
        );
    }

    #[test]
    fn styled_panel_padding_and_gap_affect_arranged_child_geometry() {
        let mut state = UiStateStore::default();
        let stylesheet = UiStyleSheet {
            panel: UiComponentStyle {
                base: UiStyleOverride {
                    padding: Some(16),
                    vertical_gap: Some(9),
                    ..UiStyleOverride::default()
                },
                ..UiComponentStyle::default()
            },
            ..UiStyleSheet::default()
        };

        let (output, _) = run_headless_styled(
            size(),
            &mut state,
            UiNavInput::default(),
            theme(),
            stylesheet,
            |ui| {
                ui.panel(|ui| {
                    ui.button("A");
                    ui.button("B");
                });
            },
        );

        let panel_rect = rect_for_kind(output.dump(), "Panel");
        let first = rect_for_label(output.dump(), "A");
        let second = rect_for_label(output.dump(), "B");

        assert_eq!(first.x, panel_rect.x + 16);
        assert_eq!(first.y, panel_rect.y + 16);
        assert_eq!(second.y, first.y + first.height as i32 + 9);
    }

    #[test]
    fn visual_style_state_precedence_is_focused_then_hovered_then_active() {
        let stylesheet = UiStyleSheet {
            button: UiComponentStyle {
                focused: UiStyleOverride {
                    background: Some(color(30)),
                    ..UiStyleOverride::default()
                },
                hovered: UiStyleOverride {
                    background: Some(color(60)),
                    ..UiStyleOverride::default()
                },
                active: UiStyleOverride {
                    background: Some(color(90)),
                    ..UiStyleOverride::default()
                },
                ..UiComponentStyle::default()
            },
            ..UiStyleSheet::default()
        };

        let mut focused_state = UiStateStore::default();
        let mut focused_framebuffer = Framebuffer::new(140, 80);
        let (focused_output, _) = run_styled(
            &mut focused_framebuffer,
            &mut focused_state,
            UiNavInput::default(),
            theme(),
            stylesheet,
            |ui| ui.button("BUTTON"),
        );
        let focused_rect = rect_for_label(focused_output.dump(), "BUTTON");
        assert_eq!(
            focused_framebuffer.pixel(focused_rect.x + 2, focused_rect.y + 2),
            Some(color(30))
        );

        let mut hovered_state = UiStateStore::default();
        let mut hovered_framebuffer = Framebuffer::new(140, 80);
        let (hovered_output, _) = run_with_input_styled(
            &mut hovered_framebuffer,
            &mut hovered_state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(first_control_position()),
                    ..UiPointerInput::default()
                },
                ..UiInput::default()
            },
            theme(),
            stylesheet,
            |ui| ui.button("BUTTON"),
        );
        let hovered_rect = rect_for_label(hovered_output.dump(), "BUTTON");
        assert_eq!(hovered_output.hovered_id(), hovered_output.focused_id());
        assert_eq!(
            hovered_framebuffer.pixel(hovered_rect.x + 2, hovered_rect.y + 2),
            Some(color(60))
        );

        let mut active_state = UiStateStore::default();
        let mut active_framebuffer = Framebuffer::new(140, 80);
        let (active_output, button) = run_with_input_styled(
            &mut active_framebuffer,
            &mut active_state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some(first_control_position()),
                    pressed: true,
                    released: false,
                },
                ..UiInput::default()
            },
            theme(),
            stylesheet,
            |ui| ui.button("BUTTON"),
        );
        let active_rect = rect_for_label(active_output.dump(), "BUTTON");
        assert_eq!(active_output.hovered_id(), Some(button.id()));
        assert_eq!(active_output.focused_id(), Some(button.id()));
        assert!(!active_output.activated(button));
        assert_eq!(
            active_framebuffer.pixel(active_rect.x + 2, active_rect.y + 2),
            Some(color(90))
        );
    }

    fn rect_for_kind(dump: &str, kind: &str) -> Rect {
        let line = dump
            .lines()
            .find(|line| line.contains(kind))
            .expect("kind in dump");
        let rect_text = line
            .split_once("rect=[")
            .expect("rect prefix")
            .1
            .split_once(']')
            .expect("rect suffix")
            .0;
        parse_rect_text(rect_text)
    }

    fn parse_rect_text(rect_text: &str) -> Rect {
        let (coordinates, size) = rect_text.split_once(' ').expect("rect separator");
        let (x, y) = coordinates.split_once(',').expect("coordinate separator");
        let (width, height) = size.split_once('x').expect("size separator");
        Rect {
            x: x.parse().expect("x"),
            y: y.parse().expect("y"),
            width: width.parse().expect("width"),
            height: height.parse().expect("height"),
        }
    }
}
