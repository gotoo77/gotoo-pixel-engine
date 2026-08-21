const HIT_REACTION_DURATION: f32 = 0.11;
const HIT_FLASH_WINDOW: f32 = 0.075;
const HIT_RECOIL_X: f32 = 4.0;
const HIT_RECOIL_Y: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitLayer {
    Barrier,
    Hull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitFlashKind {
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
struct HitBudget {
    barrier: u32,
    hull: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct PlayerHitBudget {
    barrier: f32,
    hull: f32,
}

#[derive(Debug, Clone, Copy)]
struct HitReaction {
    remaining: f32,
    layer: HitLayer,
}

#[derive(Debug, Clone, Copy)]
struct HitVisual {
    offset_x: i32,
    offset_y: i32,
    flash: f32,
    layer: Option<HitLayer>,
}

impl Default for HitVisual {
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
struct HitSnapshot {
    carrion: std::collections::BTreeMap<CarrionDefenseKey, HitBudget>,
    special: std::collections::BTreeMap<SpecialDefenseKey, HitBudget>,
    threat: std::collections::BTreeMap<ThreatDefenseKey, HitBudget>,
    boss: Option<HitBudget>,
    player: PlayerHitBudget,
}

impl HitSnapshot {
    fn capture(game: &GameplayRuntime) -> Self {
        let defenses = game.presentation_defense_model();
        let encounter = game.presentation_encounter_model();
        let mut snapshot = Self::default();

        for enemy in game
            .presentation_base()
            .enemies
            .iter()
            .filter(|enemy| enemy.alive)
        {
            let key = vc20_carrion_key(enemy);
            let armor_max = vc20_carrion_armor_max(enemy.pattern);
            let barrier = defenses
                .carrion_armor
                .get(&key)
                .copied()
                .unwrap_or(armor_max);
            snapshot.carrion.insert(key, HitBudget { barrier, hull: 1 });
        }

        for enemy in encounter
            .combat
            .specials
            .iter()
            .filter(|enemy| enemy.alive)
        {
            let key = vc20_special_key(enemy);
            let armor_max = vc20_special_armor_max(enemy.kind);
            let barrier = defenses
                .special_armor
                .get(&key)
                .copied()
                .unwrap_or(armor_max);
            snapshot.special.insert(
                key,
                HitBudget {
                    barrier,
                    hull: enemy.hp,
                },
            );
        }

        for threat in encounter.threats.iter().filter(|threat| threat.alive) {
            let key = vc20_threat_key(threat);
            let armor_max = vc20_threat_armor_max(threat.kind);
            let barrier = defenses
                .threat_armor
                .get(&key)
                .copied()
                .unwrap_or(armor_max);
            snapshot.threat.insert(
                key,
                HitBudget {
                    barrier,
                    hull: threat.hp,
                },
            );
        }

        if let Some(boss) = game.presentation_base().boss {
            snapshot.boss = Some(HitBudget {
                barrier: defenses.boss_shield,
                hull: boss.hp,
            });
        }

        let combat = game.combat_model();
        snapshot.player = PlayerHitBudget {
            barrier: combat.player_shield,
            hull: combat.player_hull,
        };
        snapshot
    }
}

#[derive(Default)]
struct HitReactionState {
    carrion: std::collections::BTreeMap<CarrionDefenseKey, HitReaction>,
    special: std::collections::BTreeMap<SpecialDefenseKey, HitReaction>,
    threat: std::collections::BTreeMap<ThreatDefenseKey, HitReaction>,
    boss: Option<HitReaction>,
    player: Option<HitReaction>,
}

impl HitReactionState {
    fn update(&mut self, dt: f32, before: &HitSnapshot, game: &GameplayRuntime) {
        decay_hit_map(&mut self.carrion, dt);
        decay_hit_map(&mut self.special, dt);
        decay_hit_map(&mut self.threat, dt);
        decay_hit_slot(&mut self.boss, dt);
        decay_hit_slot(&mut self.player, dt);

        let after = HitSnapshot::capture(game);
        for (key, budget) in &after.carrion {
            if let Some(layer) = before
                .carrion
                .get(key)
                .and_then(|previous| detect_hit(*previous, *budget))
            {
                self.carrion.insert(*key, new_hit_reaction(layer));
            }
        }
        for (key, budget) in &after.special {
            if let Some(layer) = before
                .special
                .get(key)
                .and_then(|previous| detect_hit(*previous, *budget))
            {
                self.special.insert(*key, new_hit_reaction(layer));
            }
        }
        for (key, budget) in &after.threat {
            if let Some(layer) = before
                .threat
                .get(key)
                .and_then(|previous| detect_hit(*previous, *budget))
            {
                self.threat.insert(*key, new_hit_reaction(layer));
            }
        }
        if let (Some(previous), Some(current)) = (before.boss, after.boss)
            && let Some(layer) = detect_hit(previous, current)
        {
            self.boss = Some(new_hit_reaction(layer));
        }
        if after.boss.is_none() {
            self.boss = None;
        }

        if after.player.hull > before.player.hull + f32::EPSILON
            || after.player.barrier > before.player.barrier + f32::EPSILON
        {
            self.player = None;
        } else if let Some(layer) = detect_player_hit(before.player, after.player) {
            self.player = Some(new_hit_reaction(layer));
        }
    }

    fn carrion_visual(&self, key: CarrionDefenseKey) -> HitVisual {
        hit_visual(self.carrion.get(&key).copied(), carrion_hit_seed(key))
    }

    fn special_visual(&self, key: SpecialDefenseKey) -> HitVisual {
        hit_visual(self.special.get(&key).copied(), special_hit_seed(key))
    }

    fn threat_visual(&self, key: ThreatDefenseKey) -> HitVisual {
        hit_visual(self.threat.get(&key).copied(), threat_hit_seed(key))
    }

    fn boss_visual(&self) -> HitVisual {
        hit_visual(self.boss, 0xB311_7E11)
    }

    fn player_visual(&self) -> HitVisual {
        hit_visual(self.player, 0xC417_1C1E)
    }
}

fn new_hit_reaction(layer: HitLayer) -> HitReaction {
    HitReaction {
        remaining: HIT_REACTION_DURATION,
        layer,
    }
}

fn detect_hit(before: HitBudget, after: HitBudget) -> Option<HitLayer> {
    if after.hull < before.hull {
        Some(HitLayer::Hull)
    } else if after.barrier < before.barrier {
        Some(HitLayer::Barrier)
    } else {
        None
    }
}

fn detect_player_hit(before: PlayerHitBudget, after: PlayerHitBudget) -> Option<HitLayer> {
    if after.hull + f32::EPSILON < before.hull {
        Some(HitLayer::Hull)
    } else if after.barrier + f32::EPSILON < before.barrier {
        Some(HitLayer::Barrier)
    } else {
        None
    }
}

fn decay_hit_map<K: Ord>(reactions: &mut std::collections::BTreeMap<K, HitReaction>, dt: f32) {
    reactions.retain(|_, reaction| {
        reaction.remaining = (reaction.remaining - dt).max(0.0);
        reaction.remaining > 0.0
    });
}

fn decay_hit_slot(reaction: &mut Option<HitReaction>, dt: f32) {
    let Some(active) = reaction.as_mut() else {
        return;
    };
    active.remaining = (active.remaining - dt).max(0.0);
    if active.remaining <= 0.0 {
        *reaction = None;
    }
}

fn hit_visual(reaction: Option<HitReaction>, seed: u32) -> HitVisual {
    let Some(reaction) = reaction else {
        return HitVisual::default();
    };
    let remaining_ratio = (reaction.remaining / HIT_REACTION_DURATION).clamp(0.0, 1.0);
    let elapsed = HIT_REACTION_DURATION - reaction.remaining;
    let flash = if elapsed < HIT_FLASH_WINDOW {
        1.0 - elapsed / HIT_FLASH_WINDOW
    } else {
        0.0
    };
    let direction = if seed & 1 == 0 { -1.0 } else { 1.0 };

    HitVisual {
        offset_x: (direction * HIT_RECOIL_X * remaining_ratio).round() as i32,
        offset_y: (-HIT_RECOIL_Y * remaining_ratio).round() as i32,
        flash,
        layer: Some(reaction.layer),
    }
}

fn carrion_hit_seed(key: CarrionDefenseKey) -> u32 {
    key.0 ^ key.1.rotate_left(7) ^ key.2.rotate_left(13) ^ u32::from(key.3)
}

fn special_hit_seed(key: SpecialDefenseKey) -> u32 {
    u32::from(key.0) ^ key.1.rotate_left(5) ^ key.2.rotate_left(11) ^ key.3.rotate_left(17)
}

fn threat_hit_seed(key: ThreatDefenseKey) -> u32 {
    u32::from(key.0) ^ key.1.rotate_left(3) ^ key.2.rotate_left(9) ^ key.3.rotate_left(15)
}

fn render_hit_flash(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    kind: HitFlashKind,
    visual: HitVisual,
) {
    if visual.flash <= 0.0 {
        return;
    }

    let primary = match visual.layer {
        Some(HitLayer::Barrier) => VC20_ARMOR_LIGHT,
        Some(HitLayer::Hull) => CANTICLE_COLOR,
        None => return,
    };
    let secondary = match visual.layer {
        Some(HitLayer::Barrier) => TEXT,
        Some(HitLayer::Hull) => DANGER,
        None => return,
    };
    let hot = visual.flash > 0.45;

    match kind {
        HitFlashKind::Carrion => {
            framebuffer.draw_circle(x, y, 8, primary);
            framebuffer.draw_line(x - 8, y - 4, x - 25, y - 7, secondary);
            framebuffer.draw_line(x + 8, y - 4, x + 25, y - 7, secondary);
            if hot {
                framebuffer.draw_line(x - 6, y + 5, x - 20, y + 8, primary);
                framebuffer.draw_line(x + 6, y + 5, x + 20, y + 8, primary);
            }
        }
        HitFlashKind::GraveKnight => {
            framebuffer.draw_rect(x - 17, y - 23, 35, 46, primary);
            framebuffer.draw_line(x - 18, y - 8, x - 34, y + 2, secondary);
            framebuffer.draw_line(x + 18, y - 8, x + 34, y + 2, secondary);
            if hot {
                framebuffer.draw_rect(x - 7, y - 11, 15, 15, primary);
            }
        }
        HitFlashKind::BellWraith => {
            framebuffer.draw_circle(x, y - 5, 19, primary);
            framebuffer.draw_line(x - 15, y - 8, x - 9, y - 22, secondary);
            framebuffer.draw_line(x + 15, y - 8, x + 9, y - 22, secondary);
            if hot {
                framebuffer.draw_line(x, y + 8, x, y + 28, primary);
            }
        }
        HitFlashKind::RelicCarrier => {
            framebuffer.draw_rect(x - 18, y - 15, 37, 31, primary);
            framebuffer.draw_circle(x, y, 11, secondary);
            if hot {
                framebuffer.draw_line(x - 33, y - 16, x - 18, y - 8, primary);
                framebuffer.draw_line(x + 33, y - 16, x + 18, y - 8, primary);
            }
        }
        HitFlashKind::ChoirNode => {
            framebuffer.draw_circle(x, y, 21, primary);
            framebuffer.draw_line(x - 28, y, x + 28, y, secondary);
            framebuffer.draw_line(x, y - 28, x, y + 28, secondary);
            if hot {
                framebuffer.draw_circle(x, y, 10, primary);
            }
        }
        HitFlashKind::VoidLeech => {
            framebuffer.draw_circle(x, y, 21, primary);
            framebuffer.draw_line(x - 17, y - 12, x - 32, y - 22, secondary);
            framebuffer.draw_line(x + 17, y - 12, x + 32, y - 22, secondary);
            if hot {
                framebuffer.draw_circle(x, y, 9, primary);
            }
        }
        HitFlashKind::Bellkeeper => {
            framebuffer.draw_circle(x, y - 6, 40, primary);
            framebuffer.draw_line(x - 25, y - 30, x, y - 48, secondary);
            framebuffer.draw_line(x + 25, y - 30, x, y - 48, secondary);
            framebuffer.draw_line(x - 31, y + 17, x + 31, y + 17, primary);
            if hot {
                framebuffer.draw_circle(x, y - 9, 15, secondary);
            }
        }
        HitFlashKind::Pilgrim => {
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
mod hit_reaction_tests {
    use super::*;

    #[test]
    fn hull_damage_has_priority_over_barrier_damage() {
        assert_eq!(
            detect_hit(
                HitBudget { barrier: 3, hull: 5 },
                HitBudget { barrier: 0, hull: 4 },
            ),
            Some(HitLayer::Hull)
        );
        assert_eq!(
            detect_hit(
                HitBudget { barrier: 3, hull: 5 },
                HitBudget { barrier: 2, hull: 5 },
            ),
            Some(HitLayer::Barrier)
        );
    }

    #[test]
    fn hit_visual_recoils_then_returns_to_anchor() {
        let fresh = hit_visual(Some(new_hit_reaction(HitLayer::Hull)), 1);
        let ending = hit_visual(
            Some(HitReaction {
                remaining: 0.001,
                layer: HitLayer::Hull,
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
            detect_player_hit(
                PlayerHitBudget {
                    barrier: 20.0,
                    hull: 80.0,
                },
                PlayerHitBudget {
                    barrier: 12.0,
                    hull: 80.0,
                },
            ),
            Some(HitLayer::Barrier)
        );
    }
}
