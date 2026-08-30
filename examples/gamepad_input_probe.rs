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
    controller_layout: ControllerReferenceLayout,
}

impl Default for Probe {
    fn default() -> Self {
        let controller_image = Image::decode_png(CONTROLLER_PNG)
            .expect("embedded generic controller PNG must decode");
        let controller_layout = ControllerReferenceLayout::for_image(
            controller_image.width(),
            controller_image.height(),
        );

        Self {
            profile: GamepadProfile::standard(),
            controller_image,
            controller_layout,
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
            self.controller_layout,
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
    controller_layout: ControllerReferenceLayout,
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
            draw_panel(
                fb,
                input,
                id,
                inset(rect, 2),
                controller_image,
                controller_layout,
            );
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
struct SourcePoint {
    x: f32,
    y: f32,
}

impl SourcePoint {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SourceRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl SourceRect {
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
struct SourceCircle {
    center: SourcePoint,
    radius: f32,
}

// The human validation capture renders the reference image at exactly 786 x 590,
// so calibration coordinates below map one-for-one to pixels in that raster. They
// are converted once to the committed PNG's actual native dimensions.
const CALIBRATION_REFERENCE_WIDTH: f32 = 786.0;
const CALIBRATION_REFERENCE_HEIGHT: f32 = 590.0;
const SHAPE_POINTS: usize = 9;

type SourceShape = [SourcePoint; SHAPE_POINTS];

#[derive(Debug, Clone, Copy, PartialEq)]
struct ControllerReferenceLayout {
    left_stick: SourceCircle,
    right_stick: SourceCircle,
    stick_cursor_range: f32,
    stick_cursor_radius: f32,
    dpad_up: SourceCircle,
    dpad_down: SourceCircle,
    dpad_left: SourceCircle,
    dpad_right: SourceCircle,
    north: SourceCircle,
    south: SourceCircle,
    east: SourceCircle,
    west: SourceCircle,
    select: SourceCircle,
    guide: SourceCircle,
    start: SourceCircle,
    left_trigger: SourceShape,
    right_trigger: SourceShape,
    left_trigger_meter: SourceRect,
    right_trigger_meter: SourceRect,
    left_trigger_label: SourcePoint,
    right_trigger_label: SourcePoint,
    left_shoulder: SourceShape,
    right_shoulder: SourceShape,
    left_shoulder_label: SourcePoint,
    right_shoulder_label: SourcePoint,
}

impl ControllerReferenceLayout {
    fn for_image(image_width: u32, image_height: u32) -> Self {
        let scale_x = image_width as f32 / CALIBRATION_REFERENCE_WIDTH;
        let scale_y = image_height as f32 / CALIBRATION_REFERENCE_HEIGHT;
        let scale = scale_x.min(scale_y);
        let point = |x: f32, y: f32| SourcePoint::new(x * scale_x, y * scale_y);
        let circle = |x: f32, y: f32, radius: f32| SourceCircle {
            center: point(x, y),
            radius: radius * scale,
        };
        let rect = |x: f32, y: f32, width: f32, height: f32| {
            SourceRect::new(x * scale_x, y * scale_y, width * scale_x, height * scale_y)
        };
        let shape = |points: [(f32, f32); SHAPE_POINTS]| {
            points.map(|(x, y)| point(x, y))
        };

        Self {
            left_stick: circle(212.0, 187.0, 27.0),
            right_stick: circle(491.0, 295.0, 27.0),
            stick_cursor_range: 22.0 * scale,
            stick_cursor_radius: 4.0 * scale,
            dpad_up: circle(300.0, 267.0, 10.0),
            dpad_down: circle(300.0, 334.0, 10.0),
            dpad_left: circle(267.0, 300.0, 10.0),
            dpad_right: circle(334.0, 300.0, 10.0),
            north: circle(575.0, 142.0, 18.0),
            south: circle(576.0, 234.0, 18.0),
            east: circle(623.0, 184.0, 18.0),
            west: circle(529.0, 188.0, 18.0),
            select: circle(340.0, 190.0, 11.0),
            guide: circle(394.0, 153.0, 23.0),
            start: circle(447.0, 190.0, 11.0),
            left_trigger: shape([
                (164.0, 91.0),
                (166.0, 82.0),
                (174.0, 73.0),
                (188.0, 66.0),
                (207.0, 60.0),
                (230.0, 56.0),
                (250.0, 56.0),
                (266.0, 58.0),
                (266.0, 91.0),
            ]),
            right_trigger: shape([
                (522.0, 91.0),
                (522.0, 58.0),
                (542.0, 56.0),
                (562.0, 58.0),
                (583.0, 63.0),
                (601.0, 70.0),
                (615.0, 79.0),
                (624.0, 89.0),
                (624.0, 91.0),
            ]),
            left_trigger_meter: rect(170.0, 87.0, 90.0, 3.0),
            right_trigger_meter: rect(528.0, 87.0, 90.0, 3.0),
            left_trigger_label: point(166.0, 58.0),
            right_trigger_label: point(524.0, 58.0),
            left_shoulder: shape([
                (151.0, 111.0),
                (158.0, 103.0),
                (174.0, 96.0),
                (197.0, 92.0),
                (222.0, 92.0),
                (246.0, 96.0),
                (267.0, 103.0),
                (282.0, 112.0),
                (286.0, 119.0),
            ]),
            right_shoulder: shape([
                (502.0, 119.0),
                (506.0, 112.0),
                (521.0, 103.0),
                (542.0, 96.0),
                (566.0, 92.0),
                (591.0, 92.0),
                (614.0, 96.0),
                (630.0, 103.0),
                (637.0, 111.0),
            ]),
            left_shoulder_label: point(211.0, 101.0),
            right_shoulder_label: point(569.0, 101.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SourceTransform {
    image_bounds: Rect,
    source_width: f32,
    source_height: f32,
}

impl SourceTransform {
    fn new(image_bounds: Rect, source_width: u32, source_height: u32) -> Self {
        Self {
            image_bounds,
            source_width: source_width as f32,
            source_height: source_height as f32,
        }
    }

    fn point(self, point: SourcePoint) -> (i32, i32) {
        if self.source_width <= 0.0 || self.source_height <= 0.0 {
            return (self.image_bounds.x, self.image_bounds.y);
        }

        (
            self.image_bounds.x
                + (self.image_bounds.width as f32 * point.x / self.source_width).round() as i32,
            self.image_bounds.y
                + (self.image_bounds.height as f32 * point.y / self.source_height).round() as i32,
        )
    }

    fn rect(self, rect: SourceRect) -> Rect {
        let (x, y) = self.point(SourcePoint::new(rect.x, rect.y));
        if self.source_width <= 0.0 || self.source_height <= 0.0 {
            return Rect {
                x,
                y,
                width: 0,
                height: 0,
            };
        }

        Rect {
            x,
            y,
            width: (self.image_bounds.width as f32 * rect.width / self.source_width)
                .round()
                .max(1.0) as u32,
            height: (self.image_bounds.height as f32 * rect.height / self.source_height)
                .round()
                .max(1.0) as u32,
        }
    }

    fn radius(self, source_radius: f32) -> u32 {
        if self.source_width <= 0.0 || self.source_height <= 0.0 {
            return 1;
        }

        let scale_x = self.image_bounds.width as f32 / self.source_width;
        let scale_y = self.image_bounds.height as f32 / self.source_height;
        (source_radius * scale_x.min(scale_y)).round().max(1.0) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GamepadVisualLayout {
    source: ControllerReferenceLayout,
    transform: SourceTransform,
}

impl GamepadVisualLayout {
    fn within(
        panel: Rect,
        image_width: u32,
        image_height: u32,
        source: ControllerReferenceLayout,
    ) -> Self {
        let image_bounds = contained_rect(controller_destination(panel), image_width, image_height);
        Self {
            source,
            transform: SourceTransform::new(image_bounds, image_width, image_height),
        }
    }

    fn point(self, point: SourcePoint) -> (i32, i32) {
        self.transform.point(point)
    }

    fn rect(self, rect: SourceRect) -> Rect {
        self.transform.rect(rect)
    }

    fn circle(self, circle: SourceCircle) -> ((i32, i32), u32) {
        (self.point(circle.center), self.transform.radius(circle.radius))
    }

    fn source_distance(self, distance: f32) -> u32 {
        self.transform.radius(distance)
    }

    #[cfg(test)]
    fn bounds(self) -> Rect {
        self.transform.image_bounds
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
    if destination.width == 0 || destination.height == 0 || source_width == 0 || source_height == 0
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
        (destination.width, height.max(1).min(destination.height))
    } else {
        let width = (u128::from(destination.height) * u128::from(source_width)
            / u128::from(source_height)) as u32;
        (width.max(1).min(destination.width), destination.height)
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
    controller_layout: ControllerReferenceLayout,
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

    let layout = GamepadVisualLayout::within(
        rect,
        controller_image.width(),
        controller_image.height(),
        controller_layout,
    );
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
        layout.source.left_stick,
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
        layout.source.right_stick,
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

struct TriggerOverlaySpec<'a> {
    shape: &'a [SourcePoint],
    meter: SourceRect,
    label_position: SourcePoint,
    label: &'a str,
    axis: GamepadAxis,
    button: GamepadButton,
}

struct ShoulderOverlaySpec<'a> {
    shape: &'a [SourcePoint],
    label_position: SourcePoint,
    label: &'a str,
    button: GamepadButton,
}

fn draw_triggers(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    draw_trigger_overlay(
        fb,
        input,
        id,
        layout,
        TriggerOverlaySpec {
            shape: &layout.source.left_trigger,
            meter: layout.source.left_trigger_meter,
            label_position: layout.source.left_trigger_label,
            label: "LT",
            axis: GamepadAxis::LeftTrigger,
            button: GamepadButton::LeftTrigger,
        },
    );
    draw_trigger_overlay(
        fb,
        input,
        id,
        layout,
        TriggerOverlaySpec {
            shape: &layout.source.right_trigger,
            meter: layout.source.right_trigger_meter,
            label_position: layout.source.right_trigger_label,
            label: "RT",
            axis: GamepadAxis::RightTrigger,
            button: GamepadButton::RightTrigger,
        },
    );
}

fn draw_trigger_overlay(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
    spec: TriggerOverlaySpec<'_>,
) {
    let analog = input.gamepad_axis_capability(id, spec.axis);
    let digital = input.gamepad_button_capability(id, spec.button);
    let value = input.gamepad_axis(id, spec.axis).clamp(0.0, 1.0);
    let held = input.gamepad_button(id, spec.button).held();
    let capability = merge_capabilities(analog, digital);
    let active = held || (analog == GamepadCapability::Available && value > 0.02);
    let color = capability_color(capability, active);

    draw_source_polygon(fb, layout, spec.shape, color);
    let meter = layout.rect(spec.meter);
    if analog == GamepadCapability::Available {
        let progress = (meter.width as f32 * value).round() as u32;
        if progress > 0 {
            fb.fill_rect(meter.x, meter.y, progress.min(meter.width), 1, color);
        }
    } else if held {
        fb.fill_rect(meter.x, meter.y, meter.width, 1, color);
    }
    let label_position = layout.point(spec.label_position);
    fb.draw_text_with_font(
        Font::Mini3x5,
        label_position.0,
        label_position.1,
        spec.label,
        color,
    );
}

fn draw_shoulders(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    draw_shoulder_overlay(
        fb,
        input,
        id,
        layout,
        ShoulderOverlaySpec {
            shape: &layout.source.left_shoulder,
            label_position: layout.source.left_shoulder_label,
            label: "LB",
            button: GamepadButton::LeftShoulder,
        },
    );
    draw_shoulder_overlay(
        fb,
        input,
        id,
        layout,
        ShoulderOverlaySpec {
            shape: &layout.source.right_shoulder,
            label_position: layout.source.right_shoulder_label,
            label: "RB",
            button: GamepadButton::RightShoulder,
        },
    );
}

fn draw_shoulder_overlay(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
    spec: ShoulderOverlaySpec<'_>,
) {
    let color = button_color(input, id, spec.button);
    draw_source_polygon(fb, layout, spec.shape, color);
    let label_position = layout.point(spec.label_position);
    fb.draw_text_with_font(
        Font::Mini3x5,
        label_position.0,
        label_position.1,
        spec.label,
        color,
    );
}

fn draw_source_polygon(
    fb: &mut Framebuffer,
    layout: GamepadVisualLayout,
    points: &[SourcePoint],
    color: Pixel,
) {
    if points.len() < 2 {
        return;
    }

    for index in 0..points.len() {
        let start = layout.point(points[index]);
        let end = layout.point(points[(index + 1) % points.len()]);
        draw_line_segment(fb, start, end, color);
    }
}

fn draw_line_segment(
    fb: &mut Framebuffer,
    start: (i32, i32),
    end: (i32, i32),
    color: Pixel,
) {
    let (mut x0, mut y0) = start;
    let (x1, y1) = end;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        fb.draw(x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let twice_error = error * 2;
        if twice_error >= dy {
            error += dy;
            x0 += sx;
        }
        if twice_error <= dx {
            error += dx;
            y0 += sy;
        }
    }
}

fn draw_stick(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
    source: SourceCircle,
    controls: ((GamepadAxis, GamepadAxis), GamepadButton, &str),
) {
    let (axes, press, label) = controls;
    let (center, radius) = layout.circle(source);
    let cursor_range = layout.source_distance(layout.source.stick_cursor_range) as f32;
    let capability = merge_capabilities(
        input.gamepad_axis_capability(id, axes.0),
        input.gamepad_axis_capability(id, axes.1),
    );
    let pressed = input.gamepad_button(id, press).held();
    let axis_x = input.gamepad_axis(id, axes.0);
    let axis_y = input.gamepad_axis(id, axes.1);
    let moved = axis_x.abs() > 0.02 || axis_y.abs() > 0.02;
    let color = capability_color(capability, pressed || moved);

    fb.draw_circle(center.0, center.1, radius, color);
    if pressed && radius > 2 {
        fb.draw_circle(center.0, center.1, radius - 2, ACTIVE);
    }
    fb.fill_circle(
        center.0 + (axis_x * cursor_range).round() as i32,
        center.1 - (axis_y * cursor_range).round() as i32,
        layout.source_distance(layout.source.stick_cursor_radius),
        color,
    );
    fb.draw_text_with_font(
        Font::Mini3x5,
        center.0 - radius as i32,
        center.1 + radius as i32 + 2,
        label,
        color,
    );
}

fn draw_dpad(fb: &mut Framebuffer, input: &Input, id: GamepadId, layout: GamepadVisualLayout) {
    for (source, button) in [
        (layout.source.dpad_up, GamepadButton::DPadUp),
        (layout.source.dpad_down, GamepadButton::DPadDown),
        (layout.source.dpad_left, GamepadButton::DPadLeft),
        (layout.source.dpad_right, GamepadButton::DPadRight),
    ] {
        let (center, radius) = layout.circle(source);
        draw_control_marker(fb, input, id, center, radius, button);
    }
}

fn draw_face_buttons(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
) {
    for (source, button, label) in [
        (layout.source.north, GamepadButton::North, "N"),
        (layout.source.south, GamepadButton::South, "S"),
        (layout.source.east, GamepadButton::East, "E"),
        (layout.source.west, GamepadButton::West, "W"),
    ] {
        let (center, radius) = layout.circle(source);
        draw_labeled_marker(fb, input, id, center, radius, label, button);
    }
}

fn draw_center_buttons(
    fb: &mut Framebuffer,
    input: &Input,
    id: GamepadId,
    layout: GamepadVisualLayout,
) {
    for (source, button, label) in [
        (layout.source.select, GamepadButton::Select, "-"),
        (layout.source.guide, GamepadButton::Guide, "G"),
        (layout.source.start, GamepadButton::Start, "+"),
    ] {
        let (center, radius) = layout.circle(source);
        draw_labeled_marker(fb, input, id, center, radius, label, button);
    }
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
    if held && radius > 2 {
        fb.draw_circle(center.0, center.1, radius - 2, ACTIVE);
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
    fb.draw_text_with_font(Font::Mini3x5, center.0 - 2, center.1 - 2, label, color);
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
    use super::{
        CONTROLLER_PNG, ControllerReferenceLayout, GamepadVisualLayout, SourcePoint,
        SourceTransform, contained_rect, panel_assignments,
    };
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
    fn source_projection_maps_image_edges_to_contained_bounds() {
        let bounds = Rect {
            x: 10,
            y: 20,
            width: 400,
            height: 300,
        };
        let transform = SourceTransform::new(bounds, 800, 600);

        assert_eq!(transform.point(SourcePoint::new(0.0, 0.0)), (10, 20));
        assert_eq!(transform.point(SourcePoint::new(800.0, 600.0)), (410, 320));
    }

    #[test]
    fn registered_controls_stay_inside_source_image() {
        let image = Image::decode_png(CONTROLLER_PNG).expect("committed reference PNG must decode");
        let width = image.width();
        let height = image.height();
        let layout = ControllerReferenceLayout::for_image(width, height);

        for circle in [
            layout.left_stick,
            layout.right_stick,
            layout.dpad_up,
            layout.dpad_down,
            layout.dpad_left,
            layout.dpad_right,
            layout.north,
            layout.south,
            layout.east,
            layout.west,
            layout.select,
            layout.guide,
            layout.start,
        ] {
            assert!(circle.center.x - circle.radius >= 0.0);
            assert!(circle.center.y - circle.radius >= 0.0);
            assert!(circle.center.x + circle.radius <= width as f32);
            assert!(circle.center.y + circle.radius <= height as f32);
        }

        for shape in [
            layout.left_trigger,
            layout.right_trigger,
            layout.left_shoulder,
            layout.right_shoulder,
        ] {
            for point in shape {
                assert!(point.x >= 0.0 && point.x <= width as f32);
                assert!(point.y >= 0.0 && point.y <= height as f32);
            }
        }

        for rect in [layout.left_trigger_meter, layout.right_trigger_meter] {
            assert!(rect.x >= 0.0);
            assert!(rect.y >= 0.0);
            assert!(rect.x + rect.width <= width as f32);
            assert!(rect.y + rect.height <= height as f32);
        }
    }

    #[test]
    fn source_registration_is_deterministic_and_inside_every_panel_shape() {
        let image = Image::decode_png(CONTROLLER_PNG).expect("committed reference PNG must decode");
        let width = image.width();
        let height = image.height();
        let source = ControllerReferenceLayout::for_image(width, height);
        for count in 1..=4 {
            let ids = (0..count).map(GamepadId::new).collect::<Vec<_>>();
            for (_, panel) in panel_assignments(&ids, BOUNDS) {
                let layout = GamepadVisualLayout::within(panel, width, height, source);
                assert_eq!(
                    layout,
                    GamepadVisualLayout::within(panel, width, height, source)
                );
                let bounds = layout.bounds();
                assert!(panel.contains((bounds.x, bounds.y)));
                assert!(panel.contains((
                    bounds.x + bounds.width as i32 - 1,
                    bounds.y + bounds.height as i32 - 1,
                )));

                for circle in [
                    source.left_stick,
                    source.right_stick,
                    source.dpad_up,
                    source.dpad_down,
                    source.dpad_left,
                    source.dpad_right,
                    source.north,
                    source.south,
                    source.east,
                    source.west,
                    source.select,
                    source.guide,
                    source.start,
                ] {
                    assert!(bounds.contains(layout.point(circle.center)));
                }

                for shape in [
                    source.left_trigger,
                    source.right_trigger,
                    source.left_shoulder,
                    source.right_shoulder,
                ] {
                    for point in shape {
                        assert!(bounds.contains(layout.point(point)));
                    }
                }

                for rect in [source.left_trigger_meter, source.right_trigger_meter] {
                    let rect = layout.rect(rect);
                    assert!(bounds.contains((rect.x, rect.y)));
                    assert!(bounds.contains((
                        rect.x + rect.width as i32 - 1,
                        rect.y + rect.height as i32 - 1,
                    )));
                }
            }
        }
    }

    #[test]
    fn source_projection_scales_without_changing_relative_position() {
        let source = SourcePoint::new(256.0, 192.0);
        let small = SourceTransform::new(
            Rect {
                x: 10,
                y: 20,
                width: 320,
                height: 240,
            },
            1024,
            768,
        );
        let large = SourceTransform::new(
            Rect {
                x: 30,
                y: 40,
                width: 640,
                height: 480,
            },
            1024,
            768,
        );

        assert_eq!(small.point(source), (90, 80));
        assert_eq!(large.point(source), (190, 160));
    }
}
