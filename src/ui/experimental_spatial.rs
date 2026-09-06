use std::collections::HashMap;

use crate::{ActionId, Framebuffer, Image, ImageFilter, ImageFit, Pixel, Rect, TextRenderer};
#[cfg(test)]
use crate::{Touch, TouchPhase};

use super::{
    UiStyleOverride, UiStyleSheet, UiTheme,
    kernel::{
        UiId, UiInput, UiInteractionOutput, UiInteractionState, UiNavigationPolicy, UiPointerInput,
        UiResolvedTarget, run_interaction_pass,
    },
    layout::{UiGridSpec, inset_rect, layout_responsive_grid},
    style::{UiResolvedStyle, resolve_style},
};

/// Compatibility name retained while the MFE-001B spatial adapter converges
/// into the production kernel.
pub type PointerInput = UiPointerInput;

/// Compatibility name retained while the MFE-001B spatial adapter converges
/// into the production kernel.
pub type SpatialInput<'a> = UiInput<'a>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSpec {
    pub min_cell_width: u32,
    pub preferred_cell_height: u32,
    pub gap: u32,
    pub padding: u32,
}

impl Default for GridSpec {
    fn default() -> Self {
        Self {
            min_cell_width: 96,
            preferred_cell_height: 72,
            gap: 8,
            padding: 8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpatialCard<'a> {
    pub id: UiId,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub image: Option<&'a Image>,
    pub action: ActionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardLayout {
    pub id: UiId,
    pub rect: Rect,
    pub image_rect: Rect,
    pub text_rect: Rect,
}

impl UiResolvedTarget for CardLayout {
    fn ui_id(&self) -> UiId {
        self.id
    }

    fn ui_rect(&self) -> Rect {
        self.rect
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardVisualState {
    pub focused: bool,
    pub hovered: bool,
    pub active: bool,
}

pub trait CardPainter {
    fn paint(
        &self,
        framebuffer: &mut Framebuffer,
        card: &SpatialCard<'_>,
        layout: CardLayout,
        visual: CardVisualState,
        theme: UiTheme,
    );
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultCardPainter;

impl DefaultCardPainter {
    pub fn paint_styled(
        &self,
        framebuffer: &mut Framebuffer,
        card: &SpatialCard<'_>,
        layout: CardLayout,
        visual: CardVisualState,
        theme: UiTheme,
        stylesheet: UiStyleSheet,
    ) {
        let style = resolve_card_style(theme, stylesheet, visual);
        framebuffer.fill_rect(
            layout.rect.x,
            layout.rect.y,
            layout.rect.width,
            layout.rect.height,
            style.background,
        );
        draw_border(framebuffer, layout.rect, style.border, style.border_width);

        if let Some(image) = card.image
            && layout.image_rect.width > 0
            && layout.image_rect.height > 0
        {
            framebuffer.draw_image_fit(
                image,
                layout.image_rect,
                ImageFit::Cover,
                ImageFilter::Nearest,
            );
        }

        if visual.active {
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                layout.rect.width,
                3_u32.min(layout.rect.height),
                style.accent,
            );
        }

        let text = TextRenderer::new(theme.font);
        draw_centered(
            framebuffer,
            text,
            Rect {
                x: layout.text_rect.x,
                y: layout.text_rect.y,
                width: layout.text_rect.width,
                height: layout.text_rect.height / 2,
            },
            card.title,
            theme.text_scale.max(1),
            style.text,
        );
        if !card.subtitle.is_empty() {
            draw_centered(
                framebuffer,
                text,
                Rect {
                    x: layout.text_rect.x,
                    y: layout
                        .text_rect
                        .y
                        .saturating_add(u32_to_i32(layout.text_rect.height / 2)),
                    width: layout.text_rect.width,
                    height: layout
                        .text_rect
                        .height
                        .saturating_sub(layout.text_rect.height / 2),
                },
                card.subtitle,
                theme.text_scale.max(1),
                style.muted_text,
            );
        }
    }
}

impl CardPainter for DefaultCardPainter {
    fn paint(
        &self,
        framebuffer: &mut Framebuffer,
        card: &SpatialCard<'_>,
        layout: CardLayout,
        visual: CardVisualState,
        theme: UiTheme,
    ) {
        self.paint_styled(
            framebuffer,
            card,
            layout,
            visual,
            theme,
            UiStyleSheet::default(),
        );
    }
}

#[derive(Debug, Default)]
pub struct SpatialState {
    interaction: UiInteractionState,
    capture_debug_dump: bool,
}

impl SpatialState {
    pub const fn focused_id(&self) -> Option<UiId> {
        self.interaction.focused_id()
    }

    pub const fn pointer_capture_id(&self) -> Option<UiId> {
        self.interaction.pointer_capture_id()
    }

    pub fn touch_capture_count(&self) -> usize {
        self.interaction.touch_capture_count()
    }

    pub const fn debug_capture_enabled(&self) -> bool {
        self.capture_debug_dump
    }

    pub fn set_debug_capture(&mut self, enabled: bool) {
        self.capture_debug_dump = enabled;
    }
}

/// Compatibility wrapper retained for the MFE-001B probes.
///
/// Semantic interaction state is owned by the shared kernel output. This type
/// only adds Grid-specific geometry and an optional deterministic textual dump.
#[derive(Debug, Clone)]
pub struct SpatialOutput {
    interaction: UiInteractionOutput,
    columns: usize,
    rows: usize,
    layouts: Vec<CardLayout>,
    dump: String,
}

impl SpatialOutput {
    pub const fn focused_id(&self) -> Option<UiId> {
        self.interaction.focused_id()
    }

    pub const fn hovered_id(&self) -> Option<UiId> {
        self.interaction.hovered_id()
    }

    pub fn activated(&self, id: UiId) -> bool {
        self.interaction.activated(id)
    }

    pub fn action_pressed(&self, action: ActionId) -> bool {
        self.interaction.action_pressed(action)
    }

    pub const fn cancel_requested(&self) -> bool {
        self.interaction.cancel_requested()
    }

    pub const fn columns(&self) -> usize {
        self.columns
    }

    pub const fn rows(&self) -> usize {
        self.rows
    }

    pub fn layouts(&self) -> &[CardLayout] {
        &self.layouts
    }

    pub fn layout_for(&self, id: UiId) -> Option<CardLayout> {
        self.layouts.iter().copied().find(|layout| layout.id == id)
    }

    pub fn dump(&self) -> &str {
        &self.dump
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_card_grid<P: CardPainter>(
    framebuffer: &mut Framebuffer,
    bounds: Rect,
    state: &mut SpatialState,
    input: SpatialInput<'_>,
    spec: GridSpec,
    theme: UiTheme,
    cards: &[SpatialCard<'_>],
    painter: &P,
) -> SpatialOutput {
    let output = resolve_card_grid(bounds, state, input, spec, cards);

    for (card, layout) in cards.iter().zip(output.layouts.iter().copied()) {
        let visual = CardVisualState {
            focused: output.focused_id() == Some(card.id),
            hovered: output.hovered_id() == Some(card.id),
            active: state.interaction.is_active(card.id),
        };
        painter.paint(framebuffer, card, layout, visual, theme);
    }

    output
}

#[allow(clippy::too_many_arguments)]
pub fn run_default_card_grid_styled(
    framebuffer: &mut Framebuffer,
    bounds: Rect,
    state: &mut SpatialState,
    input: SpatialInput<'_>,
    spec: GridSpec,
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    cards: &[SpatialCard<'_>],
) -> SpatialOutput {
    let output = resolve_card_grid(bounds, state, input, spec, cards);
    let painter = DefaultCardPainter;

    for (card, layout) in cards.iter().zip(output.layouts.iter().copied()) {
        let visual = CardVisualState {
            focused: output.focused_id() == Some(card.id),
            hovered: output.hovered_id() == Some(card.id),
            active: state.interaction.is_active(card.id),
        };
        painter.paint_styled(framebuffer, card, layout, visual, theme, stylesheet);
    }

    output
}

pub fn run_card_grid_headless(
    bounds: Rect,
    state: &mut SpatialState,
    input: SpatialInput<'_>,
    spec: GridSpec,
    cards: &[SpatialCard<'_>],
) -> SpatialOutput {
    resolve_card_grid(bounds, state, input, spec, cards)
}

fn resolve_card_grid(
    bounds: Rect,
    state: &mut SpatialState,
    input: SpatialInput<'_>,
    spec: GridSpec,
    cards: &[SpatialCard<'_>],
) -> SpatialOutput {
    let (columns, rows, layouts) = layout_cards(bounds, spec, cards);
    let interaction = run_interaction_pass(
        &mut state.interaction,
        input,
        &layouts,
        UiNavigationPolicy::Spatial,
        |id| {
            cards
                .iter()
                .find(|card| card.id == id)
                .map(|card| card.action)
        },
    );

    let focused = interaction.focused_id();
    let hovered = interaction.hovered_id();
    let dump = if state.capture_debug_dump {
        dump_layout(
            bounds,
            columns,
            rows,
            &layouts,
            focused,
            hovered,
            state.interaction.pointer_capture_id(),
            state.interaction.touch_captures(),
            interaction.activated_ids(),
        )
    } else {
        String::new()
    };

    SpatialOutput {
        interaction,
        columns,
        rows,
        layouts,
        dump,
    }
}

fn layout_cards(
    bounds: Rect,
    spec: GridSpec,
    cards: &[SpatialCard<'_>],
) -> (usize, usize, Vec<CardLayout>) {
    let grid = layout_responsive_grid(
        bounds,
        UiGridSpec {
            min_cell_width: spec.min_cell_width,
            preferred_cell_height: spec.preferred_cell_height,
            gap: spec.gap,
            padding: spec.padding,
        },
        cards.len(),
    );

    let mut layouts = Vec::with_capacity(cards.len());
    for (card, rect) in cards.iter().zip(grid.rects.iter().copied()) {
        let inner = inset_rect(rect, 4);
        let image_height = if card.image.is_some() {
            inner.height.saturating_mul(3) / 5
        } else {
            0
        };
        let image_rect = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: image_height,
        };
        let text_rect = Rect {
            x: inner.x,
            y: inner.y.saturating_add(u32_to_i32(image_height)),
            width: inner.width,
            height: inner.height.saturating_sub(image_height),
        };

        layouts.push(CardLayout {
            id: card.id,
            rect,
            image_rect,
            text_rect,
        });
    }

    (grid.columns, grid.rows, layouts)
}

fn draw_centered(
    framebuffer: &mut Framebuffer,
    text: TextRenderer,
    rect: Rect,
    label: &str,
    scale: u32,
    color: Pixel,
) {
    let (width, height) = text.text_size(label, scale);
    text.draw_scaled(
        framebuffer,
        centered_coordinate(rect.x, rect.width, width),
        centered_coordinate(rect.y, rect.height, height),
        label,
        scale,
        color,
    );
}

fn resolve_card_style(
    theme: UiTheme,
    stylesheet: UiStyleSheet,
    visual: CardVisualState,
) -> UiResolvedStyle {
    let visual_state = super::UiVisualState {
        focused: visual.focused,
        hovered: visual.hovered,
        active: visual.active,
    };
    let mut resolved = resolve_style(
        theme,
        stylesheet.button,
        UiStyleOverride::default(),
        visual_state,
    );

    let later_border = (visual.hovered && stylesheet.button.hovered.border.is_some())
        || (visual.active && stylesheet.button.active.border.is_some());
    if (visual.focused || visual.hovered) && !later_border {
        resolved.border = resolved.accent;
    }

    resolved
}

fn draw_border(framebuffer: &mut Framebuffer, rect: Rect, color: Pixel, width: u32) {
    for inset in 0..width.max(1) {
        let rect = inset_rect(rect, inset);
        if rect.width == 0 || rect.height == 0 {
            break;
        }
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
    }
}

fn centered_coordinate(origin: i32, extent: u32, content_extent: u32) -> i32 {
    let coordinate = i64::from(origin) + (i64::from(extent) - i64::from(content_extent)) / 2;
    coordinate.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[allow(clippy::too_many_arguments)]
fn dump_layout(
    bounds: Rect,
    columns: usize,
    rows: usize,
    layouts: &[CardLayout],
    focused: Option<UiId>,
    hovered: Option<UiId>,
    pointer_capture: Option<UiId>,
    touch_capture: &HashMap<u64, UiId>,
    activated: &std::collections::HashSet<UiId>,
) -> String {
    use std::fmt::Write as _;

    let mut dump = String::new();
    let _ = writeln!(
        dump,
        "Grid bounds=[{},{} {}x{}] columns={} rows={}",
        bounds.x, bounds.y, bounds.width, bounds.height, columns, rows
    );
    for layout in layouts {
        let focused_marker = if focused == Some(layout.id) {
            " focused"
        } else {
            ""
        };
        let hovered_marker = if hovered == Some(layout.id) {
            " hovered"
        } else {
            ""
        };
        let active_marker = if pointer_capture == Some(layout.id)
            || touch_capture.values().any(|id| *id == layout.id)
        {
            " active"
        } else {
            ""
        };
        let activated_marker = if activated.contains(&layout.id) {
            " activated"
        } else {
            ""
        };
        let _ = writeln!(
            dump,
            "Card id={:016x} rect=[{},{} {}x{}]{}{}{}{}",
            layout.id.raw(),
            layout.rect.x,
            layout.rect.y,
            layout.rect.width,
            layout.rect.height,
            focused_marker,
            hovered_marker,
            active_marker,
            activated_marker
        );
    }
    dump
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{Image, Size};

    use super::*;
    use crate::ui::experimental::{UiStateStore, run_headless};
    use crate::ui::kernel::UiNavInput;

    const ACTION_A: ActionId = ActionId::new("mfe001b.a");
    const ACTION_B: ActionId = ActionId::new("mfe001b.b");
    const ACTION_C: ActionId = ActionId::new("mfe001b.c");
    const ACTION_D: ActionId = ActionId::new("mfe001b.d");
    const ACTION_E: ActionId = ActionId::new("mfe001b.e");
    const ACTION_F: ActionId = ActionId::new("mfe001b.f");
    const ACTIONS: [ActionId; 6] = [ACTION_A, ACTION_B, ACTION_C, ACTION_D, ACTION_E, ACTION_F];
    const TITLES: [&str; 6] = ["A", "B", "C", "D", "E", "F"];

    fn bounds(width: u32) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width,
            height: 220,
        }
    }

    fn ids() -> Vec<UiId> {
        let mut state = UiStateStore::default();
        let (_, ids) = run_headless(
            Size {
                width: 320,
                height: 200,
            },
            &mut state,
            UiNavInput::default(),
            UiTheme::default(),
            |ui| {
                TITLES
                    .iter()
                    .map(|key| ui.keyed(key, |ui| ui.button(*key).id()))
                    .collect::<Vec<_>>()
            },
        );
        ids
    }

    fn cards<'a>(ids: &'a [UiId]) -> Vec<SpatialCard<'a>> {
        ids.iter()
            .enumerate()
            .map(|(index, id)| SpatialCard {
                id: *id,
                title: TITLES[index],
                subtitle: "",
                image: None,
                action: ACTIONS[index],
            })
            .collect()
    }

    fn input(nav: UiNavInput) -> SpatialInput<'static> {
        SpatialInput {
            nav,
            pointer: PointerInput::default(),
            touches: &[],
        }
    }

    fn color(value: u8) -> Pixel {
        Pixel::rgb(value, value, value)
    }

    fn debug_state() -> SpatialState {
        let mut state = SpatialState::default();
        state.set_debug_capture(true);
        state
    }

    #[test]
    fn responsive_grid_changes_column_count_deterministically() {
        let ids = ids();
        let cards = cards(&ids);
        let spec = GridSpec {
            min_cell_width: 100,
            preferred_cell_height: 70,
            gap: 8,
            padding: 8,
        };

        let mut small_state = SpatialState::default();
        let small = run_card_grid_headless(
            bounds(120),
            &mut small_state,
            SpatialInput::default(),
            spec,
            &cards,
        );
        let mut medium_state = SpatialState::default();
        let medium = run_card_grid_headless(
            bounds(240),
            &mut medium_state,
            SpatialInput::default(),
            spec,
            &cards,
        );
        let mut wide_state = debug_state();
        let wide = run_card_grid_headless(
            bounds(360),
            &mut wide_state,
            SpatialInput::default(),
            spec,
            &cards,
        );

        assert_eq!(small.columns(), 1);
        assert_eq!(medium.columns(), 2);
        assert_eq!(wide.columns(), 3);
        assert_eq!(wide.rows(), 2);
        assert!(wide.dump().contains("Grid bounds="));

        let mut replay_state = debug_state();
        let replay = run_card_grid_headless(
            bounds(360),
            &mut replay_state,
            SpatialInput::default(),
            spec,
            &cards,
        );
        assert_eq!(wide.dump(), replay.dump());
    }

    #[test]
    fn spatial_debug_dump_capture_is_opt_in_without_changing_semantics() {
        let ids = ids();
        let cards = cards(&ids);
        let spec = GridSpec::default();
        let mut off_state = SpatialState::default();
        let mut on_state = debug_state();

        let off = run_card_grid_headless(
            bounds(360),
            &mut off_state,
            SpatialInput::default(),
            spec,
            &cards,
        );
        let on = run_card_grid_headless(
            bounds(360),
            &mut on_state,
            SpatialInput::default(),
            spec,
            &cards,
        );

        assert!(!off_state.debug_capture_enabled());
        assert!(on_state.debug_capture_enabled());
        assert!(off.dump().is_empty());
        assert!(on.dump().contains("Grid bounds="));
        assert_eq!(off.columns(), on.columns());
        assert_eq!(off.rows(), on.rows());
        assert_eq!(off.layouts(), on.layouts());
        assert_eq!(off.focused_id(), on.focused_id());
        assert_eq!(off.hovered_id(), on.hovered_id());
    }

    #[test]
    fn keyed_focus_survives_reorder() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();

        let _ = run_card_grid_headless(
            bounds(360),
            &mut state,
            input(UiNavInput {
                right: true,
                ..UiNavInput::default()
            }),
            GridSpec::default(),
            &cards,
        );
        assert_eq!(state.focused_id(), Some(ids[1]));

        let reordered = vec![cards[2], cards[0], cards[1], cards[3], cards[4], cards[5]];
        let output = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            &reordered,
        );
        assert_eq!(output.focused_id(), Some(ids[1]));
    }

    #[test]
    fn removing_focused_card_uses_previous_index_as_fallback() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();
        let _ = run_card_grid_headless(
            bounds(360),
            &mut state,
            input(UiNavInput {
                right: true,
                ..UiNavInput::default()
            }),
            GridSpec::default(),
            &cards,
        );
        assert_eq!(state.focused_id(), Some(ids[1]));

        let without_b = vec![cards[0], cards[2], cards[3], cards[4], cards[5]];
        let output = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            &without_b,
        );
        assert_eq!(output.focused_id(), Some(ids[2]));
    }

    #[test]
    fn spatial_navigation_uses_resolved_geometry() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();
        let spec = GridSpec {
            min_cell_width: 100,
            ..GridSpec::default()
        };

        let _ = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput::default(),
            spec,
            &cards[..5],
        );
        assert_eq!(state.focused_id(), Some(ids[0]));
        let right = run_card_grid_headless(
            bounds(360),
            &mut state,
            input(UiNavInput {
                right: true,
                ..UiNavInput::default()
            }),
            spec,
            &cards[..5],
        );
        assert_eq!(right.focused_id(), Some(ids[1]));
        let down = run_card_grid_headless(
            bounds(360),
            &mut state,
            input(UiNavInput {
                down: true,
                ..UiNavInput::default()
            }),
            spec,
            &cards[..5],
        );
        assert_eq!(down.focused_id(), Some(ids[4]));
    }

    #[test]
    fn mouse_press_release_on_same_card_activates_semantic_action() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();
        let baseline = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            &cards,
        );
        let position = center_i32(baseline.layouts()[0].rect);

        let _ = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput {
                nav: UiNavInput::default(),
                pointer: PointerInput {
                    position: Some(position),
                    pressed: true,
                    released: false,
                },
                touches: &[],
            },
            GridSpec::default(),
            &cards,
        );
        assert_eq!(state.pointer_capture_id(), Some(ids[0]));

        let released = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput {
                nav: UiNavInput::default(),
                pointer: PointerInput {
                    position: Some(position),
                    pressed: false,
                    released: true,
                },
                touches: &[],
            },
            GridSpec::default(),
            &cards,
        );
        assert!(released.activated(ids[0]));
        assert!(released.action_pressed(ACTION_A));
        assert_eq!(state.pointer_capture_id(), None);
    }

    #[test]
    fn cancel_flows_through_shared_semantic_output() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();
        let output = run_card_grid_headless(
            bounds(360),
            &mut state,
            input(UiNavInput {
                cancel: true,
                ..UiNavInput::default()
            }),
            GridSpec::default(),
            &cards,
        );

        assert!(output.cancel_requested());
    }

    #[test]
    fn touch_move_off_card_cancels_capture() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();
        let baseline = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            &cards,
        );
        let position = center_i32(baseline.layouts()[0].rect);

        let started = [Touch {
            id: 7,
            phase: TouchPhase::Started,
            position: Some(position),
        }];
        let _ = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput {
                nav: UiNavInput::default(),
                pointer: PointerInput::default(),
                touches: &started,
            },
            GridSpec::default(),
            &cards,
        );
        assert_eq!(state.touch_capture_count(), 1);

        let moved = [Touch {
            id: 7,
            phase: TouchPhase::Moved,
            position: Some((359, 219)),
        }];
        let _ = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput {
                nav: UiNavInput::default(),
                pointer: PointerInput::default(),
                touches: &moved,
            },
            GridSpec::default(),
            &cards,
        );
        assert_eq!(state.touch_capture_count(), 0);
    }

    #[test]
    fn touch_start_end_inside_card_activates() {
        let ids = ids();
        let cards = cards(&ids);
        let mut state = SpatialState::default();
        let baseline = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            &cards,
        );
        let position = center_i32(baseline.layouts()[0].rect);
        let touches = [
            Touch {
                id: 9,
                phase: TouchPhase::Started,
                position: Some(position),
            },
            Touch {
                id: 9,
                phase: TouchPhase::Ended,
                position: Some(position),
            },
        ];
        let output = run_card_grid_headless(
            bounds(360),
            &mut state,
            SpatialInput {
                nav: UiNavInput::default(),
                pointer: PointerInput::default(),
                touches: &touches,
            },
            GridSpec::default(),
            &cards,
        );
        assert!(output.action_pressed(ACTION_A));
        assert_eq!(state.touch_capture_count(), 0);
    }

    struct CountingPainter {
        calls: Cell<usize>,
    }

    impl CardPainter for CountingPainter {
        fn paint(
            &self,
            _framebuffer: &mut Framebuffer,
            _card: &SpatialCard<'_>,
            _layout: CardLayout,
            _visual: CardVisualState,
            _theme: UiTheme,
        ) {
            self.calls.set(self.calls.get() + 1);
        }
    }

    #[test]
    fn custom_card_painter_is_a_first_class_escape_hatch() {
        let ids = ids();
        let cards = cards(&ids[..3]);
        let mut state = SpatialState::default();
        let mut framebuffer = Framebuffer::new(360, 220);
        let painter = CountingPainter {
            calls: Cell::new(0),
        };
        let output = run_card_grid(
            &mut framebuffer,
            bounds(360),
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            UiTheme::default(),
            &cards,
            &painter,
        );
        assert_eq!(painter.calls.get(), cards.len());
        assert!(output.dump().is_empty());
    }

    #[test]
    fn default_card_painter_renders_existing_image_primitive() {
        let ids = ids();
        let image = Image::from_rgba8(1, 1, vec![220, 20, 20, 255]).expect("valid image");
        let cards = [SpatialCard {
            id: ids[0],
            title: "IMAGE",
            subtitle: "",
            image: Some(&image),
            action: ACTION_A,
        }];
        let mut state = SpatialState::default();
        let mut framebuffer = Framebuffer::new(180, 120);
        let output = run_card_grid(
            &mut framebuffer,
            Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 120,
            },
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            UiTheme::default(),
            &cards,
            &DefaultCardPainter,
        );
        let center = center_i32(output.layouts()[0].image_rect);
        assert_eq!(
            framebuffer.pixel(center.0, center.1),
            Some(Pixel::rgb(220, 20, 20))
        );
    }

    #[test]
    fn styled_default_card_painter_uses_button_style_semantics() {
        let ids = ids();
        let cards = cards(&ids[..2]);
        let mut state = SpatialState::default();
        let mut framebuffer = Framebuffer::new(220, 120);
        let background = color(41);
        let border = color(51);
        let accent = color(61);
        let stylesheet = UiStyleSheet {
            button: super::super::UiComponentStyle {
                base: UiStyleOverride {
                    background: Some(background),
                    border: Some(border),
                    accent: Some(accent),
                    border_width: Some(2),
                    ..UiStyleOverride::default()
                },
                ..super::super::UiComponentStyle::default()
            },
            ..UiStyleSheet::default()
        };

        let output = run_default_card_grid_styled(
            &mut framebuffer,
            Rect {
                x: 0,
                y: 0,
                width: 220,
                height: 120,
            },
            &mut state,
            SpatialInput::default(),
            GridSpec {
                min_cell_width: 90,
                preferred_cell_height: 80,
                gap: 8,
                padding: 8,
            },
            UiTheme::default(),
            stylesheet,
            &cards,
        );

        let focused = output.layout_for(ids[0]).expect("focused card layout").rect;
        let unfocused = output
            .layout_for(ids[1])
            .expect("unfocused card layout")
            .rect;
        assert_eq!(output.focused_id(), Some(ids[0]));
        assert_eq!(
            framebuffer.pixel(focused.x + 2, focused.y + 2),
            Some(background)
        );
        assert_eq!(framebuffer.pixel(focused.x, focused.y), Some(accent));
        assert_eq!(
            framebuffer.pixel(unfocused.x + 1, unfocused.y + 1),
            Some(border)
        );
    }

    #[test]
    fn styled_default_card_painter_applies_active_over_hovered_over_focused() {
        let ids = ids();
        let cards = cards(&ids[..1]);
        let mut state = SpatialState::default();
        let baseline = run_card_grid_headless(
            Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 120,
            },
            &mut state,
            SpatialInput::default(),
            GridSpec::default(),
            &cards,
        );
        let position = center_i32(baseline.layouts()[0].rect);
        let mut framebuffer = Framebuffer::new(180, 120);
        let stylesheet = UiStyleSheet {
            button: super::super::UiComponentStyle {
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
                    accent: Some(color(120)),
                    ..UiStyleOverride::default()
                },
                ..super::super::UiComponentStyle::default()
            },
            ..UiStyleSheet::default()
        };

        let output = run_default_card_grid_styled(
            &mut framebuffer,
            Rect {
                x: 0,
                y: 0,
                width: 180,
                height: 120,
            },
            &mut state,
            SpatialInput {
                pointer: PointerInput {
                    position: Some(position),
                    pressed: true,
                    released: false,
                },
                ..SpatialInput::default()
            },
            GridSpec::default(),
            UiTheme::default(),
            stylesheet,
            &cards,
        );

        let rect = output.layouts()[0].rect;
        assert_eq!(output.focused_id(), Some(ids[0]));
        assert_eq!(output.hovered_id(), Some(ids[0]));
        assert!(!output.activated(ids[0]));
        assert_eq!(framebuffer.pixel(rect.x + 2, rect.y + 4), Some(color(90)));
        assert_eq!(framebuffer.pixel(rect.x + 2, rect.y + 1), Some(color(120)));
    }

    fn center_i32(rect: Rect) -> (i32, i32) {
        (
            rect.x.saturating_add(u32_to_i32(rect.width / 2)),
            rect.y.saturating_add(u32_to_i32(rect.height / 2)),
        )
    }
}
