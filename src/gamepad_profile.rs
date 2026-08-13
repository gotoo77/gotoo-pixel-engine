#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCalibration {
    raw_negative: f32,
    raw_center: f32,
    raw_positive: f32,
    dead_zone: f32,
}

impl AxisCalibration {
    pub const fn new(
        raw_negative: f32,
        raw_center: f32,
        raw_positive: f32,
        dead_zone: f32,
    ) -> Self {
        Self {
            raw_negative,
            raw_center,
            raw_positive,
            dead_zone,
        }
    }

    pub const fn standard(dead_zone: f32) -> Self {
        Self::new(-1.0, 0.0, 1.0, dead_zone)
    }

    pub const fn raw_negative(self) -> f32 {
        self.raw_negative
    }

    pub const fn raw_center(self) -> f32 {
        self.raw_center
    }

    pub const fn raw_positive(self) -> f32 {
        self.raw_positive
    }

    pub const fn dead_zone(self) -> f32 {
        self.dead_zone
    }

    pub fn normalize(self, raw: f32) -> f32 {
        let center = self.raw_center;
        let positive_span = self.raw_positive - center;
        let negative_span = self.raw_negative - center;
        let delta = raw - center;

        let normalized = if same_direction(delta, positive_span) && positive_span.abs() > f32::EPSILON
        {
            delta / positive_span
        } else if same_direction(delta, negative_span) && negative_span.abs() > f32::EPSILON {
            -(delta / negative_span)
        } else {
            0.0
        }
        .clamp(-1.0, 1.0);

        if normalized.abs() <= self.dead_zone.clamp(0.0, 0.95) {
            0.0
        } else {
            normalized
        }
    }

    pub fn inverted(self) -> Self {
        Self {
            raw_negative: self.raw_positive,
            raw_positive: self.raw_negative,
            ..self
        }
    }
}

fn same_direction(value: f32, reference: f32) -> bool {
    value == 0.0 || value.signum() == reference.signum()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamepadProfile {
    pub left_stick_x: AxisCalibration,
    pub left_stick_y: AxisCalibration,
    pub dpad_x: AxisCalibration,
    pub dpad_y: AxisCalibration,
    pub digital_threshold: f32,
}

impl GamepadProfile {
    pub const fn standard() -> Self {
        Self {
            left_stick_x: AxisCalibration::standard(0.20),
            left_stick_y: AxisCalibration::standard(0.20),
            dpad_x: AxisCalibration::standard(0.10),
            dpad_y: AxisCalibration::standard(0.10),
            digital_threshold: 0.50,
        }
    }

    pub fn with_left_stick_x(mut self, calibration: AxisCalibration) -> Self {
        self.left_stick_x = calibration;
        self
    }

    pub fn with_left_stick_y(mut self, calibration: AxisCalibration) -> Self {
        self.left_stick_y = calibration;
        self
    }

    pub fn with_dpad_x(mut self, calibration: AxisCalibration) -> Self {
        self.dpad_x = calibration;
        self
    }

    pub fn with_dpad_y(mut self, calibration: AxisCalibration) -> Self {
        self.dpad_y = calibration;
        self
    }

    pub fn with_digital_threshold(mut self, threshold: f32) -> Self {
        self.digital_threshold = threshold.clamp(0.05, 0.95);
        self
    }
}

impl Default for GamepadProfile {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::{AxisCalibration, GamepadProfile};

    #[test]
    fn standard_axis_normalizes_both_directions() {
        let axis = AxisCalibration::standard(0.10);

        assert_eq!(axis.normalize(-1.0), -1.0);
        assert_eq!(axis.normalize(0.0), 0.0);
        assert_eq!(axis.normalize(1.0), 1.0);
        assert!(axis.normalize(0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn inverted_axis_swaps_logical_polarity() {
        let axis = AxisCalibration::standard(0.0).inverted();

        assert_eq!(axis.normalize(-1.0), 1.0);
        assert_eq!(axis.normalize(1.0), -1.0);
    }

    #[test]
    fn asymmetric_center_is_supported() {
        let axis = AxisCalibration::new(0.0, 0.431, 1.0, 0.10);

        assert_eq!(axis.normalize(0.431), 0.0);
        assert_eq!(axis.normalize(0.0), -1.0);
        assert_eq!(axis.normalize(1.0), 1.0);
    }

    #[test]
    fn inverted_asymmetric_axis_supports_linux_hat_y_polarity() {
        let axis = AxisCalibration::new(1.0, 0.431, 0.0, 0.10);

        assert_eq!(axis.normalize(0.0), 1.0);
        assert_eq!(axis.normalize(1.0), -1.0);
    }

    #[test]
    fn gamepad_profile_can_override_one_axis_without_touching_others() {
        let standard = GamepadProfile::standard();
        let inverted_y = standard.left_stick_y.inverted();
        let profile = standard.with_left_stick_y(inverted_y);

        assert_eq!(profile.left_stick_x, standard.left_stick_x);
        assert_eq!(profile.left_stick_y, inverted_y);
    }
}
