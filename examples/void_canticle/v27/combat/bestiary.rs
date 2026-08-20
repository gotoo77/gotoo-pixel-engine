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
mod v27_bestiary_tests {
    use super::*;

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
