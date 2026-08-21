impl VoidCanticlePresentation {
    fn render_pickups(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.presentation_base();
        for cinder in &base.cinders {
            let x = presentation_coord(cinder.x);
            let y = presentation_coord(cinder.y);
            framebuffer.fill_circle(x, y, 3, CINDER);
            framebuffer.draw_line(x, y - 7, x, y - 4, CANTICLE_COLOR);
            framebuffer.draw(x - 2, y, CANTICLE_COLOR);
            framebuffer.draw(x + 2, y, CANTICLE_COLOR);
        }

        let progression = self.game.presentation_progression();
        let game = &progression.progression.combat.combat.ui.inner.inner;
        for relic in &game.relics {
            let x = presentation_coord(relic.x);
            let y = presentation_coord(relic.y);
            let pulse = ((relic.age * 7.0 + relic.phase).sin() * 2.0).round() as i32;
            framebuffer.draw_line(x, y - 9 - pulse, x + 8 + pulse, y, POWER_RELIC_LIGHT);
            framebuffer.draw_line(x + 8 + pulse, y, x, y + 9 + pulse, POWER_RELIC_LIGHT);
            framebuffer.draw_line(x, y + 9 + pulse, x - 8 - pulse, y, POWER_RELIC_LIGHT);
            framebuffer.draw_line(x - 8 - pulse, y, x, y - 9 - pulse, POWER_RELIC_LIGHT);
            framebuffer.fill_circle(x, y, 3, POWER_RELIC);
            framebuffer.draw(x, y, CANTICLE_COLOR);
        }

        for orb in &progression.progression.xp_orbs {
            let x = presentation_coord(orb.x);
            let y = presentation_coord(orb.y);
            framebuffer.draw_line(x, y - 5, x + 4, y, XP_SHARD_EDGE);
            framebuffer.draw_line(x + 4, y, x, y + 5, XP_SHARD_EDGE);
            framebuffer.draw_line(x, y + 5, x - 4, y, XP_SHARD_EDGE);
            framebuffer.draw_line(x - 4, y, x, y - 5, XP_SHARD_EDGE);
            framebuffer.fill_circle(x, y, 1, XP_SHARD_CORE);
        }

        for (x, y) in progression.orbital_positions() {
            let x = presentation_coord(x);
            let y = presentation_coord(y);
            framebuffer.draw_circle(x, y, 5, POWER_RELIC_LIGHT);
            framebuffer.fill_circle(x, y, 2, PILGRIM_VIOLET);
            framebuffer.draw(x, y, BOLT_CORE);
        }
    }

    fn render_projectiles(&self, framebuffer: &mut Framebuffer) {
        let base = self.game.presentation_base();
        for bullet in &base.player_bullets {
            let x = presentation_coord(bullet.x);
            let y = presentation_coord(bullet.y);
            framebuffer.draw_line(x, y + 6, x, y - 7, BOLT_CORE);
            framebuffer.draw_line(x - 1, y + 4, x - 1, y - 4, BOLT_EDGE);
            framebuffer.draw_line(x + 1, y + 4, x + 1, y - 4, BOLT_EDGE);
        }

        let progression = self.game.presentation_progression();
        let game = &progression.progression.combat.combat.ui.inner.inner;
        for shot in &game.power_shots {
            let x = presentation_coord(shot.x);
            let y = presentation_coord(shot.y);
            let radius = if shot.radius >= 2 || shot.damage > 1 { 4 } else { 2 };
            framebuffer.fill_circle(x, y, radius, BOLT_EDGE);
            framebuffer.fill_circle(x, y, 1, BOLT_CORE);
            framebuffer.draw_line(x, y + 5, x, y + 11, BOLT_RELIC);
            if shot.vx.abs() > 6.0 {
                let wing = if shot.vx > 0.0 { -5 } else { 5 };
                framebuffer.draw_line(x, y + 2, x + wing, y + 7, BOLT_RELIC);
            }
        }

        let pressure = self.game.presentation_void_pressure();
        let boss_phase = base.boss.map(Bellkeeper::phase);
        for (index, bullet) in base.enemy_bullets.iter().enumerate() {
            let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt().max(1.0);
            let style = self
                .projectile_provenance
                .style_for(index)
                .unwrap_or_else(|| enemy_shot_style(base.encounter_phase, speed));
            render_enemy_bullet(framebuffer, *bullet, style, pressure, boss_phase);
        }
    }
}

fn enemy_shot_style(encounter_phase: EncounterPhase, speed: f32) -> EnemyShotStyle {
    if encounter_phase == EncounterPhase::BossFight {
        return EnemyShotStyle::Bellkeeper;
    }
    if (speed - 48.0).abs() <= 1.0 {
        EnemyShotStyle::Wraith
    } else if (speed - 62.0).abs() <= 1.0 {
        EnemyShotStyle::VoidPulse
    } else if (speed - ENEMY_SHOT_SPEED).abs() <= 1.0 {
        EnemyShotStyle::Carrion
    } else {
        EnemyShotStyle::Void
    }
}

fn render_enemy_bullet(
    framebuffer: &mut Framebuffer,
    bullet: Bullet,
    style: EnemyShotStyle,
    pressure: VoidPressure,
    boss_phase: Option<BellPhase>,
) {
    let x = presentation_coord(bullet.x);
    let y = presentation_coord(bullet.y);
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
        EnemyShotStyle::Carrion => {
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
        EnemyShotStyle::Wraith => {
            let (tail_x, tail_y) = tail(5.0);
            framebuffer.draw_line(tail_x, tail_y, x, y, WRAITH_GLOW);
            framebuffer.draw_circle(x, y, 4, WRAITH_GLOW);
            framebuffer.draw_circle(x, y, 2, WRAITH_CORE);
            framebuffer.draw(x, y, CANTICLE_COLOR);
        }
        EnemyShotStyle::VoidPulse => {
            let (tail_x, tail_y) = tail(4.0);
            framebuffer.draw_line(tail_x, tail_y, x, y, ART_VOID);
            framebuffer.draw_circle(x, y, 5, LEECH_GLOW);
            framebuffer.fill_circle(x, y, 2, DANGER);
            framebuffer.draw(x, y, VOID_LIGHT);
        }
        EnemyShotStyle::Void => {
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
        EnemyShotStyle::Bellkeeper => {
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

#[cfg(test)]
mod presentation_projectile_tests {
    use super::*;

    #[test]
    fn enemy_shot_styles_follow_existing_pattern_speeds() {
        assert_eq!(
            enemy_shot_style(EncounterPhase::Waves, 48.0),
            EnemyShotStyle::Wraith
        );
        assert_eq!(
            enemy_shot_style(EncounterPhase::Waves, 62.0),
            EnemyShotStyle::VoidPulse
        );
        assert_eq!(
            enemy_shot_style(EncounterPhase::Waves, ENEMY_SHOT_SPEED),
            EnemyShotStyle::Carrion
        );
        assert_eq!(
            enemy_shot_style(EncounterPhase::Waves, 96.0),
            EnemyShotStyle::Void
        );
        assert_eq!(
            enemy_shot_style(EncounterPhase::BossFight, 48.0),
            EnemyShotStyle::Bellkeeper
        );
    }
}
