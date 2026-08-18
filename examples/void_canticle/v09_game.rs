const VC09_VERSION: &str = "VC0.9";
const V09_INTERMISSION: f32 = 0.85;
const V09_WIPE_BANNER_DURATION: f32 = 1.35;

#[derive(Debug, Clone, Copy)]
struct WaveDef {
    name: &'static str,
    spawns: &'static [SpawnSpec],
}

const V09_WAVES: &[WaveDef] = &[
    WaveDef {
        name: "PROCESSION",
        spawns: &[
            SpawnSpec { at: 0.20, base_x: 38.0, phase: 0.0, curve_amplitude: 18.0, first_shot: 1.90, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 0.55, base_x: 62.0, phase: 0.6, curve_amplitude: 18.0, first_shot: 1.90, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 0.90, base_x: 118.0, phase: 2.8, curve_amplitude: -18.0, first_shot: 1.90, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 1.25, base_x: 142.0, phase: 3.4, curve_amplitude: -18.0, first_shot: 1.90, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "TWIN GRAVES",
        spawns: &[
            SpawnSpec { at: 0.20, base_x: 34.0, phase: 0.2, curve_amplitude: 24.0, first_shot: 1.45, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.20, base_x: 146.0, phase: 3.3, curve_amplitude: -24.0, first_shot: 1.45, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.95, base_x: 58.0, phase: 1.1, curve_amplitude: 26.0, first_shot: 1.55, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 0.95, base_x: 122.0, phase: 4.2, curve_amplitude: -26.0, first_shot: 1.55, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "CROSSING",
        spawns: &[
            SpawnSpec { at: 0.15, base_x: 28.0, phase: 0.0, curve_amplitude: 42.0, first_shot: 1.25, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.45, base_x: 152.0, phase: 3.14, curve_amplitude: -42.0, first_shot: 1.25, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.95, base_x: 42.0, phase: 1.0, curve_amplitude: 36.0, first_shot: 1.20, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 1.25, base_x: 138.0, phase: 4.1, curve_amplitude: -36.0, first_shot: 1.20, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 1.75, base_x: 90.0, phase: 0.4, curve_amplitude: 54.0, first_shot: 1.10, pattern: ShotPattern::Fan3 },
        ],
    },
    WaveDef {
        name: "BELL ECHO",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 90.0, phase: 0.0, curve_amplitude: 58.0, first_shot: 1.00, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.55, base_x: 50.0, phase: 1.2, curve_amplitude: 32.0, first_shot: 1.05, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.55, base_x: 130.0, phase: 4.3, curve_amplitude: -32.0, first_shot: 1.05, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 1.35, base_x: 34.0, phase: 2.2, curve_amplitude: 22.0, first_shot: 1.00, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 1.35, base_x: 146.0, phase: 5.3, curve_amplitude: -22.0, first_shot: 1.00, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "SCAVENGER CHOIR",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 32.0, phase: 0.3, curve_amplitude: 28.0, first_shot: 0.95, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.10, base_x: 148.0, phase: 3.4, curve_amplitude: -28.0, first_shot: 0.95, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.70, base_x: 60.0, phase: 1.0, curve_amplitude: 38.0, first_shot: 0.95, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.70, base_x: 120.0, phase: 4.1, curve_amplitude: -38.0, first_shot: 0.95, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 1.55, base_x: 90.0, phase: 2.4, curve_amplitude: 62.0, first_shot: 0.90, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "DEAD ORBIT",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 24.0, phase: 0.0, curve_amplitude: 34.0, first_shot: 0.90, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.10, base_x: 156.0, phase: 3.14, curve_amplitude: -34.0, first_shot: 0.90, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.55, base_x: 50.0, phase: 0.8, curve_amplitude: 42.0, first_shot: 0.88, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.55, base_x: 130.0, phase: 3.9, curve_amplitude: -42.0, first_shot: 0.88, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 1.20, base_x: 76.0, phase: 1.6, curve_amplitude: 50.0, first_shot: 0.86, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 1.20, base_x: 104.0, phase: 4.7, curve_amplitude: -50.0, first_shot: 0.86, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "WRAITH LINE",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 34.0, phase: 0.0, curve_amplitude: 20.0, first_shot: 0.85, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.42, base_x: 62.0, phase: 0.7, curve_amplitude: 24.0, first_shot: 0.85, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.74, base_x: 90.0, phase: 1.4, curve_amplitude: 28.0, first_shot: 0.82, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 1.06, base_x: 118.0, phase: 2.1, curve_amplitude: -24.0, first_shot: 0.85, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 1.38, base_x: 146.0, phase: 2.8, curve_amplitude: -20.0, first_shot: 0.85, pattern: ShotPattern::Fan5 },
        ],
    },
    WaveDef {
        name: "RELIC RUN",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 30.0, phase: 0.4, curve_amplitude: 40.0, first_shot: 0.82, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.10, base_x: 150.0, phase: 3.5, curve_amplitude: -40.0, first_shot: 0.82, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.62, base_x: 48.0, phase: 1.2, curve_amplitude: 48.0, first_shot: 0.78, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.62, base_x: 132.0, phase: 4.3, curve_amplitude: -48.0, first_shot: 0.78, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 1.22, base_x: 66.0, phase: 2.0, curve_amplitude: 54.0, first_shot: 0.80, pattern: ShotPattern::Aimed },
            SpawnSpec { at: 1.22, base_x: 114.0, phase: 5.1, curve_amplitude: -54.0, first_shot: 0.80, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "RESONANCE",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 26.0, phase: 0.0, curve_amplitude: 44.0, first_shot: 0.76, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.10, base_x: 154.0, phase: 3.14, curve_amplitude: -44.0, first_shot: 0.76, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.72, base_x: 54.0, phase: 1.3, curve_amplitude: 50.0, first_shot: 0.76, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.72, base_x: 126.0, phase: 4.4, curve_amplitude: -50.0, first_shot: 0.76, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 1.45, base_x: 90.0, phase: 2.6, curve_amplitude: 68.0, first_shot: 0.72, pattern: ShotPattern::Fan5 },
        ],
    },
    WaveDef {
        name: "BLACK HALO",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 90.0, phase: 0.0, curve_amplitude: 70.0, first_shot: 0.70, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.35, base_x: 32.0, phase: 1.1, curve_amplitude: 26.0, first_shot: 0.72, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.35, base_x: 148.0, phase: 4.2, curve_amplitude: -26.0, first_shot: 0.72, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.95, base_x: 55.0, phase: 2.0, curve_amplitude: 44.0, first_shot: 0.70, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.95, base_x: 125.0, phase: 5.1, curve_amplitude: -44.0, first_shot: 0.70, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 1.55, base_x: 90.0, phase: 3.14, curve_amplitude: 58.0, first_shot: 0.68, pattern: ShotPattern::Aimed },
        ],
    },
    WaveDef {
        name: "GRAVE CROSS",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 28.0, phase: 0.2, curve_amplitude: 34.0, first_shot: 0.68, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.10, base_x: 152.0, phase: 3.3, curve_amplitude: -34.0, first_shot: 0.68, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.55, base_x: 58.0, phase: 1.1, curve_amplitude: 52.0, first_shot: 0.66, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.55, base_x: 122.0, phase: 4.2, curve_amplitude: -52.0, first_shot: 0.66, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 1.15, base_x: 90.0, phase: 2.4, curve_amplitude: 72.0, first_shot: 0.64, pattern: ShotPattern::Fan5 },
        ],
    },
    WaveDef {
        name: "IRON RAIN",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 24.0, phase: 0.0, curve_amplitude: 30.0, first_shot: 0.64, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.10, base_x: 156.0, phase: 3.14, curve_amplitude: -30.0, first_shot: 0.64, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.45, base_x: 46.0, phase: 0.8, curve_amplitude: 42.0, first_shot: 0.64, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.45, base_x: 134.0, phase: 3.9, curve_amplitude: -42.0, first_shot: 0.64, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.90, base_x: 68.0, phase: 1.6, curve_amplitude: 54.0, first_shot: 0.62, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.90, base_x: 112.0, phase: 4.7, curve_amplitude: -54.0, first_shot: 0.62, pattern: ShotPattern::Fan5 },
        ],
    },
    WaveDef {
        name: "VOID LITANY",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 30.0, phase: 0.0, curve_amplitude: 50.0, first_shot: 0.60, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.10, base_x: 150.0, phase: 3.14, curve_amplitude: -50.0, first_shot: 0.60, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.48, base_x: 50.0, phase: 1.0, curve_amplitude: 56.0, first_shot: 0.62, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.48, base_x: 130.0, phase: 4.1, curve_amplitude: -56.0, first_shot: 0.62, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.96, base_x: 70.0, phase: 2.0, curve_amplitude: 62.0, first_shot: 0.60, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.96, base_x: 110.0, phase: 5.1, curve_amplitude: -62.0, first_shot: 0.60, pattern: ShotPattern::Fan5 },
        ],
    },
    WaveDef {
        name: "LAST PROCESSION",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 28.0, phase: 0.4, curve_amplitude: 40.0, first_shot: 0.58, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.10, base_x: 152.0, phase: 3.5, curve_amplitude: -40.0, first_shot: 0.58, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.42, base_x: 52.0, phase: 1.2, curve_amplitude: 52.0, first_shot: 0.58, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.42, base_x: 128.0, phase: 4.3, curve_amplitude: -52.0, first_shot: 0.58, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.82, base_x: 76.0, phase: 2.0, curve_amplitude: 66.0, first_shot: 0.56, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.82, base_x: 104.0, phase: 5.1, curve_amplitude: -66.0, first_shot: 0.56, pattern: ShotPattern::Fan3 },
        ],
    },
    WaveDef {
        name: "FINAL CHOIR",
        spawns: &[
            SpawnSpec { at: 0.10, base_x: 24.0, phase: 0.0, curve_amplitude: 44.0, first_shot: 0.54, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.10, base_x: 156.0, phase: 3.14, curve_amplitude: -44.0, first_shot: 0.54, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.38, base_x: 46.0, phase: 0.8, curve_amplitude: 56.0, first_shot: 0.54, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.38, base_x: 134.0, phase: 3.9, curve_amplitude: -56.0, first_shot: 0.54, pattern: ShotPattern::Fan5 },
            SpawnSpec { at: 0.72, base_x: 68.0, phase: 1.6, curve_amplitude: 68.0, first_shot: 0.52, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 0.72, base_x: 112.0, phase: 4.7, curve_amplitude: -68.0, first_shot: 0.52, pattern: ShotPattern::Fan3 },
            SpawnSpec { at: 1.20, base_x: 90.0, phase: 2.6, curve_amplitude: 76.0, first_shot: 0.50, pattern: ShotPattern::Fan5 },
        ],
    },
];

struct VoidCanticleV09 {
    inner: VoidCanticleV07,
    wave_index: usize,
    wave_time: f32,
    next_spawn: usize,
    intermission: f32,
    wave_kills_start: u32,
    canticle_used_in_wave: bool,
    full_wipe_chain: u32,
    full_wipes: u32,
    wipe_banner_timer: f32,
    wipe_banner_chain: u32,
    wipe_banner_bonus: u32,
}

impl VoidCanticleV09 {
    fn new() -> Self {
        Self {
            inner: VoidCanticleV07::new(),
            wave_index: 0,
            wave_time: 0.0,
            next_spawn: 0,
            intermission: 0.65,
            wave_kills_start: 0,
            canticle_used_in_wave: false,
            full_wipe_chain: 0,
            full_wipes: 0,
            wipe_banner_timer: 0.0,
            wipe_banner_chain: 0,
            wipe_banner_bonus: 0,
        }
    }

    fn reset_run(&mut self) {
        self.inner.reset_run();
        self.wave_index = 0;
        self.wave_time = 0.0;
        self.next_spawn = 0;
        self.intermission = 0.65;
        self.wave_kills_start = 0;
        self.canticle_used_in_wave = false;
        self.full_wipe_chain = 0;
        self.full_wipes = 0;
        self.wipe_banner_timer = 0.0;
        self.wipe_banner_chain = 0;
        self.wipe_banner_bonus = 0;
    }

    fn update_wave_flow(&mut self, dt: f32, frame: &mut Frame<'_>) {
        if self.inner.base.encounter_phase != EncounterPhase::Waves {
            self.inner.base.update_encounter(dt, frame);
            return;
        }

        if self.wave_index >= V09_WAVES.len() {
            self.intermission = (self.intermission - dt).max(0.0);
            if self.intermission <= 0.0 {
                self.inner.base.begin_boss_intro(frame);
            }
            return;
        }

        if self.intermission > 0.0 {
            self.intermission = (self.intermission - dt).max(0.0);
            return;
        }

        self.wave_time += dt;
        let wave = V09_WAVES[self.wave_index];
        while self.next_spawn < wave.spawns.len()
            && wave.spawns[self.next_spawn].at <= self.wave_time
        {
            self.inner
                .base
                .enemies
                .push(CarrionDrone::new(wave.spawns[self.next_spawn]));
            self.next_spawn += 1;
        }
    }

    fn finish_wave_if_clear(&mut self) {
        if self.inner.base.encounter_phase != EncounterPhase::Waves
            || self.wave_index >= V09_WAVES.len()
            || self.intermission > 0.0
        {
            return;
        }

        let wave = V09_WAVES[self.wave_index];
        if self.next_spawn < wave.spawns.len() || !self.inner.base.enemies.is_empty() {
            return;
        }

        let gun_kills = self
            .inner
            .destroyed_enemies
            .saturating_sub(self.wave_kills_start);
        let full_wipe = !self.canticle_used_in_wave && gun_kills == wave.spawns.len() as u32;

        if full_wipe {
            self.full_wipe_chain = self.full_wipe_chain.saturating_add(1);
            self.full_wipes = self.full_wipes.saturating_add(1);
            let bonus = full_wipe_bonus(self.full_wipe_chain);
            self.inner.base.score = self.inner.base.score.saturating_add(bonus);
            self.wipe_banner_timer = V09_WIPE_BANNER_DURATION;
            self.wipe_banner_chain = self.full_wipe_chain;
            self.wipe_banner_bonus = bonus;
        } else {
            self.full_wipe_chain = 0;
        }

        self.wave_index += 1;
        self.wave_time = 0.0;
        self.next_spawn = 0;
        self.intermission = V09_INTERMISSION;
        self.wave_kills_start = self.inner.destroyed_enemies;
        self.canticle_used_in_wave = false;
    }

    fn resolve_power_shots(&mut self, frame: &mut Frame<'_>) {
        let mut destroyed = Vec::new();
        let mut boss_damage = 0_u32;

        for shot in &mut self.inner.power_shots {
            if !shot.alive {
                continue;
            }

            for enemy in &mut self.inner.base.enemies {
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
                if let Some(boss) = self.inner.base.boss.as_mut() {
                    if self.inner.base.encounter_phase == EncounterPhase::BossFight
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

        self.inner.power_shots.retain(|shot| shot.alive);
        self.inner.base.enemies.retain(|enemy| enemy.alive);

        for (x, y) in destroyed {
            self.inner.destroyed_enemies = self.inner.destroyed_enemies.saturating_add(1);
            self.inner.base.score = self.inner.base.score.saturating_add(100);
            self.inner.base.cinders.push(CinderDrop {
                x,
                y,
                age: 0.0,
                phase: x * 0.11 + y * 0.07,
                alive: true,
            });
            self.inner
                .base
                .bursts
                .push(Burst::new(x, y, 0.24, ENEMY_EYE));

            if should_drop_relic_v09(self.inner.destroyed_enemies) {
                self.inner.relics.push(RelicDrop {
                    x,
                    y,
                    age: 0.0,
                    phase: x * 0.09 + y * 0.13,
                    alive: true,
                });
            }

            let _ = self.inner.base.sounds.play(frame.audio, ENEMY_HIT_SOUND);
        }

        if boss_damage > 0 {
            self.inner.base.score = self.inner.base.score.saturating_add(boss_damage * 5);
            if let Some(boss) = self.inner.base.boss {
                self.inner
                    .base
                    .bursts
                    .push(Burst::new(boss.x, boss.y + 3.0, 0.11, BELL_LIGHT));
            }
        }

        self.inner.base.finish_boss_if_destroyed(frame);
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        self.inner.render(framebuffer, focused);

        // Replace the legacy VC0.7 label rendered by the reused gameplay slice.
        framebuffer.fill_rect(88, 15, 42, 7, BG);
        framebuffer.draw_text(88, 15, VC09_VERSION, TEXT);

        if self.inner.base.encounter_phase == EncounterPhase::Waves {
            if self.wave_index < V09_WAVES.len() {
                framebuffer.draw_text(
                    4,
                    39,
                    &format!(
                        "WAVE {}/15 {}",
                        self.wave_index + 1,
                        V09_WAVES[self.wave_index].name
                    ),
                    WRECK_LIGHT,
                );
            } else {
                framebuffer.draw_text(4, 39, "BELLKEEPER APPROACHES", BELL_LIGHT);
            }
            framebuffer.draw_text(4, 49, &format!("FULL WIPES {}", self.full_wipes), WRECK_LIGHT);
        }

        if self.wipe_banner_timer > 0.0 {
            framebuffer.fill_rect(38, 82, 104, 30, Pixel::rgb(10, 8, 16));
            framebuffer.draw_rect(38, 82, 104, 30, POWER_RELIC_LIGHT);
            framebuffer.draw_text(54, 89, "FULL WIPE", POWER_RELIC_LIGHT);
            framebuffer.draw_text(
                58,
                101,
                &format!("CHAIN {} +{}", self.wipe_banner_chain, self.wipe_banner_bonus),
                POWER_RELIC,
            );
        }
    }
}

impl Game for VoidCanticleV09 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.inner.base.controls.update(frame.input);

        if self.inner.base.game_over {
            if self.inner.base.controls.action(FIRE).pressed() {
                self.reset_run();
            }
            self.render(frame.framebuffer, false);
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let focused = self.inner.base.focus_held(frame);
        let canticle_pressed = self.inner.base.canticle_pressed(frame);

        self.wipe_banner_timer = (self.wipe_banner_timer - dt).max(0.0);
        self.inner.base.scroll =
            (self.inner.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.inner.base.update_feedback(dt);
        self.inner.base.update_player(dt, focused);
        self.inner.update_player_fire(dt, frame);
        self.update_wave_flow(dt, frame);
        self.inner.base.update_enemies(dt, frame);
        self.inner.base.update_projectiles(dt);
        self.inner.update_power_shots(dt);
        self.resolve_power_shots(frame);
        self.inner.base.update_cinders(dt, frame);
        self.inner.update_relics(dt, frame);

        if canticle_pressed {
            if self.inner.base.encounter_phase == EncounterPhase::Waves
                && self.inner.base.core_charge >= CORE_MAX
                && self.inner.base.canticle_timer <= 0.0
            {
                self.canticle_used_in_wave = true;
            }
            self.inner.base.trigger_canticle(frame);
        }

        self.finish_wave_if_clear();

        let previous_lives = self.inner.base.lives;
        self.inner.base.resolve_player_hits(frame);
        self.inner.apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

fn should_drop_relic_v09(destroyed_enemies: u32) -> bool {
    matches!(destroyed_enemies, 12 | 30 | 50 | 70)
}

fn full_wipe_bonus(chain: u32) -> u32 {
    500_u32.saturating_mul(chain.max(1))
}

fn v09_total_spawns() -> usize {
    V09_WAVES.iter().map(|wave| wave.spawns.len()).sum()
}

pub fn run_v09() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC09_VERSION} [{BUILD_ID}] - Gotoo Pixel Engine"
            ),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        PauseGame::new(
            VoidCanticleV09::new(),
            PauseConfig::new(Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }),
        ),
    )
}

#[cfg(test)]
mod v09_tests {
    use super::*;

    #[test]
    fn grave_orbit_has_fifteen_authored_waves() {
        assert_eq!(V09_WAVES.len(), 15);
        assert_eq!(v09_total_spawns(), 81);
        assert!(V09_WAVES.iter().all(|wave| !wave.spawns.is_empty()));
        assert!(V09_WAVES.iter().all(|wave| {
            wave.spawns
                .windows(2)
                .all(|pair| pair[0].at <= pair[1].at)
        }));
    }

    #[test]
    fn relic_powerups_are_spread_across_the_long_stage() {
        let thresholds: Vec<u32> = (1..=v09_total_spawns() as u32)
            .filter(|kill| should_drop_relic_v09(*kill))
            .collect();
        assert_eq!(thresholds, vec![12, 30, 50, 70]);
        assert!(thresholds[0] > 10);
        assert!(thresholds[3] > 3 * v09_total_spawns() as u32 / 4);
    }

    #[test]
    fn full_wipe_chain_bonus_scales_linearly() {
        assert_eq!(full_wipe_bonus(1), 500);
        assert_eq!(full_wipe_bonus(2), 1_000);
        assert_eq!(full_wipe_bonus(5), 2_500);
    }
}
