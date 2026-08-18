const VC15_VERSION: &str = "VC1.5";

const SYNERGY_CANTOR_STORM: u8 = 1 << 0;
const SYNERGY_TWIN_REQUIEM: u8 = 1 << 1;
const SYNERGY_GRAVITY_WAKE: u8 = 1 << 2;
const SYNERGY_CANTICLE_CHOIR: u8 = 1 << 3;

const SYNERGY_COLOR: Pixel = Pixel::rgb(92, 224, 238);
const SYNERGY_LIGHT: Pixel = Pixel::rgb(224, 255, 246);
const SYNERGY_GOLD: Pixel = Pixel::rgb(255, 206, 96);
const SYNERGY_BANNER_DURATION: f32 = 2.2;
const CHOIR_OVERCHARGE_DURATION: f32 = 2.6;

struct VoidCanticleV15 {
    combat: VoidCanticleV14,
    offer_nonce: u32,
    run_counter: u32,
    last_level_offer_tuned: u32,
    last_mutation_offer_tuned: u32,
    known_synergies: u8,
    synergy_banner_timer: f32,
    synergy_banner_name: Option<&'static str>,
    cantor_timer: f32,
    requiem_flash: f32,
    requiem_offset: f32,
    chorus_overcharge: f32,
    chorus_timer: f32,
}

impl VoidCanticleV15 {
    fn new() -> Self {
        Self {
            combat: VoidCanticleV14::new(),
            offer_nonce: 0xC41C_1E55,
            run_counter: 0,
            last_level_offer_tuned: 0,
            last_mutation_offer_tuned: 0,
            known_synergies: 0,
            synergy_banner_timer: 0.0,
            synergy_banner_name: None,
            cantor_timer: 0.0,
            requiem_flash: 0.0,
            requiem_offset: 0.0,
            chorus_overcharge: 0.0,
            chorus_timer: 0.0,
        }
    }

    fn reset_synergy_layer(&mut self) {
        self.run_counter = self.run_counter.saturating_add(1);
        self.offer_nonce = self
            .offer_nonce
            .wrapping_add(0x9E37_79B9_u32.wrapping_mul(self.run_counter.max(1)));
        self.last_level_offer_tuned = 0;
        self.last_mutation_offer_tuned = 0;
        self.known_synergies = 0;
        self.synergy_banner_timer = 0.0;
        self.synergy_banner_name = None;
        self.cantor_timer = 0.0;
        self.requiem_flash = 0.0;
        self.requiem_offset = 0.0;
        self.chorus_overcharge = 0.0;
        self.chorus_timer = 0.0;
    }

    fn reset_run(&mut self) {
        self.combat.reset_run();
        self.reset_synergy_layer();
    }

    fn build(&self) -> BuildState {
        self.combat.progression.build
    }

    fn mutations(&self) -> MutationBuild {
        self.combat.mutations
    }

    fn synergy_mask(&self) -> u8 {
        synergy_mask(self.build(), self.mutations())
    }

    fn next_offer_entropy(&mut self) -> u32 {
        let base = &self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base;
        let timing = base.animation_time.to_bits();
        let score = base.score;
        let level = self.combat.progression.level;
        self.offer_nonce = self
            .offer_nonce
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)
            ^ timing.rotate_left(7)
            ^ score.rotate_left(13)
            ^ level.wrapping_mul(0x45D9_F3B);
        self.offer_nonce
    }

    fn preferred_upgrade(&self) -> Option<UpgradeKind> {
        let build = self.build();
        let mutations = self.mutations();
        if mutations.split_volley > 0 && build.rapid_fire == 0 {
            return Some(UpgradeKind::RapidFire);
        }
        if mutations.piercing_lance > 0
            && build.stellar_power == 0
            && self.combat.progression.combat.combat.ui.inner.inner.power_level < MAX_POWER_LEVEL
        {
            return Some(UpgradeKind::StellarPower);
        }
        if mutations.death_nova > 0 && build.magnet_field == 0 {
            return Some(UpgradeKind::MagnetField);
        }
        if mutations.orbitals > 0 && build.core_surge == 0 {
            return Some(UpgradeKind::CoreSurge);
        }
        None
    }

    fn preferred_mutation(&self) -> Option<MutationKind> {
        let build = self.build();
        let mutations = self.mutations();
        if build.rapid_fire > 0 && mutations.split_volley == 0 {
            return Some(MutationKind::SplitVolley);
        }
        if build.stellar_power > 0 && mutations.piercing_lance == 0 {
            return Some(MutationKind::PiercingLance);
        }
        if build.magnet_field > 0 && mutations.death_nova == 0 {
            return Some(MutationKind::DeathNova);
        }
        if build.core_surge > 0 && mutations.orbitals == 0 {
            return Some(MutationKind::Orbitals);
        }
        None
    }

    fn retune_level_offer_if_needed(&mut self) {
        let level = self.combat.progression.level;
        if self.combat.progression.level_choice.is_none() || self.last_level_offer_tuned == level {
            return;
        }

        let preferred = self.preferred_upgrade();
        let seed = self.next_offer_entropy();
        let stellar_available =
            self.combat.progression.combat.combat.ui.inner.inner.power_level < MAX_POWER_LEVEL;
        let offers = controlled_upgrade_offers(seed, preferred, stellar_available);
        if let Some(choice) = self.combat.progression.level_choice.as_mut() {
            choice.offers = offers;
        }
        self.last_level_offer_tuned = level;
    }

    fn retune_mutation_offer_if_needed(&mut self) {
        let level = self.combat.progression.level;
        if self.combat.mutation_choice.is_none() || self.last_mutation_offer_tuned == level {
            return;
        }

        let preferred = self.preferred_mutation();
        let seed = self.next_offer_entropy();
        let offers = controlled_mutation_offers(seed, preferred);
        if let Some(choice) = self.combat.mutation_choice.as_mut() {
            choice.offers = offers;
        }
        self.last_mutation_offer_tuned = level;
    }

    fn refresh_synergy_discovery(&mut self, frame: &mut Frame<'_>) {
        let active = self.synergy_mask();
        let discovered = active & !self.known_synergies;
        self.known_synergies |= active;
        if discovered == 0 {
            return;
        }

        let name = first_synergy_name(discovered);
        self.synergy_banner_name = Some(name);
        self.synergy_banner_timer = SYNERGY_BANNER_DURATION;
        let base = &mut self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base;
        base.bursts.push(Burst::new(
            base.player_x,
            base.player_y - 4.0,
            0.52,
            SYNERGY_COLOR,
        ));
        let _ = base.sounds.play(frame.audio, POWERUP_SOUND);
    }

    fn gameplay_is_running(&self) -> bool {
        !self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .game_over
            && self.combat.progression.level_choice.is_none()
            && self.combat.mutation_choice.is_none()
    }

    fn fire_held(&self) -> bool {
        self.combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .controls
            .action(FIRE)
            .held()
    }

    fn update_cantor_storm(&mut self, dt: f32) {
        self.cantor_timer = (self.cantor_timer - dt).max(0.0);
        if self.synergy_mask() & SYNERGY_CANTOR_STORM == 0
            || !self.fire_held()
            || self.cantor_timer > 0.0
        {
            return;
        }

        let build = self.build();
        let mutations = self.mutations();
        let pairs = mutations.split_volley.min(3);
        let game = &mut self.combat.progression.combat.combat.ui.inner.inner;
        for pair in 0..pairs {
            let origin = 13.0 + pair as f32 * 5.0;
            let inward = 52.0 + pair as f32 * 18.0;
            for direction in [-1.0_f32, 1.0] {
                game.power_shots.push(PowerShot {
                    x: game.base.player_x + direction * origin,
                    y: game.base.player_y - 10.0,
                    vx: -direction * inward,
                    vy: -PLAYER_SHOT_SPEED * 0.90,
                    damage: 1,
                    radius: 1,
                    alive: true,
                });
            }
        }

        let stacks = build.rapid_fire.min(5) as f32;
        self.cantor_timer = (0.46 - stacks * 0.055).max(0.18);
    }

    fn maybe_fire_twin_requiem(&mut self, previous_piercing_flash: f32, frame: &mut Frame<'_>) {
        if self.synergy_mask() & SYNERGY_TWIN_REQUIEM == 0 {
            return;
        }
        if self.combat.piercing_flash <= previous_piercing_flash + 0.03 {
            return;
        }

        let stellar = self.build().stellar_power.max(1);
        let offset = 7.0 + stellar as f32 * 1.5;
        self.requiem_offset = offset;
        self.requiem_flash = 0.13;
        self.fire_requiem_echoes(offset, stellar, frame);
    }

    fn fire_requiem_echoes(&mut self, offset: f32, stellar: u32, frame: &mut Frame<'_>) {
        let damage = 1 + stellar / 2;
        let half_width = 2.0 + stellar as f32 * 0.7;
        let (center_x, player_y) = {
            let base = &self.combat.progression.combat.combat.ui.inner.inner.base;
            (base.player_x, base.player_y)
        };
        let lanes = [center_x - offset, center_x + offset];
        let mut carrion_destroyed = Vec::new();
        let mut special_destroyed = Vec::new();
        let mut threat_destroyed = Vec::new();
        let mut boss_damage = 0_u32;

        {
            let game = &mut self.combat.progression.combat.combat.ui.inner.inner;
            for enemy in &mut game.base.enemies {
                if enemy.alive
                    && lanes.iter().any(|lane| {
                        in_piercing_lane(enemy.x, enemy.y, *lane, player_y, half_width)
                    })
                {
                    enemy.alive = false;
                    carrion_destroyed.push((enemy.x, enemy.y));
                }
            }
            game.base.enemies.retain(|enemy| enemy.alive);

            if let Some(boss) = game.base.boss.as_mut()
                && game.base.encounter_phase == EncounterPhase::BossFight
                && lanes.iter().any(|lane| {
                    in_piercing_lane(boss.x, boss.y, *lane, player_y, 19.0 + half_width)
                })
            {
                boss.hp = boss.hp.saturating_sub(damage);
                boss_damage = damage;
            }
        }

        {
            let specials = &mut self.combat.progression.combat.combat.specials;
            for enemy in specials.iter_mut() {
                if !enemy.alive
                    || !lanes.iter().any(|lane| {
                        in_piercing_lane(enemy.x, enemy.y, *lane, player_y, half_width)
                    })
                {
                    continue;
                }
                enemy.hp = enemy.hp.saturating_sub(damage);
                if enemy.hp == 0 {
                    enemy.alive = false;
                    special_destroyed.push((enemy.kind, enemy.x, enemy.y));
                }
            }
            specials.retain(|enemy| enemy.alive);
        }

        {
            let threats = &mut self.combat.progression.combat.threats;
            for threat in threats.iter_mut() {
                if !threat.alive
                    || !lanes.iter().any(|lane| {
                        in_piercing_lane(threat.x, threat.y, *lane, player_y, half_width)
                    })
                {
                    continue;
                }
                threat.hp = threat.hp.saturating_sub(damage);
                if threat.hp == 0 {
                    threat.alive = false;
                    threat_destroyed.push((threat.kind, threat.x, threat.y));
                }
            }
            threats.retain(|threat| threat.alive);
        }

        self.combat.reward_piercing_kills(
            &carrion_destroyed,
            &special_destroyed,
            &threat_destroyed,
            boss_damage,
            frame,
        );
    }

    fn apply_gravity_wake(&mut self) {
        if self.synergy_mask() & SYNERGY_GRAVITY_WAKE == 0 {
            return;
        }

        let nova_positions: Vec<(f32, f32)> = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .bursts
            .iter()
            .filter(|burst| {
                (burst.remaining - burst.duration).abs() <= 0.0001
                    && (burst.duration - 0.19).abs() < 0.0001
            })
            .map(|burst| (burst.x, burst.y))
            .collect();
        if nova_positions.is_empty() {
            return;
        }

        let player_x = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .player_x;
        let player_y = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .player_y;
        let radius = self.combat.progression.magnet_radius()
            + self.combat.mutations.death_nova.min(4) as f32 * 12.0;
        let radius_sq = radius * radius;

        for orb in &mut self.combat.progression.xp_orbs {
            if nova_positions.iter().any(|(x, y)| {
                let dx = orb.x - *x;
                let dy = orb.y - *y;
                dx * dx + dy * dy <= radius_sq
            }) {
                let dx = player_x - orb.x;
                let dy = player_y - orb.y;
                let distance = (dx * dx + dy * dy).sqrt().max(0.001);
                orb.vx = dx / distance * 330.0;
                orb.vy = dy / distance * 330.0;
            }
        }

        let base = &mut self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base;
        for (x, y) in nova_positions {
            base.bursts.push(Burst::new(x, y, 0.22, SYNERGY_COLOR));
        }
    }

    fn update_canticle_choir(&mut self, dt: f32, canticle_was_active: bool) {
        self.chorus_overcharge = (self.chorus_overcharge - dt).max(0.0);
        self.chorus_timer = (self.chorus_timer - dt).max(0.0);
        if self.synergy_mask() & SYNERGY_CANTICLE_CHOIR == 0 {
            return;
        }

        let canticle_active = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .canticle_timer
            > 0.0;
        if !canticle_was_active && canticle_active {
            self.chorus_overcharge = CHOIR_OVERCHARGE_DURATION;
            self.chorus_timer = 0.0;
            let base = &mut self
                .combat
                .progression
                .combat
                .combat
                .ui
                .inner
                .inner
                .base;
            base.bursts.push(Burst::new(
                base.player_x,
                base.player_y - 4.0,
                0.56,
                SYNERGY_GOLD,
            ));
        }

        if self.chorus_overcharge <= 0.0 || !self.fire_held() || self.chorus_timer > 0.0 {
            return;
        }

        let positions = self.combat.orbital_positions();
        let game = &mut self.combat.progression.combat.combat.ui.inner.inner;
        for (x, y) in positions {
            for vx in [-72.0_f32, 0.0, 72.0] {
                game.power_shots.push(PowerShot {
                    x,
                    y: y - 4.0,
                    vx,
                    vy: -PLAYER_SHOT_SPEED * 0.90,
                    damage: 1,
                    radius: 1,
                    alive: true,
                });
            }
        }

        let core_stacks = self.build().core_surge.min(5) as f32;
        self.chorus_timer = (0.23 - core_stacks * 0.02).max(0.11);
    }

    fn render_synergy_fx(&self, framebuffer: &mut Framebuffer) {
        if self.requiem_flash > 0.0 {
            let base = &self.combat.progression.combat.combat.ui.inner.inner.base;
            let center = base.player_x.round() as i32;
            let offset = self.requiem_offset.round() as i32;
            let bottom = (base.player_y - 10.0).round() as i32;
            for x in [center - offset, center + offset] {
                framebuffer.draw_line(x - 1, bottom, x - 1, 20, SYNERGY_COLOR);
                framebuffer.draw_line(x, bottom, x, 20, SYNERGY_LIGHT);
                framebuffer.draw_line(x + 1, bottom, x + 1, 20, SYNERGY_COLOR);
            }
        }

        if self.chorus_overcharge > 0.0 {
            for (x, y) in self.combat.orbital_positions() {
                let x = x.round() as i32;
                let y = y.round() as i32;
                framebuffer.draw_circle(x, y, 7, SYNERGY_GOLD);
                framebuffer.draw_circle(x, y, 9, SYNERGY_COLOR);
            }
        }

        if self.synergy_banner_timer > 0.0
            && let Some(name) = self.synergy_banner_name
        {
            gotoo_pixel_engine::ui::draw_panel(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 12,
                    y: 34,
                    width: 156,
                    height: 34,
                },
                Pixel::rgb(7, 15, 20),
                SYNERGY_COLOR,
            );
            framebuffer.draw_text(61, 41, "SYNERGY", SYNERGY_LIGHT);
            let text_width = name.len() as i32 * 6;
            let x = ((FRAMEBUFFER_WIDTH as i32 - text_width) / 2).max(16);
            framebuffer.draw_text(x, 54, name, SYNERGY_GOLD);
        }
    }

    fn active_synergy_count(&self) -> u32 {
        self.synergy_mask().count_ones()
    }
}

impl Game for VoidCanticleV15 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let was_game_over = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .game_over;
        let canticle_was_active = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .canticle_timer
            > 0.0;
        let previous_piercing_flash = self.combat.piercing_flash;
        let dt = frame.delta_time.as_secs_f32().min(0.05);

        let result = self.combat.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let game_over = self
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .game_over;
        if was_game_over && !game_over {
            self.reset_synergy_layer();
        }

        self.retune_level_offer_if_needed();
        self.retune_mutation_offer_if_needed();
        self.refresh_synergy_discovery(frame);

        self.synergy_banner_timer = (self.synergy_banner_timer - dt).max(0.0);
        self.requiem_flash = (self.requiem_flash - dt).max(0.0);

        if self.gameplay_is_running() {
            self.update_cantor_storm(dt);
            self.maybe_fire_twin_requiem(previous_piercing_flash, frame);
            self.apply_gravity_wake();
            self.update_canticle_choir(dt, canticle_was_active);
        }

        self.render_synergy_fx(frame.framebuffer);
        GameResult::Continue
    }
}

fn synergy_mask(build: BuildState, mutations: MutationBuild) -> u8 {
    let mut mask = 0_u8;
    if build.rapid_fire > 0 && mutations.split_volley > 0 {
        mask |= SYNERGY_CANTOR_STORM;
    }
    if build.stellar_power > 0 && mutations.piercing_lance > 0 {
        mask |= SYNERGY_TWIN_REQUIEM;
    }
    if build.magnet_field > 0 && mutations.death_nova > 0 {
        mask |= SYNERGY_GRAVITY_WAKE;
    }
    if build.core_surge > 0 && mutations.orbitals > 0 {
        mask |= SYNERGY_CANTICLE_CHOIR;
    }
    mask
}

fn first_synergy_name(mask: u8) -> &'static str {
    if mask & SYNERGY_CANTOR_STORM != 0 {
        "CANTOR STORM"
    } else if mask & SYNERGY_TWIN_REQUIEM != 0 {
        "TWIN REQUIEM"
    } else if mask & SYNERGY_GRAVITY_WAKE != 0 {
        "GRAVITY WAKE"
    } else {
        "CANTICLE CHOIR"
    }
}

fn controlled_upgrade_offers(
    seed: u32,
    preferred: Option<UpgradeKind>,
    stellar_available: bool,
) -> [UpgradeKind; 3] {
    let mut offers = [UpgradeKind::CoreSurge; 3];
    let mut count = 0_usize;

    if let Some(candidate) = preferred
        && (candidate != UpgradeKind::StellarPower || stellar_available)
    {
        offers[count] = candidate;
        count += 1;
    }

    let start = seed as usize % UPGRADE_POOL.len();
    for offset in 0..UPGRADE_POOL.len() * 2 {
        let candidate = UPGRADE_POOL[(start + offset * 5) % UPGRADE_POOL.len()];
        if candidate == UpgradeKind::StellarPower && !stellar_available {
            continue;
        }
        if offers[..count].contains(&candidate) {
            continue;
        }
        offers[count] = candidate;
        count += 1;
        if count == offers.len() {
            break;
        }
    }

    debug_assert_eq!(count, offers.len());
    offers
}

fn controlled_mutation_offers(seed: u32, preferred: Option<MutationKind>) -> [MutationKind; 3] {
    let mut offers = [MutationKind::Orbitals; 3];
    let mut count = 0_usize;

    if let Some(candidate) = preferred {
        offers[count] = candidate;
        count += 1;
    }

    let start = seed as usize % MUTATION_POOL.len();
    for offset in 0..MUTATION_POOL.len() * 2 {
        let candidate = MUTATION_POOL[(start + offset * 3) % MUTATION_POOL.len()];
        if offers[..count].contains(&candidate) {
            continue;
        }
        offers[count] = candidate;
        count += 1;
        if count == offers.len() {
            break;
        }
    }

    debug_assert_eq!(count, offers.len());
    offers
}

struct VoidCanticlePauseV15 {
    game: VoidCanticleV15,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV15 {
    fn new(game: VoidCanticleV15) -> Self {
        let mut controls = gotoo_pixel_engine::ui::standard_menu_controls(
            VC_PAUSE_UP,
            VC_PAUSE_DOWN,
            VC_PAUSE_CONFIRM,
        );
        controls
            .bind_key(VC_PAUSE_TOGGLE, Key::Escape)
            .bind_gamepad(VC_PAUSE_TOGGLE, GamepadButton::Start);
        Self {
            game,
            state: VcPauseState::Running,
            menu: gotoo_pixel_engine::ui::MenuState::new(5),
            controls,
        }
    }

    fn pause_input_held(&self) -> bool {
        [VC_PAUSE_TOGGLE, VC_PAUSE_UP, VC_PAUSE_DOWN, VC_PAUSE_CONFIRM]
            .into_iter()
            .any(|action| self.controls.action(action).held())
    }

    fn render_menu(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 18,
                y: 48,
                width: 144,
                height: 220,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(60, 62, "PAUSED", POWER_RELIC_LIGHT);

        for (index, (y, label)) in [
            (90, "RESUME"),
            (118, "RESTART"),
            (146, "CONTROLS"),
            (174, "BUILD INFO"),
            (202, "QUIT"),
        ]
        .into_iter()
        .enumerate()
        {
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 34,
                    y,
                    width: 112,
                    height: 18,
                },
                label,
                self.menu.selected() == Some(index),
                1,
                TEXT,
                POWER_RELIC_LIGHT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 10,
                y: 46,
                width: 160,
                height: 224,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(52, 59, "CONTROLS", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 86, "MOVE  ARROWS WASD", TEXT);
        framebuffer.draw_text(20, 106, "FIRE  SPACE SOUTH", TEXT);
        framebuffer.draw_text(20, 126, "FOCUS SHIFT LB", TEXT);
        framebuffer.draw_text(20, 146, "CANTICLE X EAST", TEXT);
        framebuffer.draw_text(20, 166, "PAUSE ESC START", TEXT);
        framebuffer.draw_text(31, 231, "ESC START  BACK", WRECK_LIGHT);
    }

    fn render_build_info(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 10,
                y: 58,
                width: 160,
                height: 196,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(46, 72, "BUILD INFO", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 103, &format!("VERSION {VC15_VERSION}"), TEXT);
        framebuffer.draw_text(20, 123, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(
            20,
            143,
            &format!("SYNERGIES {}", self.game.active_synergy_count()),
            SYNERGY_LIGHT,
        );
        framebuffer.draw_text(20, 163, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(31, 219, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV15 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);

        match self.state {
            VcPauseState::Running => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::Menu;
                    self.menu = gotoo_pixel_engine::ui::MenuState::new(5);
                    self.render_menu(frame.framebuffer);
                    GameResult::Continue
                } else {
                    self.game.update(frame)
                }
            }
            VcPauseState::Menu => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::ResumeGate;
                } else {
                    if self.controls.action(VC_PAUSE_UP).pressed() {
                        self.menu.select_previous();
                    }
                    if self.controls.action(VC_PAUSE_DOWN).pressed() {
                        self.menu.select_next();
                    }
                    if self.controls.action(VC_PAUSE_CONFIRM).pressed() {
                        match self.menu.selected() {
                            Some(0) => self.state = VcPauseState::ResumeGate,
                            Some(1) => {
                                self.game.reset_run();
                                self.state = VcPauseState::ResumeGate;
                            }
                            Some(2) => self.state = VcPauseState::Controls,
                            Some(3) => self.state = VcPauseState::BuildInfo,
                            Some(4) => return GameResult::Exit,
                            _ => {}
                        }
                    }
                }

                match self.state {
                    VcPauseState::Controls => self.render_controls(frame.framebuffer),
                    VcPauseState::BuildInfo => self.render_build_info(frame.framebuffer),
                    _ => self.render_menu(frame.framebuffer),
                }
                GameResult::Continue
            }
            VcPauseState::Controls => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_controls(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::BuildInfo => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_build_info(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::ResumeGate => {
                if self.pause_input_held() {
                    self.render_menu(frame.framebuffer);
                } else {
                    self.state = VcPauseState::Running;
                }
                GameResult::Continue
            }
        }
    }
}

pub fn run_v15_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePauseV15::new(VoidCanticleV15::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v15_tests {
    use super::*;

    #[test]
    fn synergies_require_both_halves() {
        let build = BuildState {
            rapid_fire: 1,
            magnet_field: 1,
            stellar_power: 1,
            core_surge: 1,
            ..BuildState::default()
        };
        assert_eq!(synergy_mask(build, MutationBuild::default()), 0);

        let mutations = MutationBuild {
            split_volley: 1,
            piercing_lance: 1,
            death_nova: 1,
            orbitals: 1,
        };
        assert_eq!(synergy_mask(build, mutations).count_ones(), 4);
    }

    #[test]
    fn controlled_upgrade_offer_preserves_requested_build_path() {
        let offers = controlled_upgrade_offers(0x1234, Some(UpgradeKind::MagnetField), true);
        assert!(offers.contains(&UpgradeKind::MagnetField));
        assert_ne!(offers[0], offers[1]);
        assert_ne!(offers[0], offers[2]);
        assert_ne!(offers[1], offers[2]);
    }

    #[test]
    fn controlled_upgrade_offer_respects_stellar_cap() {
        let offers = controlled_upgrade_offers(0x4321, Some(UpgradeKind::StellarPower), false);
        assert!(!offers.contains(&UpgradeKind::StellarPower));
    }

    #[test]
    fn controlled_mutation_offer_preserves_requested_build_path() {
        let offers = controlled_mutation_offers(0xBEEF, Some(MutationKind::Orbitals));
        assert!(offers.contains(&MutationKind::Orbitals));
        assert_ne!(offers[0], offers[1]);
        assert_ne!(offers[0], offers[2]);
        assert_ne!(offers[1], offers[2]);
    }

    #[test]
    fn vc15_version_is_explicit() {
        assert_eq!(VC15_VERSION, "VC1.5");
    }
}
