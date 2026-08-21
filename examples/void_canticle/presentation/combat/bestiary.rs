impl VoidCanticlePresentation {
    fn render_choir_links(&self, framebuffer: &mut Framebuffer) {
        let encounter = self.game.presentation_encounter_model();
        let link_color = Pixel::rgb(34, 62, 82);

        for node in encounter
            .threats
            .iter()
            .filter(|threat| threat.alive && threat.kind == ThreatKind::ChoirNode)
        {
            let x = presentation_coord(node.x);
            let y = presentation_coord(node.y);
            for enemy in &self.game.presentation_base().enemies {
                if point_near(enemy.x, enemy.y, node.x, node.y, CHOIR_BUFF_RADIUS) {
                    framebuffer.draw_line(
                        x,
                        y,
                        presentation_coord(enemy.x),
                        presentation_coord(enemy.y),
                        link_color,
                    );
                }
            }
            for enemy in &encounter.combat.specials {
                if enemy.kind == SpecialKind::BellWraith
                    && point_near(enemy.x, enemy.y, node.x, node.y, CHOIR_BUFF_RADIUS)
                {
                    framebuffer.draw_line(
                        x,
                        y,
                        presentation_coord(enemy.x),
                        presentation_coord(enemy.y),
                        link_color,
                    );
                }
            }
        }
    }

    fn render_presentation_bestiary(&self, framebuffer: &mut Framebuffer) {
        let defenses = self.game.presentation_defense_model();
        let encounter = self.game.presentation_encounter_model();
        for enemy in &self.game.presentation_base().enemies {
            let key = vc20_carrion_key(enemy);
            let hit = self.hit_reactions.carrion_visual(key);
            let x = presentation_coord(enemy.x) + hit.offset_x;
            let y = presentation_coord(enemy.y) + hit.offset_y;
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let armor = defenses
                .carrion_armor
                .get(&key)
                .copied()
                .unwrap_or(armor_max);
            render_carrion(framebuffer, x, y, enemy.age, enemy.phase);
            render_damage_marks(
                framebuffer,
                x,
                y,
                damage_state(armor.saturating_add(1), armor_max.saturating_add(1)),
                enemy.age,
                34,
            );
            render_hit_flash(framebuffer, x, y, HitFlashKind::Carrion, hit);
        }

        for enemy in &encounter.combat.specials {
            let key = vc20_special_key(enemy);
            let hit = self.hit_reactions.special_visual(key);
            let x = presentation_coord(enemy.x) + hit.offset_x;
            let y = presentation_coord(enemy.y) + hit.offset_y;
            let armor_max = vc20_special_armor_max(enemy.kind);
            let armor = defenses
                .special_armor
                .get(&key)
                .copied()
                .unwrap_or(armor_max);
            let hp_max = vc20_special_hp_max(enemy.kind);
            let damage = damage_state(
                armor.saturating_add(enemy.hp),
                armor_max.saturating_add(hp_max),
            );
            let flash_kind = match enemy.kind {
                SpecialKind::GraveKnight => {
                    render_grave_knight(framebuffer, x, y, enemy.age);
                    HitFlashKind::GraveKnight
                }
                SpecialKind::BellWraith => {
                    render_bell_wraith(framebuffer, x, y, enemy.age, enemy.phase);
                    HitFlashKind::BellWraith
                }
                SpecialKind::RelicCarrier => {
                    render_relic_carrier(
                        framebuffer,
                        x,
                        y,
                        enemy.age,
                        enemy.phase,
                        enemy.direction,
                    );
                    HitFlashKind::RelicCarrier
                }
            };
            render_damage_marks(framebuffer, x, y, damage, enemy.age, 40);
            render_hit_flash(framebuffer, x, y, flash_kind, hit);
        }

        for threat in &encounter.threats {
            let key = vc20_threat_key(threat);
            let hit = self.hit_reactions.threat_visual(key);
            let x = presentation_coord(threat.x) + hit.offset_x;
            let y = presentation_coord(threat.y) + hit.offset_y;
            let armor_max = vc20_threat_armor_max(threat.kind);
            let armor = defenses
                .threat_armor
                .get(&key)
                .copied()
                .unwrap_or(armor_max);
            let hp_max = vc20_threat_hp_max(threat.kind);
            let damage = damage_state(
                armor.saturating_add(threat.hp),
                armor_max.saturating_add(hp_max),
            );
            let flash_kind = match threat.kind {
                ThreatKind::ChoirNode => {
                    render_choir_node(framebuffer, x, y, threat.age, threat.phase);
                    HitFlashKind::ChoirNode
                }
                ThreatKind::VoidLeech => {
                    render_void_leech(
                        framebuffer,
                        x,
                        y,
                        threat.age,
                        threat.phase,
                        threat.charge,
                    );
                    HitFlashKind::VoidLeech
                }
            };
            render_damage_marks(framebuffer, x, y, damage, threat.age, 44);
            render_hit_flash(framebuffer, x, y, flash_kind, hit);
        }

        let base = self.game.presentation_base();
        if base.encounter_phase != EncounterPhase::Cleared
            && let Some(boss) = base.boss
        {
            let hit = self.hit_reactions.boss_visual();
            let mut rendered_boss = boss;
            rendered_boss.x += hit.offset_x as f32 / VC_VISUAL_PRESENTATION_SCALE as f32;
            rendered_boss.y += hit.offset_y as f32 / VC_VISUAL_PRESENTATION_SCALE as f32;
            render_bellkeeper(framebuffer, rendered_boss);
            render_hit_flash(
                framebuffer,
                presentation_coord(boss.x) + hit.offset_x,
                presentation_coord(boss.y) + hit.offset_y,
                HitFlashKind::Bellkeeper,
                hit,
            );
        }
    }

    fn render_enemy_defenses(&self, framebuffer: &mut Framebuffer) {
        let defenses = self.game.presentation_defense_model();
        let encounter = self.game.presentation_encounter_model();
        for enemy in &self.game.presentation_base().enemies {
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let armor = defenses
                .carrion_armor
                .get(&vc20_carrion_key(enemy))
                .copied()
                .unwrap_or(armor_max);
            dual_bar(
                framebuffer,
                presentation_coord(enemy.x),
                presentation_coord(enemy.y) - 39,
                armor,
                armor_max,
                1,
                1,
            );
        }

        for enemy in &encounter.combat.specials {
            let armor_max = vc20_special_armor_max(enemy.kind);
            let armor = defenses
                .special_armor
                .get(&vc20_special_key(enemy))
                .copied()
                .unwrap_or(armor_max);
            dual_bar(
                framebuffer,
                presentation_coord(enemy.x),
                presentation_coord(enemy.y) - 43,
                armor,
                armor_max,
                enemy.hp,
                vc20_special_hp_max(enemy.kind),
            );
        }

        for threat in &encounter.threats {
            let armor_max = vc20_threat_armor_max(threat.kind);
            let armor = defenses
                .threat_armor
                .get(&vc20_threat_key(threat))
                .copied()
                .unwrap_or(armor_max);
            dual_bar(
                framebuffer,
                presentation_coord(threat.x),
                presentation_coord(threat.y) - 47,
                armor,
                armor_max,
                threat.hp,
                vc20_threat_hp_max(threat.kind),
            );
        }

        let base = self.game.presentation_base();
        if base.encounter_phase == EncounterPhase::BossFight
            && let Some(boss) = base.boss
        {
            let x = presentation_coord(boss.x);
            let y = presentation_coord(boss.y);
            let color = if defenses.boss_shield_flash_timer > 0.0 {
                PRESENTATION_ARMOR_LIGHT
            } else {
                PRESENTATION_ARMOR_COLOR
            };
            if defenses.boss_shield > 0 {
                let pulse = ((base.animation_time * 7.0).sin().abs() * 4.0) as u32;
                framebuffer.draw_circle(x, y - 6, 63 + pulse, color);
            } else if defenses.boss_shield_break_timer > 0.0 {
                framebuffer.draw_circle(x, y - 6, 65, CANTICLE_COLOR);
            }
            dual_bar(
                framebuffer,
                x,
                y - 76,
                defenses.boss_shield,
                PRESENTATION_BOSS_SHIELD_MAX,
                boss.hp,
                BELLKEEPER_MAX_HP,
            );
        }
    }
}

fn dual_bar(
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
    framebuffer.fill_rect(left, y, width, 3, PRESENTATION_ARMOR_BG);
    if armor_max > 0 && armor > 0 {
        let fill = width.saturating_mul(armor.min(armor_max)) / armor_max;
        framebuffer.fill_rect(left, y, fill, 3, PRESENTATION_ARMOR_COLOR);
    }
    framebuffer.fill_rect(left, y + 5, width, 3, CORE_BG);
    if hp_max > 0 && hp > 0 {
        let fill = width.saturating_mul(hp.min(hp_max)) / hp_max;
        framebuffer.fill_rect(left, y + 5, fill, 3, PRESENTATION_HULL_COLOR);
    }
}

#[cfg(test)]
mod presentation_bestiary_tests {
    use super::*;

    #[test]
    fn damage_states_follow_effective_health_thirds() {
        assert_eq!(damage_state(10, 10), DamageState::Intact);
        assert_eq!(damage_state(6, 10), DamageState::Damaged);
        assert_eq!(damage_state(3, 10), DamageState::Critical);
    }

    #[test]
    fn hd_choir_node_keeps_changes_local() {
        let mut framebuffer = Framebuffer::new(96, 96);
        framebuffer.clear(Pixel::BLUE);
        render_choir_node(&mut framebuffer, 48, 48, 1.0, 0.0);
        assert_ne!(framebuffer.pixel(48, 48), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(95, 95), Some(Pixel::BLUE));
    }

    #[test]
    fn dual_bar_stays_local() {
        let mut framebuffer = Framebuffer::new(80, 24);
        framebuffer.clear(Pixel::BLUE);
        dual_bar(&mut framebuffer, 40, 8, 2, 3, 4, 6);
        assert_eq!(framebuffer.pixel(0, 0), Some(Pixel::BLUE));
        assert_eq!(framebuffer.pixel(79, 23), Some(Pixel::BLUE));
    }
}
