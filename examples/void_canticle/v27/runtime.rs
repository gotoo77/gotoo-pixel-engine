const VC27_TITLE_LAUNCH_DURATION: f32 = 0.38;
const VC27_TITLE_PROMPT_Y: i32 = 226;
const VC27_TITLE_FOOTER_Y: i32 = 292;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Vc27FrontState {
    Title,
    TitleLaunch { timer: f32 },
    Run,
}

fn vc27_advance_front_state(
    state: Vc27FrontState,
    dt: f32,
    launch_pressed: bool,
) -> Vc27FrontState {
    match state {
        Vc27FrontState::Title if launch_pressed => Vc27FrontState::TitleLaunch { timer: 0.0 },
        Vc27FrontState::Title => Vc27FrontState::Title,
        Vc27FrontState::TitleLaunch { timer } => {
            let timer = timer + dt.max(0.0);
            if timer >= VC27_TITLE_LAUNCH_DURATION {
                Vc27FrontState::Run
            } else {
                Vc27FrontState::TitleLaunch { timer }
            }
        }
        Vc27FrontState::Run => Vc27FrontState::Run,
    }
}

fn vc27_title_launch_pressed(input: &gotoo_pixel_engine::Input) -> bool {
    gotoo_pixel_engine::ui::menu_confirm_pressed(input)
        || input
            .mouse_button(gotoo_pixel_engine::MouseButton::Left)
            .pressed()
        || input
            .touches()
            .iter()
            .any(|touch| matches!(touch.phase, gotoo_pixel_engine::TouchPhase::Started))
}

fn vc27_title_prompt_intensity(time: f32) -> f32 {
    let frame = (time.max(0.0) * 30.0).floor() as u32;
    let hash = frame
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223)
        ^ frame.rotate_left(13);
    let pulse = 0.78 + (time * 2.35).sin().abs() * 0.18;
    match hash % 37 {
        0 => 0.22,
        1 => 0.46,
        2 => 0.68,
        _ => pulse.min(1.0),
    }
}

fn vc27_title_glitch_offset(time: f32, scale: u32) -> i32 {
    let frame = (time.max(0.0) * 30.0).floor() as u32;
    let hash = frame.wrapping_mul(2_654_435_761).rotate_left(7);
    if hash % 101 == 0 {
        let offset = scale.max(1) as i32;
        if hash & 1 == 0 {
            offset
        } else {
            -offset
        }
    } else {
        0
    }
}

fn vc27_title_color(red: u8, green: u8, blue: u8, intensity: f32) -> Pixel {
    let intensity = intensity.clamp(0.0, 1.0);
    Pixel::rgb(
        (red as f32 * intensity).round() as u8,
        (green as f32 * intensity).round() as u8,
        (blue as f32 * intensity).round() as u8,
    )
}

fn vc27_title_layout_fits(scale: u32) -> bool {
    let scale = scale.max(1);
    let width = FRAMEBUFFER_WIDTH * scale;
    let height = FRAMEBUFFER_HEIGHT * scale;
    let (logo_width, logo_height) = Framebuffer::text_size("VOID CANTICLE", 2 * scale);
    let (prompt_width, prompt_height) = Framebuffer::text_size("START GAME", 2 * scale);
    let (footer_width, footer_height) = Framebuffer::text_size("SPACE / SOUTH / TAP", scale);

    logo_width <= width
        && prompt_width <= width
        && footer_width <= width
        && 38 * scale + logo_height <= height
        && VC27_TITLE_PROMPT_Y as u32 * scale + prompt_height <= height
        && VC27_TITLE_FOOTER_Y as u32 * scale + footer_height <= height
}

fn vc27_render_title_screen(framebuffer: &mut Framebuffer, time: f32, launch_progress: f32) {
    let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
    let scale_i = scale as i32;
    let width = VC_VISUAL_PRESENTATION_WIDTH;
    let height = VC_VISUAL_PRESENTATION_HEIGHT;
    let center_x = (width / 2) as i32;
    let center_y = 151 * scale_i;

    framebuffer.clear(Pixel::rgb(2, 3, 9));

    let drift = (time * 7.0).floor() as i32;
    for i in 0..30_i32 {
        let x = (i * 53 + 17).rem_euclid(FRAMEBUFFER_WIDTH as i32) * scale_i;
        let y = (i * 97 + 29 + drift).rem_euclid(FRAMEBUFFER_HEIGHT as i32) * scale_i;
        let color = if (i + drift).rem_euclid(11) == 0 {
            Pixel::rgb(105, 160, 210)
        } else {
            Pixel::rgb(35, 46, 70)
        };
        framebuffer.draw(x, y, color);
    }

    framebuffer.draw_rect(
        6 * scale_i,
        6 * scale_i,
        width.saturating_sub(12 * scale),
        height.saturating_sub(12 * scale),
        Pixel::rgb(49, 18, 68),
    );
    framebuffer.draw_rect(
        8 * scale_i,
        8 * scale_i,
        width.saturating_sub(16 * scale),
        height.saturating_sub(16 * scale),
        Pixel::rgb(10, 72, 88),
    );

    let portal_pulse = ((time * 2.1).sin() * 2.0).round() as i32;
    framebuffer.draw_circle(
        center_x,
        center_y,
        (53 + portal_pulse).max(48) as u32 * scale,
        Pixel::rgb(75, 17, 96),
    );
    framebuffer.draw_circle(
        center_x,
        center_y,
        (42 - portal_pulse / 2).max(36) as u32 * scale,
        Pixel::rgb(17, 116, 132),
    );
    framebuffer.draw_circle(
        center_x,
        center_y,
        30 * scale,
        Pixel::rgb(174, 25, 143),
    );
    framebuffer.fill_circle(
        center_x,
        center_y,
        4 * scale,
        Pixel::rgb(232, 220, 255),
    );

    vc_visual_draw_centered_text(
        framebuffer,
        38 * scale_i,
        "VOID CANTICLE",
        2 * scale,
        Pixel::rgb(232, 224, 240),
    );

    let launch_progress = launch_progress.clamp(0.0, 1.0);
    let intensity = if launch_progress > 0.0 {
        1.0
    } else {
        vc27_title_prompt_intensity(time)
    };
    let glitch = if launch_progress > 0.0 {
        (((launch_progress * 24.0).floor() as i32).rem_euclid(3) - 1) * scale_i
    } else {
        vc27_title_glitch_offset(time, scale)
    };
    let prompt_scale = 2 * scale;
    let (prompt_width, _) = Framebuffer::text_size("START GAME", prompt_scale);
    let prompt_x = ((width.saturating_sub(prompt_width)) / 2) as i32 + glitch;
    let prompt_y = VC27_TITLE_PROMPT_Y * scale_i;
    let glow = vc27_title_color(220, 22, 178, 0.48 * intensity);
    let halo = vc27_title_color(45, 210, 255, 0.68 * intensity);
    let core = vc27_title_color(255, 226, 250, 0.42 + 0.58 * intensity);

    framebuffer.draw_text_scaled(
        prompt_x - 2 * scale_i,
        prompt_y,
        "START GAME",
        prompt_scale,
        glow,
    );
    framebuffer.draw_text_scaled(
        prompt_x + 2 * scale_i,
        prompt_y,
        "START GAME",
        prompt_scale,
        glow,
    );
    framebuffer.draw_text_scaled(
        prompt_x,
        prompt_y - scale_i,
        "START GAME",
        prompt_scale,
        halo,
    );
    framebuffer.draw_text_scaled(
        prompt_x,
        prompt_y + scale_i,
        "START GAME",
        prompt_scale,
        halo,
    );
    framebuffer.draw_text_scaled(prompt_x, prompt_y, "START GAME", prompt_scale, core);

    vc_visual_draw_centered_text(
        framebuffer,
        VC27_TITLE_FOOTER_Y * scale_i,
        "SPACE / SOUTH / TAP",
        scale,
        Pixel::rgb(112, 130, 150),
    );

    if launch_progress > 0.0 {
        let sweep_y = ((248.0 - 116.0 * launch_progress) * scale as f32).round() as i32;
        framebuffer.draw_line(
            10 * scale_i,
            sweep_y,
            width as i32 - 10 * scale_i,
            sweep_y,
            Pixel::rgb(220, 244, 255),
        );
    }
}

impl VoidCanticleV27DirectPresentation {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23Sustain::new(),
            legacy_sink: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            clean_background: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            presentation_time: 0.0,
            front_state: Vc27FrontState::Title,
            hit_reactions: Vc27HitReactionState::default(),
            projectile_provenance: Vc27ProjectileProvenance::default(),
        }
    }

    fn chassis_selection_active(&self) -> bool {
        self.game.game.game.game.game.chassis.is_none()
    }

    fn reset_to_fresh_launch(&mut self) {
        self.game = VoidCanticleV23Sustain::new();
        self.presentation_time = 0.0;
        self.front_state = Vc27FrontState::Run;
        self.hit_reactions = Vc27HitReactionState::default();
        self.projectile_provenance = Vc27ProjectileProvenance::default();
    }

    fn render_chassis_selection_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let selector = &self.game.game.game.game.game;
        vc27_render_chassis_showcase(
            &mut self.clean_background,
            framebuffer,
            selector,
            self.presentation_time,
        );
    }

    fn visual_mode(&self) -> VcVisualMode {
        if self.game.choosing_support {
            return VcVisualMode::SupportChoice;
        }

        let v14 = self.game.game.v20().game.v14();
        if v14.progression.level_choice.is_some() {
            return VcVisualMode::LevelChoice;
        }
        if v14.mutation_choice.is_some() {
            return VcVisualMode::MutationChoice;
        }
        if self.game.game.base().game_over {
            return VcVisualMode::Death;
        }

        let stabilized = &self.game.survival_model().game;
        if stabilized.stage_clear_visible() {
            return VcVisualMode::StageClear;
        }
        if !matches!(&stabilized.pause_ui().state, VcPauseState::Running) {
            return VcVisualMode::Pause;
        }

        VcVisualMode::Combat
    }

    fn render_clean_background(&mut self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        let color = if base.canticle_timer > 0.46 {
            BG_CANTICLE
        } else {
            BG
        };
        let scroll = base.scroll;

        self.clean_background.clear(color);
        render_grave_orbit_background(&mut self.clean_background, scroll);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
    }
}

fn vc27_full_restart_completed(
    was_game_over: bool,
    is_game_over: bool,
    was_stage_clear: bool,
    encounter_phase: EncounterPhase,
    pause_restart_selected: bool,
    pause_confirm_pressed: bool,
) -> bool {
    let death_restart = was_game_over && !is_game_over;
    let stage_restart = was_stage_clear && encounter_phase != EncounterPhase::Cleared;
    let pause_restart = pause_restart_selected && pause_confirm_pressed;

    death_restart || stage_restart || pause_restart
}

impl Game for VoidCanticleV27DirectPresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.presentation_time += dt;

        if !matches!(self.front_state, Vc27FrontState::Run) {
            if frame
                .input
                .key(gotoo_pixel_engine::Key::Escape)
                .pressed()
            {
                return GameResult::Exit;
            }

            let launch_pressed = vc27_title_launch_pressed(frame.input);
            self.front_state = vc27_advance_front_state(self.front_state, dt, launch_pressed);

            match self.front_state {
                Vc27FrontState::Title => {
                    vc27_render_title_screen(frame.framebuffer, self.presentation_time, 0.0);
                }
                Vc27FrontState::TitleLaunch { timer } => {
                    vc27_render_title_screen(
                        frame.framebuffer,
                        self.presentation_time,
                        timer / VC27_TITLE_LAUNCH_DURATION,
                    );
                }
                Vc27FrontState::Run => {
                    self.render_chassis_selection_presentation(frame.framebuffer);
                }
            }
            return GameResult::Continue;
        }

        let was_game_over = self.game.game.base().game_over;
        let was_stage_clear = self.game.survival_model().game.stage_clear_visible();
        let pause_restart_selected = {
            let pause = self.game.survival_model().game.pause_ui();
            matches!(&pause.state, VcPauseState::Menu) && pause.menu.selected() == Some(1)
        };
        let hit_snapshot = Vc27HitSnapshot::capture(&self.game);
        let projectile_snapshot = Vc27ProjectileSourceSnapshot::capture(&self.game);

        let result = {
            let mut legacy_audio = Vc27LegacyAttackAudioFilter::new(&mut *frame.audio);
            let mut legacy_frame = Frame {
                framebuffer: &mut self.legacy_sink,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut legacy_audio,
                surface_size: frame.surface_size,
                viewport: gotoo_pixel_engine::Viewport::new(
                    frame.surface_size,
                    Size {
                        width: FRAMEBUFFER_WIDTH,
                        height: FRAMEBUFFER_HEIGHT,
                    },
                ),
            };
            self.game.update(&mut legacy_frame)
        };
        if result == GameResult::Exit {
            return result;
        }

        let pause_confirm_pressed = self
            .game
            .survival_model()
            .game
            .pause_ui()
            .controls
            .action(VC_PAUSE_CONFIRM)
            .pressed();
        if vc27_full_restart_completed(
            was_game_over,
            self.game.game.base().game_over,
            was_stage_clear,
            self.game.game.base().encounter_phase,
            pause_restart_selected,
            pause_confirm_pressed,
        ) {
            self.reset_to_fresh_launch();
            self.render_chassis_selection_presentation(frame.framebuffer);
            return GameResult::Continue;
        }

        self.hit_reactions.update(dt, &hit_snapshot, &self.game);
        let attack_sounds =
            self.projectile_provenance
                .reconcile(dt, &projectile_snapshot, &self.game);
        self.game.play_attack_sounds(frame, &attack_sounds);

        if self.chassis_selection_active() {
            self.render_chassis_selection_presentation(frame.framebuffer);
            return GameResult::Continue;
        }

        let mode = self.visual_mode();
        match mode {
            VcVisualMode::Combat => self.render_combat_presentation(frame.framebuffer),
            VcVisualMode::Death => self.render_death_presentation(frame.framebuffer),
            VcVisualMode::Pause
            | VcVisualMode::LevelChoice
            | VcVisualMode::MutationChoice
            | VcVisualMode::SupportChoice
            | VcVisualMode::StageClear => self.render_clean_modal(frame.framebuffer, mode),
        }

        GameResult::Continue
    }
}

pub fn run_v27_direct_presentation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC27_PRESENTATION_VERSION} Direct HUD - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV27DirectPresentation::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v27_runtime_tests {
    use super::*;

    #[test]
    fn version_is_explicit() {
        assert_eq!(VC27_PRESENTATION_VERSION, "VC3.2");
    }

    #[test]
    fn fresh_process_starts_on_title_screen() {
        let game = VoidCanticleV27DirectPresentation::new();
        assert_eq!(game.front_state, Vc27FrontState::Title);
    }

    #[test]
    fn title_launch_transitions_to_run_after_short_flash() {
        let launch = vc27_advance_front_state(Vc27FrontState::Title, 0.0, true);
        assert_eq!(launch, Vc27FrontState::TitleLaunch { timer: 0.0 });

        let almost =
            vc27_advance_front_state(launch, VC27_TITLE_LAUNCH_DURATION - 0.01, false);
        assert!(matches!(almost, Vc27FrontState::TitleLaunch { .. }));

        let run = vc27_advance_front_state(almost, 0.02, false);
        assert_eq!(run, Vc27FrontState::Run);
    }

    #[test]
    fn title_layout_fits_native_and_web_surfaces() {
        assert!(vc27_title_layout_fits(1));
        assert!(vc27_title_layout_fits(2));
    }

    #[test]
    fn chassis_selection_is_explicit_precombat_state() {
        let game = VoidCanticleV27DirectPresentation::new();
        assert!(game.chassis_selection_active());
    }

    #[test]
    fn every_restart_path_requests_a_fresh_launch() {
        assert!(vc27_full_restart_completed(
            true,
            false,
            false,
            EncounterPhase::Waves,
            false,
            false,
        ));
        assert!(vc27_full_restart_completed(
            false,
            false,
            true,
            EncounterPhase::Waves,
            false,
            false,
        ));
        assert!(vc27_full_restart_completed(
            false,
            false,
            false,
            EncounterPhase::Waves,
            true,
            true,
        ));
        assert!(!vc27_full_restart_completed(
            false,
            false,
            false,
            EncounterPhase::Waves,
            true,
            false,
        ));
    }

    #[test]
    fn fresh_launch_resets_presentation_state_and_returns_to_chassis_select() {
        let mut game = VoidCanticleV27DirectPresentation::new();
        game.front_state = Vc27FrontState::Run;
        game.presentation_time = 42.0;
        game.reset_to_fresh_launch();

        assert_eq!(game.presentation_time, 0.0);
        assert_eq!(game.front_state, Vc27FrontState::Run);
        assert!(game.chassis_selection_active());
    }

    #[test]
    fn simulation_coordinates_map_to_presentation_space() {
        assert_eq!(vc27_present(0.0), 0);
        assert_eq!(vc27_present(90.0), 180);
        assert_eq!(vc27_present(319.5), 639);
    }
}
