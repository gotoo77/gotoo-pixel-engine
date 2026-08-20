const PRESENTATION_VERSION: &str = "VC3.2";
const TITLE_LAUNCH_DURATION: f32 = 0.42;
const TITLE_BACKGROUND_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/assets/title_background_90x160.png"
));
const CREDITS_BACKGROUND_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/assets/credits_background_90x160.png"
));

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
        scale_presentation_rect(self.base_rect)
    }

    fn render(
        self,
        framebuffer: &mut Framebuffer,
        time: f32,
        selected: bool,
        forced_intensity: Option<f32>,
        launch_progress: f32,
    ) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
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

type GameplayPresentation = VoidCanticleV27ChoicePresentation;
type LegacyPresentationFrontState = Vc27FrontState;

struct VoidCanticleApp {
    game: GameplayPresentation,
    title_background: Framebuffer,
    credits_background: Framebuffer,
    screen: FrontScreen,
    title_choice: TitleChoice,
    presentation_time: f32,
    launch_timer: f32,
}

impl VoidCanticleApp {
    fn new() -> Self {
        let title_image = gotoo_pixel_engine::Image::decode_png(TITLE_BACKGROUND_PNG)
            .expect("checked-in Void Canticle title background should decode");
        let credits_image = gotoo_pixel_engine::Image::decode_png(CREDITS_BACKGROUND_PNG)
            .expect("checked-in Void Canticle credits background should decode");
        assert_eq!((title_image.width(), title_image.height()), (90, 160));
        assert_eq!((credits_image.width(), credits_image.height()), (90, 160));
        let mut title_background = Framebuffer::new(90, 160);
        title_background.draw_image(0, 0, &title_image);
        let mut credits_background = Framebuffer::new(90, 160);
        credits_background.draw_image(0, 0, &credits_image);
        let mut game = GameplayPresentation::new();

        game.presentation.front_state = LegacyPresentationFrontState::Run;
        Self {
            game,
            title_background,
            credits_background,
            screen: FrontScreen::Title,
            title_choice: TitleChoice::StartGame,
            presentation_time: 0.0,
            launch_timer: 0.0,
        }
    }

    fn begin_launch(&mut self, _frame: &mut Frame<'_>) {
        self.screen = FrontScreen::TitleLaunch;
        self.launch_timer = 0.0;
    }

    fn render_title(&self, framebuffer: &mut Framebuffer, launch_progress: f32) {
        vc_visual_blit_nearest(
            &self.title_background,
            framebuffer,
            2 * VC_VISUAL_PRESENTATION_SCALE,
            false,
        );

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
            let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
            let scale_i = scale as i32;
            let center_x = (VC_VISUAL_PRESENTATION_WIDTH / 2) as i32;
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
                VC_VISUAL_PRESENTATION_WIDTH as i32 - 11 * scale_i,
                sweep_y,
                Pixel::rgb(235, 244, 255),
            );
        }
    }

    fn render_credits(&self, framebuffer: &mut Framebuffer) {
        vc_visual_blit_nearest(
            &self.credits_background,
            framebuffer,
            2 * VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let scale_i = scale as i32;

        draw_neon_heading(framebuffer, 82 * scale_i, "CREDITS", 2 * scale);
        draw_credit_line(framebuffer, 126 * scale_i, "A GAME BY GOTOO", scale);
        draw_credit_line(
            framebuffer,
            158 * scale_i,
            "BUILT WITH GOTOO PIXEL ENGINE",
            scale,
        );
        draw_credit_line(
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
                self.begin_launch(frame);
            } else if CREDITS_PROMPT.rect().contains(position) {
                self.title_choice = TitleChoice::Credits;
                self.screen = FrontScreen::Credits;
            }
        } else if front_confirm_pressed(frame.input) {
            match self.title_choice {
                TitleChoice::StartGame => self.begin_launch(frame),
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
            self.game.presentation.render_chassis_selection_presentation(frame.framebuffer);
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
}

impl Game for VoidCanticleApp {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.presentation_time += dt;

        match self.screen {
            FrontScreen::Title => self.update_title(frame),
            FrontScreen::TitleLaunch => self.update_title_launch(frame, dt),
            FrontScreen::Credits => self.update_credits(frame),
            FrontScreen::Game => self.game.update(frame),
        }
    }
}

fn scale_presentation_rect(rect: gotoo_pixel_engine::Rect) -> gotoo_pixel_engine::Rect {
    let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
    gotoo_pixel_engine::Rect {
        x: rect.x * scale as i32,
        y: rect.y * scale as i32,
        width: rect.width * scale,
        height: rect.height * scale,
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

fn draw_neon_heading(framebuffer: &mut Framebuffer, y: i32, text: &str, scale: u32) {
    let (width, _) = Framebuffer::text_size(text, scale);
    let x = ((VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(width)) / 2) as i32;
    let pixel = VC_VISUAL_PRESENTATION_SCALE.max(1) as i32;
    framebuffer.draw_text_scaled(x - pixel, y, text, scale, Pixel::rgb(111, 20, 111));
    framebuffer.draw_text_scaled(x + pixel, y, text, scale, Pixel::rgb(24, 112, 132));
    framebuffer.draw_text_scaled(x, y, text, scale, Pixel::rgb(255, 218, 255));
}

fn draw_credit_line(framebuffer: &mut Framebuffer, y: i32, text: &str, scale: u32) {
    vc_visual_draw_centered_text(framebuffer, y, text, scale, Pixel::rgb(236, 218, 246));
}

pub fn run_void_canticle_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {PRESENTATION_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleApp::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod presentation_frontend_tests {
    use super::*;

    #[test]
    fn checked_in_front_backgrounds_match_current_presentation_size() {
        let title = gotoo_pixel_engine::Image::decode_png(TITLE_BACKGROUND_PNG).unwrap();
        let credits = gotoo_pixel_engine::Image::decode_png(CREDITS_BACKGROUND_PNG).unwrap();
        assert_eq!((title.width(), title.height()), (90, 160));
        assert_eq!((credits.width(), credits.height()), (90, 160));
    }

    #[test]
    fn prompts_fit_inside_portrait_frame() {
        for rect in [START_PROMPT.rect(), CREDITS_PROMPT.rect(), BACK_PROMPT.rect()] {
            assert!(rect.x >= 0);
            assert!(rect.y >= 0);
            assert!(rect.x as u32 + rect.width <= VC_VISUAL_PRESENTATION_WIDTH);
            assert!(rect.y as u32 + rect.height <= VC_VISUAL_PRESENTATION_HEIGHT);
        }
    }

    #[test]
    fn fresh_application_starts_on_title_and_bypasses_legacy_title() {
        let app = VoidCanticleApp::new();
        assert_eq!(app.screen, FrontScreen::Title);
        assert_eq!(app.title_choice, TitleChoice::StartGame);
        assert!(matches!(app.game.presentation.front_state, LegacyPresentationFrontState::Run));
    }
}
