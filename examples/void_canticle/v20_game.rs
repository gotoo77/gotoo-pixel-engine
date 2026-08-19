const VC20_VERSION: &str = "VC2.0";
const VC20_SFX_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/void_canticle/sfx.json"
));
const VC20_REQUIRED_SFX: [&str; 14] = [
    "player_fire",
    "enemy_hit",
    "enemy_destroy",
    "boss_hit",
    "boss_phase",
    "player_hit",
    "cinder_pickup",
    "echo_pickup",
    "canticle_ready",
    "canticle_cast",
    "void_pressure",
    "level_up",
    "mutation",
    "synergy",
];

struct VoidCanticleV20 {
    game: VoidCanticleV19,
    sfx_manifest: gotoo_pixel_engine::SfxManifest,
}

impl VoidCanticleV20 {
    fn new() -> Self {
        let sfx_manifest = gotoo_pixel_engine::SfxManifest::parse(VC20_SFX_MANIFEST)
            .expect("checked-in VC2.0 SFX manifest should parse");
        sfx_manifest
            .require_keys(&VC20_REQUIRED_SFX)
            .expect("checked-in VC2.0 SFX manifest should contain required events");

        Self {
            game: VoidCanticleV19::new(),
            sfx_manifest,
        }
    }

    fn render_lore_hud(&self, framebuffer: &mut Framebuffer) {
        if !self.game.art_can_overlay_game() {
            return;
        }

        let progression = &self.game.v14().progression;
        framebuffer.draw_text(4, 269, "CINDERS > CORE", CINDER);
        framebuffer.fill_rect(103, 303, 73, 10, BG);
        framebuffer.draw_text(
            104,
            305,
            &format!("ECHOES {}", progression.xp),
            XP_ORB_CORE,
        );
    }

    fn render_build_info_overlay(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 97, &format!("VERSION {VC20_VERSION}"), CANTICLE_COLOR);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        let status = if self.sfx_manifest.contains_key("boss_hit") {
            "SFX MANIFEST"
        } else {
            "FEEDBACK PASS"
        };
        framebuffer.draw_text(20, 177, status, CANTICLE_COLOR);
    }
}

impl Game for VoidCanticleV20 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.game.art_can_overlay_game() {
            self.render_lore_hud(frame.framebuffer);
        } else if matches!(&self.game.ui.state, VcPauseState::BuildInfo) {
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

    #[test]
    fn checked_in_sfx_manifest_covers_vc20_events() {
        let manifest = gotoo_pixel_engine::SfxManifest::parse(VC20_SFX_MANIFEST)
            .expect("VC2.0 SFX manifest should parse");
        manifest
            .require_keys(&VC20_REQUIRED_SFX)
            .expect("VC2.0 SFX manifest should contain every required event");

        assert_eq!(
            manifest.path("canticle_ready").unwrap(),
            "assets/void_canticle/sfx/canticle_ready.wav"
        );
        assert_eq!(
            manifest.path("echo_pickup").unwrap(),
            "assets/void_canticle/sfx/echo_pickup.wav"
        );
    }
}
