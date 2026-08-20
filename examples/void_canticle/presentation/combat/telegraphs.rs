impl VoidCanticleV27DirectPresentation {
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
}

fn vc27_telegraph_progress(timer: f32, window: f32) -> Option<f32> {
    if window <= 0.0 || timer <= 0.0 || timer > window {
        return None;
    }
    Some((1.0 - timer / window).clamp(0.0, 1.0))
}

#[cfg(test)]
mod v27_telegraph_tests {
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
}
