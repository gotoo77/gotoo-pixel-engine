const VC19_VERSION: &str = "VC1.9";

// VC1.9 used to own a PNG-backed enemy overlay. The current VC2.7 pipeline
// owns every combat sprite in presentation space, so this historical layer is
// now only a compatibility wrapper around the V17 pause/runtime state.
struct VoidCanticleV19 {
    ui: VoidCanticlePauseV17,
}

impl VoidCanticleV19 {
    fn new() -> Self {
        Self {
            ui: VoidCanticlePauseV17::new(VoidCanticleV17::new()),
        }
    }

    fn base(&self) -> &VoidCanticleGame {
        self.ui.game.base()
    }

    fn v14(&self) -> &VoidCanticleV14 {
        self.ui.game.v14()
    }

    fn v12(&self) -> &VoidCanticleV12 {
        &self.v14().progression.combat
    }

    // Kept as a gameplay-state predicate because V20 still uses it to decide
    // when its defense model is active. It no longer implies any V19 art pass.
    fn art_can_overlay_game(&self) -> bool {
        if !matches!(&self.ui.state, VcPauseState::Running) || self.base().game_over {
            return false;
        }

        self.v14().progression.level_choice.is_none() && self.v14().mutation_choice.is_none()
    }

    fn render_build_info_overlay(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 97, &format!("VERSION {VC19_VERSION}"), TEXT);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 177, "LEGACY ART RETIRED", ART_GOLD);
    }
}

impl Game for VoidCanticleV19 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.ui.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if matches!(&self.ui.state, VcPauseState::BuildInfo) {
            self.render_build_info_overlay(frame.framebuffer);
        }

        GameResult::Continue
    }
}

pub fn run_v19_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Void Canticle - Gotoo Pixel Engine".to_string(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV19::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v19_tests {
    use super::*;

    #[test]
    fn vc19_version_is_explicit() {
        assert_eq!(VC19_VERSION, "VC1.9");
    }

    #[test]
    fn v19_no_longer_owns_enemy_art() {
        let game = VoidCanticleV19::new();
        assert!(game.v12().threats.is_empty());
    }
}
