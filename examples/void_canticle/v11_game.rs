const VC11_VERSION: &str = "VC1.1";

const KNIGHT_METAL: Pixel = Pixel::rgb(118, 124, 142);
const WRAITH_GLOW: Pixel = Pixel::rgb(174, 105, 232);
const CARRIER_GOLD: Pixel = Pixel::rgb(232, 184, 86);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecialKind {
    GraveKnight,
    BellWraith,
    RelicCarrier,
}

#[derive(Debug, Clone, Copy)]
struct SpecialSpawn {
    at: f32,
    kind: SpecialKind,
    x: f32,
    y: f32,
    direction: f32,
}

#[derive(Debug, Clone, Copy)]
struct SpecialEnemy {
    kind: SpecialKind,
    base_x: f32,
    x: f32,
    y: f32,
    age: f32,
    phase: f32,
    direction: f32,
    fire_timer: f32,
    hp: u32,
    alive: bool,
}

impl SpecialEnemy {
    fn new(spec: SpecialSpawn) -> Self {
        let hp = match spec.kind {
            SpecialKind::GraveKnight => 3,
            SpecialKind::BellWraith => 4,
            SpecialKind::RelicCarrier => 2,
        };
        let fire_timer = match spec.kind {
            SpecialKind::BellWraith => 1.25,
            SpecialKind::GraveKnight | SpecialKind::RelicCarrier => 99.0,
        };
        let x = if spec.kind == SpecialKind::RelicCarrier {
            if spec.direction >= 0.0 {
                -14.0
            } else {
                FRAMEBUFFER_WIDTH as f32 + 14.0
            }
        } else {
            spec.x
        };

        Self {
            kind: spec.kind,
            base_x: spec.x,
            x,
            y: spec.y,
            age: 0.0,
            phase: spec.x * 0.071 + spec.at * 0.53,
            direction: spec.direction,
            fire_timer,
            hp,
            alive: true,
        }
    }

    fn hit_radius(self) -> f32 {
        match self.kind {
            SpecialKind::GraveKnight => 10.0,
            SpecialKind::BellWraith => 10.0,
            SpecialKind::RelicCarrier => 9.0,
        }
    }
}

const V11_SPECIAL_WAVES: &[&[SpecialSpawn]] = &[
    &[],
    &[],
    &[SpecialSpawn {
        at: 0.65,
        kind: SpecialKind::GraveKnight,
        x: 90.0,
        y: -16.0,
        direction: 0.0,
    }],
    &[SpecialSpawn {
        at: 0.45,
        kind: SpecialKind::BellWraith,
        x: 90.0,
        y: -18.0,
        direction: 0.0,
    }],
    &[SpecialSpawn {
        at: 0.55,
        kind: SpecialKind::RelicCarrier,
        x: 90.0,
        y: 58.0,
        direction: 1.0,
    }],
    &[
        SpecialSpawn {
            at: 0.35,
            kind: SpecialKind::GraveKnight,
            x: 54.0,
            y: -16.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 1.20,
            kind: SpecialKind::BellWraith,
            x: 126.0,
            y: -18.0,
            direction: 0.0,
        },
    ],
    &[SpecialSpawn {
        at: 0.70,
        kind: SpecialKind::BellWraith,
        x: 52.0,
        y: -18.0,
        direction: 0.0,
    }],
    &[SpecialSpawn {
        at: 0.45,
        kind: SpecialKind::RelicCarrier,
        x: 90.0,
        y: 72.0,
        direction: -1.0,
    }],
    &[
        SpecialSpawn {
            at: 0.30,
            kind: SpecialKind::GraveKnight,
            x: 52.0,
            y: -16.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 0.95,
            kind: SpecialKind::GraveKnight,
            x: 128.0,
            y: -16.0,
            direction: 0.0,
        },
    ],
    &[
        SpecialSpawn {
            at: 0.30,
            kind: SpecialKind::BellWraith,
            x: 90.0,
            y: -18.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 1.10,
            kind: SpecialKind::GraveKnight,
            x: 90.0,
            y: -16.0,
            direction: 0.0,
        },
    ],
    &[SpecialSpawn {
        at: 0.42,
        kind: SpecialKind::RelicCarrier,
        x: 90.0,
        y: 52.0,
        direction: 1.0,
    }],
    &[
        SpecialSpawn {
            at: 0.25,
            kind: SpecialKind::BellWraith,
            x: 48.0,
            y: -18.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 0.85,
            kind: SpecialKind::GraveKnight,
            x: 132.0,
            y: -16.0,
            direction: 0.0,
        },
    ],
    &[
        SpecialSpawn {
            at: 0.25,
            kind: SpecialKind::GraveKnight,
            x: 44.0,
            y: -16.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 0.65,
            kind: SpecialKind::BellWraith,
            x: 90.0,
            y: -18.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 1.05,
            kind: SpecialKind::GraveKnight,
            x: 136.0,
            y: -16.0,
            direction: 0.0,
        },
    ],
    &[
        SpecialSpawn {
            at: 0.30,
            kind: SpecialKind::RelicCarrier,
            x: 90.0,
            y: 62.0,
            direction: -1.0,
        },
        SpecialSpawn {
            at: 0.85,
            kind: SpecialKind::BellWraith,
            x: 90.0,
            y: -18.0,
            direction: 0.0,
        },
    ],
    &[
        SpecialSpawn {
            at: 0.18,
            kind: SpecialKind::GraveKnight,
            x: 48.0,
            y: -16.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 0.48,
            kind: SpecialKind::BellWraith,
            x: 90.0,
            y: -18.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 0.78,
            kind: SpecialKind::GraveKnight,
            x: 132.0,
            y: -16.0,
            direction: 0.0,
        },
        SpecialSpawn {
            at: 1.08,
            kind: SpecialKind::RelicCarrier,
            x: 90.0,
            y: 48.0,
            direction: 1.0,
        },
    ],
];

struct VoidCanticleV11 {
    ui: VoidCanticleV10,
    specials: Vec<SpecialEnemy>,
    next_special: usize,
    special_gun_kills_wave: u32,
    special_missed_in_wave: bool,
}

impl VoidCanticleV11 {
    fn new() -> Self {
        Self {
            ui: VoidCanticleV10::new(),
            specials: Vec::new(),
            next_special: 0,
            special_gun_kills_wave: 0,
            special_missed_in_wave: false,
        }
    }

    fn reset_run(&mut self) {
        self.ui.reset_run();
        self.specials.clear();
        self.next_special = 0;
        self.special_gun_kills_wave = 0;
        self.special_missed_in_wave = false;
    }

    fn special_specs(&self) -> &'static [SpecialSpawn] {
        V11_SPECIAL_WAVES
            .get(self.ui.inner.wave_index)
            .copied()
            .unwrap_or(&[])
    }

    fn spawn_specials_for_wave(&mut self) {
        if self.ui.inner.inner.base.encounter_phase != EncounterPhase::Waves
            || self.ui.inner.wave_index >= V09_WAVES.len()
            || self.ui.inner.intermission > 0.0
        {
            return;
        }

        let specs = self.special_specs();
        while self.next_special < specs.len() && specs[self.next_special].at <= self.ui.inner.wave_time
        {
            self.specials.push(SpecialEnemy::new(specs[self.next_special]));
            self.next_special += 1;
        }
    }

    fn update_specials(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let player_x = self.ui.inner.inner.base.player_x;
        let mut spawned_bullets = Vec::new();
        let mut fired = false;
        let mut missed = false;

        for enemy in &mut self.specials {
            enemy.age += dt;

            match enemy.kind {
                SpecialKind::GraveKnight => {
                    if enemy.age < 0.90 {
                        enemy.y += 88.0 * dt;
                        enemy.x += (player_x - enemy.x) * 1.7 * dt;
                    } else if enemy.age < 1.55 {
                        enemy.y += (70.0 - enemy.y) * 4.0 * dt;
                        enemy.x += (player_x - enemy.x) * 2.3 * dt;
                    } else {
                        enemy.y += 176.0 * dt;
                    }
                }
                SpecialKind::BellWraith => {
                    if enemy.age < 1.0 {
                        enemy.y += 78.0 * dt;
                    } else {
                        enemy.y += (72.0 - enemy.y) * 2.8 * dt;
                        enemy.x = (enemy.base_x + (enemy.age * 1.55 + enemy.phase).sin() * 25.0)
                            .clamp(15.0, FRAMEBUFFER_WIDTH as f32 - 15.0);
                        enemy.fire_timer -= dt;
                        if enemy.fire_timer <= 0.0 {
                            spawn_ring(
                                &mut spawned_bullets,
                                enemy.x,
                                enemy.y,
                                8,
                                enemy.age * 0.72,
                                48.0,
                            );
                            enemy.fire_timer += 1.35;
                            fired = true;
                        }
                    }

                    if enemy.age > 7.0 {
                        enemy.y += 58.0 * dt;
                    }
                }
                SpecialKind::RelicCarrier => {
                    enemy.x += enemy.direction * 82.0 * dt;
                    enemy.y = enemy.y + (enemy.age * 4.2 + enemy.phase).sin() * 5.0 * dt;
                }
            }

            let escaped = match enemy.kind {
                SpecialKind::RelicCarrier => {
                    enemy.x < -22.0 || enemy.x > FRAMEBUFFER_WIDTH as f32 + 22.0
                }
                SpecialKind::GraveKnight | SpecialKind::BellWraith => {
                    enemy.y > FRAMEBUFFER_HEIGHT as f32 + 24.0
                }
            };
            if escaped {
                enemy.alive = false;
                missed = true;
            }
        }

        self.ui
            .inner
            .inner
            .base
            .enemy_bullets
            .extend(spawned_bullets);
        if fired {
            let _ = self
                .ui
                .inner
                .inner
                .base
                .sounds
                .play(frame.audio, ENEMY_FIRE_SOUND);
        }
        if missed {
            self.special_missed_in_wave = true;
        }
        self.specials.retain(|enemy| enemy.alive);
    }

    fn resolve_power_shots(&mut self, frame: &mut Frame<'_>) {
        let mut destroyed_carrion = Vec::new();
        let mut destroyed_special = Vec::new();
        let mut boss_damage = 0_u32;

        {
            let game = &mut self.ui.inner.inner;

            for shot in &mut game.power_shots {
                if !shot.alive {
                    continue;
                }

                for enemy in &mut self.specials {
                    if enemy.alive
                        && point_near(
                            shot.x,
                            shot.y,
                            enemy.x,
                            enemy.y,
                            enemy.hit_radius() + shot.radius as f32,
                        )
                    {
                        shot.alive = false;
                        enemy.hp = enemy.hp.saturating_sub(shot.damage.max(1));
                        if enemy.hp == 0 {
                            enemy.alive = false;
                            destroyed_special.push((enemy.kind, enemy.x, enemy.y));
                        }
                        break;
                    }
                }

                if !shot.alive {
                    continue;
                }

                for enemy in &mut game.base.enemies {
                    if enemy.alive
                        && point_near(
                            shot.x,
                            shot.y,
                            enemy.x,
                            enemy.y,
                            8.0 + shot.radius as f32,
                        )
                    {
                        shot.alive = false;
                        enemy.alive = false;
                        destroyed_carrion.push((enemy.x, enemy.y));
                        break;
                    }
                }

                if shot.alive
                    && let Some(boss) = game.base.boss.as_mut()
                    && game.base.encounter_phase == EncounterPhase::BossFight
                    && point_near(
                        shot.x,
                        shot.y,
                        boss.x,
                        boss.y,
                        19.0 + shot.radius as f32,
                    )
                {
                    shot.alive = false;
                    let damage = shot.damage.max(1);
                    boss.hp = boss.hp.saturating_sub(damage);
                    boss_damage = boss_damage.saturating_add(damage);
                }
            }

            game.power_shots.retain(|shot| shot.alive);
            game.base.enemies.retain(|enemy| enemy.alive);
        }
        self.specials.retain(|enemy| enemy.alive);

        let game = &mut self.ui.inner.inner;
        for (x, y) in destroyed_carrion {
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
            let _ = game.base.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }

        for (kind, x, y) in destroyed_special {
            self.special_gun_kills_wave = self.special_gun_kills_wave.saturating_add(1);
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
            let _ = game.base.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }

        if boss_damage > 0 {
            game.base.score = game.base.score.saturating_add(boss_damage * 5);
            if let Some(boss) = game.base.boss {
                game.base
                    .bursts
                    .push(Burst::new(boss.x, boss.y + 3.0, 0.11, BELL_LIGHT));
            }
        }

        game.base.finish_boss_if_destroyed(frame);
    }

    fn apply_canticle_to_specials(&mut self) {
        let mut destroyed = Vec::new();
        for enemy in &mut self.specials {
            if enemy.alive && enemy.y >= 18.0 && enemy.y < FRAMEBUFFER_HEIGHT as f32 {
                enemy.alive = false;
                destroyed.push((enemy.kind, enemy.x, enemy.y));
            }
        }
        self.specials.retain(|enemy| enemy.alive);

        let game = &mut self.ui.inner.inner;
        for (kind, x, y) in destroyed {
            game.base.score = game.base.score.saturating_add(100);
            game.base
                .bursts
                .push(Burst::new(x, y, 0.42, CANTICLE_COLOR));
            if kind == SpecialKind::RelicCarrier {
                game.relics.push(RelicDrop {
                    x,
                    y,
                    age: 0.0,
                    phase: x * 0.09 + y * 0.13,
                    alive: true,
                });
            }
        }
    }

    fn finish_wave_if_clear(&mut self) {
        if self.ui.inner.inner.base.encounter_phase != EncounterPhase::Waves
            || self.ui.inner.wave_index >= V09_WAVES.len()
            || self.ui.inner.intermission > 0.0
        {
            return;
        }

        let wave = V09_WAVES[self.ui.inner.wave_index];
        let special_count = self.special_specs().len();
        if self.ui.inner.next_spawn < wave.spawns.len()
            || self.next_special < special_count
            || !self.ui.inner.inner.base.enemies.is_empty()
            || !self.specials.is_empty()
        {
            return;
        }

        let carrion_gun_kills = self
            .ui
            .inner
            .inner
            .destroyed_enemies
            .saturating_sub(self.ui.inner.wave_kills_start);
        let full_wipe = !self.ui.inner.canticle_used_in_wave
            && !self.special_missed_in_wave
            && carrion_gun_kills == wave.spawns.len() as u32
            && self.special_gun_kills_wave == special_count as u32;

        if full_wipe {
            self.ui.inner.full_wipe_chain = self.ui.inner.full_wipe_chain.saturating_add(1);
            self.ui.inner.full_wipes = self.ui.inner.full_wipes.saturating_add(1);
            let bonus = full_wipe_bonus(self.ui.inner.full_wipe_chain);
            self.ui.inner.inner.base.score = self.ui.inner.inner.base.score.saturating_add(bonus);
            self.ui.inner.wipe_banner_timer = V09_WIPE_BANNER_DURATION;
            self.ui.inner.wipe_banner_chain = self.ui.inner.full_wipe_chain;
            self.ui.inner.wipe_banner_bonus = bonus;
        } else {
            self.ui.inner.full_wipe_chain = 0;
        }

        self.ui.inner.wave_index += 1;
        self.ui.inner.wave_time = 0.0;
        self.ui.inner.next_spawn = 0;
        self.ui.inner.intermission = V09_INTERMISSION;
        self.ui.inner.wave_kills_start = self.ui.inner.inner.destroyed_enemies;
        self.ui.inner.canticle_used_in_wave = false;
        self.next_special = 0;
        self.special_gun_kills_wave = 0;
        self.special_missed_in_wave = false;
    }

    fn resolve_special_player_hit(&mut self, frame: &mut Frame<'_>) {
        let game = &mut self.ui.inner.inner;
        if game.base.invulnerability > 0.0 || game.base.game_over {
            return;
        }

        let Some(hit_index) = self.specials.iter().position(|enemy| {
            point_near(
                enemy.x,
                enemy.y,
                game.base.player_x,
                game.base.player_y,
                enemy.hit_radius(),
            )
        }) else {
            return;
        };

        self.specials[hit_index].alive = false;
        self.special_missed_in_wave = true;
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
        self.specials.retain(|enemy| enemy.alive);
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        let game = &self.ui.inner.inner;
        let base = &game.base;

        framebuffer.clear(if base.canticle_timer > 0.46 {
            BG_CANTICLE
        } else {
            BG
        });
        render_grave_orbit_background(framebuffer, base.scroll);

        for cinder in &base.cinders {
            let x = cinder.x.round() as i32;
            let y = cinder.y.round() as i32;
            framebuffer.fill_circle(x, y, 2, CINDER);
            framebuffer.draw(x, y - 4, CANTICLE_COLOR);
        }

        for relic in &game.relics {
            render_relic(framebuffer, *relic);
        }

        for enemy in &base.enemies {
            base.visuals.render_carrion(framebuffer, *enemy);
        }
        for enemy in &self.specials {
            render_special(framebuffer, *enemy);
        }

        if let Some(boss) = base.boss
            && base.encounter_phase != EncounterPhase::Cleared
        {
            base.visuals.render_bellkeeper(framebuffer, boss);
        }

        for shot in &game.power_shots {
            render_power_shot(framebuffer, *shot, game.power_level);
        }

        for bullet in &base.enemy_bullets {
            let color = if bullet.alternate {
                ENEMY_SHOT_ALT
            } else {
                ENEMY_SHOT
            };
            framebuffer.fill_circle(
                bullet.x.round() as i32,
                bullet.y.round() as i32,
                2,
                color,
            );
        }

        for burst in &base.bursts {
            render_burst(framebuffer, *burst);
        }

        game.pilgrim_visuals.render(
            framebuffer,
            base.player_x.round() as i32,
            base.player_y.round() as i32,
            focused,
            base.invulnerability,
            base.animation_time,
        );

        if base.canticle_timer > 0.0 {
            render_canticle(
                framebuffer,
                base.player_x.round() as i32,
                base.player_y.round() as i32,
                base.canticle_timer,
            );
        }

        self.ui.render_minimal_hud(framebuffer);
        self.ui.render_side_notifications(framebuffer);

        if base.game_over {
            framebuffer.fill_rect(4, 132, 92, 42, Pixel::rgb(10, 8, 16));
            framebuffer.draw_rect(4, 132, 92, 42, DANGER);
            framebuffer.draw_text(12, 141, "PILGRIM FALLEN", DANGER);
            framebuffer.draw_text(12, 158, "SPACE RETRY", TEXT);
        }
    }
}

impl Game for VoidCanticleV11 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.ui.inner.inner.base.controls.update(frame.input);

        if self.ui.inner.inner.base.game_over {
            if self.ui.inner.inner.base.controls.action(FIRE).pressed() {
                self.reset_run();
            }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.ui.inner.inner.base.focus_held(frame);
        let canticle_pressed = self.ui.inner.inner.base.canticle_pressed(frame);

        self.ui.inner.wipe_banner_timer = (self.ui.inner.wipe_banner_timer - dt).max(0.0);
        self.ui.inner.inner.base.scroll =
            (self.ui.inner.inner.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.ui.inner.inner.base.update_feedback(dt);
        self.ui.inner.inner.base.update_player(dt, focused);
        self.ui.inner.inner.update_player_fire(dt, frame);
        self.ui.inner.update_wave_flow(dt, frame);
        self.spawn_specials_for_wave();
        self.ui.inner.inner.base.update_enemies(dt, frame);
        self.update_specials(dt, frame);
        self.ui.inner.inner.base.update_projectiles(dt);
        self.ui.inner.inner.update_power_shots(dt);
        self.resolve_power_shots(frame);
        self.ui.inner.inner.base.update_cinders(dt, frame);
        self.ui.inner.inner.update_relics(dt, frame);

        if canticle_pressed {
            let can_trigger = self.ui.inner.inner.base.encounter_phase == EncounterPhase::Waves
                && self.ui.inner.inner.base.core_charge >= CORE_MAX
                && self.ui.inner.inner.base.canticle_timer <= 0.0;
            if can_trigger {
                self.ui.inner.canticle_used_in_wave = true;
                self.apply_canticle_to_specials();
            }
            self.ui.inner.inner.base.trigger_canticle(frame);
        }

        self.finish_wave_if_clear();
        self.ui.update_wave_toast(dt);

        let previous_lives = self.ui.inner.inner.base.lives;
        self.resolve_special_player_hit(frame);
        self.ui.inner.inner.base.resolve_player_hits(frame);
        self.ui.inner.inner.apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

fn special_color(kind: SpecialKind) -> Pixel {
    match kind {
        SpecialKind::GraveKnight => KNIGHT_METAL,
        SpecialKind::BellWraith => WRAITH_GLOW,
        SpecialKind::RelicCarrier => CARRIER_GOLD,
    }
}

fn render_special(framebuffer: &mut Framebuffer, enemy: SpecialEnemy) {
    let x = enemy.x.round() as i32;
    let y = enemy.y.round() as i32;

    match enemy.kind {
        SpecialKind::GraveKnight => {
            if (0.90..1.55).contains(&enemy.age) {
                framebuffer.draw_line(x, y + 8, x, FRAMEBUFFER_HEIGHT as i32 - 18, DANGER);
            }
            framebuffer.fill_rect(x - 3, y - 8, 7, 17, KNIGHT_METAL);
            framebuffer.fill_rect(x - 8, y - 4, 17, 5, KNIGHT_METAL);
            framebuffer.draw_rect(x - 5, y - 10, 11, 7, PILGRIM_GOLD);
            framebuffer.fill_rect(x - 1, y - 7, 3, 3, DANGER);
            framebuffer.draw_line(x - 4, y + 8, x - 6, y + 13, WRECK_LIGHT);
            framebuffer.draw_line(x + 4, y + 8, x + 6, y + 13, WRECK_LIGHT);
        }
        SpecialKind::BellWraith => {
            let pulse = 7 + ((enemy.age * 5.0).sin().abs() * 3.0) as u32;
            framebuffer.draw_circle(x, y, pulse, WRAITH_GLOW);
            framebuffer.draw_circle(x, y, pulse.saturating_add(4), ENEMY_DARK);
            framebuffer.draw_line(x - 8, y, x, y - 10, WRAITH_GLOW);
            framebuffer.draw_line(x, y - 10, x + 8, y, WRAITH_GLOW);
            framebuffer.draw_line(x + 8, y, x, y + 10, WRAITH_GLOW);
            framebuffer.draw_line(x, y + 10, x - 8, y, WRAITH_GLOW);
            framebuffer.fill_rect(x - 1, y - 1, 3, 3, POWER_RELIC_LIGHT);
        }
        SpecialKind::RelicCarrier => {
            framebuffer.draw_rect(x - 9, y - 6, 19, 13, CARRIER_GOLD);
            framebuffer.draw_line(x - 9, y - 6, x - 14, y, WRECK_LIGHT);
            framebuffer.draw_line(x + 9, y - 6, x + 14, y, WRECK_LIGHT);
            framebuffer.draw_line(x - 9, y + 6, x - 14, y, WRECK_LIGHT);
            framebuffer.draw_line(x + 9, y + 6, x + 14, y, WRECK_LIGHT);
            framebuffer.draw_circle(x, y, 4, POWER_RELIC);
            framebuffer.fill_rect(x - 1, y - 3, 3, 7, POWER_RELIC_LIGHT);
        }
    }
}

fn v11_total_specials() -> usize {
    V11_SPECIAL_WAVES.iter().map(|wave| wave.len()).sum()
}

fn v11_carrier_waves() -> Vec<usize> {
    V11_SPECIAL_WAVES
        .iter()
        .enumerate()
        .filter_map(|(index, wave)| {
            wave.iter()
                .any(|spawn| spawn.kind == SpecialKind::RelicCarrier)
                .then_some(index + 1)
        })
        .collect()
}

pub fn run_v11_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePause::new(VoidCanticleV11::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v11_tests {
    use super::*;

    #[test]
    fn grave_orbit_keeps_fifteen_waves_and_adds_specialists() {
        assert_eq!(V09_WAVES.len(), 15);
        assert_eq!(V11_SPECIAL_WAVES.len(), 15);
        assert!(v11_total_specials() >= 18);
        assert!(V11_SPECIAL_WAVES[0].is_empty());
        assert!(V11_SPECIAL_WAVES[1].is_empty());
    }

    #[test]
    fn relic_carriers_space_power_progression_across_stage() {
        assert_eq!(v11_carrier_waves(), vec![5, 8, 11, 14, 15]);
    }

    #[test]
    fn specialists_have_distinct_durability() {
        let knight = SpecialEnemy::new(SpecialSpawn {
            at: 0.0,
            kind: SpecialKind::GraveKnight,
            x: 90.0,
            y: -16.0,
            direction: 0.0,
        });
        let wraith = SpecialEnemy::new(SpecialSpawn {
            at: 0.0,
            kind: SpecialKind::BellWraith,
            x: 90.0,
            y: -18.0,
            direction: 0.0,
        });
        let carrier = SpecialEnemy::new(SpecialSpawn {
            at: 0.0,
            kind: SpecialKind::RelicCarrier,
            x: 90.0,
            y: 58.0,
            direction: 1.0,
        });

        assert_eq!(knight.hp, 3);
        assert_eq!(wraith.hp, 4);
        assert_eq!(carrier.hp, 2);
    }

    #[test]
    fn vc11_version_is_explicit() {
        assert_eq!(VC11_VERSION, "VC1.1");
    }
}
