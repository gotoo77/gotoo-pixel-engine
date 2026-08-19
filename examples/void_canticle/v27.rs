const VC27_PRESENTATION_VERSION: &str = "VC2.7";

struct VoidCanticleV27DirectPresentation {
    game: VoidCanticleV23Sustain,
    simulation_framebuffer: Framebuffer,
    player_overlay: Framebuffer,
}

impl VoidCanticleV27DirectPresentation {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23Sustain::new(),
            simulation_framebuffer: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            player_overlay: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
        }
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

    fn render_event_announcement(&self, framebuffer: &mut Framebuffer) {
        let v16b = &self.game.game.v20().game.ui.game.combat;

        if v16b.pressure_reveal_timer > 0.0 && v16b.pressure_reveal != VoidPressure::Dormant {
            let (headline, consequence) = pressure_transition_copy(v16b.pressure_reveal);
            vc_visual_announcement(
                framebuffer,
                headline,
                consequence,
                void_pressure_color(v16b.pressure_reveal),
                VOID_LIGHT,
            );
            return;
        }

        if v16b.boss_phase_banner_timer > 0.0
            && let Some(phase) = v16b.boss_phase_banner
        {
            vc_visual_announcement(
                framebuffer,
                "BELLKEEPER",
                bell_phase_name(phase),
                BELL_LIGHT,
                CANTICLE_COLOR,
            );
            return;
        }

        let v15 = &v16b.combat.combat;
        if v15.synergy_banner_timer > 0.0
            && let Some(name) = v15.synergy_banner_name
        {
            vc_visual_announcement(
                framebuffer,
                "SYNERGY",
                name,
                SYNERGY_COLOR,
                SYNERGY_GOLD,
            );
            return;
        }

        if self.game.game.base().canticle_timer > 0.0 {
            vc_visual_announcement(
                framebuffer,
                "FULL WIPE",
                "CANTICLE",
                CANTICLE_COLOR,
                CANTICLE_COLOR,
            );
            return;
        }

        if let Some(attack) = v16b.combat.pending_attack {
            vc_visual_announcement(
                framebuffer,
                void_attack_name(attack.kind),
                "VOID ATTACK",
                void_pressure_color(v16b.combat.pressure),
                VOID_LIGHT,
            );
        }
    }

    fn render_canticle_charge(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let base = self.game.game.base();
        let x = 10 * scale;
        let y = 306 * scale;
        let width = 62 * scale;
        let height = scale;
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let filled = (width as f32 * ratio).round() as u32;
        let color = if base.core_charge >= CORE_MAX {
            CANTICLE_COLOR
        } else {
            CINDER
        };

        framebuffer.fill_rect(x as i32, y as i32, width, height, WRECK_MID);
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(width), height, color);
        }
        for segment in 1..5 {
            let sx = x + width * segment / 5;
            framebuffer.fill_rect(sx as i32, y as i32, scale, height, BG);
        }
    }

    fn render_survival_bars(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let model = self.game.combat_model();
        let hull_cap = self.game.hull_cap().max(1.0);
        let shield_cap = self.game.shield_cap().max(1.0);

        let margin = 8 * scale;
        let width = 72 * scale;
        let height = scale;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(4 * scale);
        let hull_x = margin;
        let shield_x = VC_VISUAL_PRESENTATION_WIDTH
            .saturating_sub(margin)
            .saturating_sub(width);

        vc27_bar(
            framebuffer,
            hull_x as i32,
            y as i32,
            width,
            height,
            model.player_hull,
            hull_cap,
            if model.player_hull_flash_timer > 0.0 {
                DANGER
            } else {
                VC20_HULL
            },
        );
        vc27_bar(
            framebuffer,
            shield_x as i32,
            y as i32,
            width,
            height,
            model.player_shield,
            shield_cap,
            if model.player_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            },
        );
    }

    fn render_player_last(&mut self, framebuffer: &mut Framebuffer) {
        if self.game.combat_model().player_hull <= 0.0 {
            return;
        }

        self.player_overlay.clear(Pixel::TRANSPARENT);
        let focused = self
            .game
            .game
            .game
            .game
            .movement_controls
            .action(FOCUS)
            .held();
        let base = self.game.game.base();
        base.visuals.render_pilgrim(
            &mut self.player_overlay,
            base.player_x.round() as i32,
            base.player_y.round() as i32,
            focused,
            base.invulnerability,
            base.animation_time,
        );
        vc_visual_blit_nearest(
            &self.player_overlay,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            true,
        );
    }

    fn render_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() {
            return;
        }

        self.render_event_announcement(framebuffer);
        self.render_canticle_charge(framebuffer);
        self.render_survival_bars(framebuffer);
        self.render_player_last(framebuffer);
    }
}

impl Game for VoidCanticleV27DirectPresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = {
            let mut simulation_frame = Frame {
                framebuffer: &mut self.simulation_framebuffer,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut *frame.audio,
                surface_size: frame.surface_size,
                viewport: gotoo_pixel_engine::Viewport::new(
                    frame.surface_size,
                    Size {
                        width: FRAMEBUFFER_WIDTH,
                        height: FRAMEBUFFER_HEIGHT,
                    },
                ),
            };
            self.game.update(&mut simulation_frame)
        };
        if result == GameResult::Exit {
            return result;
        }

        let mode = self.visual_mode();

        // Critical VC2.7 rule: never erase parts of the gameplay framebuffer in
        // order to remove HUD. Legacy HUD producers are disabled at their source.
        // The raw simulation is therefore copied intact, including bullets/FX.
        vc_visual_blit_nearest(
            &self.simulation_framebuffer,
            frame.framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );

        if mode == VcVisualMode::Combat {
            self.render_combat_presentation(frame.framebuffer);
        }

        GameResult::Continue
    }
}

fn vc27_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    value: f32,
    max_value: f32,
    fill: Pixel,
) {
    let ratio = (value / max_value.max(1.0)).clamp(0.0, 1.0);
    let filled = (width as f32 * ratio).round() as u32;
    framebuffer.fill_rect(x, y, width, height, WRECK_MID);
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
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
mod v27_tests {
    use super::*;

    #[test]
    fn version_is_explicit() {
        assert_eq!(VC27_PRESENTATION_VERSION, "VC2.7");
    }

    #[test]
    fn bar_only_touches_its_own_row() {
        let mut framebuffer = Framebuffer::new(20, 8);
        framebuffer.clear(Pixel::BLUE);
        vc27_bar(&mut framebuffer, 2, 3, 12, 1, 3.0, 12.0, Pixel::RED);
        assert_eq!(framebuffer.pixel(2, 2), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(2, 4), Some(Pixel::BLUE));
    }
}