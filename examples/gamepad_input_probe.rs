use gotoo_pixel_engine::{
    EngineConfig, Font, Frame, Framebuffer, Game, GameResult, GamepadAxis, GamepadButton,
    GamepadCapability, GamepadConnectionEvent, GamepadId, GamepadMappingSource, GamepadProfile,
    Image, ImageFilter, ImageFit, Input, Key, Pixel, Rect, run, split_view_layout,
};

const CONTROLLER_PNG: &[u8] = include_bytes!("../assets/debug/gamepad/generic_controller.png");

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const BG: Pixel = Pixel::rgb(7, 11, 17);
const PANEL: Pixel = Pixel::rgb(13, 22, 31);
const FG: Pixel = Pixel::rgb(126, 174, 170);
const BRIGHT: Pixel = Pixel::rgb(218, 235, 225);
const ACTIVE: Pixel = Pixel::rgb(88, 255, 145);
const UNKNOWN: Pixel = Pixel::rgb(245, 190, 70);
const UNAVAILABLE: Pixel = Pixel::rgb(48, 62, 69);

struct Probe {
    profile: GamepadProfile,
    controller_image: Image,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            profile: GamepadProfile::standard(),
            controller_image: Image::decode_png(CONTROLLER_PNG)
                .expect("embedded generic controller PNG must decode"),
        }
    }
}

impl Game for Probe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }
        let decrease = frame.input.key(Key::A).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::LeftShoulder)
                .pressed();
        let increase = frame.input.key(Key::D).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::RightShoulder)
                .pressed();
        let reset = frame.input.key(Key::Space).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::Start)
                .pressed();
        if reset {
            self.profile = GamepadProfile::standard();
        } else if decrease {
            self.profile = self
                .profile
                .with_digital_threshold(self.profile.digital_threshold - 0.05);
        } else if increase {
            self.profile = self
                .profile
                .with_digital_threshold(self.profile.digital_threshold + 0.05);
        }

        let ids = sorted_ids(frame.input);
        for id in ids.iter().copied() {
            frame.set_gamepad_profile(id, self.profile);
        }
        log_events(frame.input, &ids);
        render(
            frame.framebuffer,
            frame.input,
            self.profile,
            &ids,
            &self.controller_image,
        );
        GameResult::Continue
    }
}

const BUTTONS: [(GamepadButton, &str); 25] = [
    (GamepadButton::South, "SOUTH"),
    (GamepadButton::East, "EAST"),
    (GamepadButton::North, "NORTH"),
    (GamepadButton::West, "WEST"),
    (GamepadButton::LeftShoulder, "LB"),
    (GamepadButton::RightShoulder, "RB"),
    (GamepadButton::LeftTrigger, "LT"),
    (GamepadButton::RightTrigger, "RT"),
    (GamepadButton::Start, "START"),
    (GamepadButton::Select, "SELECT"),
    (GamepadButton::Guide, "GUIDE"),
    (GamepadButton::LeftStickPress, "L3"),
    (GamepadButton::RightStickPress, "R3"),
    (GamepadButton::DPadUp, "DPAD UP"),
    (GamepadButton::DPadDown, "DPAD DOWN"),
    (GamepadButton::DPadLeft, "DPAD LEFT"),
    (GamepadButton::DPadRight, "DPAD RIGHT"),
    (GamepadButton::LeftStickUp, "LEFT UP"),
    (GamepadButton::LeftStickDown, "LEFT DOWN"),
    (GamepadButton::LeftStickLeft, "LEFT LEFT"),
    (GamepadButton::LeftStickRight, "LEFT RIGHT"),
    (GamepadButton::RightStickUp, "RIGHT UP"),
    (GamepadButton::RightStickDown, "RIGHT DOWN"),
    (GamepadButton::RightStickLeft, "RIGHT LEFT"),
    (GamepadButton::RightStickRight, "RIGHT RIGHT"),
];

fn sorted_ids(input: &Input) -> Vec<GamepadId> {
    let mut ids = input.gamepad_ids().collect::<Vec<_>>();
    ids.sort_by_key(|id| id.as_usize());
    ids
}

fn panel_assignments(ids: &[GamepadId], bounds: Rect) -> Vec<(GamepadId, Rect)> {
    ids.iter()
        .copied()
        .zip(split_view_layout(bounds, ids.len()))
        .collect()
}

fn log_events(input: &Input, ids: &[GamepadId]) {
    for event in input.gamepad_connection_events() {
        match event {
            GamepadConnectionEvent::Connected(info) => {
                println!("CONNECTED {:?}: {}", info.id, info.name)
            }
            GamepadConnectionEvent::Disconnected(info) => {
                println!("DISCONNECTED {:?}: {}", info.id, info.name)
            }
        }
    }
    for id in ids.iter().copied() {
        for (button, label) in BUTTONS {
            let state = input.gamepad_button(id, button);
            if state.pressed() {
                println!("{:?}: {label} pressed", id);
            }
            if state.released() {
                println!("{:?}: {label} released", id);
            }
        }
    }
}

fn render(
    fb: &mut Framebuffer,
    input: &Input,
    profile: GamepadProfile,
    ids: &[GamepadId],
    controller_image: &Image,
) {
    fb.clear(BG);
    fb.draw_text(6, 5, "GPE GAMEPAD VISUAL PROBE", BRIGHT);
    fb.draw_text(
        470,
        5,
        &format!("THR {:02}%", (profile.digital_threshold * 100.0) as u32),
        FG,
    );
    if ids.is_empty() {
        fb.draw_text_scaled(172, 155, "WAITING FOR GAMEPAD", 2, FG);
    } else {
        let bounds = Rect {
            x: 2,
            y: 19,
            width: WIDTH - 4,
            height: HEIGHT - 35,
        };
        for (id, rect) in panel_assignments(ids, bounds) {
            draw_panel(fb, input, id, inset(rect, 2), controller_image);
        }
        if ids.len() > 4 {
            fb.draw_text(600, 5, &format!("+{}", ids.len() - 4), UNKNOWN);
        }
    }
    fb.draw_text_with_font(
        Font::Mini3x5,
        6,
        351,
        "LB/RB OR A/D THRESHOLD  START/SPACE RESET  ESC QUIT",
        FG,
    );
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NormalizedPoint {
    x: f32,
    y: f32,
}

impl NormalizedPoint {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NormalizedRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl NormalizedRect {
    const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GamepadVisualLayout {
    bounds: Rect,
    left_trigger: NormalizedRect,
    right_trigger: NormalizedRect,
    left_shoulder: NormalizedRect,
    right_shoulder: NormalizedRect,
    left_stick: NormalizedPoint,
    right_stick: NormalizedPoint,
    dpad: NormalizedPoint,
    face: NormalizedPoint,
    select: NormalizedPoint,
    guide: NormalizedPoint,
    start: NormalizedPoint,
}

impl GamepadVisualLayout {
    fn within(panel: Rect, image_width: u32, image_height: u32) -> Self {
        let bounds = contained_rect(controller_destination(panel), image_width, image_height);

        Self {
            bounds,
            left_trigger: NormalizedRect::new(0.12, 0.01, 0.18, 0.13),
            right_trigger: NormalizedRect::new(0.70, 0.01, 0.18, 0.13),
            left_shoulder: NormalizedRect::new(0.08, 0.11, 0.27, 0.10),
            right_shoulder: NormalizedRect::new(0.65, 0.11, 0.27, 0.10),
            left_stick: NormalizedPoint::new(0.27, 0.43),
            right_stick: NormalizedPoint::new(0.63, 0.70),
            dpad: NormalizedPoint::new(0.34, 0.70),
            face: NormalizedPoint::new(0.77, 0.43),
            select: NormalizedPoint::new(0.42, 0.43),
            guide: NormalizedPoint::new(0.50, 0.38),
            start: NormalizedPoint::new(0.58, 0.43),
        }
    }

    fn point(self, point: NormalizedPoint) -> (i32, i32) {
        (
            self.bounds.x + (self.bounds.width as f32 * point.x).round() as i32,
            self.bounds.y + (self.bounds.height as f32 * point.y).round() as i32,
        )
    }

    fn rect(self, rect: NormalizedRect) -> Rect {
        Rect {
            x: self.bounds.x + (self.bounds.width as f32 * rect.x).round() as i32,
            y: self.bounds.y + (self.bounds.height as f32 * rect.y).round() as i32,
            width: (self.bounds.width as f32 * rect.width).round().max(1.0) as u32,
            height: (self.bounds.height as f32 * rect.height).round().max(1.0) as u32,
        }
    }

    fn unit(self, fraction: f32) -> u32 {
        (self.bounds.width.min(self.bounds.height) as f32 * fraction)
            .round()
            .max(1.0) as u32
    }
}

fn controller_destination(panel: Rect) -> Rect {
    Rect {
        x: panel.x + 6,
        y: panel.y + 20,
        width: panel.width.saturating_sub(12),
        height: panel.height.saturating_sub(26),
    }
}

fn contained_rect(destination: Rect, source_width: u32, source_height: u32) -> Rect {
    if destination.width == 0
        || destination.height == 0
        || source_width == 0
        || source_height == 0
    {
        return Rect {
            x: destination.x,
            y: destination.y,
            width: 0,
            height: 0,
        };
    }

    let width_limited = u128::from(destination.width) * u128::from(source_height)
        <= u128::from(destination.height) * u128::from(source_width);
    let (width, height) = if width_limited {
        let height = (u128::from(destination.width) * u128::from(source_height)
            / u128::from(source_width)) as u32;
        (
            destination.width,
            height.max(1).min(destination.height),
        )
    } else {
        let width = (u128::from(destination.height) * u128::from(source_width)
            / u128::from(source_height)) as u32;
        (
            width.max(1).min(destination.width),
            destination.height,
        )
    };

    Rect {
        x: destination.x + (destination.width - width) as i32 / 2,
        y: destination.y + (destination.height - height) as i32 / 2,
        width,
        height,
    }
}

fn draw_panel(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    rect: Rect,
    controller_image: &Image,
) {
    fb.fill_rect(rect.x, rect.y, rect.width, rect.height, PANEL);
    fb.draw_rect(rect.x, rect.y, rect.width, rect.height, FG);
    let info = input.gamepad_info(id).expect("panel ID comes from Input");
    let limit = (rect.width.saturating_sub(48) / 4) as usize;
    fb.draw_text_with_font(
        Font::Mini3x5,
        rect.x + 5,
        rect.y + 4,
        &format!("PAD {} {}", id.as_usize(), display_name(&info.name, limit)),
        BRIGHT,
    );
    fb.draw_text_with_font(
        Font::Mini3x5,
        rect.x + 5,
        rect.y + 11,
        &mapping_label(info.mapping_source, info.mapping_name.as_deref()),
        if info.mapping_source == GamepadMappingSource::Unknown {
            UNKNOWN
        } else {
            FG
        },
    );

    let layout = GamepadVisualLayout::within(rect, controller_image.width(), controller_image.height());
    fb.draw_image_fit(
        controller_image,
        controller_destination(rect),
        ImageFit::Contain,
        ImageFilter::Linear,
    );

    draw_triggers(fb, input, id, layout);
    draw_shoulders(fb, input, id, layout);
    draw_stick(
        fb,
        input,
        id,
        layout,
        layout.left_stick,
        (
            (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
            GamepadButton::LeftStickPress,
            "L3",
        ),
    );
    draw_stick(
        fb,
        input,
        id,
        layout,
        layout.right_stick,
        (
            (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
            GamepadButton::RightStickPress,
            "R3",
        ),
    );
    draw_dpad(fb, input, id, layout);
    draw_face_buttons(fb, input, id, layout);
    draw_center_buttons(fb, input, id, layout);
}

fn draw_triggers(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    draw_trigger_overlay(
        fb,
        input,
        id,
        layout.rect(layout.left_trigger),
        "LT",
        GamepadAxis::LeftTrigger,
        GamepadButton::LeftTrigger,
    );
    draw_trigger_overlay(
        fb,
        input,
        id,
        layout.rect(layout.right_trigger),
        "RT",
        GamepadAxis::RightTrigger,
        GamepadButton::RightTrigger,
    );
}

fn draw_trigger_overlay(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    rect: Rect,
    label: &str,
    axis: GamepadAxis,
    button: GamepadButton,
) {
    let analog = input.gamepad_axis_capability(id, axis);
    let digital = input.gamepad_button_capability(id, button);
    let value = input.gamepad_axis(id, axis).clamp(0.0, 1.0);
    let held = input.gamepad_button(id, button).held();
    let capability = merge_capabilities(analog, digital);
    let active = held || (analog == GamepadCapability::Available && value > 0.02);
    let color = capability_color(capability, active);

    fb.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
    if analog == GamepadCapability::Available {
        let progress = (rect.width.saturating_sub(2) as f32 * value).round() as u32;
        if progress > 0 {
            fb.fill_rect(
                rect.x + 1,
                rect.y + rect.height as i32 - 2,
                progress,
                1,
                color,
            );
        }
    } else if held && rect.width > 2 {
        fb.fill_rect(rect.x + 1, rect.y + rect.height as i32 - 2, rect.width - 2, 1, color);
    }
    fb.draw_text_with_font(Font::Mini3x5, rect.x + 2, rect.y + 2, label, color);
}

fn draw_shoulders(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    draw_shoulder_overlay(
        fb,
        input,
        id,
        layout.rect(layout.left_shoulder),
        "LB",
        GamepadButton::LeftShoulder,
    );
    draw_shoulder_overlay(
        fb,
        input,
        id,
        layout.rect(layout.right_shoulder),
        "RB",
        GamepadButton::RightShoulder,
    );
}

fn draw_shoulder_overlay(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    rect: Rect,
    label: &str,
    button: GamepadButton,
) {
    let held = input.gamepad_button(id, button).held();
    let color = button_color(input, id, button);
    fb.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
    if held && rect.width > 2 {
        fb.fill_rect(rect.x + 1, rect.y + 1, rect.width - 2, 1, color);
    }
    fb.draw_text_with_font(
        Font::Mini3x5,
        rect.x + rect.width as i32 / 2 - 4,
        rect.y + 2,
        label,
        color,
    );
}

fn draw_stick(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
    normalized: NormalizedPoint,
    controls: ((GamepadAxis, GamepadAxis), GamepadButton, &str),
) {
    let (axes, press, label) = controls;
    let center = layout.point(normalized);
    let radius = layout.unit(0.055).max(4);
    let cursor_range = radius.saturating_sub(2) as f32;
    let capability = merge_capabilities(
        input.gamepad_axis_capability(id, axes.0),
        input.gamepad_axis_capability(id, axes.1),
    );
    let pressed = input.gamepad_button(id, press).held();
    let axis_x = input.gamepad_axis(id, axes.0);
    let axis_y = input.gamepad_axis(id, axes.1);
    let moved = axis_x.abs() > 0.02 || axis_y.abs() > 0.02;
    let color = capability_color(capability, pressed || moved);

    fb.draw_circle(center.0, center.1, radius + 2, color);
    if pressed {
        fb.draw_circle(center.0, center.1, radius, ACTIVE);
    }
    fb.fill_circle(
        center.0 + (axis_x * cursor_range).round() as i32,
        center.1 - (axis_y * cursor_range).round() as i32,
        layout.unit(0.012).max(1),
        color,
    );
    fb.draw_text_with_font(
        Font::Mini3x5,
        center.0 - radius as i32,
        center.1 + radius as i32 + 4,
        label,
        color,
    );
}

fn draw_dpad(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    let (x, y) = layout.point(layout.dpad);
    let offset = layout.unit(0.055).max(5) as i32;
    let marker = layout.unit(0.018).max(2);
    for (center, button) in [
        ((x, y - offset), GamepadButton::DPadUp),
        ((x, y + offset), GamepadButton::DPadDown),
        ((x - offset, y), GamepadButton::DPadLeft),
        ((x + offset, y), GamepadButton::DPadRight),
    ] {
        draw_control_marker(fb, input, id, center, marker, button);
    }
}

fn draw_face_buttons(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
) {
    let (x, y) = layout.point(layout.face);
    let offset = layout.unit(0.055).max(6) as i32;
    let radius = layout.unit(0.024).max(3);
    for (center, button, label) in [
        ((x, y - offset), GamepadButton::North, "N"),
        ((x, y + offset), GamepadButton::South, "S"),
        ((x + offset, y), GamepadButton::East, "E"),
        ((x - offset, y), GamepadButton::West, "W"),
    ] {
        draw_labeled_marker(fb, input, id, center, radius, label, button);
    }
}

fn draw_center_buttons(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
) {
    let radius = layout.unit(0.018).max(2);
    draw_labeled_marker(
        fb,
        input,
        id,
        layout.point(layout.select),
        radius,
        "-",
        GamepadButton::Select,
    );
    draw_labeled_marker(
        fb,
        input,
        id,
        layout.point(layout.guide),
        radius + 1,
        "G",
        GamepadButton::Guide,
    );
    draw_labeled_marker(
        fb,
        input,
        id,
        layout.point(layout.start),
        radius,
        "+",
        GamepadButton::Start,
    );
}

fn draw_control_marker(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    center: (i32, i32),
    radius: u32,
    button: GamepadButton,
) {
    let held = input.gamepad_button(id, button).held();
    let color = button_color(input, id, button);
    fb.draw_circle(center.0, center.1, radius, color);
    if held {
        fb.fill_circle(center.0, center.1, radius.saturating_sub(1).max(1), color);
    }
}

fn draw_labeled_marker(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    center: (i32, i32),
    radius: u32,
    label: &str,
    button: GamepadButton,
) {
    draw_control_marker(fb, input, id, center, radius, button);
    let color = button_color(input, id, button);
    fb.draw_text_with_font(
        Font::Mini3x5,
        center.0 - 2,
        center.1 - 2,
        label,
        color,
    );
}

fn button_color(input: &Input, id: GamepadId, button: GamepadButton) -> Pixel {
    capability_color(
        input.gamepad_button_capability(id, button),
        input.gamepad_button(id, button).held(),
    )
}

fn capability_color(capability: GamepadCapability, active: bool) -> Pixel {
    match (capability, active) {
        (GamepadCapability::Available, true) | (GamepadCapability::Unknown, true) => ACTIVE,
        (GamepadCapability::Available, false) => BRIGHT,
        (GamepadCapability::Unavailable, _) => UNAVAILABLE,
        (GamepadCapability::Unknown, false) => UNKNOWN,
    }
}

fn merge_capabilities(a: GamepadCapability, b: GamepadCapability) -> GamepadCapability {
    if a == GamepadCapability::Available || b == GamepadCapability::Available {
        GamepadCapability::Available
    } else if a == GamepadCapability::Unknown || b == GamepadCapability::Unknown {
        GamepadCapability::Unknown
    } else {
        GamepadCapability::Unavailable
    }
}

fn mapping_label(source: GamepadMappingSource, name: Option<&str>) -> String {
    match source {
        GamepadMappingSource::SdlMappings => format!("SDL {}", name.unwrap_or("MAPPING")),
        GamepadMappingSource::Driver => "DRIVER MAPPING".to_owned(),
        GamepadMappingSource::BrowserStandard => "WEB STANDARD".to_owned(),
        GamepadMappingSource::Unknown => "GENERIC / UNKNOWN MAPPING".to_owned(),
    }
}

fn display_name(name: &str, limit: usize) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ' ' {
                c.to_ascii_uppercase()
            } else {
                ' '
            }
        })
        .take(limit.max(8))
        .collect()
}

fn inset(rect: Rect, amount: u32) -> Rect {
    Rect {
        x: rect.x.saturating_add(amount as i32),
        y: rect.y.saturating_add(amount as i32),
        width: rect.width.saturating_sub(amount * 2),
        height: rect.height.saturating_sub(amount * 2),
    }
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE Gamepad Input Probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 2,
            window_height: HEIGHT * 2,
        },
        Probe::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::{CONTROLLER_PNG, GamepadVisualLayout, contained_rect, panel_assignments};
    use gotoo_pixel_engine::{GamepadId, Image, Rect};

    const BOUNDS: Rect = Rect {
        x: 0,
        y: 0,
        width: 640,
        height: 320,
    };

    #[test]
    fn hotplug_reflow_preserves_device_identity() {
        let two = panel_assignments(&[GamepadId::new(0), GamepadId::new(1)], BOUNDS);
        assert_eq!(
            two.iter().map(|entry| entry.0).collect::<Vec<_>>(),
            [GamepadId::new(0), GamepadId::new(1)]
        );
        assert_eq!(
            panel_assignments(&[GamepadId::new(1)], BOUNDS)[0],
            (GamepadId::new(1), BOUNDS)
        );
        assert_eq!(
            panel_assignments(&[GamepadId::new(0), GamepadId::new(1)], BOUNDS),
            two
        );
    }

    #[test]
    fn more_than_four_devices_reports_only_first_four_panels() {
        let ids = (0..6).map(GamepadId::new).collect::<Vec<_>>();
        let assignments = panel_assignments(&ids, BOUNDS);
        assert_eq!(assignments.len(), 4);
        assert_eq!(assignments[3].0, GamepadId::new(3));
    }

    #[test]
    fn reference_controller_png_decodes() {
        let image = Image::decode_png(CONTROLLER_PNG).expect("committed reference PNG must decode");
        assert!(image.width() > 0);
        assert!(image.height() > 0);
    }

    #[test]
    fn contained_controller_rect_preserves_containment() {
        let destination = Rect {
            x: 10,
            y: 20,
            width: 300,
            height: 200,
        };
        let rect = contained_rect(destination, 1600, 1000);
        assert_eq!(rect.width, 300);
        assert_eq!(rect.height, 187);
        assert!(destination.contains((rect.x, rect.y)));
        assert!(destination.contains((
            rect.x + rect.width as i32 - 1,
            rect.y + rect.height as i32 - 1,
        )));
    }

    #[test]
    fn normalized_controller_layout_is_deterministic_and_inside_every_panel_shape() {
        for count in 1..=4 {
            let ids = (0..count).map(GamepadId::new).collect::<Vec<_>>();
            for (_, panel) in panel_assignments(&ids, BOUNDS) {
                let layout = GamepadVisualLayout::within(panel, 1600, 1000);
                assert_eq!(layout, GamepadVisualLayout::within(panel, 1600, 1000));
                assert!(panel.contains((layout.bounds.x, layout.bounds.y)));
                let last_x = layout.bounds.x + layout.bounds.width as i32 - 1;
                let last_y = layout.bounds.y + layout.bounds.height as i32 - 1;
                assert!(panel.contains((last_x, last_y)));
                for point in [
                    layout.left_stick,
                    layout.right_stick,
                    layout.dpad,
                    layout.face,
                    layout.select,
                    layout.guide,
                    layout.start,
                ] {
                    assert!(layout.bounds.contains(layout.point(point)));
                }
            }
        }
    }

    #[test]
    fn controller_layout_scales_instead_of_using_fixed_probe_coordinates() {
        let small = GamepadVisualLayout::within(
            Rect {
                x: 10,
                y: 20,
                width: 240,
                height: 140,
            },
            1600,
            1000,
        );
        let large = GamepadVisualLayout::within(
            Rect {
                x: 30,
                y: 40,
                width: 600,
                height: 300,
            },
            1600,
            1000,
        );

        assert!(large.bounds.width > small.bounds.width);
        assert!(large.bounds.height > small.bounds.height);
        assert_ne!(large.point(large.left_stick), small.point(small.left_stick));
    }
}
