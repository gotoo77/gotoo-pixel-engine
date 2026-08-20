const V08_BOSS_GATE_TIME: f32 = 27.0;

const V08_SPAWNS: &[SpawnSpec] = &[
    // WAVE I — PROCESSION: readable opening, then first fan pressure.
    SpawnSpec { at: 0.45, base_x: 34.0, phase: 0.0, curve_amplitude: 22.0, first_shot: 1.85, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 0.78, base_x: 34.0, phase: 0.5, curve_amplitude: 22.0, first_shot: 2.00, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 1.11, base_x: 34.0, phase: 1.0, curve_amplitude: 22.0, first_shot: 1.70, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 2.10, base_x: 146.0, phase: 3.1, curve_amplitude: -22.0, first_shot: 1.75, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 2.45, base_x: 146.0, phase: 3.7, curve_amplitude: -24.0, first_shot: 1.55, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 3.35, base_x: 90.0, phase: 0.0, curve_amplitude: 48.0, first_shot: 1.35, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 3.85, base_x: 90.0, phase: 2.1, curve_amplitude: 48.0, first_shot: 1.20, pattern: ShotPattern::Aimed },

    // WAVE II — CROSSFIRE: mirrored entries force horizontal reads.
    SpawnSpec { at: 5.75, base_x: 28.0, phase: 0.4, curve_amplitude: 28.0, first_shot: 1.30, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 5.75, base_x: 152.0, phase: 3.5, curve_amplitude: -28.0, first_shot: 1.30, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 6.55, base_x: 48.0, phase: 0.9, curve_amplitude: 36.0, first_shot: 1.20, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 6.55, base_x: 132.0, phase: 4.0, curve_amplitude: -36.0, first_shot: 1.20, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 7.55, base_x: 90.0, phase: 0.0, curve_amplitude: 58.0, first_shot: 1.05, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 8.55, base_x: 35.0, phase: 1.8, curve_amplitude: 22.0, first_shot: 1.10, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 8.55, base_x: 145.0, phase: 4.9, curve_amplitude: -22.0, first_shot: 1.10, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 9.75, base_x: 90.0, phase: 3.0, curve_amplitude: 54.0, first_shot: 0.95, pattern: ShotPattern::Fan5 },

    // WAVE III — RESONANCE: wide oscillations and simultaneous fan fire.
    SpawnSpec { at: 11.95, base_x: 24.0, phase: 0.0, curve_amplitude: 30.0, first_shot: 0.95, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 11.95, base_x: 156.0, phase: 3.14, curve_amplitude: -30.0, first_shot: 0.95, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 12.85, base_x: 52.0, phase: 1.0, curve_amplitude: 38.0, first_shot: 1.00, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 12.85, base_x: 128.0, phase: 4.1, curve_amplitude: -38.0, first_shot: 1.00, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 14.00, base_x: 90.0, phase: 0.3, curve_amplitude: 65.0, first_shot: 0.85, pattern: ShotPattern::Aimed },
    SpawnSpec { at: 15.25, base_x: 40.0, phase: 2.2, curve_amplitude: 48.0, first_shot: 0.85, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 15.25, base_x: 140.0, phase: 5.3, curve_amplitude: -48.0, first_shot: 0.85, pattern: ShotPattern::Fan5 },

    // WAVE IV — FINAL CHOIR: dense closing phrase before Bellkeeper.
    SpawnSpec { at: 18.80, base_x: 30.0, phase: 0.2, curve_amplitude: 34.0, first_shot: 0.80, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 18.80, base_x: 150.0, phase: 3.3, curve_amplitude: -34.0, first_shot: 0.80, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 19.65, base_x: 62.0, phase: 1.2, curve_amplitude: 46.0, first_shot: 0.78, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 19.65, base_x: 118.0, phase: 4.3, curve_amplitude: -46.0, first_shot: 0.78, pattern: ShotPattern::Fan3 },
    SpawnSpec { at: 20.70, base_x: 90.0, phase: 0.0, curve_amplitude: 68.0, first_shot: 0.72, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 21.75, base_x: 28.0, phase: 2.0, curve_amplitude: 28.0, first_shot: 0.72, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 21.75, base_x: 152.0, phase: 5.1, curve_amplitude: -28.0, first_shot: 0.72, pattern: ShotPattern::Fan5 },
    SpawnSpec { at: 23.05, base_x: 90.0, phase: 3.14, curve_amplitude: 58.0, first_shot: 0.68, pattern: ShotPattern::Fan5 },
];

struct VoidCanticleV08 {
    inner: VoidCanticleV07,
}

impl VoidCanticleV08 {
    fn new() -> Self {
        Self {
            inner: VoidCanticleV07::new(),
        }
    }

    fn reset_run(&mut self) {
        self.inner.reset_run();
    }

    fn update_encounter(&mut self, dt: f32, frame: &mut Frame<'_>) {
        match self.inner.base.encounter_phase {
            EncounterPhase::Waves => {
                self.inner.base.stage_time += dt;

                while self.inner.base.next_spawn < V08_SPAWNS.len()
                    && V08_SPAWNS[self.inner.base.next_spawn].at <= self.inner.base.stage_time
                {
                    self.inner
                        .base
                        .enemies
                        .push(CarrionDrone::new(V08_SPAWNS[self.inner.base.next_spawn]));
                    self.inner.base.next_spawn += 1;
                }

                if self.inner.base.next_spawn == V08_SPAWNS.len()
                    && self.inner.base.stage_time >= V08_BOSS_GATE_TIME
                    && self.inner.base.enemies.is_empty()
                {
                    self.inner.base.begin_boss_intro(frame);
                }
            }
            EncounterPhase::BossIntro => self.inner.base.update_encounter(dt, frame),
            EncounterPhase::BossFight => self.inner.base.update_boss(dt, frame),
            EncounterPhase::Cleared => {}
        }
    }

    fn render(&self, framebuffer: &mut Framebuffer, focused: bool) {
        self.inner.render(framebuffer, focused);

        if self.inner.base.encounter_phase == EncounterPhase::Waves {
            let (wave, name) = v08_wave_descriptor(self.inner.base.stage_time);
            framebuffer.draw_text(
                4,
                39,
                &format!("WAVE {wave}/4 {name}"),
                WRECK_LIGHT,
            );
        }
    }
}

impl Game for VoidCanticleV08 {
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

        self.inner.base.scroll =
            (self.inner.base.scroll + 34.0 * dt) % FRAMEBUFFER_HEIGHT as f32;
        self.inner.base.update_feedback(dt);
        self.inner.base.update_player(dt, focused);
        self.inner.update_player_fire(dt, frame);
        self.update_encounter(dt, frame);
        self.inner.base.update_enemies(dt, frame);
        self.inner.base.update_projectiles(dt);
        self.inner.update_power_shots(dt);
        self.inner.resolve_power_shots(frame);
        self.inner.base.update_cinders(dt, frame);
        self.inner.update_relics(dt, frame);

        if canticle_pressed {
            self.inner.base.trigger_canticle(frame);
        }

        let previous_lives = self.inner.base.lives;
        self.inner.base.resolve_player_hits(frame);
        self.inner.apply_death_penalty(previous_lives);

        self.render(frame.framebuffer, focused);
        GameResult::Continue
    }
}

fn v08_wave_descriptor(stage_time: f32) -> (u8, &'static str) {
    if stage_time < 5.75 {
        (1, "PROCESSION")
    } else if stage_time < 11.95 {
        (2, "CROSSFIRE")
    } else if stage_time < 18.80 {
        (3, "RESONANCE")
    } else {
        (4, "FINAL CHOIR")
    }
}

pub fn run_v08() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC07_VERSION} [{BUILD_ID}] - Gotoo Pixel Engine"
            ),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        PauseGame::new(
            VoidCanticleV08::new(),
            PauseConfig::new(Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }),
        ),
    )
}

#[cfg(test)]
mod v08_tests {
    use super::*;

    #[test]
    fn grave_orbit_v08_has_a_real_multi_wave_stage() {
        assert!(V08_SPAWNS.len() >= 28);
        assert!(V08_SPAWNS.windows(2).all(|pair| pair[0].at <= pair[1].at));
        assert!(V08_SPAWNS.last().is_some_and(|spawn| spawn.at > 20.0));
        assert!(V08_SPAWNS.last().is_some_and(|spawn| spawn.at < V08_BOSS_GATE_TIME));
    }

    #[test]
    fn opening_wave_cannot_already_reach_max_power() {
        let opening_kills = 7_u32;
        let opening_drops = (1..=opening_kills)
            .filter(|kill| should_drop_relic(*kill))
            .count() as u8;
        assert_eq!(START_POWER_LEVEL + opening_drops, 2);

        let full_stage_drops = (1..=V08_SPAWNS.len() as u32)
            .filter(|kill| should_drop_relic(*kill))
            .count() as u8;
        assert_eq!(START_POWER_LEVEL + full_stage_drops, MAX_POWER_LEVEL);
    }

    #[test]
    fn wave_labels_cover_all_four_sections() {
        assert_eq!(v08_wave_descriptor(0.0).0, 1);
        assert_eq!(v08_wave_descriptor(6.0).0, 2);
        assert_eq!(v08_wave_descriptor(12.0).0, 3);
        assert_eq!(v08_wave_descriptor(19.0).0, 4);
    }
}
