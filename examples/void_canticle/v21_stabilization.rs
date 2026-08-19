const VC21_STABLE_VERSION: &str = "VC2.1";
const VC21_PIXEL_VERSION: &str = "VC2-1";
const VC21_AUDIO_KEEPALIVE_SOUND: SoundId = SoundId::new("void_canticle.audio_keepalive");
const VC21_AUDIO_KEEPALIVE_PERIOD: f32 = 0.18;
const VC21_AUDIO_KEEPALIVE_DURATION_SAMPLES: usize = 22_050;
const VC21_SHOT_ESCAPE_Y: f32 = 18.0;
const VC21_SHOT_DESPAWN_Y: f32 = -12.0;

#[derive(Debug, Clone, Copy)]
struct Vc21EscapedShot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    damage: u32,
    radius: u32,
}

impl From<PowerShot> for Vc21EscapedShot {
    fn from(shot: PowerShot) -> Self {
        Self {
            x: shot.x,
            y: shot.y,
            vx: shot.vx,
            vy: shot.vy,
            damage: shot.damage,
            radius: shot.radius,
        }
    }
}

struct VoidCanticleV21Stabilized {
    game: VoidCanticleV21,
    escaped_shots: Vec<Vc21EscapedShot>,
    audio_keepalive_timer: f32,
    stage_clear_seen: bool,
}

impl VoidCanticleV21Stabilized {
    fn new() -> Self {
        let mut game = VoidCanticleV21::new();
        let silence = vec![0_i16; VC21_AUDIO_KEEPALIVE_DURATION_SAMPLES];
        let wav = pcm16_mono_wav(AUDIO_SAMPLE_RATE, &silence)
            .expect("VC2.1 audio keepalive WAV should encode");
        game.base_mut()
            .sounds
            .insert_wav(VC21_AUDIO_KEEPALIVE_SOUND, wav)
            .expect("VC2.1 audio keepalive sound id should be unique");

        Self {
            game,
            escaped_shots: Vec::new(),
            audio_keepalive_timer: 0.0,
            stage_clear_seen: false,
        }
    }

    fn power_game(&self) -> &VoidCanticleV07 {
        &self
            .game
            .combat
            .game
            .ui
            .game
            .combat
            .combat
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
    }

    fn power_game_mut(&mut self) -> &mut VoidCanticleV07 {
        &mut self
            .game
            .combat
            .game
            .ui
            .game
            .combat
            .combat
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
    }

    fn gameplay_running(&self) -> bool {
        self.game.combat.game.art_can_overlay_game()
    }

    fn collect_shots_crossing_hud_cutoff(&mut self, dt: f32) {
        if !self.gameplay_running() {
            return;
        }

        let crossing: Vec<Vc21EscapedShot> = self
            .power_game()
            .power_shots
            .iter()
            .filter(|shot| {
                shot.alive
                    && shot.y > VC21_SHOT_ESCAPE_Y
                    && shot.y + shot.vy * dt <= VC21_SHOT_ESCAPE_Y
            })
            .copied()
            .map(|shot| {
                let mut escaped = Vc21EscapedShot::from(shot);
                escaped.x += escaped.vx * dt;
                escaped.y += escaped.vy * dt;
                escaped
            })
            .collect();
        self.escaped_shots.extend(crossing);
    }

    fn update_escaped_shots(&mut self, dt: f32) {
        if !self.gameplay_running() {
            return;
        }
        for shot in &mut self.escaped_shots {
            shot.x += shot.vx * dt;
            shot.y += shot.vy * dt;
        }
        self.escaped_shots.retain(|shot| {
            shot.y > VC21_SHOT_DESPAWN_Y
                && shot.x > -12.0
                && shot.x < FRAMEBUFFER_WIDTH as f32 + 12.0
        });
    }

    fn render_escaped_shots(&self, framebuffer: &mut Framebuffer) {
        if !self.gameplay_running() {
            return;
        }
        let power_level = self.power_game().power_level;
        for shot in &self.escaped_shots {
            render_power_shot(
                framebuffer,
                PowerShot {
                    x: shot.x,
                    y: shot.y,
                    vx: shot.vx,
                    vy: shot.vy,
                    damage: shot.damage,
                    radius: shot.radius,
                    alive: true,
                },
                power_level,
            );
        }
    }

    fn keep_audio_alive(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.audio_keepalive_timer -= dt;
        if self.audio_keepalive_timer > 0.0 {
            return;
        }
        let _ = self
            .game
            .base_mut()
            .sounds
            .play(frame.audio, VC21_AUDIO_KEEPALIVE_SOUND);
        self.audio_keepalive_timer = VC21_AUDIO_KEEPALIVE_PERIOD;
    }

    fn handle_stage_clear_transition(&mut self) {
        let cleared = self.game.base().encounter_phase == EncounterPhase::Cleared;
        if !cleared {
            self.stage_clear_seen = false;
            return;
        }
        if self.stage_clear_seen {
            return;
        }

        self.stage_clear_seen = true;
        self.escaped_shots.clear();
        self.game.base_mut().player_bullets.clear();
        self.power_game_mut().power_shots.clear();
        self.game.base_mut().enemy_bullets.clear();
    }

    fn render_pixel_safe_version(&self, framebuffer: &mut Framebuffer) {
        if self.gameplay_running() {
            framebuffer.fill_rect(3, 13, 102, 10, BG);
            framebuffer.draw_text(
                4,
                15,
                &format!("GRAVE ORBIT / {VC21_PIXEL_VERSION}"),
                TEXT,
            );
        } else if matches!(&self.game.combat.game.ui.state, VcPauseState::BuildInfo) {
            framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
            framebuffer.draw_text(
                20,
                97,
                &format!("VERSION {VC21_PIXEL_VERSION}"),
                CANTICLE_COLOR,
            );
        }
    }

    fn render_stage_clear(&self, framebuffer: &mut Framebuffer) {
        if self.game.base().encounter_phase != EncounterPhase::Cleared {
            return;
        }

        framebuffer.clear(BG);
        render_grave_orbit_background(framebuffer, self.game.base().scroll);
        framebuffer.fill_rect(12, 55, 156, 206, Pixel::rgb(9, 8, 15));
        framebuffer.draw_rect(12, 55, 156, 206, CANTICLE_COLOR);
        framebuffer.draw_rect(16, 59, 148, 198, ART_GOLD);

        framebuffer.draw_text_scaled(30, 73, "STAGE CLEAR", 2, CANTICLE_COLOR);
        framebuffer.draw_text(54, 98, "GRAVE ORBIT", TEXT);
        framebuffer.draw_text(60, 116, "PATH OPENS", ART_GOLD);

        framebuffer.draw_text(32, 151, "SCORE", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            151,
            &format!("{}", self.game.base().score),
            PILGRIM_CORE,
        );
        framebuffer.draw_text(32, 168, "LEVEL", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            168,
            &format!("{}", self.game.combat.game.v14().progression.level),
            XP_ORB_CORE,
        );
        framebuffer.draw_text(32, 185, "ECHOES", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            185,
            &format!("{}", self.game.combat.game.v14().progression.xp),
            XP_ORB_CORE,
        );
        framebuffer.draw_text(32, 202, "HULL", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            202,
            &format!("{}", self.game.player_hull.round() as u32),
            VC20_HULL,
        );
        framebuffer.draw_text(32, 219, "SHIELD", WRECK_LIGHT);
        framebuffer.draw_text(
            95,
            219,
            &format!("{}", self.game.player_shield.round() as u32),
            VC20_ARMOR_LIGHT,
        );

        framebuffer.draw_text(38, 240, "RUN PHASE COMPLETE", TEXT);
    }
}

impl Game for VoidCanticleV21Stabilized {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.keep_audio_alive(dt, frame);
        self.collect_shots_crossing_hud_cutoff(dt);

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.update_escaped_shots(dt);
        self.handle_stage_clear_transition();
        self.render_escaped_shots(frame.framebuffer);
        self.render_pixel_safe_version(frame.framebuffer);
        self.render_stage_clear(frame.framebuffer);

        GameResult::Continue
    }
}

pub fn run_v21_stabilized_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC21_STABLE_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV21Stabilized::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v21_stabilization_tests {
    use super::*;

    #[test]
    fn framebuffer_version_uses_supported_punctuation() {
        assert_eq!(VC21_PIXEL_VERSION, "VC2-1");
        assert!(!VC21_PIXEL_VERSION.contains('.'));
    }

    #[test]
    fn escaped_player_shots_have_real_screen_margin() {
        assert!(VC21_SHOT_DESPAWN_Y < 0.0);
        assert!(VC21_SHOT_ESCAPE_Y > 0.0);
    }

    #[test]
    fn audio_keepalive_overlaps_itself() {
        let duration = VC21_AUDIO_KEEPALIVE_DURATION_SAMPLES as f32 / AUDIO_SAMPLE_RATE as f32;
        assert!(duration > VC21_AUDIO_KEEPALIVE_PERIOD);
    }
}
