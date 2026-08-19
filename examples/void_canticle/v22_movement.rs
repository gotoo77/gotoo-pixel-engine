const VC22_BULWARK_ACCEL: f32 = 340.0;
const VC22_BULWARK_BRAKE: f32 = 420.0;
const VC22_PILGRIM_ACCEL: f32 = 760.0;
const VC22_PILGRIM_BRAKE: f32 = 1_020.0;
const VC22_WRAITH_ACCEL: f32 = 1_800.0;
const VC22_WRAITH_BRAKE: f32 = 2_350.0;
const VC22_FOCUS_RESPONSE_MULTIPLIER: f32 = 2.0;
const VC22_TELEPORT_THRESHOLD: f32 = 14.0;

#[derive(Debug, Clone, Copy)]
struct ChassisMovementProfile {
    acceleration: f32,
    braking: f32,
}

impl ExosuitChassis {
    fn movement_profile(self) -> ChassisMovementProfile {
        match self {
            Self::Bulwark => ChassisMovementProfile {
                acceleration: VC22_BULWARK_ACCEL,
                braking: VC22_BULWARK_BRAKE,
            },
            Self::Pilgrim => ChassisMovementProfile {
                acceleration: VC22_PILGRIM_ACCEL,
                braking: VC22_PILGRIM_BRAKE,
            },
            Self::Wraith => ChassisMovementProfile {
                acceleration: VC22_WRAITH_ACCEL,
                braking: VC22_WRAITH_BRAKE,
            },
        }
    }
}

struct VoidCanticleV22Movement {
    game: VoidCanticleV22,
    velocity_x: f32,
    velocity_y: f32,
}

impl VoidCanticleV22Movement {
    fn new() -> Self {
        Self {
            game: VoidCanticleV22::new(),
            velocity_x: 0.0,
            velocity_y: 0.0,
        }
    }

    fn reset_velocity(&mut self) {
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
    }

    fn base(&self) -> &VoidCanticleGame {
        self.game.game.game.game.game.base()
    }

    fn base_mut(&mut self) -> &mut VoidCanticleGame {
        self.game.game.game.game.game.base_mut()
    }

    fn focused(&self, frame: &Frame<'_>) -> bool {
        self.base().controls.action(FOCUS).held()
            || frame.input.mouse_button(MouseButton::Right).held()
    }

    fn apply_movement_dynamics(
        &mut self,
        before: (f32, f32),
        after: (f32, f32),
        dt: f32,
        frame: &Frame<'_>,
    ) {
        if !self.game.gameplay_running() || dt <= f32::EPSILON {
            self.reset_velocity();
            return;
        }
        let Some(chassis) = self.game.chassis else {
            self.reset_velocity();
            return;
        };

        let raw_dx = after.0 - before.0;
        let raw_dy = after.1 - before.1;
        if raw_dx.abs() > VC22_TELEPORT_THRESHOLD || raw_dy.abs() > VC22_TELEPORT_THRESHOLD {
            // Run restarts and other state transitions may reposition the
            // player instantly. They are not movement input and must not seed
            // inertia for the next frame.
            self.reset_velocity();
            return;
        }

        let target = (raw_dx / dt, raw_dy / dt);
        let moving = target.0.abs() > 0.01 || target.1.abs() > 0.01;
        let profile = chassis.movement_profile();
        let mut response = if moving {
            profile.acceleration
        } else {
            profile.braking
        };
        if self.focused(frame) {
            response *= VC22_FOCUS_RESPONSE_MULTIPLIER;
        }

        let velocity = vc22_move_towards_velocity(
            (self.velocity_x, self.velocity_y),
            target,
            response * dt,
        );
        self.velocity_x = velocity.0;
        self.velocity_y = velocity.1;

        let base = self.base_mut();
        base.player_x = (before.0 + velocity.0 * dt)
            .clamp(8.0, FRAMEBUFFER_WIDTH as f32 - 8.0);
        base.player_y = (before.1 + velocity.1 * dt)
            .clamp(30.0, FRAMEBUFFER_HEIGHT as f32 - 16.0);
    }

    fn render_motion_signature(&self, framebuffer: &mut Framebuffer) {
        if !self.game.gameplay_running() {
            return;
        }
        let Some(chassis) = self.game.chassis else {
            return;
        };

        let speed = (self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y).sqrt();
        if speed < 8.0 {
            return;
        }

        let base = self.base();
        let x = base.player_x.round() as i32;
        let y = base.player_y.round() as i32;
        let trail = match chassis {
            ExosuitChassis::Bulwark => 3,
            ExosuitChassis::Pilgrim => 5,
            ExosuitChassis::Wraith => 8,
        };
        let color = match chassis {
            ExosuitChassis::Bulwark => ART_GOLD,
            ExosuitChassis::Pilgrim => PILGRIM_THRUSTER,
            ExosuitChassis::Wraith => ART_CYAN_LIGHT,
        };

        let length = (trail as f32 * (speed / 120.0).clamp(0.55, 1.35)).round() as i32;
        if length > 0 {
            framebuffer.draw_line(x - 3, y + 10, x - 3, y + 10 + length, color);
            framebuffer.draw_line(x + 3, y + 10, x + 3, y + 10 + length, color);
        }
    }
}

impl Game for VoidCanticleV22Movement {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let before = self.game.player_position();
        let was_running = self.game.gameplay_running();
        let dt = frame.delta_time.as_secs_f32().min(0.05);

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let running = self.game.gameplay_running();
        if !was_running || !running {
            self.reset_velocity();
            return GameResult::Continue;
        }

        let after = self.game.player_position();
        self.apply_movement_dynamics(before, after, dt, frame);
        self.render_motion_signature(frame.framebuffer);
        GameResult::Continue
    }
}

fn vc22_move_towards_velocity(
    current: (f32, f32),
    target: (f32, f32),
    max_delta: f32,
) -> (f32, f32) {
    let dx = target.0 - current.0;
    let dy = target.1 - current.1;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance <= max_delta || distance <= f32::EPSILON {
        return target;
    }
    let scale = max_delta / distance;
    (current.0 + dx * scale, current.1 + dy * scale)
}

pub fn run_v22_movement_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: format!("Void Canticle {VC22_VERSION} - Gotoo Pixel Engine"),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV22Movement::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v22_movement_tests {
    use super::*;

    #[test]
    fn chassis_response_matches_mass_tradeoff() {
        let bulwark = ExosuitChassis::Bulwark.movement_profile();
        let pilgrim = ExosuitChassis::Pilgrim.movement_profile();
        let wraith = ExosuitChassis::Wraith.movement_profile();

        assert!(bulwark.acceleration < pilgrim.acceleration);
        assert!(pilgrim.acceleration < wraith.acceleration);
        assert!(bulwark.braking < pilgrim.braking);
        assert!(pilgrim.braking < wraith.braking);
    }

    #[test]
    fn velocity_approach_is_bounded() {
        let velocity = vc22_move_towards_velocity((0.0, 0.0), (100.0, 0.0), 25.0);
        assert!((velocity.0 - 25.0).abs() < 0.001);
        assert!(velocity.1.abs() < 0.001);
    }

    #[test]
    fn velocity_approach_snaps_when_target_is_close() {
        let velocity = vc22_move_towards_velocity((90.0, 5.0), (100.0, 0.0), 20.0);
        assert_eq!(velocity, (100.0, 0.0));
    }

    #[test]
    fn focus_increases_control_response() {
        assert!(VC22_FOCUS_RESPONSE_MULTIPLIER > 1.0);
    }
}
