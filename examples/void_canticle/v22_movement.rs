const VC22_BULWARK_ACCEL: f32 = 340.0;
const VC22_BULWARK_BRAKE: f32 = 420.0;
const VC22_PILGRIM_ACCEL: f32 = 760.0;
const VC22_PILGRIM_BRAKE: f32 = 1_020.0;
const VC22_WRAITH_ACCEL: f32 = 1_800.0;
const VC22_WRAITH_BRAKE: f32 = 2_350.0;
const VC22_FOCUS_RESPONSE_MULTIPLIER: f32 = 2.0;

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
    movement_controls: ControlMap,
    velocity_x: f32,
    velocity_y: f32,
}

impl VoidCanticleV22Movement {
    fn new() -> Self {
        let mut game = VoidCanticleV22::new();

        // VC2.2 owns movement feel. Keep the historical movement code intact,
        // but detach its four physical direction bindings so it cannot apply
        // a second instantaneous displacement after the chassis velocity has
        // already been integrated.
        {
            let controls = &mut game.game.game.game.game.base_mut().controls;
            controls
                .clear_bindings(MOVE_LEFT)
                .clear_bindings(MOVE_RIGHT)
                .clear_bindings(MOVE_UP)
                .clear_bindings(MOVE_DOWN);
        }

        let mut movement_controls = ControlMap::new();
        movement_controls
            .bind_key(MOVE_LEFT, Key::Left)
            .bind_key(MOVE_LEFT, Key::A)
            .bind_gamepad(MOVE_LEFT, GamepadButton::DPadLeft)
            .bind_gamepad(MOVE_LEFT, GamepadButton::LeftStickLeft)
            .bind_key(MOVE_RIGHT, Key::Right)
            .bind_key(MOVE_RIGHT, Key::D)
            .bind_gamepad(MOVE_RIGHT, GamepadButton::DPadRight)
            .bind_gamepad(MOVE_RIGHT, GamepadButton::LeftStickRight)
            .bind_key(MOVE_UP, Key::Up)
            .bind_key(MOVE_UP, Key::W)
            .bind_gamepad(MOVE_UP, GamepadButton::DPadUp)
            .bind_gamepad(MOVE_UP, GamepadButton::LeftStickUp)
            .bind_key(MOVE_DOWN, Key::Down)
            .bind_key(MOVE_DOWN, Key::S)
            .bind_gamepad(MOVE_DOWN, GamepadButton::DPadDown)
            .bind_gamepad(MOVE_DOWN, GamepadButton::LeftStickDown)
            .bind_key(FOCUS, Key::LeftShift)
            .bind_gamepad(FOCUS, GamepadButton::LeftShoulder);

        Self {
            game,
            movement_controls,
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
        self.movement_controls.action(FOCUS).held()
            || frame.input.mouse_button(MouseButton::Right).held()
    }

    fn target_velocity(&self, chassis: ExosuitChassis, focused: bool) -> (f32, f32) {
        let left = self.movement_controls.action(MOVE_LEFT).held();
        let right = self.movement_controls.action(MOVE_RIGHT).held();
        let up = self.movement_controls.action(MOVE_UP).held();
        let down = self.movement_controls.action(MOVE_DOWN).held();

        let mut dx = (right as i8 - left as i8) as f32;
        let mut dy = (down as i8 - up as i8) as f32;
        if dx != 0.0 && dy != 0.0 {
            const INV_SQRT_2: f32 = 0.707_106_77;
            dx *= INV_SQRT_2;
            dy *= INV_SQRT_2;
        }

        let profile = chassis.profile();
        let speed = if focused {
            FOCUS_SPEED * profile.focus_multiplier
        } else {
            PLAYER_SPEED * profile.move_multiplier
        };
        (dx * speed, dy * speed)
    }

    fn integrate_chassis_movement(&mut self, dt: f32, frame: &Frame<'_>) {
        self.movement_controls.update(frame.input);

        if !self.game.gameplay_running() || dt <= f32::EPSILON {
            self.reset_velocity();
            return;
        }
        let Some(chassis) = self.game.chassis else {
            self.reset_velocity();
            return;
        };

        let focused = self.focused(frame);
        let target = self.target_velocity(chassis, focused);
        let moving = target.0.abs() > 0.01 || target.1.abs() > 0.01;
        let profile = chassis.movement_profile();
        let mut response = if moving {
            profile.acceleration
        } else {
            profile.braking
        };
        if focused {
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
        let unclamped_x = base.player_x + velocity.0 * dt;
        let unclamped_y = base.player_y + velocity.1 * dt;
        let next_x = unclamped_x.clamp(8.0, FRAMEBUFFER_WIDTH as f32 - 8.0);
        let next_y = unclamped_y.clamp(30.0, FRAMEBUFFER_HEIGHT as f32 - 16.0);
        base.player_x = next_x;
        base.player_y = next_y;

        if (next_x - unclamped_x).abs() > f32::EPSILON {
            self.velocity_x = 0.0;
        }
        if (next_y - unclamped_y).abs() > f32::EPSILON {
            self.velocity_y = 0.0;
        }
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
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        self.integrate_chassis_movement(dt, frame);

        let result = self.game.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if !self.game.gameplay_running() {
            self.reset_velocity();
        }
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

    #[test]
    fn bulwark_takes_longer_to_reach_cruise_speed_than_wraith() {
        let bulwark = ExosuitChassis::Bulwark;
        let wraith = ExosuitChassis::Wraith;
        let bulwark_target = PLAYER_SPEED * bulwark.profile().move_multiplier;
        let wraith_target = PLAYER_SPEED * wraith.profile().move_multiplier;
        let bulwark_time = bulwark_target / bulwark.movement_profile().acceleration;
        let wraith_time = wraith_target / wraith.movement_profile().acceleration;
        assert!(bulwark_time > wraith_time * 2.0);
    }
}
