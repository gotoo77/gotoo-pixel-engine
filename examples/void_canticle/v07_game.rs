struct VoidCanticleV07 {
    base: VoidCanticleGame,
    pilgrim_visuals: PilgrimV07Visuals,
    power_level: u8,
    power_shots: Vec<PowerShot>,
    relics: Vec<RelicDrop>,
    destroyed_enemies: u32,
}

impl VoidCanticleV07 {
    fn new() -> Self {
        let mut base = VoidCanticleGame::new();
        base.sounds
            .insert_wav(
                POWERUP_SOUND,
                synthesize_chirp(480.0, 1520.0, 0.17, 0.27),
            )
            .expect("Void Canticle power-up sound id should be unique");

        Self {
            base,
            pilgrim_visuals: PilgrimV07Visuals::new(),
            power_level: START_POWER_LEVEL,
            power_shots: Vec::new(),
            relics: Vec::new(),
            destroyed_enemies: 0,
        }
    }

    fn reset_run(&mut self) {
        self.base.reset_run();
        self.power_level = START_POWER_LEVEL;
        self.power_shots.clear();
        self.relics.clear();
        self.destroyed_enemies = 0;
    }

    fn update_player_fire(&mut self, dt: f32, frame: &mut Frame<'_>) {
        self.base.fire_cooldown = (self.base.fire_cooldown - dt).max(0.0);
        if !self.base.controls.action(FIRE).held() || self.base.fire_cooldown > 0.0 {
            return;
        }

        spawn_power_volley(
            &mut self.power_shots,
            self.power_level,
            self.base.player_x,
            self.base.player_y - 12.0,
        );
        self.base.fire_cooldown = weapon_profile(self.power_level).period;
        let _ = self.base.sounds.play(frame.audio, FIRE_SOUND);
    }

    fn update_power_shots(&mut self, dt: f32) {
        for shot in &mut self.power_shots {
            shot.x += shot.vx * dt;
            shot.y += shot.vy * dt;
            shot.alive = shot.y > 18.0
                && shot.x > -12.0
                && shot.x < FRAMEBUFFER_WIDTH as f32 + 12.0;
        }
        self.power_shots.retain(|shot| shot.alive);
    }

    fn resolve_power_shots(&mut self, frame: &mut Frame<'_>) {
        let mut destroyed = Vec::new();
        let mut boss_damage = 0_u32;

        for shot in &mut self.power_shots {
            if !shot.alive {
                continue;
            }

            for enemy in &mut self.base.enemies {
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
                    destroyed.push((enemy.x, enemy.y));
                    break;
                }
            }

            if shot.alive {
                if let Some(boss) = self.base.boss.as_mut() {
                    if self.base.encounter_phase == EncounterPhase::BossFight
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
            }
        }

        self.power_shots.retain(|shot| shot.alive);
        self.base.enemies.retain(|enemy| enemy.alive);

        for (x, y) in destroyed {
            self.destroyed_enemies = self.destroyed_enemies.saturating_add(1);
            self.base.score = self.base.score.saturating_add(100);
            self.base.cinders.push(CinderDrop {
                x,
                y,
                age: 0.0,
                phase: x * 0.11 + y * 0.07,
                alive: true,
            });
            self.base
                .bursts
                .push(Burst::new(x, y, 0.24, ENEMY_EYE));

            if should_drop_relic(self.destroyed_enemies) {
                self.relics.push(RelicDrop {
                    x,
                    y,
                    age: 0.0,
                    phase: x * 0.09 + y * 0.13,
                    alive: true,
                });
            }

            let _ = self.base.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }

        if boss_damage > 0 {
            self.base.score = self.base.score.saturating_add(boss_damage * 5);
            if let Some(boss) = self.base.boss {
                self.base
                    .bursts
                    .push(Burst::new(boss.x, boss.y + 3.0, 0.11, BELL_LIGHT));
            }
        }

        self.base.finish_boss_if_destroyed(frame);
    }

    fn update_relics(&mut self, dt: f32, frame: &mut Frame<'_>) {
        let mut collected = 0_u32;

        for relic in &mut self.relics {
            relic.age += dt;
            relic.y += 18.0 * dt;
            relic.x += (relic.age * 3.8 + relic.phase).sin() * 8.0 * dt;

            if point_near(
                relic.x,
                relic.y,
                self.base.player_x,
                self.base.player_y,
                11.0,
            ) {
                relic.alive = false;
                collected += 1;
            } else if relic.y > FRAMEBUFFER_HEIGHT as f32 + 10.0 {
                relic.alive = false;
            }
        }

        self.relics.retain(|relic| relic.alive);

        for _ in 0..collected {
            if self.power_level < MAX_POWER_LEVEL {
                self.power_level += 1;
                self.base.score = self.base.score.saturating_add(250);
            } else {
                self.base.score = self.base.score.saturating_add(500);
                self.base.core_charge = (self.base.core_charge + 10).min(CORE_MAX);
            }
            let _ = self.base.sounds.play(frame.audio, POWERUP_SOUND);
            self.base.bursts.push(Burst::new(
                self.base.player_x,
                self.base.player_y - 4.0,
                0.38,
                POWER_RELIC_LIGHT,
            ));
        }
    }

    fn apply_death_penalty(&mut self, previous_lives: u32) {
        if self.base.lives < previous_lives {
            self.power_level = power_after_death(self.power_level);
            self.power_shots.clear();
        }
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        framebuffer.clear(if self.base.canticle_timer > 0.46 {
            BG_CANTICLE
        } else {
            BG
        });
        render_grave_orbit_background(framebuffer, self.base.scroll);

        for cinder in &self.base.cinders {
            let x = cinder.x.round() as i32;
            let y = cinder.y.round() as i32;
            framebuffer.fill_circle(x, y, 2, CINDER);
            framebuffer.draw(x, y - 4, CANTICLE_COLOR);
        }

        for relic in &self.relics {
            render_relic(framebuffer, *relic);
        }

        for enemy in &self.base.enemies {
            self.base.visuals.render_carrion(framebuffer, *enemy);
        }

        if let Some(boss) = self.base.boss {
            if self.base.encounter_phase != EncounterPhase::Cleared {
                self.base.visuals.render_bellkeeper(framebuffer, boss);
            }
        }

        for shot in &self.power_shots {
            render_power_shot(framebuffer, *shot, self.power_level);
        }

        for bullet in &self.base.enemy_bullets {
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

        for burst in &self.base.bursts {
            render_burst(framebuffer, *burst);
        }

        self.pilgrim_visuals.render(
            framebuffer,
            self.base.player_x.round() as i32,
            self.base.player_y.round() as i32,
            focused,
            self.base.invulnerability,
            self.base.animation_time,
        );

        if self.base.canticle_timer > 0.0 {
            render_canticle(
                framebuffer,
                self.base.player_x.round() as i32,
                self.base.player_y.round() as i32,
                self.base.canticle_timer,
            );
        }

        self.render_hud(framebuffer);
        self.base.render_encounter_overlay(framebuffer);

        if self.base.game_over {
            framebuffer.fill_rect(20, 132, 140, 48, Pixel::rgb(12, 10, 18));
            framebuffer.draw_rect(20, 132, 140, 48, DANGER);
            framebuffer.draw_text(41, 143, "PILGRIM FALLEN", DANGER);
            framebuffer.draw_text(37, 160, "SPACE TO RETURN", TEXT);
        }
    }

    fn render_hud(&self, framebuffer: &mut Framebuffer) {
        framebuffer.draw_text(4, 4, "VOID CANTICLE", ACCENT);
        framebuffer.draw_text(
            4,
            15,
            &format!("GRAVE ORBIT / {VC07_VERSION}"),
            TEXT,
        );
        framebuffer.draw_text(4, 27, &format!("LIVES {}", self.base.lives), TEXT);
        framebuffer.draw_text(82, 27, &format!("BUILD {BUILD_ID}"), WRECK_LIGHT);
        framebuffer.draw_text(
            126,
            4,
            &format!("{}", self.base.score),
            PILGRIM_CORE,
        );

        if let Some(boss) = self.base.boss {
            if self.base.encounter_phase == EncounterPhase::BossFight {
                framebuffer.draw_text(4, 39, "BELLKEEPER", BELL_LIGHT);
                framebuffer.fill_rect(66, 40, 106, 5, CORE_BG);
                let width = 106_u32.saturating_mul(boss.hp) / BELLKEEPER_MAX_HP;
                framebuffer.fill_rect(66, 40, width, 5, DANGER);
                let phase = match boss.phase() {
                    BellPhase::Procession => "TOLL I",
                    BellPhase::Resonance => "TOLL II",
                    BellPhase::FinalToll => "FINAL TOLL",
                };
                framebuffer.draw_text(4, 49, phase, WRECK_LIGHT);
            }
        }

        let profile = weapon_profile(self.power_level);
        framebuffer.draw_text(
            4,
            256,
            &format!("POWER {} {}", self.power_level, profile.name),
            POWER_RELIC_LIGHT,
        );
        for index in 0..MAX_POWER_LEVEL {
            let color = if index < self.power_level {
                POWER_RELIC
            } else {
                CORE_BG
            };
            framebuffer.fill_rect(132 + i32::from(index) * 7, 258, 5, 4, color);
        }

        framebuffer.draw_text(4, 270, "CORE", TEXT);
        framebuffer.fill_rect(34, 271, 138, 6, CORE_BG);
        let charge_width = 138_u32.saturating_mul(self.base.core_charge) / CORE_MAX;
        framebuffer.fill_rect(34, 271, charge_width, 6, CINDER);
        if self.base.core_charge >= CORE_MAX {
            framebuffer.draw_text(113, 280, "READY", CANTICLE_COLOR);
        }

        framebuffer.draw_text(4, 291, "SPACE FIRE", TEXT);
        framebuffer.draw_text(76, 291, "SHIFT FOCUS", TEXT);
        framebuffer.draw_text(4, 303, "X CANTICLE", TEXT);
    }
}

impl Game for VoidCanticleV07 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.base.controls.update(frame.input);

        if self.base.game_over {
            if self.base.controls.action(FIRE).pressed() {
                self.reset_run();
            }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.base.focus_held(frame);
        let canticle_pressed = self.base.canticle_pressed(frame);

        self.base.scroll = (self.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.base.update_feedback(dt);
        self.base.update_player(dt, focused);
        self.update_player_fire(dt, frame);
        self.base.update_encounter(dt, frame);
        self.base.update_enemies(dt, frame);
        self.base.update_projectiles(dt);
        self.update_power_shots(dt);
        self.resolve_power_shots(frame);
        self.base.update_cinders(dt, frame);
        self.update_relics(dt, frame);

        if canticle_pressed {
            self.base.trigger_canticle(frame);
        }

        let previous_lives = self.base.lives;
        self.base.resolve_player_hits(frame);
        self.apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

