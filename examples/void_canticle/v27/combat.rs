impl VoidCanticleV27DirectPresentation {
    fn render_choir_links(&self, framebuffer: &mut Framebuffer) {
        let v12 = &self.game.game.v20().game.v14().progression.combat;
        let link_color = Pixel::rgb(34, 62, 82);

        for node in v12
            .threats
            .iter()
            .filter(|threat| threat.alive && threat.kind == ThreatKind::ChoirNode)
        {
            let x = vc27_present(node.x);
            let y = vc27_present(node.y);
            for enemy in &self.game.game.base().enemies {
                if point_near(enemy.x, enemy.y, node.x, node.y, CHOIR_BUFF_RADIUS) {
                    framebuffer.draw_line(
                        x,
                        y,
                        vc27_present(enemy.x),
                        vc27_present(enemy.y),
                        link_color,
                    );
                }
            }
            for enemy in &v12.combat.specials {
                if enemy.kind == SpecialKind::BellWraith
                    && point_near(enemy.x, enemy.y, node.x, node.y, CHOIR_BUFF_RADIUS)
                {
                    framebuffer.draw_line(
                        x,
                        y,
                        vc27_present(enemy.x),
                        vc27_present(enemy.y),
                        link_color,
                    );
                }
            }
        }
    }

    fn render_presentation_bestiary(&self, framebuffer: &mut Framebuffer) {
        let v20 = self.game.game.v20();
        for enemy in &self.game.game.base().enemies {
            let x = vc27_present(enemy.x);
            let y = vc27_present(enemy.y);
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let armor = v20
                .carrion_armor
                .get(&vc20_carrion_key(enemy))
                .copied()
                .unwrap_or(armor_max);
            vc27_hd_render_carrion(framebuffer, x, y, enemy.age, enemy.phase);
            vc27_hd_render_damage_marks(
                framebuffer,
                x,
                y,
                vc27_damage_state(armor.saturating_add(1), armor_max.saturating_add(1)),
                enemy.age,
                34,
            );
        }

        let v12 = &v20.game.v12();
        for enemy in &v12.combat.specials {
            let x = vc27_present(enemy.x);
            let y = vc27_present(enemy.y);
            let armor_max = vc20_special_armor_max(enemy.kind);
            let armor = v20
                .special_armor
                .get(&vc20_special_key(enemy))
                .copied()
                .unwrap_or(armor_max);
            let hp_max = vc20_special_hp_max(enemy.kind);
            let damage_state = vc27_damage_state(
                armor.saturating_add(enemy.hp),
                armor_max.saturating_add(hp_max),
            );
            match enemy.kind {
                SpecialKind::GraveKnight => {
                    vc27_hd_render_grave_knight(framebuffer, x, y, enemy.age)
                }
                SpecialKind::BellWraith => {
                    vc27_hd_render_bell_wraith(framebuffer, x, y, enemy.age, enemy.phase)
                }
                SpecialKind::RelicCarrier => vc27_hd_render_relic_carrier(
                    framebuffer,
                    x,
                    y,
                    enemy.age,
                    enemy.phase,
                    enemy.direction,
                ),
            }
            vc27_hd_render_damage_marks(framebuffer, x, y, damage_state, enemy.age, 40);
        }

        for threat in &v12.threats {
            let x = vc27_present(threat.x);
            let y = vc27_present(threat.y);
            let armor_max = vc20_threat_armor_max(threat.kind);
            let armor = v20
                .threat_armor
                .get(&vc20_threat_key(threat))
                .copied()
                .unwrap_or(armor_max);
            let hp_max = vc20_threat_hp_max(threat.kind);
            let damage_state = vc27_damage_state(
                armor.saturating_add(threat.hp),
                armor_max.saturating_add(hp_max),
            );
            match threat.kind {
                ThreatKind::ChoirNode => {
                    vc27_hd_render_choir_node(framebuffer, x, y, threat.age, threat.phase)
                }
                ThreatKind::VoidLeech => vc27_hd_render_void_leech(
                    framebuffer,
                    x,
                    y,
                    threat.age,
                    threat.phase,
                    threat.charge,
                ),
            }
            vc27_hd_render_damage_marks(framebuffer, x, y, damage_state, threat.age, 44);
        }

        let base = self.game.game.base();
        if base.encounter_phase != EncounterPhase::Cleared
            && let Some(boss) = base.boss
        {
            vc27_hd_render_bellkeeper(framebuffer, boss);
        }
    }

    fn render_attack_telegraphs(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        for enemy in &base.enemies {
            let Some(progress) = vc27_telegraph_progress(
                enemy.fire_timer,
                VC27_CARRION_TELEGRAPH_WINDOW,
            ) else {
                continue;
            };
            if !(8.0..FRAMEBUFFER_HEIGHT as f32 - 8.0).contains(&enemy.y) {
                continue;
            }

            let x = vc27_present(enemy.x);
            let y = vc27_present(enemy.y);
            let color = match enemy.pattern {
                ShotPattern::Aimed => ENEMY_EYE,
                ShotPattern::Fan3 => HOSTILE_EDGE,
                ShotPattern::Fan5 => HOSTILE_ALT_EDGE,
            };
            let radius = (14.0 - progress * 7.0).round().max(5.0) as u32;
            framebuffer.draw_circle(x, y, radius, color);
            framebuffer.fill_circle(x, y, if progress > 0.72 { 3 } else { 2 }, color);
            if enemy.pattern == ShotPattern::Aimed {
                framebuffer.draw_line(x - 5, y, x + 5, y, color);
                framebuffer.draw_line(x, y + 3, x, y + 9, color);
            }
        }

        let v12 = &self.game.game.v20().game.v12();
        for enemy in &v12.combat.specials {
            if enemy.kind != SpecialKind::BellWraith || enemy.age < 1.0 {
                continue;
            }
            let Some(progress) = vc27_telegraph_progress(
                enemy.fire_timer,
                VC27_WRAITH_TELEGRAPH_WINDOW,
            ) else {
                continue;
            };
            let x = vc27_present(enemy.x);
            let y = vc27_present(enemy.y) - 5;
            let radius = (29.0 - progress * 15.0).round().max(11.0) as u32;
            framebuffer.draw_circle(x, y, radius, WRAITH_GLOW);
            framebuffer.draw_circle(x, y, radius.saturating_sub(4), WRAITH_CORE);
            framebuffer.draw_line(x, y + 8, x, y + 15, CANTICLE_COLOR);
        }

        for threat in &v12.threats {
            let x = vc27_present(threat.x);
            let y = vc27_present(threat.y);
            match threat.kind {
                ThreatKind::ChoirNode => {
                    let buffing_carrion = base.enemies.iter().any(|enemy| {
                        enemy.alive
                            && point_near(
                                enemy.x,
                                enemy.y,
                                threat.x,
                                threat.y,
                                CHOIR_BUFF_RADIUS,
                            )
                    });
                    let buffing_wraith = v12.combat.specials.iter().any(|enemy| {
                        enemy.alive
                            && enemy.kind == SpecialKind::BellWraith
                            && point_near(
                                enemy.x,
                                enemy.y,
                                threat.x,
                                threat.y,
                                CHOIR_BUFF_RADIUS,
                            )
                    });
                    if buffing_carrion || buffing_wraith {
                        let pulse = ((threat.age * 7.0).sin().abs() * 5.0).round() as u32;
                        framebuffer.draw_circle(x, y, 32 + pulse, CHOIR_GLOW);
                        for (dx, dy) in [(0, -38), (38, 0), (0, 38), (-38, 0)] {
                            framebuffer.draw(x + dx, y + dy, ART_GOLD);
                        }
                    }
                }
                ThreatKind::VoidLeech => {
                    if threat.charge.saturating_add(1) >= LEECH_PULSE_CHARGE {
                        let pulse = ((threat.age * 10.0).sin().abs() * 5.0).round() as u32;
                        framebuffer.draw_circle(x, y, 32 + pulse, LEECH_GLOW);
                        framebuffer.draw_circle(x, y, 38 + pulse, DANGER);
                        framebuffer.draw_line(x - 8, y, x + 8, y, LEECH_CORE);
                        framebuffer.draw_line(x, y - 8, x, y + 8, LEECH_CORE);
                    }
                }
            }
        }

        if base.encounter_phase == EncounterPhase::BossFight
            && let Some(boss) = base.boss
            && let Some(progress) =
                vc27_telegraph_progress(boss.shot_timer, VC27_BOSS_TELEGRAPH_WINDOW)
        {
            let x = vc27_present(boss.x);
            let y = vc27_present(boss.y) - 6;
            let color = match boss.phase() {
                BellPhase::Procession => BELL_LIGHT,
                BellPhase::Resonance => WRAITH_GLOW,
                BellPhase::FinalToll => DANGER,
            };
            let radius = (76.0 - progress * 18.0).round().max(54.0) as u32;
            framebuffer.draw_circle(x, y, radius, color);
            framebuffer.draw_circle(x, y, radius.saturating_sub(7), ART_GOLD);
            let clapper_y = y + 38 + (progress * 8.0).round() as i32;
            framebuffer.fill_circle(x, clapper_y, 4, color);
            framebuffer.draw_line(x, y + 18, x, clapper_y, BELL_LIGHT);
        }
    }

    fn render_pickups(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        for cinder in &base.cinders {
            let x = vc27_present(cinder.x);
            let y = vc27_present(cinder.y);
            framebuffer.fill_circle(x, y, 3, CINDER);
            framebuffer.draw_line(x, y - 7, x, y - 4, CANTICLE_COLOR);
            framebuffer.draw(x - 2, y, CANTICLE_COLOR);
            framebuffer.draw(x + 2, y, CANTICLE_COLOR);
        }

        let v14 = self.game.game.v20().game.v14();
        let game = &v14.progression.combat.combat.ui.inner.inner;
        for relic in &game.relics {
            let x = vc27_present(relic.x);
            let y = vc27_present(relic.y);
            let pulse = ((relic.age * 7.0 + relic.phase).sin() * 2.0).round() as i32;
            framebuffer.draw_line(x, y - 9 - pulse, x + 8 + pulse, y, POWER_RELIC_LIGHT);
            framebuffer.draw_line(x + 8 + pulse, y, x, y + 9 + pulse, POWER_RELIC_LIGHT);
            framebuffer.draw_line(x, y + 9 + pulse, x - 8 - pulse, y, POWER_RELIC_LIGHT);
            framebuffer.draw_line(x - 8 - pulse, y, x, y - 9 - pulse, POWER_RELIC_LIGHT);
            framebuffer.fill_circle(x, y, 3, POWER_RELIC);
            framebuffer.draw(x, y, CANTICLE_COLOR);
        }

        for orb in &v14.progression.xp_orbs {
            let x = vc27_present(orb.x);
            let y = vc27_present(orb.y);
            framebuffer.draw_line(x, y - 5, x + 4, y, XP_SHARD_EDGE);
            framebuffer.draw_line(x + 4, y, x, y + 5, XP_SHARD_EDGE);
            framebuffer.draw_line(x, y + 5, x - 4, y, XP_SHARD_EDGE);
            framebuffer.draw_line(x - 4, y, x, y - 5, XP_SHARD_EDGE);
            framebuffer.fill_circle(x, y, 1, XP_SHARD_CORE);
        }

        for (x, y) in v14.orbital_positions() {
            let x = vc27_present(x);
            let y = vc27_present(y);
            framebuffer.draw_circle(x, y, 5, POWER_RELIC_LIGHT);
            framebuffer.fill_circle(x, y, 2, PILGRIM_VIOLET);
            framebuffer.draw(x, y, BOLT_CORE);
        }
    }

    fn render_projectiles(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        for bullet in &base.player_bullets {
            let x = vc27_present(bullet.x);
            let y = vc27_present(bullet.y);
            framebuffer.draw_line(x, y + 6, x, y - 7, BOLT_CORE);
            framebuffer.draw_line(x - 1, y + 4, x - 1, y - 4, BOLT_EDGE);
            framebuffer.draw_line(x + 1, y + 4, x + 1, y - 4, BOLT_EDGE);
        }

        let v14 = self.game.game.v20().game.v14();
        let game = &v14.progression.combat.combat.ui.inner.inner;
        for shot in &game.power_shots {
            let x = vc27_present(shot.x);
            let y = vc27_present(shot.y);
            let radius = if shot.radius >= 2 || shot.damage > 1 { 4 } else { 2 };
            framebuffer.fill_circle(x, y, radius, BOLT_EDGE);
            framebuffer.fill_circle(x, y, 1, BOLT_CORE);
            framebuffer.draw_line(x, y + 5, x, y + 11, BOLT_RELIC);
            if shot.vx.abs() > 6.0 {
                let wing = if shot.vx > 0.0 { -5 } else { 5 };
                framebuffer.draw_line(x, y + 2, x + wing, y + 7, BOLT_RELIC);
            }
        }

        let pressure = self.game.game.v20().game.ui.game.combat.combat.pressure;
        let boss_phase = base.boss.map(Bellkeeper::phase);
        for bullet in &base.enemy_bullets {
            let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt().max(1.0);
            let style = vc27_enemy_shot_style(base.encounter_phase, speed);
            vc27_render_enemy_bullet(framebuffer, *bullet, style, pressure, boss_phase);
        }
    }

    fn render_particles_and_bursts(&self, framebuffer: &mut Framebuffer) {
        for burst in &self.game.game.base().bursts {
            let ratio = if burst.duration > 0.0 {
                (burst.remaining / burst.duration).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let radius = (3.0 + (1.0 - ratio) * 16.0).round() as u32;
            framebuffer.draw_circle(
                vc27_present(burst.x),
                vc27_present(burst.y),
                radius,
                burst.color,
            );
        }

        let v17 = &self.game.game.v20().game.ui.game;
        for particle in &v17.particles {
            let x = vc27_present(particle.x);
            let y = vc27_present(particle.y);
            match particle.kind {
                V17ParticleKind::Spark => {
                    let tail_x = vc27_present(particle.x - particle.vx * 0.035);
                    let tail_y = vc27_present(particle.y - particle.vy * 0.035);
                    framebuffer.draw_line(tail_x, tail_y, x, y, particle.color);
                    framebuffer.draw(x, y, BOLT_CORE);
                }
                V17ParticleKind::Shard => {
                    framebuffer.draw_line(x - 2, y, x + 2, y, particle.color);
                    framebuffer.draw_line(x, y - 2, x, y + 2, particle.color);
                }
            }
        }
    }

    fn render_enemy_defenses(&self, framebuffer: &mut Framebuffer) {
        let v20 = self.game.game.v20();
        for enemy in &self.game.game.base().enemies {
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let armor = v20
                .carrion_armor
                .get(&vc20_carrion_key(enemy))
                .copied()
                .unwrap_or(armor_max);
            vc27_dual_bar(
                framebuffer,
                vc27_present(enemy.x),
                vc27_present(enemy.y) - 39,
                armor,
                armor_max,
                1,
                1,
            );
        }

        let v12 = &v20.game.v12();
        for enemy in &v12.combat.specials {
            let armor_max = vc20_special_armor_max(enemy.kind);
            let armor = v20
                .special_armor
                .get(&vc20_special_key(enemy))
                .copied()
                .unwrap_or(armor_max);
            vc27_dual_bar(
                framebuffer,
                vc27_present(enemy.x),
                vc27_present(enemy.y) - 43,
                armor,
                armor_max,
                enemy.hp,
                vc20_special_hp_max(enemy.kind),
            );
        }

        for threat in &v12.threats {
            let armor_max = vc20_threat_armor_max(threat.kind);
            let armor = v20
                .threat_armor
                .get(&vc20_threat_key(threat))
                .copied()
                .unwrap_or(armor_max);
            vc27_dual_bar(
                framebuffer,
                vc27_present(threat.x),
                vc27_present(threat.y) - 47,
                armor,
                armor_max,
                threat.hp,
                vc20_threat_hp_max(threat.kind),
            );
        }

        let base = self.game.game.base();
        if base.encounter_phase == EncounterPhase::BossFight
            && let Some(boss) = base.boss
        {
            let x = vc27_present(boss.x);
            let y = vc27_present(boss.y);
            let color = if v20.boss_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            };
            if v20.boss_shield > 0 {
                let pulse = ((base.animation_time * 7.0).sin().abs() * 4.0) as u32;
                framebuffer.draw_circle(x, y - 6, 63 + pulse, color);
            } else if v20.boss_shield_break_timer > 0.0 {
                framebuffer.draw_circle(x, y - 6, 65, CANTICLE_COLOR);
            }
            vc27_dual_bar(
                framebuffer,
                x,
                y - 76,
                v20.boss_shield,
                VC20_BOSS_SHIELD_MAX,
                boss.hp,
                BELLKEEPER_MAX_HP,
            );
        }
    }

    fn render_major_fx(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        let player_x = vc27_present(base.player_x);
        let player_y = vc27_present(base.player_y);

        if base.canticle_timer > 0.0 {
            let ratio = (base.canticle_timer / CANTICLE_DURATION).clamp(0.0, 1.0);
            let radius = (40.0 + (1.0 - ratio) * 150.0).round() as u32;
            framebuffer.draw_circle(player_x, player_y, radius, CANTICLE_COLOR);
            framebuffer.draw_circle(player_x, player_y, radius.saturating_add(12), ART_GOLD);
        }

        let emp_timer = self.game.game.emp_flash_timer;
        if emp_timer > 0.0 {
            let ratio = (emp_timer / VC23_EMP_FLASH_DURATION).clamp(0.0, 1.0);
            let radius = (28.0 + (1.0 - ratio) * 220.0).round() as u32;
            framebuffer.draw_circle(player_x, player_y, radius, ART_CYAN_LIGHT);
        }
    }

    fn render_player_last(&self, framebuffer: &mut Framebuffer) {
        if self.game.combat_model().player_hull <= 0.0 {
            return;
        }

        let focused = self
            .game
            .game
            .game
            .game
            .movement_controls
            .action(FOCUS)
            .held();
        let base = self.game.game.base();
        vc27_hd_render_pilgrim(
            framebuffer,
            vc27_present(base.player_x),
            vc27_present(base.player_y),
            focused,
            base.invulnerability,
            base.animation_time,
        );
    }

    fn render_combat_presentation(&mut self, framebuffer: &mut Framebuffer) {
        if !self.game.game.active_combat() {
            return;
        }

        self.render_clean_background(framebuffer);
        self.render_choir_links(framebuffer);
        self.render_pickups(framebuffer);
        self.render_presentation_bestiary(framebuffer);
        self.render_attack_telegraphs(framebuffer);
        self.render_enemy_defenses(framebuffer);
        self.render_projectiles(framebuffer);
        self.render_particles_and_bursts(framebuffer);
        self.render_major_fx(framebuffer);
        self.render_event_announcement(framebuffer);
        self.render_threat_meter(framebuffer);
        self.render_echo_shell(framebuffer);
        self.render_canticle_charge(framebuffer);
        self.render_survival_bars(framebuffer);
        self.render_player_last(framebuffer);
    }
}

fn vc27_telegraph_progress(timer: f32, window: f32) -> Option<f32> {
    if window <= 0.0 || timer <= 0.0 || timer > window {
        return None;
    }
    Some((1.0 - timer / window).clamp(0.0, 1.0))
}

fn vc27_enemy_shot_style(encounter_phase: EncounterPhase, speed: f32) -> Vc27EnemyShotStyle {
    if encounter_phase == EncounterPhase::BossFight {
        return Vc27EnemyShotStyle::Bellkeeper;
    }
    if (speed - 48.0).abs() <= 1.0 {
        Vc27EnemyShotStyle::Wraith
    } else if (speed - 62.0).abs() <= 1.0 {
        Vc27EnemyShotStyle::VoidPulse
    } else if (speed - ENEMY_SHOT_SPEED).abs() <= 1.0 {
        Vc27EnemyShotStyle::Carrion
    } else {
        Vc27EnemyShotStyle::Void
    }
}

fn vc27_render_enemy_bullet(
    framebuffer: &mut Framebuffer,
    bullet: Bullet,
    style: Vc27EnemyShotStyle,
    pressure: VoidPressure,
    boss_phase: Option<BellPhase>,
) {
    let x = vc27_present(bullet.x);
    let y = vc27_present(bullet.y);
    let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt().max(1.0);
    let nx = bullet.vx / speed;
    let ny = bullet.vy / speed;
    let tail = |length: f32| {
        (
            x - (nx * length).round() as i32,
            y - (ny * length).round() as i32,
        )
    };

    match style {
        Vc27EnemyShotStyle::Carrion => {
            let edge = if bullet.alternate {
                HOSTILE_ALT_EDGE
            } else {
                HOSTILE_EDGE
            };
            let core = if bullet.alternate {
                HOSTILE_ALT_CORE
            } else {
                HOSTILE_CORE
            };
            let (tail_x, tail_y) = tail(7.0);
            framebuffer.draw_line(tail_x, tail_y, x, y, edge);
            framebuffer.fill_circle(x, y, 3, edge);
            framebuffer.fill_circle(x, y, 1, core);
            framebuffer.draw(
                x - ny.round() as i32 * 3,
                y + nx.round() as i32 * 3,
                ENEMY_EYE,
            );
        }
        Vc27EnemyShotStyle::Wraith => {
            let (tail_x, tail_y) = tail(5.0);
            framebuffer.draw_line(tail_x, tail_y, x, y, WRAITH_GLOW);
            framebuffer.draw_circle(x, y, 4, WRAITH_GLOW);
            framebuffer.draw_circle(x, y, 2, WRAITH_CORE);
            framebuffer.draw(x, y, CANTICLE_COLOR);
        }
        Vc27EnemyShotStyle::VoidPulse => {
            let (tail_x, tail_y) = tail(4.0);
            framebuffer.draw_line(tail_x, tail_y, x, y, ART_VOID);
            framebuffer.draw_circle(x, y, 5, LEECH_GLOW);
            framebuffer.fill_circle(x, y, 2, DANGER);
            framebuffer.draw(x, y, VOID_LIGHT);
        }
        Vc27EnemyShotStyle::Void => {
            let color = void_pressure_color(pressure);
            let core = if bullet.alternate {
                VOID_LIGHT
            } else {
                VOID_DANGER
            };
            let (tail_x, tail_y) = tail(7.0);
            framebuffer.draw_line(tail_x, tail_y, x, y, color);
            framebuffer.draw_circle(x, y, 3, color);
            framebuffer.fill_circle(x, y, 1, core);
        }
        Vc27EnemyShotStyle::Bellkeeper => {
            let phase = boss_phase.unwrap_or(BellPhase::Procession);
            let edge = match phase {
                BellPhase::Procession => BELL_LIGHT,
                BellPhase::Resonance => WRAITH_GLOW,
                BellPhase::FinalToll => DANGER,
            };
            let core = match phase {
                BellPhase::Procession => ART_GOLD,
                BellPhase::Resonance => VOID_LIGHT,
                BellPhase::FinalToll => CANTICLE_COLOR,
            };
            let (tail_x, tail_y) = tail(if bullet.alternate { 9.0 } else { 7.0 });
            framebuffer.draw_line(tail_x, tail_y, x, y, edge);
            framebuffer.draw_circle(x, y, 4, edge);
            framebuffer.draw_circle(x, y, 2, core);
            framebuffer.draw(x, y, core);
        }
    }
}

fn vc27_dual_bar(
    framebuffer: &mut Framebuffer,
    center_x: i32,
    y: i32,
    armor: u32,
    armor_max: u32,
    hp: u32,
    hp_max: u32,
) {
    let width = 54_u32;
    let left = center_x - width as i32 / 2;
    framebuffer.fill_rect(left, y, width, 3, VC20_ARMOR_BG);
    if armor_max > 0 && armor > 0 {
        let fill = width.saturating_mul(armor.min(armor_max)) / armor_max;
        framebuffer.fill_rect(left, y, fill, 3, VC20_ARMOR);
    }
    framebuffer.fill_rect(left, y + 5, width, 3, CORE_BG);
    if hp_max > 0 && hp > 0 {
        let fill = width.saturating_mul(hp.min(hp_max)) / hp_max;
        framebuffer.fill_rect(left, y + 5, fill, 3, VC20_HULL);
    }
}

#[cfg(test)]
mod v27_combat_tests {
    use super::*;

    #[test]
    fn attack_telegraph_progress_only_exists_inside_warning_window() {
        assert_eq!(vc27_telegraph_progress(0.31, 0.30), None);
        assert_eq!(vc27_telegraph_progress(0.0, 0.30), None);
        assert_eq!(vc27_telegraph_progress(-0.1, 0.30), None);
        let start = vc27_telegraph_progress(0.30, 0.30).expect("window start should telegraph");
        let middle = vc27_telegraph_progress(0.15, 0.30).expect("window middle should telegraph");
        let end = vc27_telegraph_progress(0.03, 0.30).expect("window end should telegraph");
        assert!(start <= 0.001);
        assert!((middle - 0.5).abs() <= 0.001);
        assert!(end > middle);
    }

    #[test]
    fn enemy_shot_styles_follow_existing_pattern_speeds() {
        assert_eq!(
            vc27_enemy_shot_style(EncounterPhase::Waves, 48.0),
            Vc27EnemyShotStyle::Wraith
        );
        assert_eq!(
            vc27_enemy_shot_style(EncounterPhase::Waves, 62.0),
            Vc27EnemyShotStyle::VoidPulse
        );
        assert_eq!(
            vc27_enemy_shot_style(EncounterPhase::Waves, ENEMY_SHOT_SPEED),
            Vc27EnemyShotStyle::Carrion
        );
        assert_eq!(
            vc27_enemy_shot_style(EncounterPhase::Waves, 96.0),
            Vc27EnemyShotStyle::Void
        );
        assert_eq!(
            vc27_enemy_shot_style(EncounterPhase::BossFight, 48.0),
            Vc27EnemyShotStyle::Bellkeeper
        );
    }

    #[test]
    fn damage_states_follow_effective_health_thirds() {
        assert_eq!(vc27_damage_state(10, 10), Vc27DamageState::Intact);
        assert_eq!(vc27_damage_state(6, 10), Vc27DamageState::Damaged);
        assert_eq!(vc27_damage_state(3, 10), Vc27DamageState::Critical);
    }

    #[test]
    fn hd_choir_node_keeps_changes_local() {
        let mut framebuffer = Framebuffer::new(96, 96);
        framebuffer.clear(Pixel::BLUE);
        vc27_hd_render_choir_node(&mut framebuffer, 48, 48, 1.0, 0.0);
        assert_ne!(framebuffer.pixel(48, 48), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(95, 95), Some(Pixel::BLUE));
    }

    #[test]
    fn dual_bar_stays_local() {
        let mut framebuffer = Framebuffer::new(80, 24);
        framebuffer.clear(Pixel::BLUE);
        vc27_dual_bar(&mut framebuffer, 40, 8, 2, 3, 4, 6);
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(79, 23), Some(Pixel::BLUE));
    }
}
