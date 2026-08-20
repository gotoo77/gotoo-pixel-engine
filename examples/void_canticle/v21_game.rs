const VC21_VERSION: &str = "VC2.1";

const VC21_PLAYER_HULL_MAX: f32 = 100.0;
const VC21_PLAYER_SHIELD_MAX: f32 = 50.0;
const VC21_LEGACY_IMPACT_DAMAGE: f32 = 25.0;
const VC21_SHIELD_REGEN_DELAY: f32 = 2.75;
const VC21_SHIELD_REGEN_PER_SECOND: f32 = 15.0;
const VC21_PLAYER_SHIELD_FLASH_DURATION: f32 = 0.16;
const VC21_PLAYER_HULL_FLASH_DURATION: f32 = 0.18;
const VC21_ENEMY_HULL_IMPACT_DURATION: f32 = 0.17;
const VC21_ENEMY_HULL_IMPACT_SPARKS: usize = 5;

const VC21_ELITE_COLLAPSE_DURATION: f32 = 0.42;
const VC21_ELITE_NOVA_DURATION: f32 = 0.46;
const VC21_ELITE_NOVA_RADIUS: f32 = 68.0;
const VC21_ELITE_NOVA_DAMAGE: f32 = 35.0;

const VC21_IMPACT_SOUND_COOLDOWN: f32 = 0.055;
const VC21_ARMOR_IMPACT_SOUND: SoundId = SoundId::new("void_canticle.armor_impact");
const VC21_PLAYER_SHIELD_IMPACT_SOUND: SoundId =
    SoundId::new("void_canticle.player_shield_impact");
const VC21_PLAYER_SHIELD_BREAK_SOUND: SoundId =
    SoundId::new("void_canticle.player_shield_break");
const VC21_ELITE_IMPLOSION_SOUND: SoundId = SoundId::new("void_canticle.elite_implosion");
const VC21_ELITE_NOVA_SOUND: SoundId = SoundId::new("void_canticle.elite_nova");

const VC21_REQUIRED_SFX: [&str; 7] = [
    "armor_impact",
    "enemy_hull_impact",
    "player_shield_impact",
    "player_shield_break",
    "player_hull_impact",
    "elite_implosion",
    "elite_nova",
];

#[derive(Debug, Clone, Copy)]
enum Vc21CombatEvent {
    EnemyArmorImpact,
    EnemyHullImpact,
    EnemyShieldImpact,
    PlayerShieldImpact { x: f32, y: f32 },
    PlayerShieldBreak { x: f32, y: f32 },
    PlayerHullImpact { x: f32, y: f32, play_sound: bool },
    EliteImplosion { x: f32, y: f32 },
    EliteNova { x: f32, y: f32 },
}

#[derive(Debug, Clone, Copy)]
struct Vc21EliteSnapshot {
    key: SpecialDefenseKey,
    x: f32,
    y: f32,
    hit_radius: f32,
}

#[derive(Debug, Clone, Copy)]
struct Vc21EliteDeathFx {
    x: f32,
    y: f32,
    age: f32,
    nova_triggered: bool,
}

impl Vc21EliteDeathFx {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            age: 0.0,
            nova_triggered: false,
        }
    }
}

#[derive(Default)]
struct Vc21ImpactSnapshot {
    carrion_armor: std::collections::BTreeMap<CarrionDefenseKey, u32>,
    special_armor: std::collections::BTreeMap<SpecialDefenseKey, u32>,
    threat_armor: std::collections::BTreeMap<ThreatDefenseKey, u32>,
    special_hp: std::collections::BTreeMap<SpecialDefenseKey, u32>,
    threat_hp: std::collections::BTreeMap<ThreatDefenseKey, u32>,
    boss_shield: u32,
}

struct VoidCanticleV21 {
    combat: VoidCanticleV20,
    player_hull: f32,
    player_shield: f32,
    shield_regen_delay: f32,
    player_shield_flash_timer: f32,
    player_hull_flash_timer: f32,
    impact_sound_cooldown: f32,
    combat_events: Vec<Vc21CombatEvent>,
    elite_deaths: Vec<Vc21EliteDeathFx>,
}

impl VoidCanticleV21 {
    fn new() -> Self {
        let mut combat = VoidCanticleV20::new();
        combat
            .sfx_manifest
            .require_keys(&VC21_REQUIRED_SFX)
            .expect("checked-in VC2.1 SFX manifest should contain combat-model events");

        {
            let base = combat.game.ui.game.combat.base_mut();
            for (id, wav) in [
                (
                    VC21_ARMOR_IMPACT_SOUND,
                    synthesize_chirp(1180.0, 640.0, 0.052, 0.13),
                ),
                (
                    VC21_PLAYER_SHIELD_IMPACT_SOUND,
                    synthesize_chirp(860.0, 430.0, 0.090, 0.18),
                ),
                (
                    VC21_PLAYER_SHIELD_BREAK_SOUND,
                    synthesize_noise_burst(0.180, 0.24, 0x521E_1D01),
                ),
                (
                    VC21_ELITE_IMPLOSION_SOUND,
                    synthesize_chirp(260.0, 72.0, 0.300, 0.22),
                ),
                (
                    VC21_ELITE_NOVA_SOUND,
                    synthesize_noise_burst(0.260, 0.34, 0xE117_EA21),
                ),
            ] {
                base.sounds
                    .insert_wav(id, wav)
                    .expect("VC2.1 combat sound ids should be unique");
            }
        }

        Self {
            combat,
            player_hull: VC21_PLAYER_HULL_MAX,
            player_shield: VC21_PLAYER_SHIELD_MAX,
            shield_regen_delay: 0.0,
            player_shield_flash_timer: 0.0,
            player_hull_flash_timer: 0.0,
            impact_sound_cooldown: 0.0,
            combat_events: Vec::new(),
            elite_deaths: Vec::new(),
        }
    }

    fn base(&self) -> &VoidCanticleGame {
        self.combat.game.base()
    }

    fn base_mut(&mut self) -> &mut VoidCanticleGame {
        self.combat.game.ui.game.combat.base_mut()
    }

    fn power_level(&self) -> u8 {
        self.combat
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
            .power_level
    }

    fn set_power_level(&mut self, level: u8) {
        self.combat
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
            .power_level = level;
    }

    fn reset_combat_model(&mut self) {
        self.player_hull = VC21_PLAYER_HULL_MAX;
        self.player_shield = VC21_PLAYER_SHIELD_MAX;
        self.shield_regen_delay = 0.0;
        self.player_shield_flash_timer = 0.0;
        self.player_hull_flash_timer = 0.0;
        self.impact_sound_cooldown = 0.0;
        self.combat_events.clear();
        self.elite_deaths.clear();
    }

    fn update_timers(&mut self, dt: f32) {
        self.player_shield_flash_timer = (self.player_shield_flash_timer - dt).max(0.0);
        self.player_hull_flash_timer = (self.player_hull_flash_timer - dt).max(0.0);
        self.impact_sound_cooldown = (self.impact_sound_cooldown - dt).max(0.0);
    }

    fn capture_impact_snapshot(&mut self) -> Vc21ImpactSnapshot {
        self.combat.sync_enemy_armor();
        let special_hp = self
            .combat
            .game
            .v12()
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive)
            .map(|enemy| (vc20_special_key(enemy), enemy.hp))
            .collect();
        let threat_hp = self
            .combat
            .game
            .v12()
            .threats
            .iter()
            .filter(|threat| threat.alive)
            .map(|threat| (vc20_threat_key(threat), threat.hp))
            .collect();

        Vc21ImpactSnapshot {
            carrion_armor: self.combat.carrion_armor.clone(),
            special_armor: self.combat.special_armor.clone(),
            threat_armor: self.combat.threat_armor.clone(),
            special_hp,
            threat_hp,
            boss_shield: self.combat.boss_shield,
        }
    }

    fn snapshot_elites(&self) -> Vec<Vc21EliteSnapshot> {
        self.combat
            .game
            .v12()
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive && enemy.kind == SpecialKind::GraveKnight)
            .map(|enemy| Vc21EliteSnapshot {
                key: vc20_special_key(enemy),
                x: enemy.x,
                y: enemy.y,
                hit_radius: enemy.hit_radius(),
            })
            .collect()
    }

    fn spawn_enemy_hull_impact_fx(&mut self, impacts: &[(f32, f32)]) {
        for (impact_index, &(x, y)) in impacts.iter().enumerate() {
            self.base_mut()
                .bursts
                .push(Burst::new(x, y, VC21_ENEMY_HULL_IMPACT_DURATION, VC20_HULL));

            let seed = x * 0.053 + y * 0.079 + impact_index as f32 * 0.47;
            for spark in 0..VC21_ENEMY_HULL_IMPACT_SPARKS {
                let angle = seed
                    + spark as f32 * std::f32::consts::TAU
                        / VC21_ENEMY_HULL_IMPACT_SPARKS as f32;
                let speed = 58.0 + (spark % 3) as f32 * 21.0;
                self.combat.game.ui.game.particles.push(V17Particle {
                    x,
                    y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    life: VC21_ENEMY_HULL_IMPACT_DURATION,
                    max_life: VC21_ENEMY_HULL_IMPACT_DURATION,
                    color: if spark % 2 == 0 { VC20_HULL } else { ART_GOLD },
                    kind: V17ParticleKind::Shard,
                });
            }
        }
    }

    fn detect_impact_events(&mut self, before: &Vc21ImpactSnapshot) {
        let armor_hit = before
            .carrion_armor
            .iter()
            .any(|(key, value)| self.combat.carrion_armor.get(key).is_some_and(|now| now < value))
            || before.special_armor.iter().any(|(key, value)| {
                self.combat
                    .special_armor
                    .get(key)
                    .is_some_and(|now| now < value)
            })
            || before.threat_armor.iter().any(|(key, value)| {
                self.combat
                    .threat_armor
                    .get(key)
                    .is_some_and(|now| now < value)
            });
        if armor_hit {
            self.combat_events.push(Vc21CombatEvent::EnemyArmorImpact);
        }

        if self.combat.boss_shield < before.boss_shield {
            self.combat_events.push(Vc21CombatEvent::EnemyShieldImpact);
        }

        let hull_impacts: Vec<(f32, f32)> = {
            let v12 = self.combat.game.v12();
            let mut impacts = Vec::new();
            impacts.extend(v12.combat.specials.iter().filter_map(|enemy| {
                let key = vc20_special_key(enemy);
                before
                    .special_hp
                    .get(&key)
                    .is_some_and(|old_hp| enemy.hp < *old_hp)
                    .then_some((enemy.x, enemy.y))
            }));
            impacts.extend(v12.threats.iter().filter_map(|threat| {
                let key = vc20_threat_key(threat);
                before
                    .threat_hp
                    .get(&key)
                    .is_some_and(|old_hp| threat.hp < *old_hp)
                    .then_some((threat.x, threat.y))
            }));
            impacts
        };

        if !hull_impacts.is_empty() {
            self.spawn_enemy_hull_impact_fx(&hull_impacts);
            self.combat_events.push(Vc21CombatEvent::EnemyHullImpact);
        }
    }

    fn translate_legacy_player_hit(
        &mut self,
        lives_before: u32,
        power_before: u8,
    ) -> bool {
        let lives_after = self.base().lives;
        if lives_after >= lives_before {
            return false;
        }

        let hit_count = lives_before.saturating_sub(lives_after).max(1);
        self.set_power_level(power_before);
        for _ in 0..hit_count {
            self.apply_player_damage(VC21_LEGACY_IMPACT_DAMAGE, true);
        }

        if self.player_hull > 0.0 {
            let base = self.base_mut();
            base.lives = 3;
            base.game_over = false;
        } else {
            let base = self.base_mut();
            base.lives = 0;
            base.game_over = true;
        }
        true
    }

    fn apply_player_damage(&mut self, damage: f32, legacy_sound_already_played: bool) {
        if damage <= 0.0 || self.player_hull <= 0.0 {
            return;
        }

        let (shield_damage, hull_damage) = vc21_apply_damage_to_layers(
            &mut self.player_shield,
            &mut self.player_hull,
            damage,
        );
        self.shield_regen_delay = VC21_SHIELD_REGEN_DELAY;
        let (x, y) = (self.base().player_x, self.base().player_y);

        if shield_damage > 0.0 {
            self.player_shield_flash_timer = VC21_PLAYER_SHIELD_FLASH_DURATION;
            self.combat_events
                .push(Vc21CombatEvent::PlayerShieldImpact { x, y });
            if self.player_shield <= 0.0 {
                self.combat_events
                    .push(Vc21CombatEvent::PlayerShieldBreak { x, y });
            }
        }
        if hull_damage > 0.0 {
            self.player_hull_flash_timer = VC21_PLAYER_HULL_FLASH_DURATION;
            self.combat_events.push(Vc21CombatEvent::PlayerHullImpact {
                x,
                y,
                play_sound: !legacy_sound_already_played,
            });
        }
    }

    fn update_shield_regen(&mut self, dt: f32, damaged_this_frame: bool) {
        if damaged_this_frame || self.player_hull <= 0.0 {
            return;
        }
        if self.player_shield >= VC21_PLAYER_SHIELD_MAX {
            self.player_shield = VC21_PLAYER_SHIELD_MAX;
            self.shield_regen_delay = 0.0;
            return;
        }

        self.shield_regen_delay = (self.shield_regen_delay - dt).max(0.0);
        if self.shield_regen_delay <= 0.0 {
            self.player_shield = (self.player_shield + VC21_SHIELD_REGEN_PER_SECOND * dt)
                .min(VC21_PLAYER_SHIELD_MAX);
        }
    }

    fn detect_elite_deaths(&mut self, before: &[Vc21EliteSnapshot], legacy_hit: bool) {
        let active_keys: Vec<SpecialDefenseKey> = self
            .combat
            .game
            .v12()
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive && enemy.kind == SpecialKind::GraveKnight)
            .map(vc20_special_key)
            .collect();
        let (player_x, player_y) = (self.base().player_x, self.base().player_y);

        for elite in before {
            if active_keys.contains(&elite.key)
                || elite.y < 18.0
                || elite.y > FRAMEBUFFER_HEIGHT as f32 - 24.0
            {
                continue;
            }
            if legacy_hit
                && point_near(
                    elite.x,
                    elite.y,
                    player_x,
                    player_y,
                    elite.hit_radius + 2.0,
                )
            {
                continue;
            }

            self.elite_deaths
                .push(Vc21EliteDeathFx::new(elite.x, elite.y));
            self.combat_events.push(Vc21CombatEvent::EliteImplosion {
                x: elite.x,
                y: elite.y,
            });
        }
    }

    fn update_elite_deaths(&mut self, dt: f32) -> bool {
        let mut novas = Vec::new();
        for fx in &mut self.elite_deaths {
            fx.age += dt;
            if !fx.nova_triggered && fx.age >= VC21_ELITE_COLLAPSE_DURATION {
                fx.nova_triggered = true;
                novas.push((fx.x, fx.y));
            }
        }

        let mut player_damaged = false;
        for (x, y) in novas {
            self.combat_events.push(Vc21CombatEvent::EliteNova { x, y });
            {
                let base = self.base_mut();
                base.enemy_bullets
                    .retain(|bullet| !point_near(bullet.x, bullet.y, x, y, VC21_ELITE_NOVA_RADIUS));
            }

            let can_hit_player = {
                let base = self.base();
                base.invulnerability <= 0.0
                    && point_near(
                        x,
                        y,
                        base.player_x,
                        base.player_y,
                        VC21_ELITE_NOVA_RADIUS,
                    )
            };
            if can_hit_player {
                self.apply_player_damage(VC21_ELITE_NOVA_DAMAGE, false);
                self.base_mut().invulnerability = PLAYER_INVULNERABILITY * 0.65;
                player_damaged = true;
            }
        }

        let total_duration = VC21_ELITE_COLLAPSE_DURATION + VC21_ELITE_NOVA_DURATION;
        self.elite_deaths.retain(|fx| fx.age < total_duration);
        player_damaged
    }

    fn process_combat_events(&mut self, frame: &mut Frame<'_>) {
        let events = std::mem::take(&mut self.combat_events);
        let mut impact_sound_used = false;
        let mut implosion_sound_used = false;
        let mut nova_sound_used = false;

        for event in events {
            match event {
                Vc21CombatEvent::EnemyArmorImpact => {
                    if self.impact_sound_cooldown <= 0.0 && !impact_sound_used {
                        let _ = self.base_mut().sounds.play(frame.audio, VC21_ARMOR_IMPACT_SOUND);
                        self.impact_sound_cooldown = VC21_IMPACT_SOUND_COOLDOWN;
                        impact_sound_used = true;
                    }
                }
                Vc21CombatEvent::EnemyHullImpact => {
                    if self.impact_sound_cooldown <= 0.0 && !impact_sound_used {
                        let _ = self.base_mut().sounds.play(frame.audio, ENEMY_HIT_SOUND);
                        self.impact_sound_cooldown = VC21_IMPACT_SOUND_COOLDOWN;
                        impact_sound_used = true;
                    }
                }
                Vc21CombatEvent::EnemyShieldImpact => {
                    if self.impact_sound_cooldown <= 0.0 && !impact_sound_used {
                        let _ = self
                            .base_mut()
                            .sounds
                            .play(frame.audio, VC21_PLAYER_SHIELD_IMPACT_SOUND);
                        self.impact_sound_cooldown = VC21_IMPACT_SOUND_COOLDOWN;
                        impact_sound_used = true;
                    }
                }
                Vc21CombatEvent::PlayerShieldImpact { x, y } => {
                    let base = self.base_mut();
                    base.bursts.push(Burst::new(x, y, 0.20, VC20_ARMOR));
                    let _ = base.sounds.play(frame.audio, VC21_PLAYER_SHIELD_IMPACT_SOUND);
                }
                Vc21CombatEvent::PlayerShieldBreak { x, y } => {
                    let base = self.base_mut();
                    base.bursts
                        .push(Burst::new(x, y, 0.34, VC20_ARMOR_LIGHT));
                    let _ = base.sounds.play(frame.audio, VC21_PLAYER_SHIELD_BREAK_SOUND);
                }
                Vc21CombatEvent::PlayerHullImpact {
                    x,
                    y,
                    play_sound,
                } => {
                    let base = self.base_mut();
                    base.bursts.push(Burst::new(x, y, 0.24, DANGER));
                    if play_sound {
                        let _ = base.sounds.play(frame.audio, PLAYER_HIT_SOUND);
                    }
                }
                Vc21CombatEvent::EliteImplosion { x, y } => {
                    let base = self.base_mut();
                    base.bursts.push(Burst::new(x, y, 0.24, ART_VOID));
                    if !implosion_sound_used {
                        let _ = base.sounds.play(frame.audio, VC21_ELITE_IMPLOSION_SOUND);
                        implosion_sound_used = true;
                    }
                }
                Vc21CombatEvent::EliteNova { x, y } => {
                    let base = self.base_mut();
                    base.bursts.push(Burst::new(x, y, 0.46, CANTICLE_COLOR));
                    if !nova_sound_used {
                        let _ = base.sounds.play(frame.audio, VC21_ELITE_NOVA_SOUND);
                        nova_sound_used = true;
                    }
                }
            }
        }
    }

    fn render_player_survival(&self, framebuffer: &mut Framebuffer) {
        // Screen-space Hull/Shield HUD is owned by the active presentation.
        // V21 keeps only local hit feedback around the player.
        if self.player_shield_flash_timer > 0.0 {
            framebuffer.draw_circle(
                self.base().player_x.round() as i32,
                self.base().player_y.round() as i32,
                11,
                VC20_ARMOR_LIGHT,
            );
        } else if self.player_hull_flash_timer > 0.0 {
            framebuffer.draw_circle(
                self.base().player_x.round() as i32,
                self.base().player_y.round() as i32,
                9,
                DANGER,
            );
        }
    }

    fn render_elite_deaths(&self, framebuffer: &mut Framebuffer) {
        for fx in &self.elite_deaths {
            let x = fx.x.round() as i32;
            let y = fx.y.round() as i32;
            if fx.age < VC21_ELITE_COLLAPSE_DURATION {
                let progress = (fx.age / VC21_ELITE_COLLAPSE_DURATION).clamp(0.0, 1.0);
                let radius = (16.0 * (1.0 - progress)).max(2.0).round() as u32;
                framebuffer.draw_circle(x, y, radius + 5, ART_VOID);
                framebuffer.draw_circle(x, y, radius, VC20_ARMOR_LIGHT);
                framebuffer.fill_circle(x, y, 2 + (progress * 4.0) as u32, BG);
            } else {
                let progress = ((fx.age - VC21_ELITE_COLLAPSE_DURATION)
                    / VC21_ELITE_NOVA_DURATION)
                    .clamp(0.0, 1.0);
                let radius = (6.0 + progress * VC21_ELITE_NOVA_RADIUS).round() as u32;
                framebuffer.draw_circle(x, y, radius, CANTICLE_COLOR);
                if radius > 5 {
                    framebuffer.draw_circle(x, y, radius - 4, ART_GOLD);
                }
                framebuffer.fill_circle(x, y, 3, VC20_ARMOR_LIGHT);
            }
        }
    }

    fn render_version_overlay(&self, _framebuffer: &mut Framebuffer) {
        // Retired: presentation owns version/stage text.
    }

    fn render_build_info_overlay(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 97, &format!("VERSION {VC21_VERSION}"), CANTICLE_COLOR);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 177, "COMBAT MODEL", CANTICLE_COLOR);
    }
}

impl Game for VoidCanticleV21 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let was_game_over = self.base().game_over;
        let gameplay_active = self.combat.game.art_can_overlay_game();

        self.update_timers(if gameplay_active { dt } else { 0.0 });

        let lives_before = self.base().lives;
        let power_before = self.power_level();
        let impact_before = if gameplay_active {
            Some(self.capture_impact_snapshot())
        } else {
            None
        };
        let elites_before = if gameplay_active {
            self.snapshot_elites()
        } else {
            Vec::new()
        };

        let result = self.combat.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if was_game_over && !self.base().game_over {
            self.reset_combat_model();
        } else if gameplay_active {
            let legacy_hit = self.translate_legacy_player_hit(lives_before, power_before);
            if let Some(snapshot) = impact_before.as_ref() {
                self.detect_impact_events(snapshot);
            }
            self.detect_elite_deaths(&elites_before, legacy_hit);
            let nova_hit = self.update_elite_deaths(dt);
            self.update_shield_regen(dt, legacy_hit || nova_hit);
            self.process_combat_events(frame);
        }

        if self.combat.game.art_can_overlay_game() {
            self.render_version_overlay(frame.framebuffer);
            self.render_player_survival(frame.framebuffer);
            self.render_elite_deaths(frame.framebuffer);
        } else if matches!(&self.combat.game.ui.state, VcPauseState::BuildInfo) {
            self.render_build_info_overlay(frame.framebuffer);
        }

        GameResult::Continue
    }
}

fn vc21_apply_damage_to_layers(shield: &mut f32, hull: &mut f32, damage: f32) -> (f32, f32) {
    let damage = damage.max(0.0);
    let shield_damage = damage.min((*shield).max(0.0));
    *shield = (*shield - shield_damage).max(0.0);
    let remaining = damage - shield_damage;
    let hull_damage = remaining.min((*hull).max(0.0));
    *hull = (*hull - hull_damage).max(0.0);
    (shield_damage, hull_damage)
}

fn vc21_health_width(current: f32, maximum: f32, width: u32) -> u32 {
    if maximum <= 0.0 || current <= 0.0 {
        return 0;
    }
    ((current.min(maximum) / maximum) * width as f32).round() as u32
}

pub fn run_v21_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC21_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV21::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v21_tests {
    use super::*;

    #[test]
    fn vc21_version_is_explicit() {
        assert_eq!(VC21_VERSION, "VC2.1");
    }

    #[test]
    fn shield_absorbs_damage_before_hull() {
        let mut shield = VC21_PLAYER_SHIELD_MAX;
        let mut hull = VC21_PLAYER_HULL_MAX;
        let (shield_damage, hull_damage) =
            vc21_apply_damage_to_layers(&mut shield, &mut hull, 25.0);
        assert_eq!(shield_damage, 25.0);
        assert_eq!(hull_damage, 0.0);
        assert_eq!(shield, 25.0);
        assert_eq!(hull, VC21_PLAYER_HULL_MAX);
    }

    #[test]
    fn overflow_damage_reaches_hull() {
        let mut shield = 10.0;
        let mut hull = 100.0;
        let (shield_damage, hull_damage) =
            vc21_apply_damage_to_layers(&mut shield, &mut hull, 25.0);
        assert_eq!(shield_damage, 10.0);
        assert_eq!(hull_damage, 15.0);
        assert_eq!(shield, 0.0);
        assert_eq!(hull, 85.0);
    }

    #[test]
    fn combat_model_replaces_three_chances_with_damage_budget() {
        let total = VC21_PLAYER_HULL_MAX + VC21_PLAYER_SHIELD_MAX;
        assert!(total / VC21_LEGACY_IMPACT_DAMAGE > 3.0);
        assert!(VC21_SHIELD_REGEN_DELAY > 0.0);
        assert!(VC21_SHIELD_REGEN_PER_SECOND > 0.0);
    }

    #[test]
    fn elite_nova_is_dangerous_but_local() {
        assert!(VC21_ELITE_NOVA_DAMAGE > VC21_LEGACY_IMPACT_DAMAGE);
        assert!(VC21_ELITE_NOVA_RADIUS < FRAMEBUFFER_WIDTH as f32 / 2.0);
    }

    #[test]
    fn checked_in_manifest_covers_vc21_combat_events() {
        let manifest = gotoo_pixel_engine::SfxManifest::parse(VC20_SFX_MANIFEST)
            .expect("VC2.1 SFX manifest should parse");
        manifest
            .require_keys(&VC21_REQUIRED_SFX)
            .expect("VC2.1 SFX manifest should cover combat events");
    }

    #[test]
    fn hull_impact_fx_emits_local_burst_and_shards() {
        let mut game = VoidCanticleV21::new();
        let bursts_before = game.base().bursts.len();
        let particles_before = game.combat.game.ui.game.particles.len();

        game.spawn_enemy_hull_impact_fx(&[(90.0, 120.0)]);

        assert_eq!(game.base().bursts.len(), bursts_before + 1);
        assert_eq!(
            game.combat.game.ui.game.particles.len(),
            particles_before + VC21_ENEMY_HULL_IMPACT_SPARKS
        );
    }
}
