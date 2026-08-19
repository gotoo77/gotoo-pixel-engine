const VC20_VERSION: &str = "VC2.0";

struct VoidCanticleV20 {
    game: VoidCanticleV19,
}

impl VoidCanticleV20 {
    fn new() -> Self {
        Self {
            game: VoidCanticleV19::new(),
        }
    }

    fn render_build_info_overlay(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 97, &format!("VERSION {VC20_VERSION}"), CANTICLE_COLOR);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 177, "FEEDBACK PASS", CANTICLE_COLOR);
    }
}

impl Game for VoidCanticleV20 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if matches!(&self.game.ui.state, VcPauseState::BuildInfo) {
            self.render_build_info_overlay(frame.framebuffer);
        }

        GameResult::Continue
    }
}

pub fn run_v20_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC20_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV20::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v20_tests {
    use super::*;

    #[test]
    fn vc20_version_is_explicit() {
        assert_eq!(VC20_VERSION, "VC2.0");
    }
}
