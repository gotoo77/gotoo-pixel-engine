fn weapon_profile(level: u8) -> WeaponProfile {
    match level.clamp(START_POWER_LEVEL, MAX_POWER_LEVEL) {
        1 => WeaponProfile {
            period: 0.105,
            radius: 1,
            volley_damage: 1,
            name: "EMBER",
        },
        2 => WeaponProfile {
            period: 0.098,
            radius: 1,
            volley_damage: 2,
            name: "TWIN",
        },
        3 => WeaponProfile {
            period: 0.090,
            radius: 2,
            volley_damage: 3,
            name: "TRIAD",
        },
        4 => WeaponProfile {
            period: 0.082,
            radius: 2,
            volley_damage: 4,
            name: "CHOIR",
        },
        _ => WeaponProfile {
            period: 0.074,
            radius: 3,
            volley_damage: 6,
            name: "RELIC",
        },
    }
}

fn spawn_power_volley(output: &mut Vec<PowerShot>, level: u8, x: f32, y: f32) {
    let profile = weapon_profile(level);

    let specs: &[(f32, f32, u32)] = match level.clamp(START_POWER_LEVEL, MAX_POWER_LEVEL) {
        1 => &[(0.0, 0.0, 1)],
        2 => &[(-3.0, 0.0, 1), (3.0, 0.0, 1)],
        3 => &[(-5.0, -10.0, 1), (0.0, 0.0, 1), (5.0, 10.0, 1)],
        4 => &[(-5.0, -12.0, 1), (0.0, 0.0, 2), (5.0, 12.0, 1)],
        _ => &[
            (-7.0, -22.0, 1),
            (-3.5, -10.0, 1),
            (0.0, 0.0, 2),
            (3.5, 10.0, 1),
            (7.0, 22.0, 1),
        ],
    };

    debug_assert_eq!(
        specs.iter().map(|(_, _, damage)| *damage).sum::<u32>(),
        profile.volley_damage
    );

    for &(offset_x, velocity_x, damage) in specs {
        output.push(PowerShot {
            x: x + offset_x,
            y,
            vx: velocity_x,
            vy: -PLAYER_SHOT_SPEED,
            damage,
            radius: profile.radius,
            alive: true,
        });
    }
}

fn should_drop_relic(destroyed_enemies: u32) -> bool {
    matches!(destroyed_enemies, 2 | 4 | 6 | 8)
}

fn power_after_death(level: u8) -> u8 {
    level.saturating_sub(1).max(START_POWER_LEVEL)
}

fn render_power_shot(framebuffer: &mut Framebuffer, shot: PowerShot, power_level: u8) {
    let x = shot.x.round() as i32;
    let y = shot.y.round() as i32;
    let outer = if power_level >= 4 {
        PILGRIM_VIOLET
    } else {
        SHOT
    };

    if shot.radius <= 1 {
        framebuffer.fill_rect(x - 1, y - 4, 3, 8, outer);
        framebuffer.draw(x, y - 5, POWER_RELIC_LIGHT);
    } else {
        framebuffer.fill_circle(x, y, shot.radius, outer);
        framebuffer.fill_rect(x - 1, y - 5, 3, 8, outer);
        framebuffer.draw(x, y - 6, POWER_RELIC_LIGHT);
    }
}

fn render_relic(framebuffer: &mut Framebuffer, relic: RelicDrop) {
    let x = relic.x.round() as i32;
    let y = relic.y.round() as i32;
    let pulse = ((relic.age * 8.0) as i32 & 1) == 0;
    let glow = if pulse {
        POWER_RELIC_LIGHT
    } else {
        POWER_RELIC
    };

    framebuffer.draw_circle(x, y, 6, POWER_RELIC);
    framebuffer.draw_line(x, y - 5, x + 5, y, glow);
    framebuffer.draw_line(x + 5, y, x, y + 5, glow);
    framebuffer.draw_line(x, y + 5, x - 5, y, glow);
    framebuffer.draw_line(x - 5, y, x, y - 5, glow);
    framebuffer.fill_rect(x - 1, y - 3, 3, 7, POWER_RELIC_LIGHT);
    framebuffer.fill_rect(x - 3, y - 1, 7, 3, POWER_RELIC_LIGHT);
}

pub fn run_v07() -> Result<(), EngineError> {
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
            VoidCanticleV07::new(),
            PauseConfig::new(Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            }),
        ),
    )
}

#[cfg(test)]
mod v07_tests {
    use super::*;

    #[test]
    fn weapon_progression_gets_faster_and_stronger() {
        let mut previous = weapon_profile(START_POWER_LEVEL);
        for level in (START_POWER_LEVEL + 1)..=MAX_POWER_LEVEL {
            let current = weapon_profile(level);
            assert!(current.period <= previous.period);
            assert!(current.radius >= previous.radius);
            assert!(current.volley_damage >= previous.volley_damage);
            previous = current;
        }
    }

    #[test]
    fn authored_relic_schedule_reaches_max_power_before_boss() {
        let drops = (1..=10).filter(|kill| should_drop_relic(*kill)).count() as u8;
        assert_eq!(START_POWER_LEVEL + drops, MAX_POWER_LEVEL);
    }

    #[test]
    fn death_loses_one_power_but_never_below_start() {
        assert_eq!(power_after_death(5), 4);
        assert_eq!(power_after_death(2), 1);
        assert_eq!(power_after_death(1), 1);
    }

    #[test]
    fn maximum_volley_matches_profile_damage() {
        let mut shots = Vec::new();
        spawn_power_volley(&mut shots, MAX_POWER_LEVEL, 90.0, 250.0);
        assert_eq!(shots.len(), 5);
        assert_eq!(
            shots.iter().map(|shot| shot.damage).sum::<u32>(),
            weapon_profile(MAX_POWER_LEVEL).volley_damage
        );
    }
}
