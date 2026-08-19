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

const VC20_BOSS_SHIELD_MAX: u32 = 120;
const VC20_BOSS_SHIELD_BREAK_DURATION: f32 = 1.15;
const VC20_BOSS_SHIELD_FLASH_DURATION: f32 = 0.10;
const VC20_ARMOR: Pixel = Pixel::rgb(82, 202, 255);
const VC20_ARMOR_LIGHT: Pixel = Pixel::rgb(214, 246, 255);
const VC20_ARMOR_BG: Pixel = Pixel::rgb(24, 47, 64);
const VC20_HULL: Pixel = Pixel::rgb(245, 92, 105);
const VC20_BAR_WIDTH: u32 = 18;

type CarrionDefenseKey = (u32, u32, u32, u8);
type SpecialDefenseKey = (u8, u32, u32, u32);
type ThreatDefenseKey = (u8, u32, u32, u32);

struct VoidCanticleV20 {
    game: VoidCanticleV19,
    sfx_manifest: gotoo_pixel_engine::SfxManifest,
    carrion_armor: std::collections::BTreeMap<CarrionDefenseKey, u32>,
    special_armor: std::collections::BTreeMap<SpecialDefenseKey, u32>,
    threat_armor: std::collections::BTreeMap<ThreatDefenseKey, u32>,
    boss_shield: u32,
    boss_defense_armed: bool,
    boss_shield_flash_timer: f32,
    boss_shield_break_timer: f32,
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
            carrion_armor: std::collections::BTreeMap::new(),
            special_armor: std::collections::BTreeMap::new(),
            threat_armor: std::collections::BTreeMap::new(),
            boss_shield: 0,
            boss_defense_armed: false,
            boss_shield_flash_timer: 0.0,
            boss_shield_break_timer: 0.0,
        }
    }

    fn tune_bellkeeper_before_update(&mut self) {
        let base = self.game.ui.game.combat.base_mut();
        if base.encounter_phase != EncounterPhase::BossFight {
            return;
        }
        let Some(boss) = base.boss.as_mut() else {
            return;
        };
        boss.shot_timer = boss.shot_timer.min(vc20_bellkeeper_shot_cap(boss.phase()));
    }

    fn update_defense_timers(&mut self, dt: f32) {
        self.boss_shield_flash_timer = (self.boss_shield_flash_timer - dt).max(0.0);
        self.boss_shield_break_timer = (self.boss_shield_break_timer - dt).max(0.0);
    }

    fn prepare_boss_shield(&mut self) {
        let boss_present = {
            let base = self.game.base();
            base.boss.is_some() && base.encounter_phase != EncounterPhase::Cleared
        };

        if boss_present && !self.boss_defense_armed {
            self.boss_shield = VC20_BOSS_SHIELD_MAX;
            self.boss_defense_armed = true;
            self.boss_shield_break_timer = 0.0;
        } else if !boss_present {
            self.boss_shield = 0;
            self.boss_defense_armed = false;
            self.boss_shield_flash_timer = 0.0;
        }
    }

    fn sync_enemy_armor(&mut self) {
        let carrion_active: Vec<(CarrionDefenseKey, u32)> = self
            .game
            .base()
            .enemies
            .iter()
            .filter(|enemy| enemy.alive)
            .map(|enemy| (vc20_carrion_key(enemy), vc20_carrion_armor_max(enemy.pattern)))
            .collect();
        self.carrion_armor
            .retain(|key, _| carrion_active.iter().any(|(active, _)| active == key));
        for (key, armor) in carrion_active {
            self.carrion_armor.entry(key).or_insert(armor);
        }

        let special_active: Vec<(SpecialDefenseKey, u32)> = self
            .game
            .v12()
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive)
            .map(|enemy| (vc20_special_key(enemy), vc20_special_armor_max(enemy.kind)))
            .collect();
        self.special_armor
            .retain(|key, _| special_active.iter().any(|(active, _)| active == key));
        for (key, armor) in special_active {
            self.special_armor.entry(key).or_insert(armor);
        }

        let threat_active: Vec<(ThreatDefenseKey, u32)> = self
            .game
            .v12()
            .threats
            .iter()
            .filter(|threat| threat.alive)
            .map(|threat| (vc20_threat_key(threat), vc20_threat_armor_max(threat.kind)))
            .collect();
        self.threat_armor
            .retain(|key, _| threat_active.iter().any(|(active, _)| active == key));
        for (key, armor) in threat_active {
            self.threat_armor.entry(key).or_insert(armor);
        }
    }

    fn preprocess_projectile_defenses(&mut self, dt: f32) {
        self.sync_enemy_armor();

        let carrion_targets: Vec<(CarrionDefenseKey, f32, f32)> = self
            .game
            .base()
            .enemies
            .iter()
            .filter(|enemy| enemy.alive)
            .map(|enemy| {
                let next_age = enemy.age + dt;
                (
                    vc20_carrion_key(enemy),
                    curved_x(
                        enemy.base_x,
                        next_age,
                        enemy.phase,
                        enemy.curve_amplitude,
                    ),
                    -10.0 + next_age * 42.0,
                )
            })
            .collect();
        let special_targets: Vec<(SpecialDefenseKey, f32, f32, f32)> = self
            .game
            .v12()
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive)
            .map(|enemy| {
                (
                    vc20_special_key(enemy),
                    enemy.x,
                    enemy.y,
                    enemy.hit_radius() + 3.0,
                )
            })
            .collect();
        let threat_targets: Vec<(ThreatDefenseKey, f32, f32, f32)> = self
            .game
            .v12()
            .threats
            .iter()
            .filter(|threat| threat.alive)
            .map(|threat| {
                (
                    vc20_threat_key(threat),
                    threat.x,
                    threat.y,
                    threat.hit_radius() + 3.0,
                )
            })
            .collect();
        let boss_target = self.game.base().boss.map(|boss| (boss.x, boss.y));

        let mut shield_hit = false;
        let mut shield_broken = false;

        {
            let carrion_armor = &mut self.carrion_armor;
            let boss_shield = &mut self.boss_shield;
            let base = self.game.ui.game.combat.base_mut();
            base.player_bullets.retain_mut(|bullet| {
                let x = bullet.x + bullet.vx * dt;
                let y = bullet.y + bullet.vy * dt;

                for (key, enemy_x, enemy_y) in &carrion_targets {
                    if carrion_armor.get(key).copied().unwrap_or(0) > 0
                        && point_near(x, y, *enemy_x, *enemy_y, 10.5)
                    {
                        if let Some(armor) = carrion_armor.get_mut(key) {
                            *armor = armor.saturating_sub(1);
                        }
                        return false;
                    }
                }

                if *boss_shield > 0
                    && let Some((boss_x, boss_y)) = boss_target
                    && point_near(x, y, boss_x, boss_y, 23.0)
                {
                    let before = *boss_shield;
                    *boss_shield = boss_shield.saturating_sub(1);
                    shield_hit = true;
                    shield_broken |= before > 0 && *boss_shield == 0;
                    return false;
                }

                true
            });
        }

        {
            let carrion_armor = &mut self.carrion_armor;
            let special_armor = &mut self.special_armor;
            let threat_armor = &mut self.threat_armor;
            let boss_shield = &mut self.boss_shield;
            let game = &mut self
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
                .inner;
            game.power_shots.retain_mut(|shot| {
                let x = shot.x + shot.vx * dt;
                let y = shot.y + shot.vy * dt;
                let damage = shot.damage.max(1);

                for (key, enemy_x, enemy_y, radius) in &threat_targets {
                    if threat_armor.get(key).copied().unwrap_or(0) > 0
                        && point_near(x, y, *enemy_x, *enemy_y, *radius + shot.radius as f32)
                    {
                        if let Some(armor) = threat_armor.get_mut(key) {
                            *armor = armor.saturating_sub(damage);
                        }
                        return false;
                    }
                }

                for (key, enemy_x, enemy_y, radius) in &special_targets {
                    if special_armor.get(key).copied().unwrap_or(0) > 0
                        && point_near(x, y, *enemy_x, *enemy_y, *radius + shot.radius as f32)
                    {
                        if let Some(armor) = special_armor.get_mut(key) {
                            *armor = armor.saturating_sub(damage);
                        }
                        return false;
                    }
                }

                for (key, enemy_x, enemy_y) in &carrion_targets {
                    if carrion_armor.get(key).copied().unwrap_or(0) > 0
                        && point_near(x, y, *enemy_x, *enemy_y, 9.5 + shot.radius as f32)
                    {
                        if let Some(armor) = carrion_armor.get_mut(key) {
                            *armor = armor.saturating_sub(damage);
                        }
                        return false;
                    }
                }

                if *boss_shield > 0
                    && let Some((boss_x, boss_y)) = boss_target
                    && point_near(x, y, boss_x, boss_y, 22.0 + shot.radius as f32)
                {
                    let before = *boss_shield;
                    *boss_shield = boss_shield.saturating_sub(damage);
                    shield_hit = true;
                    shield_broken |= before > 0 && *boss_shield == 0;
                    return false;
                }

                true
            });
        }

        if shield_hit {
            self.boss_shield_flash_timer = VC20_BOSS_SHIELD_FLASH_DURATION;
        }
        if shield_broken {
            self.boss_shield_break_timer = VC20_BOSS_SHIELD_BREAK_DURATION;
        }
    }

    fn reconcile_direct_boss_damage(
        &mut self,
        hp_before: Option<u32>,
        phase_before: Option<BellPhase>,
    ) {
        if self.boss_shield == 0 {
            return;
        }

        let mut reconciled = None;
        let mut shield_broken = false;
        {
            let boss_shield = &mut self.boss_shield;
            let base = self.game.ui.game.combat.base_mut();
            let Some(boss) = base.boss.as_mut() else {
                return;
            };
            let Some(before) = hp_before else {
                return;
            };
            if boss.hp >= before {
                return;
            }

            let damage = before - boss.hp;
            let absorbed = damage.min(*boss_shield);
            if absorbed == 0 {
                return;
            }

            boss.hp = boss.hp.saturating_add(absorbed).min(before);
            *boss_shield -= absorbed;
            shield_broken = *boss_shield == 0;
            reconciled = Some((boss.hp, boss.phase()));
        }

        self.boss_shield_flash_timer = VC20_BOSS_SHIELD_FLASH_DURATION;
        if shield_broken {
            self.boss_shield_break_timer = VC20_BOSS_SHIELD_BREAK_DURATION;
        }

        if let Some((hp, phase)) = reconciled {
            let polish = &mut self.game.ui.game.combat;
            polish.last_boss_hp = Some(hp);
            polish.last_boss_phase = Some(phase);
            if phase_before == Some(phase)
                && polish
                    .boss_phase_banner
                    .is_some_and(|banner_phase| banner_phase != phase)
            {
                polish.boss_phase_banner = None;
                polish.boss_phase_banner_timer = 0.0;
            }
        }
    }

    fn render_enemy_defenses(&self, framebuffer: &mut Framebuffer) {
        for enemy in &self.game.base().enemies {
            let key = vc20_carrion_key(enemy);
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let armor = self.carrion_armor.get(&key).copied().unwrap_or(armor_max);
            vc20_render_dual_bar(
                framebuffer,
                enemy.x.round() as i32,
                enemy.y.round() as i32 - 20,
                armor,
                armor_max,
                1,
                1,
            );
        }

        for enemy in &self.game.v12().combat.specials {
            let key = vc20_special_key(enemy);
            let armor_max = vc20_special_armor_max(enemy.kind);
            let armor = self.special_armor.get(&key).copied().unwrap_or(armor_max);
            vc20_render_dual_bar(
                framebuffer,
                enemy.x.round() as i32,
                enemy.y.round() as i32 - 20,
                armor,
                armor_max,
                enemy.hp,
                vc20_special_hp_max(enemy.kind),
            );
        }

        for threat in &self.game.v12().threats {
            let key = vc20_threat_key(threat);
            let armor_max = vc20_threat_armor_max(threat.kind);
            let armor = self.threat_armor.get(&key).copied().unwrap_or(armor_max);
            vc20_render_dual_bar(
                framebuffer,
                threat.x.round() as i32,
                threat.y.round() as i32 - 20,
                armor,
                armor_max,
                threat.hp,
                vc20_threat_hp_max(threat.kind),
            );
        }
    }

    fn render_boss_defense(&self, framebuffer: &mut Framebuffer) {
        if self.game.base().encounter_phase != EncounterPhase::BossFight {
            return;
        }
        let Some(boss) = self.game.base().boss else {
            return;
        };

        let color = if self.boss_shield_flash_timer > 0.0 {
            VC20_ARMOR_LIGHT
        } else {
            VC20_ARMOR
        };
        let x = boss.x.round() as i32;
        let y = boss.y.round() as i32;

        if self.boss_shield > 0 {
            let pulse = ((self.game.base().animation_time * 7.0).sin().abs() * 2.0) as u32;
            framebuffer.draw_circle(x, y, 27 + pulse, color);
            framebuffer.draw_circle(x, y, 31 + pulse, VC20_ARMOR_BG);
        } else if self.boss_shield_break_timer > 0.0 {
            framebuffer.draw_circle(x, y, 31, CANTICLE_COLOR);
        }
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
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.update_defense_timers(dt);
        self.prepare_boss_shield();

        let gameplay_running = matches!(&self.game.ui.state, VcPauseState::Running);
        if gameplay_running {
            self.tune_bellkeeper_before_update();
            self.preprocess_projectile_defenses(dt);
        }

        let hp_before = self.game.base().boss.map(|boss| boss.hp);
        let phase_before = self.game.base().boss.map(Bellkeeper::phase);
        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if gameplay_running {
            self.reconcile_direct_boss_damage(hp_before, phase_before);
        }

        if self.game.art_can_overlay_game() {
            self.render_enemy_defenses(frame.framebuffer);
            self.render_boss_defense(frame.framebuffer);
        } else if matches!(&self.game.ui.state, VcPauseState::BuildInfo) {
            self.render_build_info_overlay(frame.framebuffer);
        }

        GameResult::Continue
    }
}

fn vc20_bellkeeper_shot_cap(phase: BellPhase) -> f32 {
    match phase {
        BellPhase::Procession => 0.72,
        BellPhase::Resonance => 0.60,
        BellPhase::FinalToll => 0.50,
    }
}

fn vc20_pattern_code(pattern: ShotPattern) -> u8 {
    match pattern {
        ShotPattern::Aimed => 0,
        ShotPattern::Fan3 => 1,
        ShotPattern::Fan5 => 2,
    }
}

fn vc20_carrion_key(enemy: &CarrionDrone) -> CarrionDefenseKey {
    (
        enemy.base_x.to_bits(),
        enemy.phase.to_bits(),
        enemy.curve_amplitude.to_bits(),
        vc20_pattern_code(enemy.pattern),
    )
}

fn vc20_carrion_armor_max(pattern: ShotPattern) -> u32 {
    match pattern {
        ShotPattern::Aimed => 1,
        ShotPattern::Fan3 => 2,
        ShotPattern::Fan5 => 3,
    }
}

fn vc20_special_kind_code(kind: SpecialKind) -> u8 {
    match kind {
        SpecialKind::GraveKnight => 0,
        SpecialKind::BellWraith => 1,
        SpecialKind::RelicCarrier => 2,
    }
}

fn vc20_special_key(enemy: &SpecialEnemy) -> SpecialDefenseKey {
    (
        vc20_special_kind_code(enemy.kind),
        enemy.base_x.to_bits(),
        enemy.phase.to_bits(),
        enemy.direction.to_bits(),
    )
}

fn vc20_special_armor_max(kind: SpecialKind) -> u32 {
    match kind {
        SpecialKind::GraveKnight => 2,
        SpecialKind::BellWraith => 2,
        SpecialKind::RelicCarrier => 1,
    }
}

fn vc20_special_hp_max(kind: SpecialKind) -> u32 {
    match kind {
        SpecialKind::GraveKnight => 3,
        SpecialKind::BellWraith => 4,
        SpecialKind::RelicCarrier => 2,
    }
}

fn vc20_threat_kind_code(kind: ThreatKind) -> u8 {
    match kind {
        ThreatKind::ChoirNode => 0,
        ThreatKind::VoidLeech => 1,
    }
}

fn vc20_threat_key(threat: &ThreatEnemy) -> ThreatDefenseKey {
    (
        vc20_threat_kind_code(threat.kind),
        threat.base_x.to_bits(),
        threat.phase.to_bits(),
        threat.target_y.to_bits(),
    )
}

fn vc20_threat_armor_max(kind: ThreatKind) -> u32 {
    match kind {
        ThreatKind::ChoirNode => 3,
        ThreatKind::VoidLeech => 3,
    }
}

fn vc20_threat_hp_max(kind: ThreatKind) -> u32 {
    match kind {
        ThreatKind::ChoirNode => 6,
        ThreatKind::VoidLeech => 7,
    }
}

fn vc20_render_dual_bar(
    framebuffer: &mut Framebuffer,
    center_x: i32,
    y: i32,
    armor: u32,
    armor_max: u32,
    hp: u32,
    hp_max: u32,
) {
    let left = center_x - VC20_BAR_WIDTH as i32 / 2;
    framebuffer.fill_rect(left, y, VC20_BAR_WIDTH, 2, VC20_ARMOR_BG);
    if armor_max > 0 && armor > 0 {
        let armor_width = VC20_BAR_WIDTH.saturating_mul(armor.min(armor_max)) / armor_max;
        framebuffer.fill_rect(left, y, armor_width, 2, VC20_ARMOR);
    }

    framebuffer.fill_rect(left, y + 3, VC20_BAR_WIDTH, 2, CORE_BG);
    if hp_max > 0 && hp > 0 {
        let hp_width = VC20_BAR_WIDTH.saturating_mul(hp.min(hp_max)) / hp_max;
        framebuffer.fill_rect(left, y + 3, hp_width, 2, VC20_HULL);
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

    #[test]
    fn bellkeeper_pressure_comes_from_real_salves_not_damage_gating() {
        assert!(vc20_bellkeeper_shot_cap(BellPhase::Procession) < 1.0);
        assert!(
            vc20_bellkeeper_shot_cap(BellPhase::Resonance)
                < vc20_bellkeeper_shot_cap(BellPhase::Procession)
        );
        assert!(
            vc20_bellkeeper_shot_cap(BellPhase::FinalToll)
                < vc20_bellkeeper_shot_cap(BellPhase::Resonance)
        );
    }

    #[test]
    fn carrion_armor_scales_with_attack_pattern() {
        assert_eq!(vc20_carrion_armor_max(ShotPattern::Aimed), 1);
        assert_eq!(vc20_carrion_armor_max(ShotPattern::Fan3), 2);
        assert_eq!(vc20_carrion_armor_max(ShotPattern::Fan5), 3);
    }

    #[test]
    fn bellkeeper_shield_doubles_initial_defense_budget() {
        assert_eq!(VC20_BOSS_SHIELD_MAX, BELLKEEPER_MAX_HP);
    }
}
