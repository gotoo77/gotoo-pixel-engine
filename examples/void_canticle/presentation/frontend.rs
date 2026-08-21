const PRESENTATION_VERSION: &str = "VC3.3";
const TITLE_LAUNCH_DURATION: f32 = 0.42;

const FRONT_WIDTH: u32 = 540;
const FRONT_HEIGHT: u32 = 960;
const FRONT_SCALE: u32 = 3;

const FRONT_ART_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/assets/front_cathedral_540x960.png"
));
const CREDITS_BACKGROUND_BRIGHTNESS: f32 = 0.42;

const START_PROMPT_BASE: gotoo_pixel_engine::Rect = gotoo_pixel_engine::Rect {
    x: 27,
    y: 254,
    width: 126,
    height: 31,
};
const CREDITS_PROMPT_BASE: gotoo_pixel_engine::Rect = gotoo_pixel_engine::Rect {
    x: 57,
    y: 294,
    width: 66,
    height: 18,
};
const BACK_PROMPT_BASE: gotoo_pixel_engine::Rect = gotoo_pixel_engine::Rect {
    x: 51,
    y: 270,
    width: 78,
    height: 28,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontScreen {
    Title,
    TitleLaunch,
    Credits,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitleChoice {
    StartGame,
    Credits,
}

impl TitleChoice {
    fn toggle(self) -> Self {
        match self {
            Self::StartGame => Self::Credits,
            Self::Credits => Self::StartGame,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct NeonPrompt {
    label: &'static str,
    base_rect: gotoo_pixel_engine::Rect,
    text_scale: u32,
}

impl NeonPrompt {
    const fn new(
        label: &'static str,
        base_rect: gotoo_pixel_engine::Rect,
        text_scale: u32,
    ) -> Self {
        Self {
            label,
            base_rect,
            text_scale,
        }
    }

    fn rect(self) -> gotoo_pixel_engine::Rect {
        scale_front_rect(self.base_rect)
    }

    fn render(
        self,
        framebuffer: &mut Framebuffer,
        time: f32,
        selected: bool,
        forced_intensity: Option<f32>,
        launch_progress: f32,
    ) {
        let scale = FRONT_SCALE.max(1);
        let scale_i = scale as i32;
        let rect = self.rect();
        let intensity = forced_intensity.unwrap_or_else(|| {
            if selected {
                neon_prompt_intensity(time)
            } else {
                0.46
            }
        });
        let launch_progress = launch_progress.clamp(0.0, 1.0);
        let glitch = if launch_progress > 0.0 {
            (((launch_progress * 30.0).floor() as i32).rem_euclid(3) - 1) * scale_i
        } else if selected {
            neon_glitch_offset(time, scale)
        } else {
            0
        };

        framebuffer.fill_rect(
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            Pixel::rgb(9, 2, 18),
        );

        let outer = neon_color(255, 25, 222, 0.26 * intensity);
        let magenta = neon_color(255, 35, 215, 0.82 * intensity);
        let cyan = neon_color(39, 218, 255, 0.48 * intensity);
        let core = neon_color(255, 235, 255, 0.55 + 0.45 * intensity);

        let glow = 3 * scale_i;
        framebuffer.draw_rect(
            rect.x - glow,
            rect.y - glow,
            rect.width + 6 * scale,
            rect.height + 6 * scale,
            outer,
        );
        framebuffer.draw_rect(
            rect.x - scale_i,
            rect.y - scale_i,
            rect.width + 2 * scale,
            rect.height + 2 * scale,
            cyan,
        );
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, magenta);

        let cut = 5 * scale_i;
        framebuffer.draw_line(rect.x, rect.y + cut, rect.x + cut, rect.y, core);
        framebuffer.draw_line(
            rect.x + rect.width as i32 - cut - 1,
            rect.y,
            rect.x + rect.width as i32 - 1,
            rect.y + cut,
            core,
        );
        framebuffer.draw_line(
            rect.x,
            rect.y + rect.height as i32 - cut - 1,
            rect.x + cut,
            rect.y + rect.height as i32 - 1,
            core,
        );
        framebuffer.draw_line(
            rect.x + rect.width as i32 - cut - 1,
            rect.y + rect.height as i32 - 1,
            rect.x + rect.width as i32 - 1,
            rect.y + rect.height as i32 - cut - 1,
            core,
        );

        let text_scale = self.text_scale * scale;
        let (text_width, text_height) = Framebuffer::text_size(self.label, text_scale);
        let x = rect.x + ((rect.width.saturating_sub(text_width)) / 2) as i32 + glitch;
        let y = rect.y + ((rect.height.saturating_sub(text_height)) / 2) as i32;

        framebuffer.draw_text_scaled(
            x - 2 * scale_i,
            y,
            self.label,
            text_scale,
            neon_color(255, 20, 201, 0.38 * intensity),
        );
        framebuffer.draw_text_scaled(
            x + 2 * scale_i,
            y,
            self.label,
            text_scale,
            neon_color(42, 220, 255, 0.42 * intensity),
        );
        framebuffer.draw_text_scaled(x, y, self.label, text_scale, core);
    }
}

const START_PROMPT: NeonPrompt = NeonPrompt::new("START GAME", START_PROMPT_BASE, 2);
const CREDITS_PROMPT: NeonPrompt = NeonPrompt::new("CREDITS", CREDITS_PROMPT_BASE, 1);
const BACK_PROMPT: NeonPrompt = NeonPrompt::new("BACK", BACK_PROMPT_BASE, 2);

type GameplayPresentation = VoidCanticleChoicePresentation;

struct VoidCanticleApp {
    game: GameplayPresentation,
    gameplay_framebuffer: Framebuffer,
    title_background: Framebuffer,
    credits_background: Framebuffer,
    screen: FrontScreen,
    title_choice: TitleChoice,
    presentation_time: f32,
    launch_timer: f32,
}

impl VoidCanticleApp {
    fn new() -> Self {
        let front_art = gotoo_pixel_engine::Image::decode_png(FRONT_ART_PNG)
            .expect("checked-in Void Canticle HD front art should decode");
        assert_eq!(
            (front_art.width(), front_art.height()),
            (FRONT_WIDTH, FRONT_HEIGHT)
        );

        let title_framebuffer = front_background(&front_art, 1.0);
        let credits_framebuffer = front_background(&front_art, CREDITS_BACKGROUND_BRIGHTNESS);
        let game = GameplayPresentation::new();

        Self {
            game,
            gameplay_framebuffer: Framebuffer::new(
                VC_VISUAL_PRESENTATION_WIDTH,
                VC_VISUAL_PRESENTATION_HEIGHT,
            ),
            title_background: title_framebuffer,
            credits_background: credits_framebuffer,
            screen: FrontScreen::Title,
            title_choice: TitleChoice::StartGame,
            presentation_time: 0.0,
            launch_timer: 0.0,
        }
    }

    fn begin_launch(&mut self) {
        self.screen = FrontScreen::TitleLaunch;
        self.launch_timer = 0.0;
    }

    fn render_title(&self, framebuffer: &mut Framebuffer, launch_progress: f32) {
        framebuffer.clone_from(&self.title_background);

        START_PROMPT.render(
            framebuffer,
            self.presentation_time,
            self.title_choice == TitleChoice::StartGame,
            (launch_progress > 0.0).then_some(1.0),
            launch_progress,
        );
        if launch_progress <= 0.0 {
            CREDITS_PROMPT.render(
                framebuffer,
                self.presentation_time + 0.37,
                self.title_choice == TitleChoice::Credits,
                None,
                0.0,
            );
        }

        if launch_progress > 0.0 {
            let scale = FRONT_SCALE.max(1);
            let scale_i = scale as i32;
            let center_x = (FRONT_WIDTH / 2) as i32;
            let portal_y = 166 * scale_i;
            let pulse_radius = ((10.0 + launch_progress * 52.0) * scale as f32).round() as u32;
            framebuffer.draw_circle(
                center_x,
                portal_y,
                pulse_radius,
                neon_color(245, 224, 255, 0.55 + launch_progress * 0.45),
            );
            framebuffer.draw_line(
                center_x,
                77 * scale_i,
                center_x,
                (254.0 * scale as f32 - launch_progress * 45.0 * scale as f32).round()
                    as i32,
                neon_color(235, 245, 255, 0.62 + launch_progress * 0.38),
            );

            let sweep_y = ((286.0 - 94.0 * launch_progress) * scale as f32).round() as i32;
            framebuffer.draw_line(
                11 * scale_i,
                sweep_y,
                FRONT_WIDTH as i32 - 11 * scale_i,
                sweep_y,
                Pixel::rgb(235, 244, 255),
            );
        }
    }

    fn render_credits(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clone_from(&self.credits_background);

        let scale = FRONT_SCALE.max(1);
        let scale_i = scale as i32;
        draw_front_neon_heading(framebuffer, 82 * scale_i, "CREDITS", 2 * scale);
        draw_front_credit_line(framebuffer, 126 * scale_i, "A GAME BY GOTOO", scale);
        draw_front_credit_line(
            framebuffer,
            158 * scale_i,
            "BUILT WITH GOTOO PIXEL ENGINE",
            scale,
        );
        draw_front_credit_line(
            framebuffer,
            190 * scale_i,
            "DESIGN / CODE / LORE - GOTOO",
            scale,
        );
        BACK_PROMPT.render(framebuffer, self.presentation_time, true, None, 0.0);
    }

    fn update_title(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if front_back_pressed(frame.input) {
            return GameResult::Exit;
        }

        if front_vertical_navigation_pressed(frame.input) {
            self.title_choice = self.title_choice.toggle();
        }

        if let Some(position) = front_pointer_press(frame) {
            if START_PROMPT.rect().contains(position) {
                self.title_choice = TitleChoice::StartGame;
                self.begin_launch();
            } else if CREDITS_PROMPT.rect().contains(position) {
                self.title_choice = TitleChoice::Credits;
                self.screen = FrontScreen::Credits;
            }
        } else if front_confirm_pressed(frame.input) {
            match self.title_choice {
                TitleChoice::StartGame => self.begin_launch(),
                TitleChoice::Credits => self.screen = FrontScreen::Credits,
            }
        }

        match self.screen {
            FrontScreen::TitleLaunch => self.render_title(frame.framebuffer, 0.001),
            FrontScreen::Credits => self.render_credits(frame.framebuffer),
            _ => self.render_title(frame.framebuffer, 0.0),
        }
        GameResult::Continue
    }

    fn update_title_launch(&mut self, frame: &mut Frame<'_>, dt: f32) -> GameResult {
        self.launch_timer += dt;
        let progress = (self.launch_timer / TITLE_LAUNCH_DURATION).clamp(0.0, 1.0);
        self.render_title(frame.framebuffer, progress);
        if self.launch_timer >= TITLE_LAUNCH_DURATION {
            self.screen = FrontScreen::Game;
        }
        GameResult::Continue
    }

    fn update_credits(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let pointer_back = front_pointer_press(frame)
            .is_some_and(|position| BACK_PROMPT.rect().contains(position));
        if pointer_back || front_confirm_pressed(frame.input) || front_back_pressed(frame.input) {
            self.screen = FrontScreen::Title;
            self.title_choice = TitleChoice::StartGame;
            self.render_title(frame.framebuffer, 0.0);
            return GameResult::Continue;
        }

        self.render_credits(frame.framebuffer);
        GameResult::Continue
    }

    fn update_game(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let gameplay_viewport = gotoo_pixel_engine::Viewport::new(
            frame.surface_size,
            gotoo_pixel_engine::Size {
                width: VC_VISUAL_PRESENTATION_WIDTH,
                height: VC_VISUAL_PRESENTATION_HEIGHT,
            },
        );
        let result = {
            let mut gameplay_frame = Frame {
                framebuffer: &mut self.gameplay_framebuffer,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut *frame.audio,
                surface_size: frame.surface_size,
                viewport: gameplay_viewport,
            };
            self.game.update(&mut gameplay_frame)
        };

        blit_gameplay_to_front(&self.gameplay_framebuffer, frame.framebuffer);
        result
    }
}

impl Game for VoidCanticleApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.presentation_time += dt;

        match self.screen {
            FrontScreen::Title => self.update_title(frame),
            FrontScreen::TitleLaunch => self.update_title_launch(frame, dt),
            FrontScreen::Credits => self.update_credits(frame),
            FrontScreen::Game => self.update_game(frame),
        }
    }
}

fn front_background(image: &gotoo_pixel_engine::Image, brightness: f32) -> Framebuffer {
    debug_assert_eq!(image.width(), FRONT_WIDTH);
    debug_assert_eq!(image.height(), FRONT_HEIGHT);

    let brightness = brightness.clamp(0.0, 1.0);
    let mut framebuffer = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
    for y in 0..FRONT_HEIGHT {
        for x in 0..FRONT_WIDTH {
            let index = ((y * FRONT_WIDTH + x) * 4) as usize;
            let source = &image.as_rgba8()[index..index + 4];
            framebuffer.set_pixel_in_bounds(
                x,
                y,
                Pixel::rgba(
                    (source[0] as f32 * brightness).round() as u8,
                    (source[1] as f32 * brightness).round() as u8,
                    (source[2] as f32 * brightness).round() as u8,
                    source[3],
                ),
            );
        }
    }
    framebuffer
}

fn scale_front_rect(rect: gotoo_pixel_engine::Rect) -> gotoo_pixel_engine::Rect {
    gotoo_pixel_engine::Rect {
        x: rect.x * FRONT_SCALE as i32,
        y: rect.y * FRONT_SCALE as i32,
        width: rect.width * FRONT_SCALE,
        height: rect.height * FRONT_SCALE,
    }
}

fn blit_gameplay_to_front(source: &Framebuffer, destination: &mut Framebuffer) {
    debug_assert_eq!(source.width(), VC_VISUAL_PRESENTATION_WIDTH);
    debug_assert_eq!(source.height(), VC_VISUAL_PRESENTATION_HEIGHT);
    debug_assert_eq!(destination.width(), FRONT_WIDTH);
    debug_assert_eq!(destination.height(), FRONT_HEIGHT);

    let source_rgba = source.as_rgba8();
    for destination_y in 0..FRONT_HEIGHT {
        let source_y = destination_y * source.height() / FRONT_HEIGHT;
        for destination_x in 0..FRONT_WIDTH {
            let source_x = destination_x * source.width() / FRONT_WIDTH;
            let source_index = ((source_y * source.width() + source_x) * 4) as usize;
            destination.set_pixel_in_bounds(
                destination_x,
                destination_y,
                Pixel::rgba(
                    source_rgba[source_index],
                    source_rgba[source_index + 1],
                    source_rgba[source_index + 2],
                    source_rgba[source_index + 3],
                ),
            );
        }
    }
}

fn front_vertical_navigation_pressed(input: &gotoo_pixel_engine::Input) -> bool {
    input.key(Key::Up).pressed()
        || input.key(Key::Down).pressed()
        || input.key(Key::W).pressed()
        || input.key(Key::S).pressed()
        || input.gamepad_button_any(GamepadButton::DPadUp).pressed()
        || input.gamepad_button_any(GamepadButton::DPadDown).pressed()
        || input.gamepad_button_any(GamepadButton::LeftStickUp).pressed()
        || input.gamepad_button_any(GamepadButton::LeftStickDown).pressed()
}

fn front_confirm_pressed(input: &gotoo_pixel_engine::Input) -> bool {
    input.key(Key::Space).pressed() || input.gamepad_button_any(GamepadButton::South).pressed()
}

fn front_back_pressed(input: &gotoo_pixel_engine::Input) -> bool {
    input.key(Key::Escape).pressed() || input.gamepad_button_any(GamepadButton::East).pressed()
}

fn front_pointer_press(frame: &Frame<'_>) -> Option<(i32, i32)> {
    if frame.input.mouse_button(MouseButton::Left).pressed() {
        return frame.input.mouse_position();
    }

    frame
        .input
        .touches()
        .iter()
        .find(|touch| matches!(touch.phase, gotoo_pixel_engine::TouchPhase::Started))
        .and_then(|touch| touch.position)
}

fn neon_prompt_intensity(time: f32) -> f32 {
    let frame = (time.max(0.0) * 30.0).floor() as u32;
    let hash = frame
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223)
        ^ frame.rotate_left(13);
    let pulse = 0.82 + (time * 2.25).sin().abs() * 0.16;
    match hash % 43 {
        0 => 0.12,
        1 => 0.36,
        2 => 0.68,
        _ => pulse.min(1.0),
    }
}

fn neon_glitch_offset(time: f32, scale: u32) -> i32 {
    let frame = (time.max(0.0) * 30.0).floor() as u32;
    let hash = frame.wrapping_mul(2_654_435_761).rotate_left(7);
    if hash.is_multiple_of(113) {
        let offset = scale.max(1) as i32;
        if hash & 1 == 0 { offset } else { -offset }
    } else {
        0
    }
}

fn neon_color(red: u8, green: u8, blue: u8, intensity: f32) -> Pixel {
    let intensity = intensity.clamp(0.0, 1.0);
    Pixel::rgb(
        (red as f32 * intensity).round() as u8,
        (green as f32 * intensity).round() as u8,
        (blue as f32 * intensity).round() as u8,
    )
}

fn draw_front_neon_heading(framebuffer: &mut Framebuffer, y: i32, text: &str, scale: u32) {
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = ((FRONT_WIDTH.saturating_sub(width)) / 2) as i32;
    let pixel = FRONT_SCALE.max(1) as i32;
    framebuffer.draw_text_scaled(x - pixel, y, text, scale, Pixel::rgb(111, 20, 111));
    framebuffer.draw_text_scaled(x + pixel, y, text, scale, Pixel::rgb(24, 112, 132));
    framebuffer.draw_text_scaled(x, y, text, scale, Pixel::rgb(255, 218, 255));
}

fn draw_front_credit_line(framebuffer: &mut Framebuffer, y: i32, text: &str, scale: u32) {
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = ((FRONT_WIDTH.saturating_sub(width)) / 2) as i32;
    framebuffer.draw_text_scaled(x, y, text, scale, Pixel::rgb(236, 218, 246));
}

pub fn run_void_canticle_with_obs_mirror() -> Result<(), EngineError> {
    run(
        EngineConfig {
            title: format!("Void Canticle {PRESENTATION_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRONT_WIDTH,
            framebuffer_height: FRONT_HEIGHT,
            window_width: FRONT_WIDTH,
            window_height: FRONT_HEIGHT,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(VoidCanticleApp::new(), FRONT_WIDTH, FRONT_HEIGHT),
    )
}

#[cfg(test)]
mod presentation_frontend_tests {
    use super::*;

    #[test]
    fn checked_in_front_art_matches_hd_front_size() {
        let art = gotoo_pixel_engine::Image::decode_png(FRONT_ART_PNG).unwrap();
        assert_eq!((art.width(), art.height()), (FRONT_WIDTH, FRONT_HEIGHT));
    }

    #[test]
    fn credits_background_is_derived_from_the_same_front_art() {
        let art = gotoo_pixel_engine::Image::decode_png(FRONT_ART_PNG).unwrap();
        let title = front_background(&art, 1.0);
        let credits = front_background(&art, CREDITS_BACKGROUND_BRIGHTNESS);
        let title_pixel = title.pixel(FRONT_WIDTH as i32 / 2, FRONT_HEIGHT as i32 / 2).unwrap();
        let credits_pixel = credits
            .pixel(FRONT_WIDTH as i32 / 2, FRONT_HEIGHT as i32 / 2)
            .unwrap();
        assert!(credits_pixel.r <= title_pixel.r);
        assert!(credits_pixel.g <= title_pixel.g);
        assert!(credits_pixel.b <= title_pixel.b);
    }

    #[test]
    fn prompts_fit_inside_hd_portrait_frame() {
        for rect in [
            START_PROMPT.rect(),
            CREDITS_PROMPT.rect(),
            BACK_PROMPT.rect(),
        ] {
            assert!(rect.x >= 0);
            assert!(rect.y >= 0);
            assert!(rect.x as u32 + rect.width <= FRONT_WIDTH);
            assert!(rect.y as u32 + rect.height <= FRONT_HEIGHT);
        }
    }

    #[test]
    fn fresh_application_starts_on_hd_title() {
        let app = VoidCanticleApp::new();
        assert_eq!(app.screen, FrontScreen::Title);
        assert_eq!(app.title_choice, TitleChoice::StartGame);
        assert_eq!(app.gameplay_framebuffer.width(), VC_VISUAL_PRESENTATION_WIDTH);
        assert_eq!(app.gameplay_framebuffer.height(), VC_VISUAL_PRESENTATION_HEIGHT);
    }

    #[test]
    fn gameplay_scaler_fills_hd_front_without_changing_source_dimensions() {
        let mut source = Framebuffer::new(
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        );
        source.draw(0, 0, Pixel::RED);
        source.draw(
            VC_VISUAL_PRESENTATION_WIDTH as i32 - 1,
            VC_VISUAL_PRESENTATION_HEIGHT as i32 - 1,
            Pixel::WHITE,
        );
        let mut destination = Framebuffer::new(FRONT_WIDTH, FRONT_HEIGHT);
        blit_gameplay_to_front(&source, &mut destination);

        assert_eq!(destination.pixel(0, 0), Some(Pixel::RED));
        assert_eq!(
            destination.pixel(FRONT_WIDTH as i32 - 1, FRONT_HEIGHT as i32 - 1),
            Some(Pixel::WHITE)
        );
        assert_eq!(source.width(), VC_VISUAL_PRESENTATION_WIDTH);
        assert_eq!(source.height(), VC_VISUAL_PRESENTATION_HEIGHT);
    }
}
