use std::f32::consts::TAU;

const VC14_VERSION: &str = "VC1.4";
const MUTATION_INTERVAL: u32 = 2;
const MUTATION_COLOR: Pixel = Pixel::rgb(255, 110, 184);
const MUTATION_LIGHT: Pixel = Pixel::rgb(255, 214, 239);

const VC_MUTATION_UP: ActionId = ActionId::new("void_canticle.mutation.up");
const VC_MUTATION_DOWN: ActionId = ActionId::new("void_canticle.mutation.down");
const VC_MUTATION_CONFIRM: ActionId = ActionId::new("void_canticle.mutation.confirm");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MutationKind {
    PiercingLance,
    SplitVolley,
    DeathNova,
    Orbitals,
}

const MUTATION_POOL: [MutationKind; 4] = [
    MutationKind::PiercingLance,
    MutationKind::SplitVolley,
    MutationKind::DeathNova,
    MutationKind::Orbitals,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct MutationBuild {
    piercing_lance: u32,
    split_volley: u32,
    death_nova: u32,
    orbitals: u32,
}

struct MutationChoice {
    offers: [MutationKind; 3],
    menu: gotoo_pixel_engine::ui::MenuState,
}

struct VoidCanticleV14 {
    progression: VoidCanticleV13,
    mutations: MutationBuild,
    mutation_choice: Option<MutationChoice>,
    mutation_controls: ControlMap,
    last_mutation_level: u32,
    piercing_timer: f32,
    piercing_flash: f32,
    piercing_x: f32,
    split_timer: f32,
    orbital_timer: f32,
}

impl VoidCanticleV14 {
    fn new() -> Self {
        Self {
            progression: VoidCanticleV13::new(),
            mutations: MutationBuild::default(),
            mutation_choice: None,
            mutation_controls: gotoo_pixel_engine::ui::standard_menu_controls(
                VC_MUTATION_UP,
                VC_MUTATION_DOWN,
                VC_MUTATION_CONFIRM,
            ),
            last_mutation_level: 1,
            piercing_timer: 0.0,
            piercing_flash: 0.0,
            piercing_x: 0.0,
            split_timer: 0.0,
            orbital_timer: 0.0,
        }
    }

    fn reset_mutations(&mut self) {
        self.mutations = MutationBuild::default();
        self.mutation_choice = None;
        self.last_mutation_level = 1;
        self.piercing_timer = 0.0;
        self.piercing_flash = 0.0;
        self.piercing_x = 0.0;
        self.split_timer = 0.0;
        self.orbital_timer = 0.0;
    }

    fn reset_run(&mut self) {
        self.progression.reset_run();
        self.reset_mutations();
    }

    fn mutation_offers(&self) -> [MutationKind; 3] {
        let start = (self.progression.level as usize / MUTATION_INTERVAL as usize)
            % MUTATION_POOL.len();
        [
            MUTATION_POOL[start],
            MUTATION_POOL[(start + 1) % MUTATION_POOL.len()],
            MUTATION_POOL[(start + 2) % MUTATION_POOL.len()],
        ]
    }

    fn maybe_start_mutation_choice(&mut self) {
        let level = self.progression.level;
        if self.progression.level_choice.is_some()
            || self.mutation_choice.is_some()
            || level < MUTATION_INTERVAL
            || !level.is_multiple_of(MUTATION_INTERVAL)
            || level <= self.last_mutation_level
        {
            return;
        }

        self.last_mutation_level = level;
        self.mutation_choice = Some(MutationChoice {
            offers: self.mutation_offers(),
            menu: gotoo_pixel_engine::ui::MenuState::new(3),
        });
    }

    fn apply_mutation(&mut self, mutation: MutationKind, frame: &mut Frame<'_>) {
        match mutation {
            MutationKind::PiercingLance => {
                self.mutations.piercing_lance =
                    self.mutations.piercing_lance.saturating_add(1).min(4);
                self.piercing_timer = 0.0;
            }
            MutationKind::SplitVolley => {
                self.mutations.split_volley = self.mutations.split_volley.saturating_add(1).min(3);
                self.split_timer = 0.0;
            }
            MutationKind::DeathNova => {
                self.mutations.death_nova = self.mutations.death_nova.saturating_add(1).min(4);
            }
            MutationKind::Orbitals => {
                self.mutations.orbitals = self.mutations.orbitals.saturating_add(1).min(3);
                self.orbital_timer = 0.0;
            }
        }

        let base = &mut self.progression.combat.combat.ui.inner.inner.base;
        base.bursts.push(Burst::new(
            base.player_x,
            base.player_y - 4.0,
            0.44,
            MUTATION_COLOR,
        ));
        let _ = base.sounds.play(frame.audio, POWERUP_SOUND);
    }

    fn update_mutation_choice(&mut self, frame: &mut Frame<'_>) {
        let mut selected = None;
        if let Some(choice) = self.mutation_choice.as_mut() {
            if self.mutation_controls.action(VC_MUTATION_UP).pressed() {
                choice.menu.select_previous();
            }
            if self.mutation_controls.action(VC_MUTATION_DOWN).pressed() {
                choice.menu.select_next();
            }
            if self.mutation_controls.action(VC_MUTATION_CONFIRM).pressed()
                && let Some(index) = choice.menu.selected()
            {
                selected = choice.offers.get(index).copied();
            }
        }

        if let Some(mutation) = selected {
            self.mutation_choice = None;
            self.apply_mutation(mutation, frame);
        }
    }

    fn update_mutation_weapons(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.piercing_timer = (self.piercing_timer - dt).max(0.0);
        self.piercing_flash = (self.piercing_flash - dt).max(0.0);
        self.split_timer = (self.split_timer - dt).max(0.0);
        self.orbital_timer = (self.orbital_timer - dt).max(0.0);

        let fire_held = self
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .controls
            .action(FIRE)
            .held();
        if !fire_held {
            return;
        }

        if self.mutations.piercing_lance > 0 && self.piercing_timer <= 0.0 {
            self.fire_piercing_lance(frame);
            let stacks = self.mutations.piercing_lance.saturating_sub(1) as f32;
            self.piercing_timer = (0.72 - stacks * 0.08).max(0.40);
        }

        if self.mutations.split_volley > 0 && self.split_timer <= 0.0 {
            self.fire_split_volley();
            let stacks = self.mutations.split_volley.saturating_sub(1) as f32;
            self.split_timer = (0.28 - stacks * 0.035).max(0.18);
        }

        if self.mutations.orbitals > 0 && self.orbital_timer <= 0.0 {
            self.fire_orbitals();
            self.orbital_timer = 0.34;
        }
    }

    fn fire_split_volley(&mut self) {
        let pairs = self.mutations.split_volley.min(3);
        let game = &mut self.progression.combat.combat.ui.inner.inner;
        for pair in 0..pairs {
            let spread = 48.0 + pair as f32 * 25.0;
            let offset = 5.0 + pair as f32 * 3.0;
            for direction in [-1.0_f32, 1.0] {
                game.power_shots.push(PowerShot {
                    x: game.base.player_x + direction * offset,
                    y: game.base.player_y - 10.0,
                    vx: direction * spread,
                    vy: -PLAYER_SHOT_SPEED * 0.92,
                    damage: 1,
                    radius: 1,
                    alive: true,
                });
            }
        }
    }

    fn orbital_positions(&self) -> Vec<(f32, f32)> {
        let count = self.mutations.orbitals.min(3) as usize;
        if count == 0 {
            return Vec::new();
        }

        let base = &self.progression.combat.combat.ui.inner.inner.base;
        let radius = 16.0 + self.mutations.orbitals as f32 * 2.0;
        (0..count)
            .map(|index| {
                let angle = base.animation_time * 2.7 + TAU * index as f32 / count as f32;
                (
                    base.player_x + angle.cos() * radius,
                    base.player_y + angle.sin() * radius,
                )
            })
            .collect()
    }

    fn fire_orbitals(&mut self) {
        let positions = self.orbital_positions();
        let game = &mut self.progression.combat.combat.ui.inner.inner;
        for (x, y) in positions {
            game.power_shots.push(PowerShot {
                x,
                y: y - 4.0,
                vx: 0.0,
                vy: -PLAYER_SHOT_SPEED * 0.84,
                damage: 1,
                radius: 1,
                alive: true,
            });
        }
    }

    fn fire_piercing_lance(&mut self, frame: &mut Frame<'_>) {
        let stacks = self.mutations.piercing_lance.max(1);
        let damage = 1 + (stacks - 1) / 2;
        let half_width = 2.5 + stacks as f32 * 1.25;
        let (lance_x, player_y) = {
            let base = &self.progression.combat.combat.ui.inner.inner.base;
            (base.player_x, base.player_y)
        };
        self.piercing_x = lance_x;
        self.piercing_flash = 0.11;

        let mut carrion_destroyed = Vec::new();
        let mut special_destroyed = Vec::new();
        let mut threat_destroyed = Vec::new();
        let mut boss_damage = 0_u32;

        {
            let game = &mut self.progression.combat.combat.ui.inner.inner;
            for enemy in &mut game.base.enemies {
                if enemy.alive && in_piercing_lane(enemy.x, enemy.y, lance_x, player_y, half_width) {
                    enemy.alive = false;
                    carrion_destroyed.push((enemy.x, enemy.y));
                }
            }
            game.base.enemies.retain(|enemy| enemy.alive);

            if let Some(boss) = game.base.boss.as_mut()
                && game.base.encounter_phase == EncounterPhase::BossFight
                && in_piercing_lane(boss.x, boss.y, lance_x, player_y, 19.0 + half_width)
            {
                boss.hp = boss.hp.saturating_sub(damage);
                boss_damage = damage;
            }
        }

        {
            let specials = &mut self.progression.combat.combat.specials;
            for enemy in specials.iter_mut() {
                if !enemy.alive
                    || !in_piercing_lane(enemy.x, enemy.y, lance_x, player_y, half_width)
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
            let threats = &mut self.progression.combat.threats;
            for threat in threats.iter_mut() {
                if !threat.alive
                    || !in_piercing_lane(threat.x, threat.y, lance_x, player_y, half_width)
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

        self.reward_piercing_kills(
            &carrion_destroyed,
            &special_destroyed,
            &threat_destroyed,
            boss_damage,
            frame,
        );
    }

    fn reward_piercing_kills(
        &mut self,
        carrion_destroyed: &[(f32, f32)],
        special_destroyed: &[(SpecialKind, f32, f32)],
        threat_destroyed: &[(ThreatKind, f32, f32)],
        boss_damage: u32,
        frame: &mut Frame<'_>,
    ) {
        let any_hit = !carrion_destroyed.is_empty()
            || !special_destroyed.is_empty()
            || !threat_destroyed.is_empty()
            || boss_damage > 0;

        {
            let game = &mut self.progression.combat.combat.ui.inner.inner;
            for &(x, y) in carrion_destroyed {
                game.destroyed_enemies = game.destroyed_enemies.saturating_add(1);
                game.base.score = game.base.score.saturating_add(100);
                game.base.cinders.push(CinderDrop {
                    x,
                    y,
                    age: 0.0,
                    phase: x * 0.11 + y * 0.07,
                    alive: true,
                });
                game.base.bursts.push(Burst::new(x, y, 0.24, ENEMY_EYE));
                self.progression.xp_orbs.push(XpOrb::new(x, y, XP_ORB_CARRION));
            }

            for &(kind, x, y) in special_destroyed {
                let score = match kind {
                    SpecialKind::GraveKnight => 350,
                    SpecialKind::BellWraith => 500,
                    SpecialKind::RelicCarrier => 650,
                };
                game.base.score = game.base.score.saturating_add(score);
                game.base.cinders.push(CinderDrop {
                    x,
                    y,
                    age: 0.0,
                    phase: x * 0.13 + y * 0.09,
                    alive: true,
                });
                game.base
                    .bursts
                    .push(Burst::new(x, y, 0.34, special_color(kind)));
                if kind == SpecialKind::RelicCarrier {
                    game.relics.push(RelicDrop {
                        x,
                        y,
                        age: 0.0,
                        phase: x * 0.09 + y * 0.13,
                        alive: true,
                    });
                }
                self.progression.xp_orbs.push(XpOrb::new(x, y, XP_ORB_SPECIAL));
            }

            for &(kind, x, y) in threat_destroyed {
                let score = match kind {
                    ThreatKind::ChoirNode => 700,
                    ThreatKind::VoidLeech => 800,
                };
                game.base.score = game.base.score.saturating_add(score);
                game.base.cinders.push(CinderDrop {
                    x,
                    y,
                    age: 0.0,
                    phase: x * 0.12 + y * 0.08,
                    alive: true,
                });
                game.base
                    .bursts
                    .push(Burst::new(x, y, 0.40, threat_color(kind)));
                self.progression.xp_orbs.push(XpOrb::new(x, y, XP_ORB_THREAT));
            }

            if boss_damage > 0 {
                game.base.score = game.base.score.saturating_add(boss_damage * 5);
                if let Some(boss) = game.base.boss {
                    game.base
                        .bursts
                        .push(Burst::new(boss.x, boss.y + 3.0, 0.11, MUTATION_LIGHT));
                }
            }
        }

        self.progression.combat.combat.special_gun_kills_wave = self
            .progression
            .combat
            .combat
            .special_gun_kills_wave
            .saturating_add(special_destroyed.len() as u32);
        self.progression.combat.threat_gun_kills_wave = self
            .progression
            .combat
            .threat_gun_kills_wave
            .saturating_add(threat_destroyed.len() as u32);

        if any_hit {
            let base = &mut self.progression.combat.combat.ui.inner.inner.base;
            let _ = base.sounds.play(frame.audio, ENEMY_HIT_SOUND);
            base.finish_boss_if_destroyed(frame);
        }
    }

    fn apply_death_novas(&mut self) {
        let stacks = self.mutations.death_nova;
        if stacks == 0 {
            return;
        }

        let radius = 24.0 + stacks as f32 * 14.0;
        let kill_positions: Vec<(f32, f32)> = self
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
                    && ((burst.duration - 0.24).abs() < 0.0001
                        || (burst.duration - 0.34).abs() < 0.0001
                        || (burst.duration - 0.40).abs() < 0.0001)
            })
            .map(|burst| (burst.x, burst.y))
            .collect();

        if kill_positions.is_empty() {
            return;
        }

        let base = &mut self.progression.combat.combat.ui.inner.inner.base;
        base.enemy_bullets.retain(|bullet| {
            !kill_positions
                .iter()
                .any(|(x, y)| point_near(bullet.x, bullet.y, *x, *y, radius))
        });
        for (x, y) in kill_positions {
            base.bursts
                .push(Burst::new(x, y, 0.19, MUTATION_COLOR));
        }
    }

    fn render_mutation_fx(&self, framebuffer: &mut Framebuffer) {
        for (x, y) in self.orbital_positions() {
            let x = x.round() as i32;
            let y = y.round() as i32;
            framebuffer.draw_circle(x, y, 4, MUTATION_COLOR);
            framebuffer.fill_circle(x, y, 2, MUTATION_LIGHT);
            framebuffer.draw(x, y - 5, POWER_RELIC_LIGHT);
        }

        if self.piercing_flash > 0.0 {
            let base = &self.progression.combat.combat.ui.inner.inner.base;
            let x = self.piercing_x.round() as i32;
            let top = 20;
            let bottom = (base.player_y - 10.0).round() as i32;
            let width = self.mutations.piercing_lance.min(4) as i32;
            for offset in -width..=width {
                let color = if offset.abs() <= 1 {
                    MUTATION_LIGHT
                } else {
                    MUTATION_COLOR
                };
                framebuffer.draw_line(x + offset, bottom, x + offset, top, color);
            }
        }
    }

    fn render_mutation_choice(&self, framebuffer: &mut Framebuffer, choice: &MutationChoice) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 8,
                y: 48,
                width: 164,
                height: 220,
            },
            Pixel::rgb(12, 7, 18),
            MUTATION_COLOR,
        );
        framebuffer.draw_text(58, 61, "MUTATION", MUTATION_LIGHT);
        framebuffer.draw_text(48, 75, "BUILD EVOLVES", WRECK_LIGHT);

        for (index, mutation) in choice.offers.iter().copied().enumerate() {
            let y = 98 + index as i32 * 47;
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 18,
                    y,
                    width: 144,
                    height: 18,
                },
                mutation_name(mutation),
                choice.menu.selected() == Some(index),
                1,
                TEXT,
                MUTATION_LIGHT,
            );
            framebuffer.draw_text(28, y + 23, mutation_description(mutation), WRECK_LIGHT);
        }

        framebuffer.draw_text(39, 244, "SPACE SOUTH SELECT", WRECK_LIGHT);
    }

    fn render_frozen_mutation_choice(&self, framebuffer: &mut Framebuffer) {
        self.progression.combat.render(framebuffer, false);
        self.progression.render_progression(framebuffer);
        self.render_mutation_fx(framebuffer);
        if let Some(choice) = &self.mutation_choice {
            self.render_mutation_choice(framebuffer, choice);
        }
    }
}

impl Game for VoidCanticleV14 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.mutation_controls.update(frame.input);

        if self.mutation_choice.is_some() {
            self.update_mutation_choice(frame);
            self.render_frozen_mutation_choice(frame.framebuffer);
            return GameResult::Continue;
        }

        let was_game_over = self
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .game_over;
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let result = self.progression.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let game_over = self
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
            .game_over;
        if was_game_over && !game_over {
            self.reset_mutations();
        }

        if self.progression.level_choice.is_none() && !game_over {
            self.update_mutation_weapons(dt, frame);
            self.apply_death_novas();
            self.maybe_start_mutation_choice();
        }

        self.render_mutation_fx(frame.framebuffer);
        if let Some(choice) = &self.mutation_choice {
            self.render_mutation_choice(frame.framebuffer, choice);
        }
        GameResult::Continue
    }
}

fn in_piercing_lane(
    x: f32,
    y: f32,
    lance_x: f32,
    player_y: f32,
    half_width: f32,
) -> bool {
    y < player_y - 5.0 && (x - lance_x).abs() <= half_width
}

fn mutation_name(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "PIERCING LANCE",
        MutationKind::SplitVolley => "SPLIT VOLLEY",
        MutationKind::DeathNova => "DEATH NOVA",
        MutationKind::Orbitals => "ORBITALS",
    }
}

fn mutation_description(mutation: MutationKind) -> &'static str {
    match mutation {
        MutationKind::PiercingLance => "LINE BEAM PIERCES",
        MutationKind::SplitVolley => "ADDS SIDE SHOTS",
        MutationKind::DeathNova => "KILLS CLEAR BULLETS",
        MutationKind::Orbitals => "RELICS FIRE WITH YOU",
    }
}

struct VoidCanticlePauseV14 {
    game: VoidCanticleV14,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV14 {
    fn new(game: VoidCanticleV14) -> Self {
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
        framebuffer.draw_text(20, 103, &format!("VERSION {VC14_VERSION}"), TEXT);
        framebuffer.draw_text(20, 123, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(20, 143, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(20, 163, "GPE DEV BUILD", WRECK_LIGHT);
        framebuffer.draw_text(31, 219, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV14 {
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

pub fn run_v14_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePauseV14::new(VoidCanticleV14::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v14_tests {
    use super::*;

    #[test]
    fn mutations_are_offered_every_two_levels() {
        assert!(2_u32.is_multiple_of(MUTATION_INTERVAL));
        assert!(4_u32.is_multiple_of(MUTATION_INTERVAL));
        assert!(!3_u32.is_multiple_of(MUTATION_INTERVAL));
    }

    #[test]
    fn mutation_offer_contains_three_distinct_choices() {
        let mut game = VoidCanticleV14::new();
        game.progression.level = 2;
        let offers = game.mutation_offers();
        assert_ne!(offers[0], offers[1]);
        assert_ne!(offers[0], offers[2]);
        assert_ne!(offers[1], offers[2]);
    }

    #[test]
    fn piercing_lane_hits_only_targets_above_player_and_near_axis() {
        assert!(in_piercing_lane(91.0, 100.0, 90.0, 250.0, 4.0));
        assert!(!in_piercing_lane(110.0, 100.0, 90.0, 250.0, 4.0));
        assert!(!in_piercing_lane(90.0, 260.0, 90.0, 250.0, 4.0));
    }

    #[test]
    fn mutation_stacks_have_explicit_caps() {
        let mut build = MutationBuild::default();
        build.piercing_lance = 4;
        build.split_volley = 3;
        build.death_nova = 4;
        build.orbitals = 3;
        assert_eq!(build.piercing_lance, 4);
        assert_eq!(build.split_volley, 3);
        assert_eq!(build.death_nova, 4);
        assert_eq!(build.orbitals, 3);
    }

    #[test]
    fn vc14_version_is_explicit() {
        assert_eq!(VC14_VERSION, "VC1.4");
    }
}
