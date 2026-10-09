use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::{ActionId, ControlMap, Framebuffer, Input, MouseButton, Pixel, Rect, Size, TouchPhase};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Insets {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Insets {
    pub const fn uniform(value: u32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlPlacement {
    pub anchor: TouchAnchor,
    pub offset: (i32, i32),
    pub visual_size: Size,
    pub hit_padding: Insets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionMode {
    FourWay,
    EightWay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectionalActions {
    pub up: ActionId,
    pub down: ActionId,
    pub left: ActionId,
    pub right: ActionId,
}

impl DirectionalActions {
    pub const fn new(up: ActionId, down: ActionId, left: ActionId, right: ActionId) -> Self {
        Self {
            up,
            down,
            left,
            right,
        }
    }

    fn all(self) -> [ActionId; 4] {
        [self.up, self.down, self.left, self.right]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DPadConfig {
    pub actions: DirectionalActions,
    pub direction_mode: DirectionMode,
    pub placement: Option<ControlPlacement>,
}

impl DPadConfig {
    pub const fn new(actions: DirectionalActions) -> Self {
        Self {
            actions,
            direction_mode: DirectionMode::FourWay,
            placement: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualStickConfig {
    pub actions: DirectionalActions,
    pub direction_mode: DirectionMode,
    pub placement: Option<ControlPlacement>,
    pub dead_zone_ratio: f32,
}

impl VirtualStickConfig {
    pub const fn new(actions: DirectionalActions) -> Self {
        Self {
            actions,
            direction_mode: DirectionMode::EightWay,
            placement: None,
            dead_zone_ratio: 0.20,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementControl {
    None,
    DPad(DPadConfig),
    VirtualStick(VirtualStickConfig),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchButtonConfig {
    pub action: ActionId,
    pub label: &'static str,
    pub placement: Option<ControlPlacement>,
}

impl TouchButtonConfig {
    pub const fn new(action: ActionId, label: &'static str) -> Self {
        Self {
            action,
            label,
            placement: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FaceButtons {
    pub a: Option<TouchButtonConfig>,
    pub b: Option<TouchButtonConfig>,
    pub x: Option<TouchButtonConfig>,
    pub y: Option<TouchButtonConfig>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TouchNavigationConfig {
    pub back: Option<TouchButtonConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceButtonArrangement {
    Diamond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceButtonsPlacement {
    pub anchor: TouchAnchor,
    pub offset: (i32, i32),
    pub button_size: Size,
    /// Distance from the group center to each canonical face-button center.
    pub center_offset: Size,
    pub hit_padding: Insets,
    pub arrangement: FaceButtonArrangement,
}

impl Default for FaceButtonsPlacement {
    fn default() -> Self {
        Self {
            anchor: TouchAnchor::BottomRight,
            offset: (0, 0),
            button_size: Size {
                width: 44,
                height: 44,
            },
            center_offset: Size {
                width: 58,
                height: 58,
            },
            hit_padding: Insets::uniform(5),
            arrangement: FaceButtonArrangement::Diamond,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchControlsLayout {
    pub safe_margin: Insets,
    pub movement: ControlPlacement,
    pub face_buttons: FaceButtonsPlacement,
    pub back: ControlPlacement,
}

impl Default for TouchControlsLayout {
    fn default() -> Self {
        Self {
            safe_margin: Insets::uniform(12),
            movement: ControlPlacement {
                anchor: TouchAnchor::BottomLeft,
                offset: (0, 0),
                visual_size: Size {
                    width: 104,
                    height: 104,
                },
                hit_padding: Insets::uniform(8),
            },
            face_buttons: FaceButtonsPlacement::default(),
            back: ControlPlacement {
                anchor: TouchAnchor::TopRight,
                offset: (0, 0),
                visual_size: Size {
                    width: 72,
                    height: 28,
                },
                hit_padding: Insets::uniform(4),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchButtonShape {
    Circle,
    RoundedRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchControlsStyle {
    pub face_button_shape: TouchButtonShape,
    pub opacity: f32,
    pub pressed_opacity: f32,
    pub border_width: u32,
    pub a_color: Pixel,
    pub b_color: Pixel,
    pub x_color: Pixel,
    pub y_color: Pixel,
    pub control_color: Pixel,
    pub border_color: Pixel,
    pub label_color: Pixel,
}

impl Default for TouchControlsStyle {
    fn default() -> Self {
        Self {
            face_button_shape: TouchButtonShape::Circle,
            opacity: 0.38,
            pressed_opacity: 0.78,
            border_width: 2,
            a_color: Pixel::rgb(62, 196, 92),
            b_color: Pixel::rgb(225, 72, 72),
            x_color: Pixel::rgb(70, 132, 226),
            y_color: Pixel::rgb(235, 196, 66),
            control_color: Pixel::rgb(205, 220, 232),
            border_color: Pixel::rgb(235, 245, 252),
            label_color: Pixel::WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TouchVisibility {
    Always,
    #[default]
    AfterFirstTouch,
    Hidden,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TouchControlsConfig {
    pub movement: MovementControl,
    pub buttons: FaceButtons,
    pub navigation: TouchNavigationConfig,
    pub layout: TouchControlsLayout,
    pub style: TouchControlsStyle,
    pub visibility: TouchVisibility,
}

impl TouchControlsConfig {
    pub fn new(movement: MovementControl) -> Self {
        Self {
            movement,
            buttons: FaceButtons::default(),
            navigation: TouchNavigationConfig::default(),
            layout: TouchControlsLayout::default(),
            style: TouchControlsStyle::default(),
            visibility: TouchVisibility::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TouchControlId {
    Movement,
    FaceA,
    FaceB,
    FaceX,
    FaceY,
    Back,
}

impl TouchControlId {
    fn name(self) -> &'static str {
        match self {
            Self::Movement => "movement",
            Self::FaceA => "A",
            Self::FaceB => "B",
            Self::FaceX => "X",
            Self::FaceY => "Y",
            Self::Back => "BACK",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTouchButton {
    pub id: TouchControlId,
    pub action: ActionId,
    pub label: &'static str,
    pub visual_rect: Rect,
    pub hit_rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedMovementControl {
    DPad {
        actions: DirectionalActions,
        direction_mode: DirectionMode,
        visual_rect: Rect,
        hit_rect: Rect,
    },
    VirtualStick {
        actions: DirectionalActions,
        direction_mode: DirectionMode,
        dead_zone_ratio: f32,
        visual_rect: Rect,
        hit_rect: Rect,
    },
}

impl ResolvedMovementControl {
    pub const fn visual_rect(self) -> Rect {
        match self {
            Self::DPad { visual_rect, .. } | Self::VirtualStick { visual_rect, .. } => visual_rect,
        }
    }

    pub const fn hit_rect(self) -> Rect {
        match self {
            Self::DPad { hit_rect, .. } | Self::VirtualStick { hit_rect, .. } => hit_rect,
        }
    }

    pub const fn actions(self) -> DirectionalActions {
        match self {
            Self::DPad { actions, .. } | Self::VirtualStick { actions, .. } => actions,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTouchControls {
    pub surface_size: Size,
    pub movement: Option<ResolvedMovementControl>,
    pub buttons: Vec<ResolvedTouchButton>,
    pub navigation: Vec<ResolvedTouchButton>,
}

impl ResolvedTouchControls {
    pub fn button(&self, id: TouchControlId) -> Option<ResolvedTouchButton> {
        self.buttons
            .iter()
            .chain(self.navigation.iter())
            .find(|button| button.id == id)
            .copied()
    }

    fn top_level_hit_rects(&self) -> Vec<(TouchControlId, Rect)> {
        let mut rects = Vec::new();
        if let Some(movement) = self.movement {
            rects.push((TouchControlId::Movement, movement.hit_rect()));
        }
        rects.extend(
            self.buttons
                .iter()
                .chain(self.navigation.iter())
                .map(|button| (button.id, button.hit_rect)),
        );
        rects
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TouchLayoutError {
    SurfaceTooSmall,
    ZeroVisualSize(&'static str),
    OutOfBounds(&'static str),
    InvalidDeadZone(f32),
    InvalidOpacity(&'static str, f32),
    Overlap(&'static str, &'static str),
    DuplicateAction(ActionId),
}

impl fmt::Display for TouchLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurfaceTooSmall => write!(formatter, "touch-control surface is too small"),
            Self::ZeroVisualSize(control) => {
                write!(formatter, "{control} has a zero-sized visual rectangle")
            }
            Self::OutOfBounds(control) => {
                write!(formatter, "{control} resolves outside the touch safe area")
            }
            Self::InvalidDeadZone(value) => {
                write!(
                    formatter,
                    "virtual-stick dead-zone ratio {value} is outside [0, 1)"
                )
            }
            Self::InvalidOpacity(field, value) => {
                write!(formatter, "{field} opacity {value} is outside [0, 1]")
            }
            Self::Overlap(first, second) => {
                write!(formatter, "touch hitboxes overlap: {first} <-> {second}")
            }
            Self::DuplicateAction(action) => {
                write!(
                    formatter,
                    "touch ActionId is assigned more than once: {}",
                    action.as_str()
                )
            }
        }
    }
}

impl std::error::Error for TouchLayoutError {}

pub fn resolve_touch_controls(
    config: &TouchControlsConfig,
    surface_size: Size,
) -> Result<ResolvedTouchControls, TouchLayoutError> {
    validate_config(config)?;

    let movement = match config.movement {
        MovementControl::None => None,
        MovementControl::DPad(dpad) => {
            let placement = dpad.placement.unwrap_or(config.layout.movement);
            let (visual_rect, hit_rect) = resolve_placement(
                "movement",
                placement,
                config.layout.safe_margin,
                surface_size,
            )?;
            Some(ResolvedMovementControl::DPad {
                actions: dpad.actions,
                direction_mode: dpad.direction_mode,
                visual_rect,
                hit_rect,
            })
        }
        MovementControl::VirtualStick(stick) => {
            let placement = stick.placement.unwrap_or(config.layout.movement);
            let (visual_rect, hit_rect) = resolve_placement(
                "movement",
                placement,
                config.layout.safe_margin,
                surface_size,
            )?;
            Some(ResolvedMovementControl::VirtualStick {
                actions: stick.actions,
                direction_mode: stick.direction_mode,
                dead_zone_ratio: stick.dead_zone_ratio,
                visual_rect,
                hit_rect,
            })
        }
    };

    let mut buttons = Vec::new();
    let has_face_buttons = config.buttons.a.is_some()
        || config.buttons.b.is_some()
        || config.buttons.x.is_some()
        || config.buttons.y.is_some();
    let group = if has_face_buttons {
        Some(resolve_face_button_group(&config.layout, surface_size)?)
    } else {
        None
    };

    for (id, button) in [
        (TouchControlId::FaceA, config.buttons.a),
        (TouchControlId::FaceB, config.buttons.b),
        (TouchControlId::FaceX, config.buttons.x),
        (TouchControlId::FaceY, config.buttons.y),
    ] {
        let Some(button) = button else {
            continue;
        };

        let (visual_rect, hit_rect) = if let Some(placement) = button.placement {
            resolve_placement(
                id.name(),
                placement,
                config.layout.safe_margin,
                surface_size,
            )?
        } else {
            let visual_rect = canonical_face_button_rect(
                id,
                group.expect("face-button group exists when a button is configured"),
                config.layout.face_buttons,
            );
            let hit_rect = expand_rect(visual_rect, config.layout.face_buttons.hit_padding)
                .ok_or(TouchLayoutError::OutOfBounds(id.name()))?;
            ensure_rect_in_safe_area(id.name(), hit_rect, surface_size, config.layout.safe_margin)?;
            (visual_rect, hit_rect)
        };

        buttons.push(ResolvedTouchButton {
            id,
            action: button.action,
            label: button.label,
            visual_rect,
            hit_rect,
        });
    }

    let mut navigation = Vec::new();
    if let Some(back) = config.navigation.back {
        let placement = back.placement.unwrap_or(config.layout.back);
        let (visual_rect, hit_rect) =
            resolve_placement("BACK", placement, config.layout.safe_margin, surface_size)?;
        navigation.push(ResolvedTouchButton {
            id: TouchControlId::Back,
            action: back.action,
            label: back.label,
            visual_rect,
            hit_rect,
        });
    }

    let resolved = ResolvedTouchControls {
        surface_size,
        movement,
        buttons,
        navigation,
    };
    validate_overlaps(&resolved)?;
    Ok(resolved)
}

fn validate_config(config: &TouchControlsConfig) -> Result<(), TouchLayoutError> {
    if !(config.style.opacity.is_finite() && (0.0..=1.0).contains(&config.style.opacity)) {
        return Err(TouchLayoutError::InvalidOpacity(
            "idle",
            config.style.opacity,
        ));
    }
    if !(config.style.pressed_opacity.is_finite()
        && (0.0..=1.0).contains(&config.style.pressed_opacity))
    {
        return Err(TouchLayoutError::InvalidOpacity(
            "pressed",
            config.style.pressed_opacity,
        ));
    }

    if let MovementControl::VirtualStick(stick) = config.movement
        && !(stick.dead_zone_ratio.is_finite() && (0.0..1.0).contains(&stick.dead_zone_ratio))
    {
        return Err(TouchLayoutError::InvalidDeadZone(stick.dead_zone_ratio));
    }

    let mut actions = HashSet::new();
    let mut add = |action: ActionId| -> Result<(), TouchLayoutError> {
        if actions.insert(action) {
            Ok(())
        } else {
            Err(TouchLayoutError::DuplicateAction(action))
        }
    };

    match config.movement {
        MovementControl::None => {}
        MovementControl::DPad(dpad) => {
            for action in dpad.actions.all() {
                add(action)?;
            }
        }
        MovementControl::VirtualStick(stick) => {
            for action in stick.actions.all() {
                add(action)?;
            }
        }
    }

    for button in [
        config.buttons.a,
        config.buttons.b,
        config.buttons.x,
        config.buttons.y,
        config.navigation.back,
    ]
    .into_iter()
    .flatten()
    {
        add(button.action)?;
    }

    Ok(())
}

fn resolve_face_button_group(
    layout: &TouchControlsLayout,
    surface_size: Size,
) -> Result<Rect, TouchLayoutError> {
    let group = layout.face_buttons;
    if group.button_size.width == 0 || group.button_size.height == 0 {
        return Err(TouchLayoutError::ZeroVisualSize("face-buttons"));
    }

    let width = group
        .button_size
        .width
        .checked_add(group.center_offset.width.saturating_mul(2))
        .ok_or(TouchLayoutError::SurfaceTooSmall)?;
    let height = group
        .button_size
        .height
        .checked_add(group.center_offset.height.saturating_mul(2))
        .ok_or(TouchLayoutError::SurfaceTooSmall)?;
    let placement = ControlPlacement {
        anchor: group.anchor,
        offset: group.offset,
        visual_size: Size { width, height },
        hit_padding: group.hit_padding,
    };
    let (visual, _) =
        resolve_placement("face-buttons", placement, layout.safe_margin, surface_size)?;
    Ok(visual)
}

fn canonical_face_button_rect(
    id: TouchControlId,
    group_rect: Rect,
    group: FaceButtonsPlacement,
) -> Rect {
    let button = group.button_size;
    let center_x = i64::from(group_rect.x) + i64::from(group_rect.width / 2);
    let center_y = i64::from(group_rect.y) + i64::from(group_rect.height / 2);
    let offset_x = i64::from(group.center_offset.width);
    let offset_y = i64::from(group.center_offset.height);
    let (cx, cy) = match (group.arrangement, id) {
        (FaceButtonArrangement::Diamond, TouchControlId::FaceA) => (center_x, center_y + offset_y),
        (FaceButtonArrangement::Diamond, TouchControlId::FaceB) => (center_x + offset_x, center_y),
        (FaceButtonArrangement::Diamond, TouchControlId::FaceX) => (center_x - offset_x, center_y),
        (FaceButtonArrangement::Diamond, TouchControlId::FaceY) => (center_x, center_y - offset_y),
        _ => (center_x, center_y),
    };

    Rect {
        x: i32::try_from(cx - i64::from(button.width / 2)).unwrap_or(i32::MAX),
        y: i32::try_from(cy - i64::from(button.height / 2)).unwrap_or(i32::MAX),
        width: button.width,
        height: button.height,
    }
}

fn resolve_placement(
    name: &'static str,
    placement: ControlPlacement,
    safe_margin: Insets,
    surface_size: Size,
) -> Result<(Rect, Rect), TouchLayoutError> {
    if placement.visual_size.width == 0 || placement.visual_size.height == 0 {
        return Err(TouchLayoutError::ZeroVisualSize(name));
    }

    let hit_width = placement
        .visual_size
        .width
        .checked_add(placement.hit_padding.left)
        .and_then(|value| value.checked_add(placement.hit_padding.right))
        .ok_or(TouchLayoutError::SurfaceTooSmall)?;
    let hit_height = placement
        .visual_size
        .height
        .checked_add(placement.hit_padding.top)
        .and_then(|value| value.checked_add(placement.hit_padding.bottom))
        .ok_or(TouchLayoutError::SurfaceTooSmall)?;

    let safe_left = i64::from(safe_margin.left);
    let safe_top = i64::from(safe_margin.top);
    let safe_right = i64::from(surface_size.width) - i64::from(safe_margin.right);
    let safe_bottom = i64::from(surface_size.height) - i64::from(safe_margin.bottom);

    if safe_left >= safe_right || safe_top >= safe_bottom {
        return Err(TouchLayoutError::SurfaceTooSmall);
    }

    let safe_width = safe_right - safe_left;
    let safe_height = safe_bottom - safe_top;
    if i64::from(hit_width) > safe_width || i64::from(hit_height) > safe_height {
        return Err(TouchLayoutError::OutOfBounds(name));
    }

    let hit_x = match placement.anchor {
        TouchAnchor::TopLeft | TouchAnchor::CenterLeft | TouchAnchor::BottomLeft => safe_left,
        TouchAnchor::TopCenter | TouchAnchor::Center | TouchAnchor::BottomCenter => {
            safe_left + (safe_width - i64::from(hit_width)) / 2
        }
        TouchAnchor::TopRight | TouchAnchor::CenterRight | TouchAnchor::BottomRight => {
            safe_right - i64::from(hit_width)
        }
    } + i64::from(placement.offset.0);

    let hit_y = match placement.anchor {
        TouchAnchor::TopLeft | TouchAnchor::TopCenter | TouchAnchor::TopRight => safe_top,
        TouchAnchor::CenterLeft | TouchAnchor::Center | TouchAnchor::CenterRight => {
            safe_top + (safe_height - i64::from(hit_height)) / 2
        }
        TouchAnchor::BottomLeft | TouchAnchor::BottomCenter | TouchAnchor::BottomRight => {
            safe_bottom - i64::from(hit_height)
        }
    } + i64::from(placement.offset.1);

    let hit_rect = Rect {
        x: i32::try_from(hit_x).map_err(|_| TouchLayoutError::OutOfBounds(name))?,
        y: i32::try_from(hit_y).map_err(|_| TouchLayoutError::OutOfBounds(name))?,
        width: hit_width,
        height: hit_height,
    };
    ensure_rect_in_safe_area(name, hit_rect, surface_size, safe_margin)?;

    let visual_rect = Rect {
        x: hit_rect
            .x
            .saturating_add(i32::try_from(placement.hit_padding.left).unwrap_or(i32::MAX)),
        y: hit_rect
            .y
            .saturating_add(i32::try_from(placement.hit_padding.top).unwrap_or(i32::MAX)),
        width: placement.visual_size.width,
        height: placement.visual_size.height,
    };
    Ok((visual_rect, hit_rect))
}

fn ensure_rect_in_safe_area(
    name: &'static str,
    rect: Rect,
    surface_size: Size,
    safe_margin: Insets,
) -> Result<(), TouchLayoutError> {
    let min_x = i64::from(rect.x);
    let min_y = i64::from(rect.y);
    let max_x = min_x + i64::from(rect.width);
    let max_y = min_y + i64::from(rect.height);
    let safe_left = i64::from(safe_margin.left);
    let safe_top = i64::from(safe_margin.top);
    let safe_right = i64::from(surface_size.width) - i64::from(safe_margin.right);
    let safe_bottom = i64::from(surface_size.height) - i64::from(safe_margin.bottom);

    if min_x < safe_left || min_y < safe_top || max_x > safe_right || max_y > safe_bottom {
        return Err(TouchLayoutError::OutOfBounds(name));
    }
    Ok(())
}

fn expand_rect(rect: Rect, padding: Insets) -> Option<Rect> {
    let x = i64::from(rect.x) - i64::from(padding.left);
    let y = i64::from(rect.y) - i64::from(padding.top);
    let width = rect
        .width
        .checked_add(padding.left)?
        .checked_add(padding.right)?;
    let height = rect
        .height
        .checked_add(padding.top)?
        .checked_add(padding.bottom)?;

    Some(Rect {
        x: i32::try_from(x).ok()?,
        y: i32::try_from(y).ok()?,
        width,
        height,
    })
}

fn validate_overlaps(resolved: &ResolvedTouchControls) -> Result<(), TouchLayoutError> {
    let rects = resolved.top_level_hit_rects();
    for (index, (first_id, first)) in rects.iter().copied().enumerate() {
        for (second_id, second) in rects.iter().copied().skip(index + 1) {
            if first.intersects(second) {
                return Err(TouchLayoutError::Overlap(first_id.name(), second_id.name()));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default, Clone)]
struct TouchRuntimeState {
    active_touches: HashSet<u64>,
    quarantined_touches: HashSet<u64>,
    owner_by_touch: HashMap<u64, TouchControlId>,
    owner_by_control: HashMap<TouchControlId, u64>,
    positions: HashMap<u64, (i32, i32)>,
    touch_seen: bool,
    mouse_contact: Option<TouchControlId>,
    mouse_position: Option<(i32, i32)>,
    mouse_suppression_frames: u8,
}

impl TouchRuntimeState {
    fn clear_ownership_into_quarantine(&mut self) {
        self.quarantined_touches
            .extend(self.active_touches.iter().copied());
        self.owner_by_touch.clear();
        self.owner_by_control.clear();
    }

    fn release_touch(&mut self, touch_id: u64) {
        if let Some(control) = self.owner_by_touch.remove(&touch_id) {
            self.owner_by_control.remove(&control);
        }
        self.positions.remove(&touch_id);
        self.active_touches.remove(&touch_id);
        self.quarantined_touches.remove(&touch_id);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TouchControlsUpdate {
    held_actions: Vec<ActionId>,
}

impl TouchControlsUpdate {
    pub fn held_actions(&self) -> &[ActionId] {
        &self.held_actions
    }
}

#[derive(Debug, Clone)]
pub struct TouchControls {
    config: TouchControlsConfig,
    resolved: ResolvedTouchControls,
    state: TouchRuntimeState,
    held_actions: HashSet<ActionId>,
}

impl TouchControls {
    pub fn new(config: TouchControlsConfig, surface_size: Size) -> Result<Self, TouchLayoutError> {
        let resolved = resolve_touch_controls(&config, surface_size)?;
        Ok(Self {
            config,
            resolved,
            state: TouchRuntimeState::default(),
            held_actions: HashSet::new(),
        })
    }

    pub fn config(&self) -> &TouchControlsConfig {
        &self.config
    }

    pub fn resolved(&self) -> &ResolvedTouchControls {
        &self.resolved
    }

    pub fn is_visible(&self) -> bool {
        match self.config.visibility {
            TouchVisibility::Always => true,
            TouchVisibility::AfterFirstTouch => self.state.touch_seen,
            TouchVisibility::Hidden => false,
        }
    }

    pub fn reset_visibility(&mut self) {
        self.state.touch_seen = false;
    }

    pub fn resize(&mut self, surface_size: Size) -> Result<(), TouchLayoutError> {
        let resolved = resolve_touch_controls(&self.config, surface_size)?;
        self.resolved = resolved;
        Ok(())
    }

    pub fn set_config(
        &mut self,
        config: TouchControlsConfig,
        surface_size: Size,
        controls: &mut ControlMap,
    ) -> Result<(), TouchLayoutError> {
        let resolved = resolve_touch_controls(&config, surface_size)?;
        for action in self.held_actions.drain() {
            controls.set_virtual(action, false);
        }
        self.state.clear_ownership_into_quarantine();
        self.config = config;
        self.resolved = resolved;
        Ok(())
    }

    pub fn reset(&mut self, controls: &mut ControlMap) {
        for action in self.held_actions.drain() {
            controls.set_virtual(action, false);
        }
        self.state = TouchRuntimeState::default();
    }

    /// Processes touch events and updates touch-derived virtual states.
    ///
    /// Call this before ControlMap::update.
    pub fn update(&mut self, input: &Input, controls: &mut ControlMap) -> TouchControlsUpdate {
        if !input.touches().is_empty() || !self.state.active_touches.is_empty() {
            self.state.mouse_suppression_frames = 3;
        } else {
            self.state.mouse_suppression_frames = self.state.mouse_suppression_frames.saturating_sub(1);
        }
        let mouse = input.mouse_button(MouseButton::Left);
        if self.state.mouse_suppression_frames > 0 || !mouse.held() {
            self.state.mouse_contact = None;
            self.state.mouse_position = None;
        } else {
            self.state.mouse_position = input.mouse_position();
            // Mouse presses are first-class inputs for desktop virtual controls.
            // Do not steal mouse gestures which started outside a control.
            if mouse.pressed() {
                self.state.mouse_contact = self.state.mouse_position.and_then(|p| self.control_at(p));
                if self.state.mouse_contact.is_some() {
                    self.state.touch_seen = true;
                }
            }
        }
        for touch in input.touches() {
            match touch.phase {
                TouchPhase::Started => {
                    self.state.active_touches.insert(touch.id);
                    if let Some(position) = touch.position {
                        self.state.positions.insert(touch.id, position);
                    }
                    self.state.touch_seen = true;

                    if self.state.quarantined_touches.contains(&touch.id)
                        || !self.interactive()
                        || self.state.owner_by_touch.contains_key(&touch.id)
                    {
                        continue;
                    }

                    let Some(position) = touch.position else {
                        continue;
                    };
                    let Some(control) = self.control_at(position) else {
                        continue;
                    };
                    if self.state.owner_by_control.contains_key(&control) {
                        continue;
                    }

                    self.state.owner_by_touch.insert(touch.id, control);
                    self.state.owner_by_control.insert(control, touch.id);
                }
                TouchPhase::Moved => {
                    if let Some(position) = touch.position {
                        self.state.positions.insert(touch.id, position);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    self.state.release_touch(touch.id);
                }
            }
        }

        let mut next_held = if self.interactive() {
            self.compute_held_actions()
        } else {
            HashSet::new()
        };
        if self.interactive()
            && let (Some(control), Some(position)) =
                (self.state.mouse_contact, self.state.mouse_position)
        {
            match control {
                TouchControlId::Movement => {
                    if let Some(movement) = self.resolved.movement {
                        next_held.extend(resolve_movement_actions(movement, position));
                    }
                }
                id => {
                    if let Some(button) = self.resolved.button(id) {
                        next_held.insert(button.action);
                    }
                }
            }
        }

        for action in self.configured_actions() {
            controls.set_virtual(action, next_held.contains(&action));
        }
        self.held_actions = next_held;

        let mut held_actions = self.held_actions.iter().copied().collect::<Vec<_>>();
        held_actions.sort_by_key(|action| action.as_str());
        TouchControlsUpdate { held_actions }
    }

    pub fn draw(&self, framebuffer: &mut Framebuffer) {
        if !self.is_visible() {
            return;
        }

        if let Some(movement) = self.resolved.movement {
            match movement {
                ResolvedMovementControl::DPad {
                    actions,
                    visual_rect,
                    ..
                } => self.draw_dpad(framebuffer, visual_rect, actions),
                ResolvedMovementControl::VirtualStick { visual_rect, .. } => {
                    self.draw_virtual_stick(framebuffer, visual_rect)
                }
            }
        }

        for button in &self.resolved.buttons {
            let color = match button.id {
                TouchControlId::FaceA => self.config.style.a_color,
                TouchControlId::FaceB => self.config.style.b_color,
                TouchControlId::FaceX => self.config.style.x_color,
                TouchControlId::FaceY => self.config.style.y_color,
                _ => self.config.style.control_color,
            };

            self.draw_face_button(
                framebuffer,
                button.visual_rect,
                button.label,
                color,
                self.held_actions.contains(&button.action),
            );
        }

        for button in &self.resolved.navigation {
            self.draw_back_button(
                framebuffer,
                button.visual_rect,
                button.label,
                self.held_actions.contains(&button.action),
            );
        }
    }

    fn interactive(&self) -> bool {
        !matches!(self.config.visibility, TouchVisibility::Hidden) && self.is_visible()
    }

    fn control_at(&self, position: (i32, i32)) -> Option<TouchControlId> {
        if let Some(movement) = self.resolved.movement
            && movement.hit_rect().contains(position)
        {
            return Some(TouchControlId::Movement);
        }

        self.resolved
            .buttons
            .iter()
            .chain(self.resolved.navigation.iter())
            .find(|button| button.hit_rect.contains(position))
            .map(|button| button.id)
    }

    fn configured_actions(&self) -> HashSet<ActionId> {
        let mut actions = HashSet::new();
        if let Some(movement) = self.resolved.movement {
            actions.extend(movement.actions().all());
        }
        actions.extend(
            self.resolved
                .buttons
                .iter()
                .chain(self.resolved.navigation.iter())
                .map(|button| button.action),
        );
        actions
    }

    fn compute_held_actions(&self) -> HashSet<ActionId> {
        let mut held = HashSet::new();

        for (control, touch_id) in &self.state.owner_by_control {
            match *control {
                TouchControlId::Movement => {
                    let Some(movement) = self.resolved.movement else {
                        continue;
                    };
                    let Some(position) = self.state.positions.get(touch_id).copied() else {
                        continue;
                    };
                    held.extend(resolve_movement_actions(movement, position));
                }
                id => {
                    if let Some(button) = self.resolved.button(id) {
                        held.insert(button.action);
                    }
                }
            }
        }

        held
    }

    fn draw_face_button(
        &self,
        framebuffer: &mut Framebuffer,
        rect: Rect,
        label: &str,
        color: Pixel,
        pressed: bool,
    ) {
        let opacity = if pressed {
            self.config.style.pressed_opacity
        } else {
            self.config.style.opacity
        };

        match self.config.style.face_button_shape {
            TouchButtonShape::Circle => {
                let radius = rect.width.min(rect.height) / 2;
                let center = rect_center(rect);
                fill_circle_blended(
                    framebuffer,
                    center.0,
                    center.1,
                    radius,
                    self.config.style.border_color,
                    opacity,
                );
                let inner = radius.saturating_sub(self.config.style.border_width.max(1));
                fill_circle_blended(framebuffer, center.0, center.1, inner, color, opacity);
            }
            TouchButtonShape::RoundedRect => {
                fill_rounded_rect_blended(
                    framebuffer,
                    rect,
                    self.config.style.border_color,
                    opacity,
                );
                let inset = self.config.style.border_width.max(1);
                if let Some(inner) = inset_rect(rect, inset) {
                    fill_rounded_rect_blended(framebuffer, inner, color, opacity);
                }
            }
        }

        draw_label(framebuffer, rect, label, self.config.style.label_color);
    }

    fn draw_back_button(
        &self,
        framebuffer: &mut Framebuffer,
        rect: Rect,
        label: &str,
        pressed: bool,
    ) {
        let opacity = if pressed {
            self.config.style.pressed_opacity
        } else {
            self.config.style.opacity
        };
        fill_rounded_rect_blended(framebuffer, rect, self.config.style.control_color, opacity);
        draw_label(framebuffer, rect, label, self.config.style.label_color);
    }

    fn draw_dpad(&self, framebuffer: &mut Framebuffer, rect: Rect, actions: DirectionalActions) {
        let third_w = (rect.width / 3).max(1);
        let third_h = (rect.height / 3).max(1);
        let center_x = rect
            .x
            .saturating_add(i32::try_from(third_w).unwrap_or(i32::MAX));
        let center_y = rect
            .y
            .saturating_add(i32::try_from(third_h).unwrap_or(i32::MAX));

        let zones = [
            (
                actions.up,
                Rect {
                    x: center_x,
                    y: rect.y,
                    width: third_w,
                    height: third_h.saturating_mul(2),
                },
                "^",
            ),
            (
                actions.down,
                Rect {
                    x: center_x,
                    y: center_y,
                    width: third_w,
                    height: rect.height.saturating_sub(third_h),
                },
                "V",
            ),
            (
                actions.left,
                Rect {
                    x: rect.x,
                    y: center_y,
                    width: third_w.saturating_mul(2),
                    height: third_h,
                },
                "<",
            ),
            (
                actions.right,
                Rect {
                    x: center_x,
                    y: center_y,
                    width: rect.width.saturating_sub(third_w),
                    height: third_h,
                },
                ">",
            ),
        ];

        for (action, zone, label) in zones {
            let opacity = if self.held_actions.contains(&action) {
                self.config.style.pressed_opacity
            } else {
                self.config.style.opacity
            };
            fill_rounded_rect_blended(framebuffer, zone, self.config.style.control_color, opacity);
            draw_label(framebuffer, zone, label, self.config.style.label_color);
        }
    }

    fn draw_virtual_stick(&self, framebuffer: &mut Framebuffer, rect: Rect) {
        let center = rect_center(rect);
        let radius = rect.width.min(rect.height) / 2;

        fill_circle_blended(
            framebuffer,
            center.0,
            center.1,
            radius,
            self.config.style.control_color,
            self.config.style.opacity,
        );

        let movement_owner = self
            .state
            .owner_by_control
            .get(&TouchControlId::Movement)
            .copied();

        let knob_center = movement_owner
            .and_then(|touch_id| self.state.positions.get(&touch_id).copied())
            .map(|position| clamp_stick_knob(center, position, radius))
            .unwrap_or(center);

        let knob_radius = (radius / 3).max(1);
        let opacity = if movement_owner.is_some() {
            self.config.style.pressed_opacity
        } else {
            self.config.style.opacity
        };

        fill_circle_blended(
            framebuffer,
            knob_center.0,
            knob_center.1,
            knob_radius,
            self.config.style.border_color,
            opacity,
        );
    }
}

fn resolve_movement_actions(
    movement: ResolvedMovementControl,
    position: (i32, i32),
) -> HashSet<ActionId> {
    let mut held = HashSet::new();

    match movement {
        ResolvedMovementControl::DPad {
            actions,
            direction_mode,
            visual_rect,
            ..
        } => {
            if !visual_rect.contains(position) {
                return held;
            }

            let center = rect_center(visual_rect);
            let dx = i64::from(position.0) - i64::from(center.0);
            let dy = i64::from(position.1) - i64::from(center.1);
            let neutral_x = i64::from(visual_rect.width / 6);
            let neutral_y = i64::from(visual_rect.height / 6);
            resolve_directional(
                &mut held,
                actions,
                direction_mode,
                dx,
                dy,
                neutral_x,
                neutral_y,
            );
        }
        ResolvedMovementControl::VirtualStick {
            actions,
            direction_mode,
            dead_zone_ratio,
            visual_rect,
            ..
        } => {
            let center = rect_center(visual_rect);
            let dx = i64::from(position.0) - i64::from(center.0);
            let dy = i64::from(position.1) - i64::from(center.1);
            let radius = f64::from(visual_rect.width.min(visual_rect.height)) / 2.0;
            if radius <= 0.0 {
                return held;
            }

            let dx_f = dx as f64;
            let dy_f = dy as f64;
            let distance = (dx_f * dx_f + dy_f * dy_f).sqrt();
            if distance < radius * f64::from(dead_zone_ratio) {
                return held;
            }

            let axis_threshold = (radius * f64::from(dead_zone_ratio)).round() as i64;
            resolve_directional(
                &mut held,
                actions,
                direction_mode,
                dx,
                dy,
                axis_threshold,
                axis_threshold,
            );
        }
    }

    held
}

fn resolve_directional(
    held: &mut HashSet<ActionId>,
    actions: DirectionalActions,
    mode: DirectionMode,
    dx: i64,
    dy: i64,
    threshold_x: i64,
    threshold_y: i64,
) {
    match mode {
        DirectionMode::FourWay => {
            if dx.abs() <= threshold_x && dy.abs() <= threshold_y {
                return;
            }

            if dx.abs() >= dy.abs() {
                held.insert(if dx < 0 { actions.left } else { actions.right });
            } else {
                held.insert(if dy < 0 { actions.up } else { actions.down });
            }
        }
        DirectionMode::EightWay => {
            if dx.abs() > threshold_x {
                held.insert(if dx < 0 { actions.left } else { actions.right });
            }
            if dy.abs() > threshold_y {
                held.insert(if dy < 0 { actions.up } else { actions.down });
            }
        }
    }
}

fn rect_center(rect: Rect) -> (i32, i32) {
    (
        rect.x
            .saturating_add(i32::try_from(rect.width / 2).unwrap_or(i32::MAX)),
        rect.y
            .saturating_add(i32::try_from(rect.height / 2).unwrap_or(i32::MAX)),
    )
}

fn inset_rect(rect: Rect, inset: u32) -> Option<Rect> {
    let double = inset.checked_mul(2)?;
    if rect.width <= double || rect.height <= double {
        return None;
    }

    Some(Rect {
        x: rect
            .x
            .saturating_add(i32::try_from(inset).unwrap_or(i32::MAX)),
        y: rect
            .y
            .saturating_add(i32::try_from(inset).unwrap_or(i32::MAX)),
        width: rect.width - double,
        height: rect.height - double,
    })
}

fn clamp_stick_knob(center: (i32, i32), position: (i32, i32), radius: u32) -> (i32, i32) {
    let dx = (i64::from(position.0) - i64::from(center.0)) as f64;
    let dy = (i64::from(position.1) - i64::from(center.1)) as f64;
    let distance = (dx * dx + dy * dy).sqrt();
    let max_distance = f64::from(radius) * 0.58;

    if distance <= max_distance || distance <= f64::EPSILON {
        return position;
    }

    let scale = max_distance / distance;
    (
        center.0.saturating_add((dx * scale).round() as i32),
        center.1.saturating_add((dy * scale).round() as i32),
    )
}

fn draw_label(framebuffer: &mut Framebuffer, rect: Rect, label: &str, color: Pixel) {
    let scale = if rect.height >= 42 { 2 } else { 1 };
    let (width, height) = Framebuffer::text_size(label, scale);
    let x = i64::from(rect.x) + (i64::from(rect.width) - i64::from(width)) / 2;
    let y = i64::from(rect.y) + (i64::from(rect.height) - i64::from(height)) / 2;
    framebuffer.draw_text_scaled(x as i32, y as i32, label, scale, color);
}

fn fill_circle_blended(
    framebuffer: &mut Framebuffer,
    center_x: i32,
    center_y: i32,
    radius: u32,
    color: Pixel,
    opacity: f32,
) {
    let radius_i64 = i64::from(radius);
    let radius_squared = radius_i64 * radius_i64;

    for dy in -radius_i64..=radius_i64 {
        for dx in -radius_i64..=radius_i64 {
            if dx * dx + dy * dy > radius_squared {
                continue;
            }
            blend_pixel(
                framebuffer,
                i64::from(center_x) + dx,
                i64::from(center_y) + dy,
                color,
                opacity,
            );
        }
    }
}

fn fill_rounded_rect_blended(
    framebuffer: &mut Framebuffer,
    rect: Rect,
    color: Pixel,
    opacity: f32,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let radius = (rect.width.min(rect.height) / 4).max(1);
    let max_x = i64::from(rect.width) - 1;
    let max_y = i64::from(rect.height) - 1;
    let radius_i64 = i64::from(radius);

    for local_y in 0..i64::from(rect.height) {
        for local_x in 0..i64::from(rect.width) {
            let corner_x = if local_x < radius_i64 {
                radius_i64 - local_x
            } else if local_x > max_x - radius_i64 {
                local_x - (max_x - radius_i64)
            } else {
                0
            };

            let corner_y = if local_y < radius_i64 {
                radius_i64 - local_y
            } else if local_y > max_y - radius_i64 {
                local_y - (max_y - radius_i64)
            } else {
                0
            };

            if corner_x != 0
                && corner_y != 0
                && corner_x * corner_x + corner_y * corner_y > radius_i64 * radius_i64
            {
                continue;
            }

            blend_pixel(
                framebuffer,
                i64::from(rect.x) + local_x,
                i64::from(rect.y) + local_y,
                color,
                opacity,
            );
        }
    }
}

fn blend_pixel(framebuffer: &mut Framebuffer, x: i64, y: i64, color: Pixel, opacity: f32) {
    if x < 0 || y < 0 || x >= i64::from(framebuffer.width()) || y >= i64::from(framebuffer.height())
    {
        return;
    }

    let mut rgba = color.to_rgba8();
    rgba[3] = ((f32::from(rgba[3]) * opacity.clamp(0.0, 1.0)).round() as u16).min(255) as u8;
    framebuffer.blend_rgba8(x as u32, y as u32, &rgba);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ButtonState, Touch};

    const UP: ActionId = ActionId::new("test.up");
    const DOWN: ActionId = ActionId::new("test.down");
    const LEFT: ActionId = ActionId::new("test.left");
    const RIGHT: ActionId = ActionId::new("test.right");
    const A: ActionId = ActionId::new("test.a");
    const B: ActionId = ActionId::new("test.b");
    const BACK: ActionId = ActionId::new("test.back");

    const SURFACE: Size = Size {
        width: 480,
        height: 270,
    };

    fn actions() -> DirectionalActions {
        DirectionalActions::new(UP, DOWN, LEFT, RIGHT)
    }

    fn test_config() -> TouchControlsConfig {
        let mut config = TouchControlsConfig::new(MovementControl::VirtualStick(
            VirtualStickConfig::new(actions()),
        ));
        config.visibility = TouchVisibility::Always;
        config.buttons.a = Some(TouchButtonConfig::new(A, "A"));
        config.buttons.b = Some(TouchButtonConfig::new(B, "B"));
        config.navigation.back = Some(TouchButtonConfig::new(BACK, "BACK"));
        config
    }

    fn input_with(touches: impl IntoIterator<Item = Touch>) -> Input {
        let mut input = Input::default();
        for touch in touches {
            input.push_touch(touch);
        }
        input
    }

    fn started(id: u64, position: (i32, i32)) -> Touch {
        Touch {
            id,
            phase: TouchPhase::Started,
            position: Some(position),
        }
    }

    fn moved(id: u64, position: (i32, i32)) -> Touch {
        Touch {
            id,
            phase: TouchPhase::Moved,
            position: Some(position),
        }
    }

    fn ended(id: u64) -> Touch {
        Touch {
            id,
            phase: TouchPhase::Ended,
            position: None,
        }
    }

    fn update_state(
        touch_controls: &mut TouchControls,
        controls: &mut ControlMap,
        input: &Input,
        action: ActionId,
    ) -> ButtonState {
        touch_controls.update(input, controls);
        controls.update(input);
        controls.action(action)
    }

    #[test]
    fn default_layout_places_movement_left_buttons_right_and_back_top_right() {
        let resolved = resolve_touch_controls(&test_config(), SURFACE).unwrap();
        let movement = resolved.movement.unwrap().hit_rect();
        let a = resolved.button(TouchControlId::FaceA).unwrap().hit_rect;
        let back = resolved.button(TouchControlId::Back).unwrap().hit_rect;

        assert!(movement.x < i32::try_from(SURFACE.width / 2).unwrap());
        assert!(a.x > i32::try_from(SURFACE.width / 2).unwrap());
        assert!(back.x > i32::try_from(SURFACE.width / 2).unwrap());
        assert!(back.y < i32::try_from(SURFACE.height / 2).unwrap());
    }

    #[test]
    fn absent_face_buttons_create_no_geometry() {
        let resolved = resolve_touch_controls(&test_config(), SURFACE).unwrap();
        assert!(resolved.button(TouchControlId::FaceA).is_some());
        assert!(resolved.button(TouchControlId::FaceB).is_some());
        assert!(resolved.button(TouchControlId::FaceX).is_none());
        assert!(resolved.button(TouchControlId::FaceY).is_none());
    }

    #[test]
    fn duplicate_touch_actions_are_rejected() {
        let mut config = test_config();
        config.buttons.b = Some(TouchButtonConfig::new(A, "B"));
        assert_eq!(
            resolve_touch_controls(&config, SURFACE),
            Err(TouchLayoutError::DuplicateAction(A))
        );
    }

    #[test]
    fn overlapping_top_level_hitboxes_are_rejected() {
        let mut config = test_config();
        let placement = config.layout.back;
        config.buttons.a = Some(TouchButtonConfig {
            action: A,
            label: "A",
            placement: Some(placement),
        });

        assert!(matches!(
            resolve_touch_controls(&config, SURFACE),
            Err(TouchLayoutError::Overlap(_, _))
        ));
    }

    #[test]
    fn after_first_touch_reveals_and_activates_in_same_frame() {
        let mut config = test_config();
        config.visibility = TouchVisibility::AfterFirstTouch;
        let mut touch_controls = TouchControls::new(config, SURFACE).unwrap();
        let a = touch_controls
            .resolved()
            .button(TouchControlId::FaceA)
            .unwrap()
            .hit_rect;
        let mut controls = ControlMap::new();

        assert!(!touch_controls.is_visible());
        let state = update_state(
            &mut touch_controls,
            &mut controls,
            &input_with([started(1, rect_center(a))]),
            A,
        );

        assert!(touch_controls.is_visible());
        assert!(state.pressed());
        assert!(state.held());
    }

    #[test]
    fn action_button_keeps_capture_when_finger_moves_outside() {
        let mut touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let a = touch_controls
            .resolved()
            .button(TouchControlId::FaceA)
            .unwrap()
            .hit_rect;
        let mut controls = ControlMap::new();

        assert!(
            update_state(
                &mut touch_controls,
                &mut controls,
                &input_with([started(1, rect_center(a))]),
                A,
            )
            .held()
        );

        assert!(
            update_state(
                &mut touch_controls,
                &mut controls,
                &input_with([moved(1, (0, 0))]),
                A,
            )
            .held()
        );

        let released = update_state(
            &mut touch_controls,
            &mut controls,
            &input_with([ended(1)]),
            A,
        );
        assert!(released.released());
        assert!(!released.held());
    }

    #[test]
    fn second_touch_cannot_steal_owned_button() {
        let mut touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let a = touch_controls
            .resolved()
            .button(TouchControlId::FaceA)
            .unwrap()
            .hit_rect;
        let mut controls = ControlMap::new();

        update_state(
            &mut touch_controls,
            &mut controls,
            &input_with([started(1, rect_center(a)), started(2, rect_center(a))]),
            A,
        );

        assert_eq!(
            touch_controls
                .state
                .owner_by_control
                .get(&TouchControlId::FaceA),
            Some(&1)
        );
    }

    #[test]
    fn virtual_stick_eight_way_can_hold_diagonal_actions() {
        let mut touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let movement = touch_controls.resolved().movement.unwrap().visual_rect();
        let center = rect_center(movement);
        let mut controls = ControlMap::new();
        let diagonal = (
            center.0 + i32::try_from(movement.width / 2).unwrap(),
            center.1 - i32::try_from(movement.height / 2).unwrap(),
        );

        let input = input_with([started(1, diagonal)]);
        touch_controls.update(&input, &mut controls);
        controls.update(&input);

        assert!(controls.action(RIGHT).held());
        assert!(controls.action(UP).held());
        assert!(!controls.action(LEFT).held());
        assert!(!controls.action(DOWN).held());
    }

    #[test]
    fn multitouch_can_hold_movement_and_action_button() {
        let mut touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let movement = touch_controls.resolved().movement.unwrap().visual_rect();
        let center = rect_center(movement);
        let a = touch_controls
            .resolved()
            .button(TouchControlId::FaceA)
            .unwrap()
            .hit_rect;
        let mut controls = ControlMap::new();

        let input = input_with([
            started(
                1,
                (
                    center.0 + i32::try_from(movement.width / 2).unwrap(),
                    center.1,
                ),
            ),
            started(2, rect_center(a)),
        ]);

        touch_controls.update(&input, &mut controls);
        controls.update(&input);

        assert!(controls.action(RIGHT).held());
        assert!(controls.action(A).held());
    }

    #[test]
    fn config_replacement_releases_and_quarantines_existing_touch() {
        let mut touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let a = touch_controls
            .resolved()
            .button(TouchControlId::FaceA)
            .unwrap()
            .hit_rect;
        let mut controls = ControlMap::new();

        let first = input_with([started(7, rect_center(a))]);
        touch_controls.update(&first, &mut controls);
        controls.update(&first);
        assert!(controls.action(A).held());

        let mut replacement = test_config();
        replacement.buttons.a = None;
        replacement.buttons.b = Some(TouchButtonConfig::new(A, "B"));
        touch_controls
            .set_config(replacement, SURFACE, &mut controls)
            .unwrap();

        let b = touch_controls
            .resolved()
            .button(TouchControlId::FaceB)
            .unwrap()
            .hit_rect;

        let moved_old_touch = input_with([moved(7, rect_center(b))]);
        touch_controls.update(&moved_old_touch, &mut controls);
        controls.update(&moved_old_touch);

        assert!(!controls.action(A).held());
        assert!(touch_controls.state.quarantined_touches.contains(&7));

        touch_controls.update(&input_with([ended(7)]), &mut controls);

        let new_touch = input_with([started(8, rect_center(b))]);
        touch_controls.update(&new_touch, &mut controls);
        controls.update(&new_touch);
        assert!(controls.action(A).held());
    }

    #[test]
    fn resizing_reanchors_top_right_navigation() {
        let mut touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let before = touch_controls
            .resolved()
            .button(TouchControlId::Back)
            .unwrap()
            .visual_rect;

        touch_controls
            .resize(Size {
                width: 640,
                height: 360,
            })
            .unwrap();

        let after = touch_controls
            .resolved()
            .button(TouchControlId::Back)
            .unwrap()
            .visual_rect;

        assert!(after.x > before.x);
        assert_eq!(after.y, before.y);
    }

    #[test]
    fn draw_changes_pixels_when_controls_are_visible() {
        let touch_controls = TouchControls::new(test_config(), SURFACE).unwrap();
        let mut framebuffer = Framebuffer::new(SURFACE.width, SURFACE.height);
        framebuffer.clear(Pixel::BLACK);
        let before = framebuffer.clone();

        touch_controls.draw(&mut framebuffer);

        assert_ne!(framebuffer, before);
    }
}
