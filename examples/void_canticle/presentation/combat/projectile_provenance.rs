const VC27_PROJECTILE_MATCH_TOLERANCE: f32 = 1.5;
const VC27_SOURCE_MATCH_RADIUS: f32 = 24.0;
const VC27_BOSS_SOURCE_MATCH_RADIUS: f32 = 52.0;
const VC27_TIMER_RESET_EPSILON: f32 = 0.20;

#[derive(Debug, Clone, Copy)]
struct Vc27TimedProjectileSource {
    timer: f32,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Vc27LeechProjectileSource {
    charge: u32,
    x: f32,
    y: f32,
}

#[derive(Default)]
struct Vc27ProjectileSourceSnapshot {
    carrion: std::collections::BTreeMap<CarrionDefenseKey, Vc27TimedProjectileSource>,
    wraith: std::collections::BTreeMap<SpecialDefenseKey, Vc27TimedProjectileSource>,
    leech: std::collections::BTreeMap<ThreatDefenseKey, Vc27LeechProjectileSource>,
    boss: Option<Vc27TimedProjectileSource>,
    pending_void: Option<VoidAttackKind>,
}

impl Vc27ProjectileSourceSnapshot {
    fn capture(game: &VoidCanticleV23Sustain) -> Self {
        let mut snapshot = Self::default();
        let v20 = game.game.v20();

        for enemy in game.game.base().enemies.iter().filter(|enemy| enemy.alive) {
            snapshot.carrion.insert(
                vc20_carrion_key(enemy),
                Vc27TimedProjectileSource {
                    timer: enemy.fire_timer,
                    x: enemy.x,
                    y: enemy.y,
                },
            );
        }

        let v12 = &v20.game.v12();
        for enemy in v12
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive && enemy.kind == SpecialKind::BellWraith)
        {
            snapshot.wraith.insert(
                vc20_special_key(enemy),
                Vc27TimedProjectileSource {
                    timer: enemy.fire_timer,
                    x: enemy.x,
                    y: enemy.y,
                },
            );
        }

        for threat in v12
            .threats
            .iter()
            .filter(|threat| threat.alive && threat.kind == ThreatKind::VoidLeech)
        {
            snapshot.leech.insert(
                vc20_threat_key(threat),
                Vc27LeechProjectileSource {
                    charge: threat.charge,
                    x: threat.x,
                    y: threat.y,
                },
            );
        }

        snapshot.boss = game.game.base().boss.map(|boss| Vc27TimedProjectileSource {
            timer: boss.shot_timer,
            x: boss.x,
            y: boss.y,
        });
        snapshot.pending_void = v20
            .game
            .ui
            .game
            .combat
            .combat
            .pending_attack
            .as_ref()
            .map(|attack| attack.kind);
        snapshot
    }
}

#[derive(Default)]
struct Vc27AttackSourceEvents {
    carrion: Vec<(CarrionDefenseKey, f32, f32)>,
    wraith: Vec<(SpecialDefenseKey, f32, f32)>,
    leech: Vec<(ThreatDefenseKey, f32, f32)>,
    void_attack: Option<VoidAttackKind>,
    bellkeeper: Option<(f32, f32)>,
}

impl Vc27AttackSourceEvents {
    fn detect(before: &Vc27ProjectileSourceSnapshot, game: &VoidCanticleV23Sustain) -> Self {
        let after = Vc27ProjectileSourceSnapshot::capture(game);
        let mut events = Self::default();

        for (key, current) in &after.carrion {
            if before.carrion.get(key).is_some_and(|previous| {
                current.timer > previous.timer + VC27_TIMER_RESET_EPSILON
            }) {
                events.carrion.push((*key, current.x, current.y));
            }
        }

        for (key, current) in &after.wraith {
            if before.wraith.get(key).is_some_and(|previous| {
                current.timer > previous.timer + VC27_TIMER_RESET_EPSILON
            }) {
                events.wraith.push((*key, current.x, current.y));
            }
        }

        for (key, current) in &after.leech {
            if before
                .leech
                .get(key)
                .is_some_and(|previous| current.charge < previous.charge)
            {
                events.leech.push((*key, current.x, current.y));
            }
        }

        if let (Some(previous), Some(current)) = (before.boss, after.boss)
            && current.timer > previous.timer + VC27_TIMER_RESET_EPSILON
        {
            events.bellkeeper = Some((current.x, current.y));
        }

        if let Some(kind) = before.pending_void
            && after.pending_void != before.pending_void
        {
            events.void_attack = Some(kind);
        }

        events
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc27ProjectileSource {
    Carrion(CarrionDefenseKey),
    Wraith(SpecialDefenseKey),
    VoidLeech(ThreatDefenseKey),
    VoidAttack(VoidAttackKind),
    Bellkeeper,
    Legacy(Vc27EnemyShotStyle),
}

impl Vc27ProjectileSource {
    fn style(self) -> Vc27EnemyShotStyle {
        match self {
            Self::Carrion(_) => Vc27EnemyShotStyle::Carrion,
            Self::Wraith(_) => Vc27EnemyShotStyle::Wraith,
            Self::VoidLeech(_) => Vc27EnemyShotStyle::VoidPulse,
            Self::VoidAttack(_) => Vc27EnemyShotStyle::Void,
            Self::Bellkeeper => Vc27EnemyShotStyle::Bellkeeper,
            Self::Legacy(style) => style,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Vc27TrackedProjectile {
    bullet: Bullet,
    source: Vc27ProjectileSource,
}

#[derive(Default)]
struct Vc27ProjectileProvenance {
    tracked: Vec<Vc27TrackedProjectile>,
}

impl Vc27ProjectileProvenance {
    fn reconcile(
        &mut self,
        dt: f32,
        before: &Vc27ProjectileSourceSnapshot,
        game: &VoidCanticleV23Sustain,
    ) -> Vec<Vc23AttackSound> {
        let events = Vc27AttackSourceEvents::detect(before, game);
        let encounter_phase = game.game.base().encounter_phase;
        let bullets = &game.game.base().enemy_bullets;
        let previous = std::mem::take(&mut self.tracked);
        let mut previous_index = 0;
        let mut next = Vec::with_capacity(bullets.len());
        let mut sounds = Vec::new();

        for bullet in bullets {
            let mut retained_source = None;
            while previous_index < previous.len() {
                let candidate = previous[previous_index];
                previous_index += 1;
                if vc27_projectile_matches(candidate.bullet, *bullet, dt) {
                    retained_source = Some(candidate.source);
                    break;
                }
            }

            let source = if let Some(source) = retained_source {
                source
            } else {
                let source = vc27_classify_new_projectile(*bullet, dt, encounter_phase, &events);
                let sound = vc27_attack_sound_for_style(source.style());
                if !sounds.contains(&sound) {
                    sounds.push(sound);
                }
                source
            };

            next.push(Vc27TrackedProjectile {
                bullet: *bullet,
                source,
            });
        }

        self.tracked = next;
        sounds
    }

    fn style_for(&self, index: usize) -> Option<Vc27EnemyShotStyle> {
        self.tracked.get(index).map(|tracked| tracked.source.style())
    }
}

fn vc27_projectile_matches(previous: Bullet, current: Bullet, dt: f32) -> bool {
    if previous.alternate != current.alternate
        || (previous.vx - current.vx).abs() > 0.01
        || (previous.vy - current.vy).abs() > 0.01
    {
        return false;
    }

    let stationary_dx = current.x - previous.x;
    let stationary_dy = current.y - previous.y;
    let moved_dx = current.x - (previous.x + previous.vx * dt);
    let moved_dy = current.y - (previous.y + previous.vy * dt);
    let tolerance_sq = VC27_PROJECTILE_MATCH_TOLERANCE * VC27_PROJECTILE_MATCH_TOLERANCE;

    stationary_dx * stationary_dx + stationary_dy * stationary_dy <= tolerance_sq
        || moved_dx * moved_dx + moved_dy * moved_dy <= tolerance_sq
}

fn vc27_classify_new_projectile(
    bullet: Bullet,
    dt: f32,
    encounter_phase: EncounterPhase,
    events: &Vc27AttackSourceEvents,
) -> Vc27ProjectileSource {
    let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt().max(1.0);

    if let Some(kind) = events.void_attack {
        if kind == VoidAttackKind::BlackSun && (speed - 62.0).abs() <= 1.0 {
            let void_distance = vc27_projectile_source_distance_sq(
                bullet,
                dt,
                FRAMEBUFFER_WIDTH as f32 / 2.0,
                72.0,
            );
            let leech = vc27_nearest_source(
                bullet,
                dt,
                &events.leech,
                VC27_SOURCE_MATCH_RADIUS,
            );
            if let Some((key, leech_distance)) = leech
                && leech_distance < void_distance
            {
                return Vc27ProjectileSource::VoidLeech(key);
            }
            return Vc27ProjectileSource::VoidAttack(kind);
        }

        if vc23_void_attack_speed(speed) {
            return Vc27ProjectileSource::VoidAttack(kind);
        }
    }

    if (speed - 48.0).abs() <= 1.0
        && let Some((key, _)) = vc27_nearest_source(
            bullet,
            dt,
            &events.wraith,
            VC27_SOURCE_MATCH_RADIUS,
        )
    {
        return Vc27ProjectileSource::Wraith(key);
    }

    if (speed - 62.0).abs() <= 1.0
        && let Some((key, _)) = vc27_nearest_source(
            bullet,
            dt,
            &events.leech,
            VC27_SOURCE_MATCH_RADIUS,
        )
    {
        return Vc27ProjectileSource::VoidLeech(key);
    }

    if (speed - ENEMY_SHOT_SPEED).abs() <= 1.0
        && let Some((key, _)) = vc27_nearest_source(
            bullet,
            dt,
            &events.carrion,
            VC27_SOURCE_MATCH_RADIUS,
        )
    {
        return Vc27ProjectileSource::Carrion(key);
    }

    if encounter_phase == EncounterPhase::BossFight
        && let Some((x, y)) = events.bellkeeper
        && vc27_projectile_source_distance_sq(bullet, dt, x, y)
            <= VC27_BOSS_SOURCE_MATCH_RADIUS * VC27_BOSS_SOURCE_MATCH_RADIUS
    {
        return Vc27ProjectileSource::Bellkeeper;
    }

    Vc27ProjectileSource::Legacy(vc27_enemy_shot_style(encounter_phase, speed))
}

fn vc27_nearest_source<K: Copy>(
    bullet: Bullet,
    dt: f32,
    sources: &[(K, f32, f32)],
    max_distance: f32,
) -> Option<(K, f32)> {
    let max_distance_sq = max_distance * max_distance;
    sources
        .iter()
        .map(|(key, x, y)| {
            (
                *key,
                vc27_projectile_source_distance_sq(bullet, dt, *x, *y),
            )
        })
        .filter(|(_, distance)| *distance <= max_distance_sq)
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
}

fn vc27_projectile_source_distance_sq(bullet: Bullet, dt: f32, x: f32, y: f32) -> f32 {
    let current_dx = bullet.x - x;
    let current_dy = bullet.y - y;
    let backtracked_dx = bullet.x - bullet.vx * dt - x;
    let backtracked_dy = bullet.y - bullet.vy * dt - y;
    (current_dx * current_dx + current_dy * current_dy)
        .min(backtracked_dx * backtracked_dx + backtracked_dy * backtracked_dy)
}

fn vc27_attack_sound_for_style(style: Vc27EnemyShotStyle) -> Vc23AttackSound {
    match style {
        Vc27EnemyShotStyle::Carrion => Vc23AttackSound::Carrion,
        Vc27EnemyShotStyle::Wraith => Vc23AttackSound::Wraith,
        Vc27EnemyShotStyle::VoidPulse => Vc23AttackSound::VoidPulse,
        Vc27EnemyShotStyle::Void => Vc23AttackSound::Void,
        Vc27EnemyShotStyle::Bellkeeper => Vc23AttackSound::Bellkeeper,
    }
}

struct Vc27LegacyAttackAudioFilter<'a> {
    inner: &'a mut dyn gotoo_pixel_engine::Audio,
}

impl<'a> Vc27LegacyAttackAudioFilter<'a> {
    fn new(inner: &'a mut dyn gotoo_pixel_engine::Audio) -> Self {
        Self { inner }
    }
}

impl gotoo_pixel_engine::Audio for Vc27LegacyAttackAudioFilter<'_> {
    fn register_wav(
        &mut self,
        id: SoundId,
        bytes: &[u8],
    ) -> Result<(), gotoo_pixel_engine::AudioError> {
        self.inner.register_wav(id, bytes)
    }

    fn play(&mut self, id: SoundId) -> Result<(), gotoo_pixel_engine::AudioError> {
        if vc27_is_family_attack_sound(id) {
            Ok(())
        } else {
            self.inner.play(id)
        }
    }
}

fn vc27_is_family_attack_sound(id: SoundId) -> bool {
    [
        VC23_CARRION_FIRE_SOUND,
        VC23_WRAITH_FIRE_SOUND,
        VC23_VOID_PULSE_FIRE_SOUND,
        VC23_VOID_FIRE_SOUND,
        VC23_BELLKEEPER_FIRE_SOUND,
    ]
    .contains(&id)
}

#[cfg(test)]
mod v27_projectile_provenance_tests {
    use super::*;

    fn bullet(x: f32, y: f32, vx: f32, vy: f32) -> Bullet {
        Bullet {
            x,
            y,
            vx,
            vy,
            alive: true,
            alternate: false,
        }
    }

    #[test]
    fn projectile_match_keeps_identity_across_expected_motion() {
        let previous = bullet(40.0, 80.0, 0.0, 68.0);
        let current = bullet(40.0, 81.7, 0.0, 68.0);
        assert!(vc27_projectile_matches(previous, current, 0.025));
    }

    #[test]
    fn black_sun_and_leech_share_speed_but_source_origin_disambiguates_them() {
        let leech_key: ThreatDefenseKey = (1, 12, 34, 56);
        let events = Vc27AttackSourceEvents {
            leech: vec![(leech_key, 128.0, 96.0)],
            void_attack: Some(VoidAttackKind::BlackSun),
            ..Vc27AttackSourceEvents::default()
        };

        assert_eq!(
            vc27_classify_new_projectile(
                bullet(FRAMEBUFFER_WIDTH as f32 / 2.0, 72.0, 0.0, 62.0),
                0.016,
                EncounterPhase::Waves,
                &events,
            ),
            Vc27ProjectileSource::VoidAttack(VoidAttackKind::BlackSun)
        );
        assert_eq!(
            vc27_classify_new_projectile(
                bullet(128.0, 96.0, 0.0, 62.0),
                0.016,
                EncounterPhase::Waves,
                &events,
            ),
            Vc27ProjectileSource::VoidLeech(leech_key)
        );
    }

    #[test]
    fn tracked_source_keeps_visual_identity_when_encounter_phase_changes() {
        let source = Vc27ProjectileSource::Wraith((1, 2, 3, 4));
        assert_eq!(source.style(), Vc27EnemyShotStyle::Wraith);
        assert_ne!(source.style(), Vc27EnemyShotStyle::Bellkeeper);
    }

    #[test]
    fn legacy_family_audio_filter_names_only_family_accents() {
        assert!(vc27_is_family_attack_sound(VC23_WRAITH_FIRE_SOUND));
        assert!(vc27_is_family_attack_sound(VC23_BELLKEEPER_FIRE_SOUND));
        assert!(!vc27_is_family_attack_sound(ENEMY_FIRE_SOUND));
    }
}
