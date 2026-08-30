use gotoo_pixel_engine::{
    EngineConfig, Font, Frame, Framebuffer, Game, GameResult, GamepadAxis, GamepadButton,
    GamepadCapability, GamepadConnectionEvent, GamepadId, GamepadMappingSource, GamepadProfile,
    Input, Key, Pixel, Rect, run, split_view_layout,
};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const BG: Pixel = Pixel::rgb(7, 11, 17);
const PANEL: Pixel = Pixel::rgb(13, 22, 31);
const BODY: Pixel = Pixel::rgb(39, 50, 59);
const BODY_EDGE: Pixel = Pixel::rgb(91, 116, 124);
const SOCKET: Pixel = Pixel::rgb(9, 15, 21);
const FG: Pixel = Pixel::rgb(126, 174, 170);
const BRIGHT: Pixel = Pixel::rgb(218, 235, 225);
const ACTIVE: Pixel = Pixel::rgb(88, 255, 145);
const UNKNOWN: Pixel = Pixel::rgb(245, 190, 70);
const UNAVAILABLE: Pixel = Pixel::rgb(48, 62, 69);

struct Probe {
    profile: GamepadProfile,
    body_cache: Vec<BodyGeometry>,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            profile: GamepadProfile::standard(),
            body_cache: Vec::new(),
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
            &mut self.body_cache,
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
    body_cache: &mut Vec<BodyGeometry>,
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
            draw_panel(fb, input, id, inset(rect, 2), body_cache);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BodyScanline {
    y: u32,
    start_x: u32,
    width: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BodyGeometry {
    size: (u32, u32),
    scanlines: Vec<BodyScanline>,
}

impl GamepadVisualLayout {
    fn within(panel: Rect) -> Self {
        let available_width = panel.width.saturating_sub(12);
        let available_height = panel.height.saturating_sub(25);
        let mut width = (available_width as f32 * 0.94).round() as u32;
        let mut height = (available_height as f32 * 0.94).round() as u32;
        height = height.min((width as f32 / 1.35).round() as u32);
        width = width.min((height as f32 * 1.90).round() as u32);
        let x = panel.x + (panel.width.saturating_sub(width) / 2) as i32;
        let y = panel.y + 20 + (available_height.saturating_sub(height) / 2) as i32;

        Self {
            bounds: Rect {
                x,
                y,
                width,
                height,
            },
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

fn draw_panel(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    rect: Rect,
    body_cache: &mut Vec<BodyGeometry>,
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

    let layout = GamepadVisualLayout::within(rect);
    draw_triggers(fb, input, id, layout);
    draw_controller_body(fb, layout, body_cache);
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

fn draw_controller_body(
    fb: &mut Framebuffer,
    layout: GamepadVisualLayout,
    cache: &mut Vec<BodyGeometry>,
) {
    let bounds = layout.bounds;
    let size = (bounds.width, bounds.height);
    let geometry_index = cache.iter().position(|geometry| geometry.size == size);
    let geometry_index = geometry_index.unwrap_or_else(|| {
        cache.push(BodyGeometry {
            size,
            scanlines: controller_body_scanlines(bounds.width, bounds.height),
        });
        cache.len() - 1
    });
    for scanline in &cache[geometry_index].scanlines {
        fb.fill_rect(
            bounds.x + scanline.start_x as i32,
            bounds.y + scanline.y as i32,
            scanline.width,
            1,
            BODY,
        );
        fb.draw(
            bounds.x + scanline.start_x as i32,
            bounds.y + scanline.y as i32,
            BODY_EDGE,
        );
        fb.draw(
            bounds.x + scanline.start_x as i32 + scanline.width as i32 - 1,
            bounds.y + scanline.y as i32,
            BODY_EDGE,
        );
    }
}

fn controller_body_scanlines(width: u32, height: u32) -> Vec<BodyScanline> {
    let mut scanlines = Vec::with_capacity(height as usize);
    for offset_y in 0..height {
        let mut run_start = None;
        for offset_x in 0..=width {
            let x = (offset_x as f32 + 0.5) / width.max(1) as f32;
            let y = (offset_y as f32 + 0.5) / height.max(1) as f32;
            let inside = offset_x < width && controller_body_contains(x, y);
            match (run_start, inside) {
                (None, true) => run_start = Some(offset_x),
                (Some(start), false) => {
                    scanlines.push(BodyScanline {
                        y: offset_y,
                        start_x: start,
                        width: offset_x - start,
                    });
                    run_start = None;
                }
                _ => {}
            }
        }
    }
    scanlines
}

fn controller_body_contains(x: f32, y: f32) -> bool {
    let main = ellipse_contains(x, y, 0.50, 0.49, 0.45, 0.34);
    let bridge = (0.19..=0.81).contains(&x) && (0.22..=0.66).contains(&y);
    let left_grip = rotated_grip_contains(x, y, 0.22, -0.276_355_65);
    let right_grip = rotated_grip_contains(x, y, 0.78, 0.276_355_65);
    let bottom_cutout = ellipse_contains(x, y, 0.50, 1.03, 0.25, 0.23);
    (main || bridge || left_grip || right_grip) && !bottom_cutout && y >= 0.14
}

fn ellipse_contains(
    x: f32,
    y: f32,
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
) -> bool {
    let normalized_x = (x - center_x) / radius_x;
    let normalized_y = (y - center_y) / radius_y;
    normalized_x * normalized_x + normalized_y * normalized_y <= 1.0
}

fn rotated_grip_contains(x: f32, y: f32, center_x: f32, sine: f32) -> bool {
    const COSINE: f32 = 0.961_055_46;
    let dx = x - center_x;
    let dy = y - 0.68;
    let rotated_x = dx * COSINE + dy * sine;
    let rotated_y = -dx * sine + dy * COSINE;
    let normalized_x = rotated_x / 0.18;
    let normalized_y = rotated_y / 0.32;
    normalized_x * normalized_x + normalized_y * normalized_y <= 1.0
}

fn draw_triggers(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    draw_trigger(
        fb,
        input,
        id,
        layout.rect(layout.left_trigger),
        "LT",
        GamepadAxis::LeftTrigger,
        GamepadButton::LeftTrigger,
    );
    draw_trigger(
        fb,
        input,
        id,
        layout.rect(layout.right_trigger),
        "RT",
        GamepadAxis::RightTrigger,
        GamepadButton::RightTrigger,
    );
}

fn draw_trigger(
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
    let held = input.gamepad_button(id, button).held();
    let capability = merge_capabilities(analog, digital);
    let color = capability_color(capability, held);
    fb.fill_rect(rect.x, rect.y, rect.width, rect.height, SOCKET);
    if analog == GamepadCapability::Available {
        let fill =
            (rect.width.saturating_sub(2) as f32 * input.gamepad_axis(id, axis)).round() as u32;
        fb.fill_rect(
            rect.x + 1,
            rect.y + 1,
            fill,
            rect.height.saturating_sub(2),
            color,
        );
    } else if held {
        fb.fill_rect(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
            color,
        );
    }
    fb.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
    let value = match analog {
        GamepadCapability::Available => format!("{label} {:.2}", input.gamepad_axis(id, axis)),
        GamepadCapability::Unavailable if digital == GamepadCapability::Available => {
            format!("{label} DIG")
        }
        GamepadCapability::Unavailable => format!("{label} N/A"),
        GamepadCapability::Unknown => format!("{label} ?"),
    };
    fb.draw_text_with_font(
        Font::Mini3x5,
        rect.x + 2,
        rect.y + 2,
        &value,
        if held { PANEL } else { color },
    );
}

fn draw_shoulders(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    draw_shoulder(
        fb,
        input,
        id,
        layout.rect(layout.left_shoulder),
        "LB",
        GamepadButton::LeftShoulder,
    );
    draw_shoulder(
        fb,
        input,
        id,
        layout.rect(layout.right_shoulder),
        "RB",
        GamepadButton::RightShoulder,
    );
}

fn draw_shoulder(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    rect: Rect,
    label: &str,
    button: GamepadButton,
) {
    let held = input.gamepad_button(id, button).held();
    let color = button_color(input, id, button);
    if held {
        fb.fill_rect(rect.x, rect.y, rect.width, rect.height, color);
    } else {
        fb.fill_rect(rect.x, rect.y, rect.width, rect.height, SOCKET);
    }
    fb.draw_rect(rect.x, rect.y, rect.width, rect.height, color);
    fb.draw_text_with_font(
        Font::Mini3x5,
        rect.x + rect.width as i32 / 2 - 4,
        rect.y + 2,
        label,
        if held { PANEL } else { color },
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
    let radius = layout.unit(0.09).max(5);
    let cursor_range = radius.saturating_sub(4) as f32;
    let capability = merge_capabilities(
        input.gamepad_axis_capability(id, axes.0),
        input.gamepad_axis_capability(id, axes.1),
    );
    let pressed = input.gamepad_button(id, press).held();
    let color = capability_color(capability, pressed);
    fb.fill_circle(center.0, center.1, radius + 3, SOCKET);
    fb.draw_circle(center.0, center.1, radius + 3, color);
    fb.draw_circle(
        center.0,
        center.1,
        radius,
        if pressed { ACTIVE } else { BODY_EDGE },
    );
    let axis_x = input.gamepad_axis(id, axes.0);
    let axis_y = input.gamepad_axis(id, axes.1);
    fb.fill_circle(
        center.0 + (axis_x * cursor_range).round() as i32,
        center.1 - (axis_y * cursor_range).round() as i32,
        layout.unit(0.025).max(2),
        color,
    );
    fb.draw_text_with_font(
        Font::Mini3x5,
        center.0 - radius as i32,
        center.1 + radius as i32 + 5,
        &format!("{label} {axis_x:+.1}/{axis_y:+.1}"),
        color,
    );
}

fn draw_dpad(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    let (x, y) = layout.point(layout.dpad);
    let half = layout.unit(0.075).max(5) as i32;
    let arm = (half * 2 / 3).max(3);
    fb.fill_rect(x - arm / 2, y - half, arm as u32, (half * 2) as u32, SOCKET);
    fb.fill_rect(x - half, y - arm / 2, (half * 2) as u32, arm as u32, SOCKET);
    for (rect, button) in [
        (
            (x - arm / 2, y - half, arm as u32, half as u32),
            GamepadButton::DPadUp,
        ),
        (
            (x - arm / 2, y, arm as u32, half as u32),
            GamepadButton::DPadDown,
        ),
        (
            (x - half, y - arm / 2, half as u32, arm as u32),
            GamepadButton::DPadLeft,
        ),
        (
            (x, y - arm / 2, half as u32, arm as u32),
            GamepadButton::DPadRight,
        ),
    ] {
        let color = button_color(input, id, button);
        if input.gamepad_button(id, button).held() {
            fb.fill_rect(rect.0, rect.1, rect.2, rect.3, color);
        }
        fb.draw_rect(rect.0, rect.1, rect.2, rect.3, color);
    }
}

fn draw_face_buttons(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
) {
    let (x, y) = layout.point(layout.face);
    let offset = layout.unit(0.075).max(7) as i32;
    let radius = layout.unit(0.038).max(4);
    for (center, button, label) in [
        ((x, y - offset), GamepadButton::North, "N"),
        ((x, y + offset), GamepadButton::South, "S"),
        ((x + offset, y), GamepadButton::East, "E"),
        ((x - offset, y), GamepadButton::West, "W"),
    ] {
        draw_round_button(fb, input, id, center, radius, label, button);
    }
}

fn draw_center_buttons(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
) {
    let radius = layout.unit(0.025).max(3);
    draw_round_button(
        fb,
        input,
        id,
        layout.point(layout.select),
        radius,
        "-",
        GamepadButton::Select,
    );
    draw_round_button(
        fb,
        input,
        id,
        layout.point(layout.guide),
        radius + 2,
        "G",
        GamepadButton::Guide,
    );
    draw_round_button(
        fb,
        input,
        id,
        layout.point(layout.start),
        radius,
        "+",
        GamepadButton::Start,
    );
}

fn draw_round_button(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    center: (i32, i32),
    radius: u32,
    label: &str,
    button: GamepadButton,
) {
    let held = input.gamepad_button(id, button).held();
    let color = button_color(input, id, button);
    if held {
        fb.fill_circle(center.0, center.1, radius, color);
    } else {
        fb.fill_circle(center.0, center.1, radius, SOCKET);
    }
    fb.draw_circle(center.0, center.1, radius, color);
    fb.draw_text_with_font(
        Font::Mini3x5,
        center.0 - 2,
        center.1 - 2,
        label,
        if held { PANEL } else { color },
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
    use super::{GamepadVisualLayout, panel_assignments};
    use gotoo_pixel_engine::{GamepadId, Rect};
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
    fn normalized_controller_layout_is_deterministic_and_inside_every_panel_shape() {
        for count in 1..=4 {
            let ids = (0..count).map(GamepadId::new).collect::<Vec<_>>();
            for (_, panel) in panel_assignments(&ids, BOUNDS) {
                let layout = GamepadVisualLayout::within(panel);
                assert_eq!(layout, GamepadVisualLayout::within(panel));
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
        let small = GamepadVisualLayout::within(Rect {
            x: 10,
            y: 20,
            width: 240,
            height: 140,
        });
        let large = GamepadVisualLayout::within(Rect {
            x: 30,
            y: 40,
            width: 600,
            height: 300,
        });

        assert!(large.bounds.width > small.bounds.width);
        assert!(large.bounds.height > small.bounds.height);
        assert_ne!(large.point(large.left_stick), small.point(small.left_stick));
    }
}
