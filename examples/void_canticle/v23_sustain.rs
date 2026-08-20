const VC23_SUPPORT_LEVEL: u32 = 3;
const VC23_NANITE_DELAY: f32 = 4.0;
const VC23_NANITE_REPAIR_PER_SECOND: f32 = 0.75;
const VC23_CAPACITOR_DELAY: f32 = 2.5;
const VC23_CAPACITOR_REGEN_PER_SECOND: f32 = 8.0;
const VC23_SUSTAIN_FLASH_DURATION: f32 = 0.32;

const VC23_ATTACK_SFX_KEYS: [&str; 5] = [
    "enemy_fire_carrion",
    "enemy_fire_wraith",
    "enemy_fire_void_pulse",
    "enemy_fire_void",
    "enemy_fire_bellkeeper",
];
const VC23_CARRION_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire_carrion");
const VC23_WRAITH_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire_wraith");
const VC23_VOID_PULSE_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire_void_pulse");
const VC23_VOID_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire_void");
const VC23_BELLKEEPER_FIRE_SOUND: SoundId = SoundId::new("void_canticle.enemy_fire_bellkeeper");

const VC23_SUPPORT_UP: ActionId = ActionId::new("void_canticle.support.up");
const VC23_SUPPORT_DOWN: ActionId = ActionId::new("void_canticle.support.down");
const VC23_SUPPORT_CONFIRM: ActionId = ActionId::new("void_canticle.support.confirm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc23AttackSound {
    Carrion,
    Wraith,
    VoidPulse,
    Void,
    Bellkeeper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc23SustainAugment {
    NaniteRepair,
    ShieldCapacitor,
}

const VC23_SUSTAIN_AUGMENTS: [Vc23SustainAugment; 2] = [
    Vc23SustainAugment::NaniteRepair,
    Vc23SustainAugment::ShieldCapacitor,
];

impl Vc23SustainAugment {
    fn name(self) -> &'static str {
        match self {
            Self::NaniteRepair => "NANITE REPAIR",
            Self::ShieldCapacitor => "SHIELD CAPACITOR",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::NaniteRepair => "HULL REPAIR AFTER 4S",
            Self::ShieldCapacitor => "SHIELD REGEN AFTER 2S",
        }
    }
}

struct VoidCanticleV23Sustain {
    game: VoidCanticleV23,
    augment: Option<Vc23SustainAugment>,
    support_offered: bool,
    choosing_support: bool,
    choice_input_armed: bool,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
    quiet_timer: f32,
    sustain_flash_timer: f32,
}

impl VoidCanticleV23Sustain {
    fn new() -> Self {
        let manifest = gotoo_pixel_engine::SfxManifest::parse(VC20_SFX_MANIFEST)
            .expect("checked-in VC SFX manifest should parse");
        manifest
            .require_keys(&VC23_ATTACK_SFX_KEYS)
            .expect("checked-in VC SFX manifest should contain attack family events");

        let mut sustain = Self {
            game: VoidCanticleV23::new(),
            augment: None,
            support_offered: false,
            choosing_support: false,
            choice_input_armed: false,
            menu: gotoo_pixel_engine::ui::MenuState::new(VC23_SUSTAIN_AUGMENTS.len()),
            controls: gotoo_pixel_engine::ui::standard_menu_controls(
                VC23_SUPPORT_UP,
                VC23_SUPPORT_DOWN,
                VC23_SUPPORT_CONFIRM,
            ),
            quiet_timer: 0.0,
            sustain_flash_timer: 0.0,
        };

        for (id, wav) in [
            (
                VC23_CARRION_FIRE_SOUND,
                synthesize_chirp(330.0, 510.0, 0.048, 0.045),
            ),
            (
                VC23_WRAITH_FIRE_SOUND,
                synthesize_chirp(880.0, 250.0, 0.095, 0.060),
            ),
            (
                VC23_VOID_PULSE_FIRE_SOUND,
                synthesize_chirp(180.0, 72.0, 0.130, 0.070),
            ),
            (
                VC23_VOID_FIRE_SOUND,
                synthesize_chirp(350.0, 112.0, 0.105, 0.055),
            ),
            (
                VC23_BELLKEEPER_FIRE_SOUND,
                synthesize_chirp(215.0, 82.0, 0.155, 0.075),
            ),
        ] {
            sustain
                .combat_model_mut()
                .base_mut()
                .sounds
                .insert_wav(id, wav)
                .expect("VC2.3 attack family sound ids should be unique");
        }

        sustain
    }

    fn combat_model(&self) -> &VoidCanticleV21 {
        &self.game.game.game.game.game.game.game.game
    }

    fn combat_model_mut(&mut self) -> &mut VoidCanticleV21 {
        &mut self.game.game.game.game.game.game.game.game
    }

    fn survival_model(&self) -> &VoidCanticleV21SurvivalCleanup {
        &self.game.game.game.game.game
    }

    fn hull_cap(&self) -> f32 {
        let survival = self.survival_model();
        survival.hull_cap_for_stacks(survival.vital_spark_stacks())
    }

    fn shield_cap(&self) -> f32 {
        self.game.game.game.game.game.game.tuning.player_shield
    }

    fn progression_level(&self) -> u32 {
        self.game.v20().game.v14().progression.level
    }

    fn attack_audio_snapshot(&self) -> (usize, bool) {
        (
            self.game.base().enemy_bullets.len(),
            self.game
                .v20()
                .game
                .ui
                .game
                .combat
                .combat
                .pending_attack
                .is_some(),
        )
    }

    fn detect_attack_sounds(
        &self,
        bullet_count_before: usize,
        void_pending_before: bool,
    ) -> Vec<Vc23AttackSound> {
        let bullets = &self.game.base().enemy_bullets;
        let new_count = bullets.len().saturating_sub(bullet_count_before);
        if new_count == 0 {
            return Vec::new();
        }

        let void_pending_after = self
            .game
            .v20()
            .game
            .ui
            .game
            .combat
            .combat
            .pending_attack
            .is_some();
        let void_attack_fired = void_pending_before && !void_pending_after;
        let encounter_phase = self.game.base().encounter_phase;
        let mut sounds = Vec::new();

        for bullet in &bullets[bullets.len() - new_count..] {
            let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt();
            let sound = vc23_attack_sound_style(encounter_phase, speed, void_attack_fired);
            if !sounds.contains(&sound) {
                sounds.push(sound);
            }
        }
        sounds
    }

    fn play_attack_sounds(&mut self, frame: &mut Frame<'_>, sounds: &[Vc23AttackSound]) {
        for sound in sounds {
            let id = vc23_attack_sound_id(*sound);
            let _ = self
                .combat_model_mut()
                .base_mut()
                .sounds
                .play(frame.audio, id);
        }
    }

    fn support_choice_can_start(&self) -> bool {
        !self.support_offered
            && !self.choosing_support
            && self.progression_level() >= VC23_SUPPORT_LEVEL
            && self.game.active_combat()
            && self.game.v20().game.v14().progression.level_choice.is_none()
            && self.game.v20().game.v14().mutation_choice.is_none()
    }

    fn maybe_start_support_choice(&mut self) {
        if self.support_choice_can_start() {
            self.support_offered = true;
            self.choosing_support = true;
            self.choice_input_armed = false;
            self.menu = gotoo_pixel_engine::ui::MenuState::new(VC23_SUSTAIN_AUGMENTS.len());
            self.quiet_timer = 0.0;
        }
    }

    fn update_support_choice(&mut self, frame: &mut Frame<'_>) {
        self.controls.update(frame.input);

        if !self.choice_input_armed {
            let any_held = [VC23_SUPPORT_UP, VC23_SUPPORT_DOWN, VC23_SUPPORT_CONFIRM]
                .into_iter()
                .any(|action| self.controls.action(action).held());
            if !any_held {
                self.choice_input_armed = true;
            }
            return;
        }

        if self.controls.action(VC23_SUPPORT_UP).pressed() {
            self.menu.select_previous();
        }
        if self.controls.action(VC23_SUPPORT_DOWN).pressed() {
            self.menu.select_next();
        }
        if self.controls.action(VC23_SUPPORT_CONFIRM).pressed()
            && let Some(index) = self.menu.selected()
            && let Some(augment) = VC23_SUSTAIN_AUGMENTS.get(index).copied()
        {
            self.augment = Some(augment);
            self.choosing_support = false;
            self.choice_input_armed = false;
            self.quiet_timer = 0.0;
            self.sustain_flash_timer = VC23_SUSTAIN_FLASH_DURATION;
        }
    }

    fn update_sustain(&mut self, dt: f32, hull_before: f32, shield_before: f32) {
        self.sustain_flash_timer = (self.sustain_flash_timer - dt).max(0.0);
        let hull_now = self.combat_model().player_hull;
        let shield_now = self.combat_model().player_shield;
        let damaged = hull_now + f32::EPSILON < hull_before
            || shield_now + f32::EPSILON < shield_before;

        if damaged {
            self.quiet_timer = 0.0;
            return;
        }
        if !self.game.active_combat() {
            return;
        }

        self.quiet_timer += dt;
        let Some(augment) = self.augment else {
            return;
        };

        match augment {
            Vc23SustainAugment::NaniteRepair if self.quiet_timer >= VC23_NANITE_DELAY => {
                let cap = self.hull_cap();
                let model = self.combat_model_mut();
                let before = model.player_hull;
                model.player_hull =
                    (model.player_hull + VC23_NANITE_REPAIR_PER_SECOND * dt).min(cap);
                if model.player_hull > before + f32::EPSILON {
                    self.sustain_flash_timer = VC23_SUSTAIN_FLASH_DURATION;
                }
            }
            Vc23SustainAugment::ShieldCapacitor
                if self.quiet_timer >= VC23_CAPACITOR_DELAY =>
            {
                let cap = self.shield_cap();
                let model = self.combat_model_mut();
                let before = model.player_shield;
                model.player_shield =
                    (model.player_shield + VC23_CAPACITOR_REGEN_PER_SECOND * dt).min(cap);
                if model.player_shield > before + f32::EPSILON {
                    self.sustain_flash_timer = VC23_SUSTAIN_FLASH_DURATION;
                }
            }
            _ => {}
        }
    }

    fn reset_for_new_run(&mut self) {
        self.augment = None;
        self.support_offered = false;
        self.choosing_support = false;
        self.choice_input_armed = false;
        self.menu = gotoo_pixel_engine::ui::MenuState::new(VC23_SUSTAIN_AUGMENTS.len());
        self.quiet_timer = 0.0;
        self.sustain_flash_timer = 0.0;
    }

    fn render_support_choice(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(8, 72, 164, 176, Pixel::rgb(7, 10, 19));
        framebuffer.draw_rect(8, 72, 164, 176, ART_CYAN);
        framebuffer.draw_text(43, 86, "SUPPORT AUGMENT", ART_CYAN_LIGHT);
        framebuffer.draw_text(54, 101, "CHOOSE MODULE", WRECK_LIGHT);

        for (index, augment) in VC23_SUSTAIN_AUGMENTS.iter().copied().enumerate() {
            let y = 126 + index as i32 * 52;
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 18,
                    y,
                    width: 144,
                    height: 18,
                },
                augment.name(),
                self.menu.selected() == Some(index),
                1,
                TEXT,
                ART_CYAN_LIGHT,
            );
            framebuffer.draw_text(28, y + 24, augment.description(), WRECK_LIGHT);
        }

        framebuffer.draw_text(39, 228, "SPACE SOUTH SELECT", TEXT);
    }
}

impl Game for VoidCanticleV23Sustain {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);

        if self.choosing_support {
            self.update_support_choice(frame);
            self.render_support_choice(frame.framebuffer);
            return GameResult::Continue;
        }

        let stage_time_before = self.game.base().stage_time;
        let hull_before = self.combat_model().player_hull;
        let shield_before = self.combat_model().player_shield;
        let (bullet_count_before, void_pending_before) = self.attack_audio_snapshot();

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let attack_sounds = self.detect_attack_sounds(bullet_count_before, void_pending_before);
        self.play_attack_sounds(frame, &attack_sounds);

        if self.game.base().stage_time + 0.05 < stage_time_before {
            self.reset_for_new_run();
        }

        self.update_sustain(dt, hull_before, shield_before);
        self.maybe_start_support_choice();

        if self.choosing_support {
            self.render_support_choice(frame.framebuffer);
        }
        GameResult::Continue
    }
}

fn vc23_attack_sound_id(sound: Vc23AttackSound) -> SoundId {
    match sound {
        Vc23AttackSound::Carrion => VC23_CARRION_FIRE_SOUND,
        Vc23AttackSound::Wraith => VC23_WRAITH_FIRE_SOUND,
        Vc23AttackSound::VoidPulse => VC23_VOID_PULSE_FIRE_SOUND,
        Vc23AttackSound::Void => VC23_VOID_FIRE_SOUND,
        Vc23AttackSound::Bellkeeper => VC23_BELLKEEPER_FIRE_SOUND,
    }
}

fn vc23_void_attack_speed(speed: f32) -> bool {
    [62.0, 70.0, 76.0, 78.0, 86.0, 90.0, 96.0, 108.0, 110.0, 132.0]
        .into_iter()
        .any(|candidate| (speed - candidate).abs() <= 1.0)
}

fn vc23_attack_sound_style(
    encounter_phase: EncounterPhase,
    speed: f32,
    void_attack_fired: bool,
) -> Vc23AttackSound {
    if void_attack_fired && vc23_void_attack_speed(speed) {
        return Vc23AttackSound::Void;
    }
    if encounter_phase == EncounterPhase::BossFight {
        return Vc23AttackSound::Bellkeeper;
    }
    if (speed - 48.0).abs() <= 1.0 {
        Vc23AttackSound::Wraith
    } else if (speed - 62.0).abs() <= 1.0 {
        Vc23AttackSound::VoidPulse
    } else if (speed - ENEMY_SHOT_SPEED).abs() <= 1.0 {
        Vc23AttackSound::Carrion
    } else {
        Vc23AttackSound::Void
    }
}

pub fn run_v23_sustain_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC23_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV23Sustain::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v23_sustain_tests {
    use super::*;

    #[test]
    fn sustain_augments_are_mutually_exclusive_choices() {
        assert_eq!(VC23_SUSTAIN_AUGMENTS.len(), 2);
        assert_ne!(VC23_SUSTAIN_AUGMENTS[0], VC23_SUSTAIN_AUGMENTS[1]);
    }

    #[test]
    fn nanites_are_slow_and_require_a_safe_window() {
        assert!(VC23_NANITE_DELAY >= 3.0);
        assert!(VC23_NANITE_REPAIR_PER_SECOND > 0.0);
        assert!(VC23_NANITE_REPAIR_PER_SECOND <= 1.0);
    }

    #[test]
    fn capacitor_is_faster_than_hull_repair_but_not_instant() {
        assert!(VC23_CAPACITOR_DELAY > 1.0);
        assert!(VC23_CAPACITOR_REGEN_PER_SECOND > VC23_NANITE_REPAIR_PER_SECOND);
        assert!(VC23_CAPACITOR_REGEN_PER_SECOND < 20.0);
    }

    #[test]
    fn support_choice_waits_until_progression_is_established() {
        assert!(VC23_SUPPORT_LEVEL >= 3);
    }

    #[test]
    fn attack_family_events_are_declared_in_sfx_manifest() {
        let manifest = gotoo_pixel_engine::SfxManifest::parse(VC20_SFX_MANIFEST)
            .expect("VC SFX manifest should parse");
        manifest
            .require_keys(&VC23_ATTACK_SFX_KEYS)
            .expect("VC SFX manifest should cover attack family events");
    }

    #[test]
    fn attack_sound_style_follows_existing_projectile_language() {
        assert_eq!(
            vc23_attack_sound_style(EncounterPhase::Waves, 68.0, false),
            Vc23AttackSound::Carrion
        );
        assert_eq!(
            vc23_attack_sound_style(EncounterPhase::Waves, 48.0, false),
            Vc23AttackSound::Wraith
        );
        assert_eq!(
            vc23_attack_sound_style(EncounterPhase::Waves, 62.0, false),
            Vc23AttackSound::VoidPulse
        );
        assert_eq!(
            vc23_attack_sound_style(EncounterPhase::Waves, 96.0, true),
            Vc23AttackSound::Void
        );
        assert_eq!(
            vc23_attack_sound_style(EncounterPhase::BossFight, 48.0, false),
            Vc23AttackSound::Bellkeeper
        );
        assert_eq!(
            vc23_attack_sound_style(EncounterPhase::BossFight, 96.0, true),
            Vc23AttackSound::Void
        );
    }
}
