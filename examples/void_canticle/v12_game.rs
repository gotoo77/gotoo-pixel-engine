const VC12_VERSION: &str = "VC1.2";
const CHOIR_GLOW: Pixel = Pixel::rgb(112, 182, 230);
const CHOIR_CORE: Pixel = Pixel::rgb(208, 235, 255);
const LEECH_GLOW: Pixel = Pixel::rgb(195, 84, 210);
const LEECH_CORE: Pixel = Pixel::rgb(245, 154, 232);
const CHOIR_BUFF_RADIUS: f32 = 72.0;
const LEECH_PULSE_CHARGE: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThreatKind {
    ChoirNode,
    VoidLeech,
}

#[derive(Debug, Clone, Copy)]
struct ThreatSpawn {
    at: f32,
    kind: ThreatKind,
    x: f32,
    target_y: f32,
}

#[derive(Debug, Clone, Copy)]
struct ThreatEnemy {
    kind: ThreatKind,
    base_x: f32,
    x: f32,
    y: f32,
    target_y: f32,
    age: f32,
    phase: f32,
    hp: u32,
    charge: u32,
    alive: bool,
}

impl ThreatEnemy {
    fn new(spec: ThreatSpawn) -> Self {
        let hp = match spec.kind {
            ThreatKind::ChoirNode => 6,
            ThreatKind::VoidLeech => 7,
        };

        Self {
            kind: spec.kind,
            base_x: spec.x,
            x: spec.x,
            y: -18.0,
            target_y: spec.target_y,
            age: 0.0,
            phase: spec.x * 0.057 + spec.at * 0.43,
            hp,
            charge: 0,
            alive: true,
        }
    }

    fn hit_radius(self) -> f32 {
        match self.kind {
            ThreatKind::ChoirNode => 10.0,
            ThreatKind::VoidLeech => 12.0,
        }
    }
}

const V12_THREAT_WAVES: &[&[ThreatSpawn]] = &[
    &[],
    &[],
    &[],
    &[],
    &[],
    &[ThreatSpawn {
        at: 0.75,
        kind: ThreatKind::ChoirNode,
        x: 90.0,
        target_y: 94.0,
    }],
    &[ThreatSpawn {
        at: 0.85,
        kind: ThreatKind::VoidLeech,
        x: 90.0,
        target_y: 86.0,
    }],
    &[],
    &[ThreatSpawn {
        at: 0.55,
        kind: ThreatKind::ChoirNode,
        x: 52.0,
        target_y: 98.0,
    }],
    &[ThreatSpawn {
        at: 0.55,
        kind: ThreatKind::VoidLeech,
        x: 126.0,
        target_y: 88.0,
    }],
    &[ThreatSpawn {
        at: 0.65,
        kind: ThreatKind::ChoirNode,
        x: 128.0,
        target_y: 96.0,
    }],
    &[
        ThreatSpawn {
            at: 0.35,
            kind: ThreatKind::ChoirNode,
            x: 48.0,
            target_y: 98.0,
        },
        ThreatSpawn {
            at: 1.10,
            kind: ThreatKind::VoidLeech,
            x: 130.0,
            target_y: 84.0,
        },
    ],
    &[ThreatSpawn {
        at: 0.48,
        kind: ThreatKind::VoidLeech,
        x: 58.0,
        target_y: 84.0,
    }],
    &[ThreatSpawn {
        at: 0.38,
        kind: ThreatKind::ChoirNode,
        x: 90.0,
        target_y: 100.0,
    }],
    &[
        ThreatSpawn {
            at: 0.25,
            kind: ThreatKind::ChoirNode,
            x: 48.0,
            target_y: 100.0,
        },
        ThreatSpawn {
            at: 0.72,
            kind: ThreatKind::VoidLeech,
            x: 132.0,
            target_y: 86.0,
        },
    ],
];

struct VoidCanticleV12 {
    combat: VoidCanticleV11,
    threats: Vec<ThreatEnemy>,
    next_threat: usize,
    threat_gun_kills_wave: u32,
    threat_missed_in_wave: bool,
}

impl VoidCanticleV12 {
    fn new() -> Self {
        Self {
            combat: VoidCanticleV11::new(),
            threats: Vec::new(),
            next_threat: 0,
            threat_gun_kills_wave: 0,
            threat_missed_in_wave: false,
        }
    }

    fn reset_run(&mut self) {
        self.combat.reset_run();
        self.threats.clear();
        self.next_threat = 0;
        self.threat_gun_kills_wave = 0;
        self.threat_missed_in_wave = false;
    }

    fn threat_specs(&self) -> &'static [ThreatSpawn] {
        V12_THREAT_WAVES
            .get(self.combat.ui.inner.wave_index)
            .copied()
            .unwrap_or(&[])
    }

    fn spawn_threats_for_wave(&mut self) {
        if self.combat.ui.inner.inner.base.encounter_phase != EncounterPhase::Waves
            || self.combat.ui.inner.wave_index >= V09_WAVES.len()
            || self.combat.ui.inner.intermission > 0.0
        {
            return;
        }

        let specs = self.threat_specs();
        while self.next_threat < specs.len()
            && specs[self.next_threat].at <= self.combat.ui.inner.wave_time
        {
            self.threats.push(ThreatEnemy::new(specs[self.next_threat]));
            self.next_threat += 1;
        }
    }

    fn update_threats(&mut self, dt: f32) {
        let mut missed = false;

        for threat in &mut self.threats {
            threat.age += dt;

            if threat.age < 1.15 {
                threat.y += 82.0 * dt;
            } else {
                threat.y += (threat.target_y - threat.y) * 2.8 * dt;
                let amplitude = match threat.kind {
                    ThreatKind::ChoirNode => 12.0,
                    ThreatKind::VoidLeech => 20.0,
                };
                let frequency = match threat.kind {
                    ThreatKind::ChoirNode => 1.05,
                    ThreatKind::VoidLeech => 1.55,
                };
                threat.x = (threat.base_x + (threat.age * frequency + threat.phase).sin() * amplitude)
                    .clamp(14.0, FRAMEBUFFER_WIDTH as f32 - 14.0);
            }

            let retreat_age = match threat.kind {
                ThreatKind::ChoirNode => 8.8,
                ThreatKind::VoidLeech => 8.0,
            };
            if threat.age > retreat_age {
                threat.y += 62.0 * dt;
            }

            if threat.y > FRAMEBUFFER_HEIGHT as f32 + 24.0 {
                threat.alive = false;
                missed = true;
            }
        }

        if missed {
            self.threat_missed_in_wave = true;
        }
        self.threats.retain(|threat| threat.alive);
    }

    fn choir_nodes(&self) -> Vec<(f32, f32)> {
        self.threats
            .iter()
            .filter(|threat| threat.alive && threat.kind == ThreatKind::ChoirNode)
            .map(|threat| (threat.x, threat.y))
            .collect()
    }

    fn apply_choir_buff(&mut self, dt: f32) {
        let nodes = self.choir_nodes();
        if nodes.is_empty() {
            return;
        }

        for enemy in &mut self.combat.ui.inner.inner.base.enemies {
            if nodes
                .iter()
                .any(|(x, y)| point_near(enemy.x, enemy.y, *x, *y, CHOIR_BUFF_RADIUS))
            {
                enemy.fire_timer -= dt * 0.85;
            }
        }

        for enemy in &mut self.combat.specials {
            if enemy.kind == SpecialKind::BellWraith
                && nodes
                    .iter()
                    .any(|(x, y)| point_near(enemy.x, enemy.y, *x, *y, CHOIR_BUFF_RADIUS))
            {
                enemy.fire_timer -= dt * 0.85;
            }
        }
    }

    fn resolve_threat_shots(&mut self, frame: &mut Frame<'_>) {
        let mut destroyed = Vec::new();
        let mut leech_pulses = Vec::new();

        {
            let game = &mut self.combat.ui.inner.inner;
            for shot in &mut game.power_shots {
                if !shot.alive {
                    continue;
                }

                for threat in &mut self.threats {
                    if !threat.alive
                        || !point_near(
                            shot.x,
                            shot.y,
                            threat.x,
                            threat.y,
                            threat.hit_radius() + shot.radius as f32,
                        )
                    {
                        continue;
                    }

                    shot.alive = false;
                    match threat.kind {
                        ThreatKind::ChoirNode => {
                            threat.hp = threat.hp.saturating_sub(shot.damage.max(1));
                        }
                        ThreatKind::VoidLeech => {
                            threat.hp = threat.hp.saturating_sub(1);
                            threat.charge = threat.charge.saturating_add(1);
                            if threat.hp > 0 && threat.charge >= LEECH_PULSE_CHARGE {
                                threat.charge = 0;
                                leech_pulses.push((threat.x, threat.y, threat.age));
                            }
                        }
                    }

                    if threat.hp == 0 {
                        threat.alive = false;
                        destroyed.push((threat.kind, threat.x, threat.y));
                    }
                    break;
                }
            }
        }

        self.threats.retain(|threat| threat.alive);

        let mut pulse_bullets = Vec::new();
        for (x, y, age) in leech_pulses {
            spawn_ring(&mut pulse_bullets, x, y, 10, age * 0.93, 62.0);
        }
        if !pulse_bullets.is_empty() {
            self.combat
                .ui
                .inner
                .inner
                .base
                .enemy_bullets
                .extend(pulse_bullets);
            let _ = self
                .combat
                .ui
                .inner
                .inner
                .base
                .sounds
                .play(frame.audio, ENEMY_FIRE_SOUND);
        }

        for (kind, x, y) in destroyed {
            self.threat_gun_kills_wave = self.threat_gun_kills_wave.saturating_add(1);
            let game = &mut self.combat.ui.inner.inner;
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
            let _ = game.base.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }

        self.combat.resolve_power_shots(frame);
    }

    fn apply_canticle_to_threats(&mut self) {
        let mut destroyed = Vec::new();
        for threat in &mut self.threats {
            if threat.alive && threat.y >= 18.0 && threat.y < FRAMEBUFFER_HEIGHT as f32 {
                threat.alive = false;
                destroyed.push((threat.x, threat.y));
            }
        }
        self.threats.retain(|threat| threat.alive);

        let game = &mut self.combat.ui.inner.inner;
        for (x, y) in destroyed {
            game.base.score = game.base.score.saturating_add(100);
            game.base
                .bursts
                .push(Burst::new(x, y, 0.46, CANTICLE_COLOR));
        }
    }

    fn finish_wave_if_clear(&mut self) {
        if self.combat.ui.inner.inner.base.encounter_phase != EncounterPhase::Waves
            || self.combat.ui.inner.wave_index >= V09_WAVES.len()
            || self.combat.ui.inner.intermission > 0.0
        {
            return;
        }

        let wave = V09_WAVES[self.combat.ui.inner.wave_index];
        let special_count = self.combat.special_specs().len();
        let threat_count = self.threat_specs().len();
        if self.combat.ui.inner.next_spawn < wave.spawns.len()
            || self.combat.next_special < special_count
            || self.next_threat < threat_count
            || !self.combat.ui.inner.inner.base.enemies.is_empty()
            || !self.combat.specials.is_empty()
            || !self.threats.is_empty()
        {
            return;
        }

        let carrion_gun_kills = self
            .combat
            .ui
            .inner
            .inner
            .destroyed_enemies
            .saturating_sub(self.combat.ui.inner.wave_kills_start);
        let full_wipe = !self.combat.ui.inner.canticle_used_in_wave
            && !self.combat.special_missed_in_wave
            && !self.threat_missed_in_wave
            && carrion_gun_kills == wave.spawns.len() as u32
            && self.combat.special_gun_kills_wave == special_count as u32
            && self.threat_gun_kills_wave == threat_count as u32;

        if full_wipe {
            self.combat.ui.inner.full_wipe_chain =
                self.combat.ui.inner.full_wipe_chain.saturating_add(1);
            self.combat.ui.inner.full_wipes = self.combat.ui.inner.full_wipes.saturating_add(1);
            let bonus = full_wipe_bonus(self.combat.ui.inner.full_wipe_chain);
            self.combat.ui.inner.inner.base.score =
                self.combat.ui.inner.inner.base.score.saturating_add(bonus);
            self.combat.ui.inner.wipe_banner_timer = V09_WIPE_BANNER_DURATION;
            self.combat.ui.inner.wipe_banner_chain = self.combat.ui.inner.full_wipe_chain;
            self.combat.ui.inner.wipe_banner_bonus = bonus;
        } else {
            self.combat.ui.inner.full_wipe_chain = 0;
        }

        self.combat.ui.inner.wave_index += 1;
        self.combat.ui.inner.wave_time = 0.0;
        self.combat.ui.inner.next_spawn = 0;
        self.combat.ui.inner.intermission = V09_INTERMISSION;
        self.combat.ui.inner.wave_kills_start = self.combat.ui.inner.inner.destroyed_enemies;
        self.combat.ui.inner.canticle_used_in_wave = false;
        self.combat.next_special = 0;
        self.combat.special_gun_kills_wave = 0;
        self.combat.special_missed_in_wave = false;
        self.next_threat = 0;
        self.threat_gun_kills_wave = 0;
        self.threat_missed_in_wave = false;
    }

    fn resolve_threat_player_hit(&mut self, frame: &mut Frame<'_>) {
        let game = &mut self.combat.ui.inner.inner;
        if game.base.invulnerability > 0.0 || game.base.game_over {
            return;
        }

        let Some(hit_index) = self.threats.iter().position(|threat| {
            point_near(
                threat.x,
                threat.y,
                game.base.player_x,
                game.base.player_y,
                threat.hit_radius(),
            )
        }) else {
            return;
        };

        self.threats[hit_index].alive = false;
        self.threat_missed_in_wave = true;
        game.base.lives = game.base.lives.saturating_sub(1);
        game.base.invulnerability = PLAYER_INVULNERABILITY;
        game.base.enemy_bullets.clear();
        game.base
            .bursts
            .push(Burst::new(game.base.player_x, game.base.player_y, 0.42, DANGER));
        let _ = game.base.sounds.play(frame.audio, PLAYER_HIT_SOUND);
        if game.base.lives == 0 {
            game.base.game_over = true;
        }
        self.threats.retain(|threat| threat.alive);
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        self.combat.render(framebuffer, focused);

        let nodes = self.choir_nodes();
        for (node_x, node_y) in nodes {
            for enemy in &self.combat.ui.inner.inner.base.enemies {
                if point_near(enemy.x, enemy.y, node_x, node_y, CHOIR_BUFF_RADIUS) {
                    framebuffer.draw_line(
                        node_x.round() as i32,
                        node_y.round() as i32,
                        enemy.x.round() as i32,
                        enemy.y.round() as i32,
                        Pixel::rgb(36, 66, 86),
                    );
                }
            }
            for enemy in &self.combat.specials {
                if enemy.kind == SpecialKind::BellWraith
                    && point_near(enemy.x, enemy.y, node_x, node_y, CHOIR_BUFF_RADIUS)
                {
                    framebuffer.draw_line(
                        node_x.round() as i32,
                        node_y.round() as i32,
                        enemy.x.round() as i32,
                        enemy.y.round() as i32,
                        Pixel::rgb(36, 66, 86),
                    );
                }
            }
        }

        for threat in &self.threats {
            render_threat(framebuffer, *threat);
        }
    }
}

impl Game for VoidCanticleV12 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.combat.ui.inner.inner.base.controls.update(frame.input);

        if self.combat.ui.inner.inner.base.game_over {
            if self
                .combat
                .ui
                .inner
                .inner
                .base
                .controls
                .action(FIRE)
                .pressed()
            {
                self.reset_run();
            }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.combat.ui.inner.inner.base.focus_held(frame);
        let canticle_pressed = self.combat.ui.inner.inner.base.canticle_pressed(frame);

        self.combat.ui.inner.wipe_banner_timer =
            (self.combat.ui.inner.wipe_banner_timer - dt).max(0.0);
        self.combat.ui.inner.inner.base.scroll =
            (self.combat.ui.inner.inner.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.combat.ui.inner.inner.base.update_feedback(dt);
        self.combat.ui.inner.inner.base.update_player(dt, focused);
        self.combat.ui.inner.inner.update_player_fire(dt, frame);
        self.combat.ui.inner.update_wave_flow(dt, frame);
        self.combat.spawn_specials_for_wave();
        self.spawn_threats_for_wave();
        self.apply_choir_buff(dt);
        self.combat.ui.inner.inner.base.update_enemies(dt, frame);
        self.combat.update_specials(dt, frame);
        self.update_threats(dt);
        self.combat.ui.inner.inner.base.update_projectiles(dt);
        self.combat.ui.inner.inner.update_power_shots(dt);
        self.resolve_threat_shots(frame);
        self.combat.ui.inner.inner.base.update_cinders(dt, frame);
        self.combat.ui.inner.inner.update_relics(dt, frame);

        if canticle_pressed {
            let can_trigger = self.combat.ui.inner.inner.base.encounter_phase == EncounterPhase::Waves
                && self.combat.ui.inner.inner.base.core_charge >= CORE_MAX
                && self.combat.ui.inner.inner.base.canticle_timer <= 0.0;
            if can_trigger {
                self.combat.ui.inner.canticle_used_in_wave = true;
                self.combat.apply_canticle_to_specials();
                self.apply_canticle_to_threats();
            }
            self.combat.ui.inner.inner.base.trigger_canticle(frame);
        }

        self.finish_wave_if_clear();
        self.combat.ui.update_wave_toast(dt);

        let previous_lives = self.combat.ui.inner.inner.base.lives;
        self.resolve_threat_player_hit(frame);
        self.combat.resolve_special_player_hit(frame);
        self.combat.ui.inner.inner.base.resolve_player_hits(frame);
        self.combat
            .ui
            .inner
            .inner
            .apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

fn threat_color(kind: ThreatKind) -> Pixel {
    match kind {
        ThreatKind::ChoirNode => CHOIR_GLOW,
        ThreatKind::VoidLeech => LEECH_GLOW,
    }
}

fn render_threat(framebuffer: &mut Framebuffer, threat: ThreatEnemy) {
    let x = threat.x.round() as i32;
    let y = threat.y.round() as i32;

    match threat.kind {
        ThreatKind::ChoirNode => {
            let pulse = 9 + ((threat.age * 4.0).sin().abs() * 3.0) as u32;
            framebuffer.draw_circle(x, y, pulse, CHOIR_GLOW);
            framebuffer.draw_circle(x, y, pulse.saturating_add(4), Pixel::rgb(40, 77, 98));
            framebuffer.draw_line(x - 9, y, x + 9, y, CHOIR_GLOW);
            framebuffer.draw_line(x, y - 9, x, y + 9, CHOIR_GLOW);
            framebuffer.fill_circle(x, y, 3, CHOIR_CORE);
        }
        ThreatKind::VoidLeech => {
            let pulse = 10 + ((threat.age * 5.2).sin().abs() * 2.0) as u32;
            framebuffer.draw_circle(x, y, pulse, LEECH_GLOW);
            framebuffer.draw_circle(x, y, pulse.saturating_add(4), ENEMY_DARK);
            framebuffer.draw_line(x - 11, y - 7, x + 11, y + 7, LEECH_GLOW);
            framebuffer.draw_line(x + 11, y - 7, x - 11, y + 7, LEECH_GLOW);
            framebuffer.fill_circle(x, y, 3, LEECH_CORE);
            for pip in 0..threat.charge.min(LEECH_PULSE_CHARGE) {
                framebuffer.draw(x - 8 + pip as i32 * 4, y + 15, LEECH_CORE);
            }
        }
    }
}

struct VoidCanticlePauseV12 {
    game: VoidCanticleV12,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV12 {
    fn new(game: VoidCanticleV12) -> Self {
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
        let panel = gotoo_pixel_engine::Rect {
            x: 18,
            y: 48,
            width: 144,
            height: 220,
        };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
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
        let panel = gotoo_pixel_engine::Rect {
            x: 10,
            y: 46,
            width: 160,
            height: 224,
        };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
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
        let panel = gotoo_pixel_engine::Rect {
            x: 10,
            y: 58,
            width: 160,
            height: 196,
        };
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            panel,
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(46, 72, "BUILD INFO", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 103, &format!("VERSION {VC12_VERSION}"), TEXT);
        framebuffer.draw_text(20, 123, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(20, 143, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(20, 163, "GPE DEV BUILD", WRECK_LIGHT);
        framebuffer.draw_text(31, 219, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV12 {
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

fn v12_total_threats() -> usize {
    V12_THREAT_WAVES.iter().map(|wave| wave.len()).sum()
}

pub fn run_v12_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePauseV12::new(VoidCanticleV12::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v12_tests {
    use super::*;

    #[test]
    fn tactical_threats_arrive_after_core_enemy_teaching_waves() {
        assert_eq!(V12_THREAT_WAVES.len(), 15);
        assert!(V12_THREAT_WAVES[..5].iter().all(|wave| wave.is_empty()));
        assert!(v12_total_threats() >= 9);
    }

    #[test]
    fn final_wave_combines_choir_and_leech() {
        let final_wave = V12_THREAT_WAVES[14];
        assert!(final_wave.iter().any(|spawn| spawn.kind == ThreatKind::ChoirNode));
        assert!(final_wave.iter().any(|spawn| spawn.kind == ThreatKind::VoidLeech));
    }

    #[test]
    fn leech_requires_multiple_absorbed_shots_before_pulse() {
        assert_eq!(LEECH_PULSE_CHARGE, 5);
    }

    #[test]
    fn vc12_version_is_explicit() {
        assert_eq!(VC12_VERSION, "VC1.2");
    }
}
