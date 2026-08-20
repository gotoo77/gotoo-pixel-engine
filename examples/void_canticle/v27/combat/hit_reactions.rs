const VC27_HIT_REACTION_DURATION: f32 = 0.11;
const VC27_HIT_FLASH_WINDOW: f32 = 0.075;
const VC27_HIT_RECOIL_X: f32 = 4.0;
const VC27_HIT_RECOIL_Y: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc27HitLayer {
    Barrier,
    Hull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vc27HitFlashKind {
    Carrion,
    GraveKnight,
    BellWraith,
    RelicCarrier,
    ChoirNode,
    VoidLeech,
    Bellkeeper,
    Pilgrim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Vc27HitBudget {
    barrier: u32,
    hull: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Vc27PlayerHitBudget {
    barrier: f32,
    hull: f32,
}

#[derive(Debug, Clone, Copy)]
struct Vc27HitReaction {
    remaining: f32,
    layer: Vc27HitLayer,
}

#[derive(Debug, Clone, Copy)]
struct Vc27HitVisual {
    offset_x: i32,
    offset_y: i32,
    flash: f32,
    layer: Option<Vc27HitLayer>,
}

impl Default for Vc27HitVisual {
    fn default() -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            flash: 0.0,
            layer: None,
        }
    }
}

#[derive(Default)]
struct Vc27HitSnapshot {
    carrion: std::collections::BTreeMap<CarrionDefenseKey, Vc27HitBudget>,
    special: std::collections::BTreeMap<SpecialDefenseKey, Vc27HitBudget>,
    threat: std::collections::BTreeMap<ThreatDefenseKey, Vc27HitBudget>,
    boss: Option<Vc27HitBudget>,
    player: Vc27PlayerHitBudget,
}

impl Vc27HitSnapshot {
    fn capture(game: &VoidCanticleV23Sustain) -> Self {
        let v20 = game.game.v20();
        let mut snapshot = Self::default();

        for enemy in game.game.base().enemies.iter().filter(|enemy| enemy.alive) {
            let key = vc20_carrion_key(enemy);
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let barrier = v20.carrion_armor.get(&key).copied().unwrap_or(armor_max);
            snapshot.carrion.insert(key, Vc27HitBudget { barrier, hull: 1 });
        }

        let v12 = &v20.game.v12();
        for enemy in v12.combat.specials.iter().filter(|enemy| enemy.alive) {
            let key = vc20_special_key(enemy);
            let armor_max = vc20_special_armor_max(enemy.kind);
            let barrier = v20.special_armor.get(&key).copied().unwrap_or(armor_max);
            snapshot.carrion.get(&(
                0,
                0,
                0,
                0,
            ));
            snapshot.special.insert(
                key,
                Vc27HitBudget {
                    barrier,
                    hull: enemy.hp,
                },
            );
        }

        for threat in v12.threats.iter().filter(|threat| threat.alive) {
            let key = vc20_threat_key(threat);
            let armor_max = vc20_threat_armor_max(threat.kind);
            let barrier = v20.threat_armor.get(&key).copied().unwrap_or(armor_max);
            snapshot.threat.insert(
                key,
                Vc27HitBudget {
                    barrier,
                    hull: threat.hp,
                },
            );
        }

        if let Some(boss) = game.game.base().boss {
            snapshot.boss = Some(Vc27HitBudget {
                barrier: v20.boss_shield,
                hull: boss.hp,
            });
        }

        let combat = game.combat_model();
        snapshot.player = Vc27PlayerHitBudget {
            barrier: combat.player_shield,
            hull: combat.player_hull,
        };
        snapshot
    }
}

#[derive(Default)]
struct Vc27HitReactionState {
    carrion: std::collections::BTreeMap<CarrionDefenseKey, Vc27HitReaction>,
    special: std::collections::BTreeMap<SpecialDefenseKey, Vc27HitReaction>,
    threat: std::collections::BTreeMap<ThreatDefenseKey, Vc27HitReaction>,
    boss: Option<Vc27HitReaction>,
    player: Option<Vc27HitReaction>,
}

impl Vc27HitReactionState {
    fn update(&mut self, dt: f32, before: &Vc27HitSnapshot, game: &VoidCanticleV23Sustain) {
        vc27_decay_hit_map(&mut self.carrion, dt);
        vc27_decay_hit_map(&mut self.special, dt);
        vc27_decay_hit_map(&mut self.threat, dt);
        vc27_decay_hit_slot(&mut self.boss, dt);
        vc27_decay_hit_slot(&mut self.player, dt);

        let after = Vc27HitSnapshot::capture(game);
        for (key, budget) in &after.carrion {
            if let Some(layer) = before
                .carrion
                .get(key)
                .and_then(|previous| vc27_detect_hit(*previous, *budget))
            {
                self.carrion.insert(*key, vc27_new_hit_reaction(layer));
            }
        }
        for (key, budget) in &after.special {
            if let Some(layer) = before
                .special
                .get(key)
                .and_then(|previous| vc27_detect_hit(*previous, *budget))
            {
                self.special.insert(*key, vc27_new_hit_reaction(layer));
            }
        }
        for (key, budget) in &after.threat {
            if let Some(layer) = before
                .threat
                .get(key)
                .and_then(|previous| vc27_detect_hit(*previous, *budget))
            {
                self.threat.insert(*key, vc27_new_hit_reaction(layer));
            }
        }
        if let (Some(previous), Some(current)) = (before.boss, after.boss)
            && let Some(layer) = vc27_detect_hit(previous, current)
        {
            self.boss = Some(vc27_new_hit_reaction(layer));
        }
        if after.boss.is_none() {
            self.boss = None;
        }

        if after.player.hull > before.player.hull + f32::EPSILON
            || after.player.barrier > before.player.barrier + f32::EPSILON
        {
            self.player = None;
        } else if let Some(layer) = vc27_detect_player_hit(before.player, after.player) {
            self.player = Some(vc27_new_hit_reaction(layer));
        }
    }

    fn carrion_visual(&self, key: CarrionDefenseKey) -> Vc27HitVisual {
        vc27_hit_visual(self.carrion.get(&key).copied(), vc27_carrion_hit_seed(key))
    }

    fn special_visual(&self, key: SpecialDefenseKey) -> Vc27HitVisual {
        vc27_hit_visual(self.special.get(&key).copied(), vc27_special_hit_seed(key))
    }

    fn threat_visual(&self, key: ThreatDefenseKey) -> Vc27HitVisual {
        vc27_hit_visual(self.threat.get(&key).copied(), vc27_threat_hit_seed(key))
    }

    fn boss_visual(&self) -> Vc27HitVisual {
        vc27_hit_visual(self.boss, 0xB311_7E11)
    }

    fn player_visual(&self) -> Vc27HitVisual {
        vc27_hit_visual(self.player, 0xC417_1C1E)
    }
}

fn vc27_new_hit_reaction(layer: Vc27HitLayer) -> Vc27HitReaction {
    Vc27HitReaction {
        remaining: VC27_HIT_REACTION_DURATION,
        layer,
    }
}

fn vc27_detect_hit(before: Vc27HitBudget, after: Vc27HitBudget) -> Option<Vc27HitLayer> {
    if after.hull < before.hull {
        Some(Vc27HitLayer::Hull)
    } else if after.barrier < before.barrier {
        Some(Vc27HitLayer::Barrier)
    } else {
        None
    }
}

fn vc27_detect_player_hit(
    before: Vc27PlayerHitBudget,
    after: Vc27PlayerHitBudget,
) -> Option<Vc27HitLayer> {
    if after.hull + f32::EPSILON < before.hull {
        Some(Vc27HitLayer::Hull)
    } else if after.barrier + f32::EPSILON < before.barrier {
        Some(Vc27HitLayer::Barrier)
    } else {
        None
    }
}

fn vc27_decay_hit_map<K: Ord>(
    reactions: &mut std::collections::BTreeMap<K, Vc27HitReaction>,
    dt: f32,
) {
    reactions.retain(|_, reaction| {
        reaction.remaining = (reaction.remaining - dt).max(0.0);
        reaction.remaining > 0.0
    });
}

fn vc27_decay_hit_slot(reaction: &mut Option<Vc27HitReaction>, dt: f32) {
    let Some(active) = reaction.as_mut() else {
        return;
    };
    active.remaining = (active.remaining - dt).max(0.0);
    if active.remaining <= 0.0 {
        *reaction = None;
    }
}

fn vc27_hit_visual(reaction: Option<Vc27HitReaction>, seed: u32) -> Vc27HitVisual {
    let Some(reaction) = reaction else {
        return Vc27HitVisual::default();
    };
    let remaining_ratio = (reaction.remaining / VC27_HIT_REACTION_DURATION).clamp(0.0, 1.0);
    let elapsed = VC27_HIT_REACTION_DURATION - reaction.remaining;
    let flash = if elapsed < VC27_HIT_FLASH_WINDOW {
        1.0 - elapsed / VC27_HIT_FLASH_WINDOW
    } else {
        0.0
    };
    let direction = if seed & 1 == 0 { -1.0 } else { 1.0 };

    Vc27HitVisual {
        offset_x: (direction * VC27_HIT_RECOIL_X * remaining_ratio).round() as i32,
        offset_y: (-VC27_HIT_RECOIL_Y * remaining_ratio).round() as i32,
        flash,
        layer: Some(reaction.layer),
    }
}

fn vc27_carrion_hit_seed(key: CarrionDefenseKey) -> u32 {
    key.0 ^ key.1.rotate_left(7) ^ key.2.rotate_left(13) ^ u32::from(key.3)
}

fn vc27_special_hit_seed(key: SpecialDefenseKey) -> u32 {
    u32::from(key.0) ^ key.1.rotate_left(5) ^ key.2.rotate_left(11) ^ key.3.rotate_left(17)
}

fn vc27_threat_hit_seed(key: ThreatDefenseKey) -> u32 {
    u32::from(key.0) ^ key.1.rotate_left(3) ^ key.2.rotate_left(9) ^ key.3.rotate_left(15)
}

fn vc27_render_hit_flash(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    kind: Vc27HitFlashKind,
    visual: Vc27HitVisual,
) {
    if visual.flash <= 0.0 {
        return;
    }

    let primary = match visual.layer {
        Some(Vc27HitLayer::Barrier) => VC20_ARMOR_LIGHT,
        Some(Vc27HitLayer::Hull) => CANTICLE_COLOR,
        None => return,
    };
    let secondary = match visual.layer {
        Some(Vc27HitLayer::Barrier) => TEXT,
        Some(Vc27HitLayer::Hull) => DANGER,
        None => return,
    };
    let hot = visual.flash > 0.45;

    match kind {
        Vc27HitFlashKind::Carrion => {
            framebuffer.draw_circle(x, y, 8, primary);
            framebuffer.draw_line(x - 8, y - 4, x - 25, y - 7, secondary);
            framebuffer.draw_line(x + 8, y - 4, x + 25, y - 7, secondary);
            if hot {
                framebuffer.draw_line(x - 6, y + 5, x - 20, y + 8, primary);
                framebuffer.draw_line(x + 6, y + 5, x + 20, y + 8, primary);
            }
        }
        Vc27HitFlashKind::GraveKnight => {
            framebuffer.draw_rect(x - 17, y - 23, 35, 46, primary);
            framebuffer.draw_line(x - 18, y - 8, x - 34, y + 2, secondary);
            framebuffer.draw_line(x + 18, y - 8, x + 34, y + 2, secondary);
            if hot {
                framebuffer.draw_rect(x - 7, y - 11, 15, 15, primary);
            }
        }
        Vc27HitFlashKind::BellWraith => {
            framebuffer.draw_circle(x, y - 5, 19, primary);
            framebuffer.draw_line(x - 15, y - 8, x - 9, y - 22, secondary);
            framebuffer.draw_line(x + 15, y - 8, x + 9, y - 22, secondary);
            if hot {
                framebuffer.draw_line(x, y + 8, x, y + 28, primary);
            }
        }
        Vc27HitFlashKind::RelicCarrier => {
            framebuffer.draw_rect(x - 18, y - 15, 37, 31, primary);
            framebuffer.draw_circle(x, y, 11, secondary);
            if hot {
                framebuffer.draw_line(x - 33, y - 16, x - 18, y - 8, primary);
                framebuffer.draw_line(x + 33, y - 16, x + 18, y - 8, primary);
            }
        }
        Vc27HitFlashKind::ChoirNode => {
            framebuffer.draw_circle(x, y, 21, primary);
            framebuffer.draw_line(x - 28, y, x + 28, y, secondary);
            framebuffer.draw_line(x, y - 28, x, y + 28, secondary);
            if hot {
                framebuffer.draw_circle(x, y, 10, primary);
            }
        }
        Vc27HitFlashKind::VoidLeech => {
            framebuffer.draw_circle(x, y, 21, primary);
            framebuffer.draw_line(x - 17, y - 12, x - 32, y - 22, secondary);
            framebuffer.draw_line(x + 17, y - 12, x + 32, y - 22, secondary);
            if hot {
                framebuffer.draw_circle(x, y, 9, primary);
            }
        }
        Vc27HitFlashKind::Bellkeeper => {
            framebuffer.draw_circle(x, y - 6, 40, primary);
            framebuffer.draw_line(x - 25, y - 30, x, y - 48, secondary);
            framebuffer.draw_line(x + 25, y - 30, x, y - 48, secondary);
            framebuffer.draw_line(x - 31, y + 17, x + 31, y + 17, primary);
            if hot {
                framebuffer.draw_circle(x, y - 9, 15, secondary);
            }
        }
        Vc27HitFlashKind::Pilgrim => {
            framebuffer.draw_line(x, y - 29, x - 10, y - 16, primary);
            framebuffer.draw_line(x, y - 29, x + 10, y - 16, primary);
            framebuffer.draw_line(x - 10, y - 16, x - 22, y + 4, secondary);
            framebuffer.draw_line(x + 10, y - 16, x + 22, y + 4, secondary);
            if hot {
                framebuffer.draw_circle(x, y - 3, 8, primary);
            }
        }
    }
}

#[cfg(test)]
mod v27_hit_reaction_tests {
    use super::*;

    #[test]
    fn hull_damage_has_priority_over_barrier_damage() {
        assert_eq!(
            vc27_detect_hit(
                Vc27HitBudget { barrier: 3, hull: 5 },
                Vc27HitBudget { barrier: 0, hull: 4 },
            ),
            Some(Vc27HitLayer::Hull)
        );
        assert_eq!(
            vc27_detect_hit(
                Vc27HitBudget { barrier: 3, hull: 5 },
                Vc27HitBudget { barrier: 2, hull: 5 },
            ),
            Some(Vc27HitLayer::Barrier)
        );
    }

    #[test]
    fn hit_visual_recoils_then_returns_to_anchor() {
        let fresh = vc27_hit_visual(Some(vc27_new_hit_reaction(Vc27HitLayer::Hull)), 1);
        let ending = vc27_hit_visual(
            Some(Vc27HitReaction {
                remaining: 0.001,
                layer: Vc27HitLayer::Hull,
            }),
            1,
        );
        assert!(fresh.offset_x.abs() > ending.offset_x.abs());
        assert!(fresh.offset_y.abs() >= ending.offset_y.abs());
        assert!(fresh.flash > ending.flash);
    }

    #[test]
    fn player_barrier_hit_is_detected_without_hull_loss() {
        assert_eq!(
            vc27_detect_player_hit(
                Vc27PlayerHitBudget {
                    barrier: 20.0,
                    hull: 80.0,
                },
                Vc27PlayerHitBudget {
                    barrier: 12.0,
                    hull: 80.0,
                },
            ),
            Some(Vc27HitLayer::Barrier)
        );
    }
}
