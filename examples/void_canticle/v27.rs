const VC27_PRESENTATION_VERSION: &str = "VC2.7";
const VC27_CARRION_TELEGRAPH_WINDOW: f32 = 0.24;
const VC27_WRAITH_TELEGRAPH_WINDOW: f32 = 0.30;
const VC27_BOSS_TELEGRAPH_WINDOW: f32 = 0.34;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/void_canticle/v27/hd_bestiary.rs"
));

struct VoidCanticleV27DirectPresentation {
    game: VoidCanticleV23Sustain,
    legacy_sink: Framebuffer,
    clean_background: Framebuffer,
}

impl VoidCanticleV27DirectPresentation {
    fn new() -> Self {
        Self {
            game: VoidCanticleV23Sustain::new(),
            legacy_sink: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
            clean_background: Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT),
        }
    }

    fn chassis_selection_active(&self) -> bool {
        self.game.game.game.game.game.chassis.is_none()
    }

    fn render_chassis_selection_presentation(&mut self, framebuffer: &mut Framebuffer) {
        self.clean_background.clear(BG);
        self.game
            .game
            .game
            .game
            .game
            .render_chassis_selection(&mut self.clean_background);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
    }

    fn visual_mode(&self) -> VcVisualMode {
        if self.game.choosing_support {
            return VcVisualMode::SupportChoice;
        }

        let v14 = self.game.game.v20().game.v14();
        if v14.progression.level_choice.is_some() {
            return VcVisualMode::LevelChoice;
        }
        if v14.mutation_choice.is_some() {
            return VcVisualMode::MutationChoice;
        }
        if self.game.game.base().game_over {
            return VcVisualMode::Death;
        }

        let stabilized = &self.game.survival_model().game;
        if stabilized.stage_clear_visible() {
            return VcVisualMode::StageClear;
        }
        if !matches!(&stabilized.pause_ui().state, VcPauseState::Running) {
            return VcVisualMode::Pause;
        }

        VcVisualMode::Combat
    }

    fn render_clean_background(&mut self, framebuffer: &mut Framebuffer) {
        let base = self.game.game.base();
        let color = if base.canticle_timer > 0.46 {
            BG_CANTICLE
        } else {
            BG
        };
        let scroll = base.scroll;

        self.clean_background.clear(color);
        render_grave_orbit_background(&mut self.clean_background, scroll);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
    }

    fn render_event_announcement(&self, framebuffer: &mut Framebuffer) {
        let v16b = &self.game.game.v20().game.ui.game.combat;

        if v16b.pressure_reveal_timer > 0.0 && v16b.pressure_reveal != VoidPressure::Dormant {
            let (headline, consequence) = pressure_transition_copy(v16b.pressure_reveal);
            vc_visual_announcement(
                framebuffer,
                headline,
                consequence,
                void_pressure_color(v16b.pressure_reveal),
                VOID_LIGHT,
            );
            return;
        }

        if v16b.boss_phase_banner_timer > 0.0
            && let Some(phase) = v16b.boss_phase_banner
        {
            vc_visual_announcement(
                framebuffer,
                "BELLKEEPER",
                bell_phase_name(phase),
                BELL_LIGHT,
                CANTICLE_COLOR,
            );
            return;
        }

        let v15 = &v16b.combat.combat;
        if v15.synergy_banner_timer > 0.0
            && let Some(name) = v15.synergy_banner_name
        {
            vc_visual_announcement(
                framebuffer,
                "SYNERGY",
                name,
                SYNERGY_COLOR,
                SYNERGY_GOLD,
            );
            return;
        }

        if self.game.game.base().canticle_timer > 0.0 {
            vc_visual_announcement(
                framebuffer,
                "FULL WIPE",
                "CANTICLE",
                CANTICLE_COLOR,
                CANTICLE_COLOR,
            );
            return;
        }

        if let Some(attack) = v16b.combat.pending_attack {
            vc_visual_announcement(
                framebuffer,
                void_attack_name(attack.kind),
                "VOID ATTACK",
                void_pressure_color(v16b.combat.pressure),
                VOID_LIGHT,
            );
        }
    }

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

        for bullet in &base.enemy_bullets {
            let x = vc27_present(bullet.x);
            let y = vc27_present(bullet.y);
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
            framebuffer.fill_circle(x, y, 3, edge);
            framebuffer.fill_circle(x, y, 1, core);
            let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt().max(1.0);
            let tail_x = x - (bullet.vx / speed * 5.0).round() as i32;
            let tail_y = y - (bullet.vy / speed * 5.0).round() as i32;
            framebuffer.draw_line(tail_x, tail_y, x, y, edge);
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

    fn render_canticle_charge(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let base = self.game.game.base();
        let width = 88 * scale;
        let height = 4 * scale;
        let x = (VC_VISUAL_PRESENTATION_WIDTH - width) / 2;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(18 * scale);
        let ratio = base.core_charge.min(CORE_MAX) as f32 / CORE_MAX as f32;
        let filled = (width as f32 * ratio).round() as u32;
        let ready = base.core_charge >= CORE_MAX;
        let color = if ready { CANTICLE_COLOR } else { CINDER };

        framebuffer.draw_text_scaled(8 * scale as i32, y as i32 - 1, "CORE", scale, TEXT);
        framebuffer.fill_rect(x as i32, y as i32, width, height, WRECK_MID);
        if filled > 0 {
            framebuffer.fill_rect(x as i32, y as i32, filled.min(width), height, color);
        }
        for segment in 1..5 {
            let separator = x + width * segment / 5;
            framebuffer.fill_rect(separator as i32, y as i32, 1, height, BG);
        }
        framebuffer.draw_rect(
            x as i32 - 1,
            y as i32 - 1,
            width + 2,
            height + 2,
            WRECK_LIGHT,
        );

        if ready && ((base.animation_time * 6.0) as i32 & 1) == 0 {
            let (text_width, text_height) = Framebuffer::text_size("READY", 1);
            let text_x = x + width.saturating_sub(text_width) / 2;
            let text_y = y + height.saturating_sub(text_height) / 2;
            framebuffer.draw_text(text_x as i32, text_y as i32, "READY", BG);
        }
    }

    fn render_survival_bars(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let model = self.game.combat_model();
        let hull_cap = self.game.hull_cap().max(1.0);
        let shield_cap = self.game.shield_cap().max(1.0);

        let margin = 8 * scale;
        let width = 72 * scale;
        let height = 4 * scale;
        let y = VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(7 * scale);
        let hull_x = margin;
        let shield_x = VC_VISUAL_PRESENTATION_WIDTH
            .saturating_sub(margin)
            .saturating_sub(width);
        let hull_segments = ((hull_cap / 10.0).ceil() as u32).clamp(4, 12);
        let shield_segments = ((shield_cap / 5.0).ceil() as u32).clamp(3, 10);

        vc27_segmented_bar(
            framebuffer,
            hull_x as i32,
            y as i32,
            width,
            height,
            model.player_hull,
            hull_cap,
            hull_segments,
            if model.player_hull_flash_timer > 0.0 {
                DANGER
            } else {
                VC20_HULL
            },
        );
        vc27_segmented_bar(
            framebuffer,
            shield_x as i32,
            y as i32,
            width,
            height,
            model.player_shield,
            shield_cap,
            shield_segments,
            if model.player_shield_flash_timer > 0.0 {
                VC20_ARMOR_LIGHT
            } else {
                VC20_ARMOR
            },
        );
    }

    fn render_threat_meter(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let pressure = self.game.game.v20().game.ui.game.combat.combat.pressure;
        let active = vc27_pressure_level(pressure);
        let brick_width = 4 * scale;
        let brick_height = 6 * scale;
        let gap = 2 * scale;
        let x = VC_VISUAL_PRESENTATION_WIDTH.saturating_sub(8 * scale);
        let base_y = 196 * scale;
        let states = [
            VoidPressure::Dormant,
            VoidPressure::Stirring,
            VoidPressure::Awake,
            VoidPressure::Hostile,
            VoidPressure::Cataclysmic,
        ];

        for (index, state) in states.into_iter().enumerate() {
            let level = index as u32 + 1;
            let inverted_index = 4_u32.saturating_sub(index as u32);
            let y = base_y + inverted_index * (brick_height + gap);
            let lit = level <= active;
            let fill = if lit {
                void_pressure_color(state)
            } else {
                WRECK_MID
            };

            if lit {
                framebuffer.fill_rect(x as i32, y as i32, brick_width, brick_height, fill);
            }
            framebuffer.draw_rect(
                x as i32 - 1,
                y as i32 - 1,
                brick_width + 2,
                brick_height + 2,
                fill,
            );
        }
    }

    fn render_echo_shell(&self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        let progression = &self.game.game.v20().game.v14().progression;
        let ratio = if progression.xp_next == 0 {
            1.0
        } else {
            progression.xp.min(progression.xp_next) as f32 / progression.xp_next as f32
        };
        let center_x = (18 * scale) as i32;
        let center_y = (VC_VISUAL_PRESENTATION_HEIGHT.saturating_sub(35 * scale)) as i32;
        let radius = 11 * scale;
        let level = progression.level;
        let tier_color = vc27_echo_level_color(level);

        vc27_echo_shell(
            framebuffer,
            center_x,
            center_y,
            radius,
            ratio,
            level,
            tier_color,
        );
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

    fn render_clean_modal(&mut self, framebuffer: &mut Framebuffer, mode: VcVisualMode) {
        self.render_clean_background(framebuffer);
        self.clean_background.clear(BG);
        render_grave_orbit_background(&mut self.clean_background, self.game.game.base().scroll);

        match mode {
            VcVisualMode::SupportChoice => {
                self.game.render_support_choice(&mut self.clean_background);
            }
            VcVisualMode::LevelChoice => {
                let v14 = self.game.game.v20().game.v14();
                if let Some(choice) = v14.progression.level_choice.as_ref() {
                    v14.progression
                        .render_level_choice(&mut self.clean_background, choice);
                }
                self.game
                    .survival_model()
                    .render_level_choice_overrides(&mut self.clean_background);
            }
            VcVisualMode::MutationChoice => {
                let v14 = self.game.game.v20().game.v14();
                if let Some(choice) = v14.mutation_choice.as_ref() {
                    v14.render_mutation_choice(&mut self.clean_background, choice);
                }
            }
            VcVisualMode::Pause => {
                let pause = self.game.survival_model().game.pause_ui();
                match &pause.state {
                    VcPauseState::Controls => pause.render_controls(&mut self.clean_background),
                    VcPauseState::BuildInfo => pause.render_build_info(&mut self.clean_background),
                    VcPauseState::Menu | VcPauseState::ResumeGate | VcPauseState::Running => {
                        pause.render_menu(&mut self.clean_background)
                    }
                }
            }
            VcVisualMode::StageClear => {
                self.game
                    .survival_model()
                    .game
                    .render_stage_clear(&mut self.clean_background);
            }
            VcVisualMode::Combat | VcVisualMode::Death => {}
        }

        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );
    }

    fn render_death_presentation(&mut self, framebuffer: &mut Framebuffer) {
        let scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
        self.clean_background.clear(BG);
        render_grave_orbit_background(&mut self.clean_background, self.game.game.base().scroll);
        vc_visual_blit_nearest(
            &self.clean_background,
            framebuffer,
            VC_VISUAL_PRESENTATION_SCALE,
            false,
        );

        let panel_width = 150 * scale;
        let panel_height = 62 * scale;
        let panel_x = ((VC_VISUAL_PRESENTATION_WIDTH - panel_width) / 2) as i32;
        let panel_y = (118 * scale) as i32;
        framebuffer.fill_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Pixel::rgb(9, 8, 15),
        );
        framebuffer.draw_rect(panel_x, panel_y, panel_width, panel_height, DANGER);
        vc_visual_draw_centered_text(
            framebuffer,
            panel_y + 15 * scale as i32,
            "PILGRIM FALLEN",
            scale,
            DANGER,
        );
        vc_visual_draw_centered_text(
            framebuffer,
            panel_y + 39 * scale as i32,
            "SPACE TO RETURN",
            scale,
            TEXT,
        );
    }
}

impl Game for VoidCanticleV27DirectPresentation {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = {
            let mut legacy_frame = Frame {
                framebuffer: &mut self.legacy_sink,
                input: frame.input,
                delta_time: frame.delta_time,
                storage: &mut *frame.storage,
                audio: &mut *frame.audio,
                surface_size: frame.surface_size,
                viewport: gotoo_pixel_engine::Viewport::new(
                    frame.surface_size,
                    Size {
                        width: FRAMEBUFFER_WIDTH,
                        height: FRAMEBUFFER_HEIGHT,
                    },
                ),
            };
            self.game.update(&mut legacy_frame)
        };
        if result == GameResult::Exit {
            return result;
        }

        if self.chassis_selection_active() {
            self.render_chassis_selection_presentation(frame.framebuffer);
            return GameResult::Continue;
        }

        let mode = self.visual_mode();
        match mode {
            VcVisualMode::Combat => self.render_combat_presentation(frame.framebuffer),
            VcVisualMode::Death => self.render_death_presentation(frame.framebuffer),
            VcVisualMode::Pause
            | VcVisualMode::LevelChoice
            | VcVisualMode::MutationChoice
            | VcVisualMode::SupportChoice
            | VcVisualMode::StageClear => self.render_clean_modal(frame.framebuffer, mode),
        }

        GameResult::Continue
    }
}

fn vc27_telegraph_progress(timer: f32, window: f32) -> Option<f32> {
    if window <= 0.0 || timer <= 0.0 || timer > window {
        return None;
    }
    Some((1.0 - timer / window).clamp(0.0, 1.0))
}

fn vc27_pressure_level(pressure: VoidPressure) -> u32 {
    match pressure {
        VoidPressure::Dormant => 1,
        VoidPressure::Stirring => 2,
        VoidPressure::Awake => 3,
        VoidPressure::Hostile => 4,
        VoidPressure::Cataclysmic => 5,
    }
}

fn vc27_echo_level_color(level: u32) -> Pixel {
    match (level / 10) % 6 {
        0 => XP_ORB_CORE,
        1 => ART_CYAN_LIGHT,
        2 => SYNERGY_GOLD,
        3 => VOID_GLOW,
        4 => CANTICLE_COLOR,
        _ => DANGER,
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

fn vc27_echo_shell(
    framebuffer: &mut Framebuffer,
    center_x: i32,
    center_y: i32,
    radius: u32,
    ratio: f32,
    level: u32,
    tier_color: Pixel,
) {
    let ratio = ratio.clamp(0.0, 1.0);
    let radius = radius.max(4);
    let steps = 48_u32;
    let active_steps = (steps as f32 * ratio).round() as u32;

    framebuffer.draw_circle(center_x, center_y, radius, tier_color);
    framebuffer.draw_circle(center_x, center_y, radius.saturating_sub(3), XP_BAR_BG);

    let mut previous = None;
    let mut active_tip = (center_x, center_y);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let angle = -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::TAU * 2.20;
        let spiral_radius = 1.5 + t * (radius as f32 - 3.0);
        let x = center_x + (angle.cos() * spiral_radius).round() as i32;
        let y = center_y + (angle.sin() * spiral_radius).round() as i32;

        if let Some((px, py)) = previous {
            let color = if step <= active_steps {
                tier_color
            } else {
                WRECK_MID
            };
            framebuffer.draw_line(px, py, x, y, color);
        }
        if step <= active_steps {
            active_tip = (x, y);
        }
        previous = Some((x, y));
    }

    if active_steps > 0 {
        framebuffer.fill_circle(active_tip.0, active_tip.1, 1, tier_color);
    }

    let lip_x = center_x + radius as i32 - 2;
    framebuffer.draw_line(
        center_x + radius as i32 / 2,
        center_y + radius as i32 / 3,
        lip_x,
        center_y + radius as i32 / 2,
        tier_color,
    );

    let badge_radius = (radius / 3).max(4);
    framebuffer.fill_circle(center_x, center_y, badge_radius, BG);
    framebuffer.draw_circle(center_x, center_y, badge_radius, tier_color);

    let label = level.to_string();
    let preferred_scale = VC_VISUAL_PRESENTATION_SCALE.max(1);
    let text_scale = if label.len() <= 2 { preferred_scale } else { 1 };
    let (text_width, text_height) = Framebuffer::text_size(&label, text_scale);
    framebuffer.draw_text_scaled(
        center_x - text_width as i32 / 2,
        center_y - text_height as i32 / 2,
        &label,
        text_scale,
        tier_color,
    );
}

fn vc27_segmented_bar(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    value: f32,
    max_value: f32,
    segments: u32,
    fill: Pixel,
) {
    let ratio = (value / max_value.max(1.0)).clamp(0.0, 1.0);
    let filled = (width as f32 * ratio).round() as u32;
    framebuffer.fill_rect(x, y, width, height, WRECK_MID);
    if filled > 0 {
        framebuffer.fill_rect(x, y, filled.min(width), height, fill);
    }

    for segment in 1..segments.max(1) {
        let separator = width.saturating_mul(segment) / segments.max(1);
        framebuffer.fill_rect(x + separator as i32, y, 1, height, BG);
    }
    framebuffer.draw_rect(x - 1, y - 1, width + 2, height + 2, WRECK_LIGHT);
}

pub fn run_v27_direct_presentation_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!(
                "Void Canticle {VC27_PRESENTATION_VERSION} Direct HUD - Gotoo Pixel Engine"
            ),
            framebuffer_width: VC_VISUAL_PRESENTATION_WIDTH,
            framebuffer_height: VC_VISUAL_PRESENTATION_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV27DirectPresentation::new(),
            VC_VISUAL_PRESENTATION_WIDTH,
            VC_VISUAL_PRESENTATION_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v27_tests {
    use super::*;

    #[test]
    fn version_is_explicit() {
        assert_eq!(VC27_PRESENTATION_VERSION, "VC2.7");
    }

    #[test]
    fn chassis_selection_is_explicit_precombat_state() {
        let game = VoidCanticleV27DirectPresentation::new();
        assert!(game.chassis_selection_active());
    }

    #[test]
    fn simulation_coordinates_map_to_presentation_space() {
        assert_eq!(vc27_present(0.0), 0);
        assert_eq!(vc27_present(90.0), 180);
        assert_eq!(vc27_present(319.5), 639);
    }

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
    fn pressure_maps_to_five_visual_bricks() {
        assert_eq!(vc27_pressure_level(VoidPressure::Dormant), 1);
        assert_eq!(vc27_pressure_level(VoidPressure::Stirring), 2);
        assert_eq!(vc27_pressure_level(VoidPressure::Awake), 3);
        assert_eq!(vc27_pressure_level(VoidPressure::Hostile), 4);
        assert_eq!(vc27_pressure_level(VoidPressure::Cataclysmic), 5);
    }

    #[test]
    fn damage_states_follow_effective_health_thirds() {
        assert_eq!(vc27_damage_state(10, 10), Vc27DamageState::Intact);
        assert_eq!(vc27_damage_state(6, 10), Vc27DamageState::Damaged);
        assert_eq!(vc27_damage_state(3, 10), Vc27DamageState::Critical);
    }

    #[test]
    fn echo_shell_changes_color_at_ten_level_boundaries() {
        assert_eq!(vc27_echo_level_color(1), vc27_echo_level_color(9));
        assert_ne!(vc27_echo_level_color(9), vc27_echo_level_color(10));
        assert_ne!(vc27_echo_level_color(19), vc27_echo_level_color(20));
        assert_ne!(vc27_echo_level_color(29), vc27_echo_level_color(30));
    }

    #[test]
    fn echo_shell_stays_local_to_its_glyph() {
        let mut framebuffer = Framebuffer::new(40, 40);
        framebuffer.clear(Pixel::BLUE);
        vc27_echo_shell(
            &mut framebuffer,
            20,
            20,
            8,
            0.5,
            12,
            vc27_echo_level_color(12),
        );
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(39, 39), Some(Pixel::BLUE));
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

    #[test]
    fn segmented_bar_only_touches_its_own_rows() {
        let mut framebuffer = Framebuffer::new(20, 8);
        framebuffer.clear(Pixel::BLUE);
        vc27_segmented_bar(&mut framebuffer, 2, 3, 12, 2, 3.0, 12.0, 4, Pixel::RED);
        assert_eq!(framebuffer.pixel(2, 1), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(2, 6), Some(Pixel::BLUE));
    }
}
