//! Validated digital binding configuration, independent of input capture and UI.
//!
//! The consumer declares context exclusivity and resolves device assignments.
//! Analog axes remain consumer-owned and are not edited by this configuration.
use super::{ActionId, ControlBinding, ControlMap};
use crate::{GamepadAxis, GamepadButton, GamepadId};

/// Contexts overlap unless explicitly declared mutually exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlContext(pub &'static str);

/// A named action and its consumer-owned default slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlAction {
    pub id: ActionId,
    pub label: &'static str,
    pub context: ControlContext,
    pub slots: Vec<BindingSlot>,
}

/// A fixed slot preserves its default; editable slots retain their source family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingSlot {
    pub default: ControlBinding,
    pub editable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationError {
    DuplicateAction(ActionId),
    UnknownAction(ActionId),
    UnknownSlot(ActionId, usize),
    FixedSlot(ActionId, usize),
    WrongSource(ActionId, usize),
    Reserved(ControlBinding),
    Conflict(ActionId, ActionId, ControlBinding, ControlBinding),
    InvalidExclusivity(ControlContext, ControlContext),
    InvalidAxis(ActionId),
    AxisConflict(ActionId, ActionId, GamepadAxis, AxisDirection),
}

/// A signed half-axis owned and evaluated by the consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisDirection {
    Negative,
    Positive,
}

/// Fixed analog source used only for conflict validation.
/// None means any device; thresholds use normalized axis coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedAxisBinding {
    pub action: ActionId,
    pub device: Option<GamepadId>,
    pub axis: GamepadAxis,
    pub direction: AxisDirection,
    pub threshold: f32,
}

/// Runtime device IDs in these bindings are session-local, never persistence IDs.
#[derive(Debug, Clone)]
pub struct ControlConfiguration {
    actions: Vec<ControlAction>,
    active: Vec<Vec<ControlBinding>>,
    exclusive: Vec<(ControlContext, ControlContext)>,
    reserved: Vec<ControlBinding>,
    fixed_axes: Vec<FixedAxisBinding>,
}

impl ControlConfiguration {
    pub fn new(
        actions: Vec<ControlAction>,
        exclusive: Vec<(ControlContext, ControlContext)>,
        reserved: Vec<ControlBinding>,
    ) -> Result<Self, ConfigurationError> {
        Self::with_fixed_axes(actions, exclusive, reserved, Vec::new())
    }

    pub fn with_fixed_axes(
        actions: Vec<ControlAction>,
        exclusive: Vec<(ControlContext, ControlContext)>,
        reserved: Vec<ControlBinding>,
        fixed_axes: Vec<FixedAxisBinding>,
    ) -> Result<Self, ConfigurationError> {
        for axis in &fixed_axes {
            if !actions.iter().any(|action| action.id == axis.action) {
                return Err(ConfigurationError::UnknownAction(axis.action));
            }
            if !axis.threshold.is_finite()
                || axis.threshold <= 0.0
                || axis.threshold > 1.0
                || (matches!(axis.axis, GamepadAxis::LeftTrigger | GamepadAxis::RightTrigger)
                    && axis.direction == AxisDirection::Negative)
            {
                return Err(ConfigurationError::InvalidAxis(axis.action));
            }
        }
        for (index, action) in actions.iter().enumerate() {
            if actions[..index].iter().any(|other| other.id == action.id) {
                return Err(ConfigurationError::DuplicateAction(action.id));
            }
        }
        for &(left, right) in &exclusive {
            if left == right
                || !actions.iter().any(|action| action.context == left)
                || !actions.iter().any(|action| action.context == right)
            {
                return Err(ConfigurationError::InvalidExclusivity(left, right));
            }
        }
        let active = actions
            .iter()
            .map(|action| action.slots.iter().map(|slot| slot.default).collect())
            .collect();
        let config = Self {
            actions,
            active,
            exclusive,
            reserved,
            fixed_axes,
        };
        config.validate(&config.active)?;
        Ok(config)
    }

    pub fn fixed_axes(&self) -> &[FixedAxisBinding] {
        &self.fixed_axes
    }

    pub fn actions(&self) -> &[ControlAction] {
        &self.actions
    }

    pub fn bindings(&self, action: ActionId) -> Option<&[ControlBinding]> {
        self.index(action)
            .ok()
            .map(|index| self.active[index].as_slice())
    }

    fn index(&self, action: ActionId) -> Result<usize, ConfigurationError> {
        self.actions
            .iter()
            .position(|entry| entry.id == action)
            .ok_or(ConfigurationError::UnknownAction(action))
    }

    /// Resolve a persisted action name against declared static IDs.
    pub fn resolve_action(&self, name: &str) -> Option<ActionId> {
        self.actions
            .iter()
            .find(|action| action.id.as_str() == name)
            .map(|action| action.id)
    }

    /// Validate all edits together, then commit once. No change on error.
    /// Repeated edits to a slot are evaluated in order, last value wins.
    pub fn replace(
        &mut self,
        edits: &[(ActionId, usize, ControlBinding)],
    ) -> Result<(), ConfigurationError> {
        let mut candidate = self.active.clone();
        for &(action, slot, binding) in edits {
            let index = self.index(action)?;
            let spec = self.actions[index]
                .slots
                .get(slot)
                .ok_or(ConfigurationError::UnknownSlot(action, slot))?;
            if !spec.editable && binding != spec.default {
                return Err(ConfigurationError::FixedSlot(action, slot));
            }
            if !same_source(spec.default, binding) {
                return Err(ConfigurationError::WrongSource(action, slot));
            }
            candidate[index][slot] = binding;
        }
        self.validate(&candidate)?;
        self.active = candidate;
        Ok(())
    }

    /// Restore the validated defaults without touching runtime input states.
    pub fn restore_defaults(&mut self) {
        self.active = self
            .actions
            .iter()
            .map(|action| action.slots.iter().map(|slot| slot.default).collect())
            .collect();
    }

    /// Replace only declared actions' physical bindings in an existing map.
    ///
    /// Call at a consumer-controlled neutral boundary, not during gameplay.
    /// Runtime states and virtual sources are deliberately unchanged.
    pub fn apply_to(&self, map: &mut ControlMap) {
        for (action, bindings) in self.actions.iter().zip(&self.active) {
            map.clear_bindings(action.id);
            for &binding in bindings {
                map.bind(action.id, binding);
            }
        }
    }

    fn validate(&self, candidate: &[Vec<ControlBinding>]) -> Result<(), ConfigurationError> {
        for (index, action) in self.actions.iter().enumerate() {
            for (slot, &binding) in action.slots.iter().zip(&candidate[index]) {
                // Fixed declarations may explicitly own a reserved control.
                if slot.editable && self.reserved.iter().any(|&r| overlaps(r, binding)) {
                    return Err(ConfigurationError::Reserved(binding));
                }
            }
            for (other_index, other) in self.actions.iter().enumerate().skip(index + 1) {
                if self.exclusive.iter().any(|&(a, b)| {
                    (a == action.context && b == other.context)
                        || (b == action.context && a == other.context)
                }) {
                    continue;
                }
                for &left in &candidate[index] {
                    for &right in &candidate[other_index] {
                        if overlaps(left, right) {
                            return Err(ConfigurationError::Conflict(
                                action.id, other.id, left, right,
                            ));
                        }
                    }
                }
            }
        }
        for (index, axis) in self.fixed_axes.iter().enumerate() {
            let owner = self.index(axis.action)?;
            for (other_index, other) in self.actions.iter().enumerate() {
                if owner == other_index
                    || self.contexts_exclusive(self.actions[owner].context, other.context)
                {
                    continue;
                }
                for &binding in &candidate[other_index] {
                    if let Some((device, button)) = pad_source(binding)
                        && devices_overlap(axis.device, device)
                        && derived_axis(button) == Some((axis.axis, axis.direction))
                    {
                        return Err(ConfigurationError::AxisConflict(
                            axis.action, other.id, axis.axis, axis.direction,
                        ));
                    }
                }
            }
            for other in &self.fixed_axes[index + 1..] {
                if axis.action != other.action
                    && axis.axis == other.axis
                    && axis.direction == other.direction
                    && devices_overlap(axis.device, other.device)
                    && !self.contexts_exclusive(
                        self.actions[owner].context,
                        self.actions[self.index(other.action)?].context,
                    )
                {
                    return Err(ConfigurationError::AxisConflict(
                        axis.action, other.action, axis.axis, axis.direction,
                    ));
                }
            }
        }
        Ok(())
    }

    fn contexts_exclusive(&self, left: ControlContext, right: ControlContext) -> bool {
        self.exclusive.iter().any(|&(a, b)| {
            (a == left && b == right) || (b == left && a == right)
        })
    }
}

fn devices_overlap(a: Option<GamepadId>, b: Option<GamepadId>) -> bool {
    a.is_none() || b.is_none() || a == b
}

fn derived_axis(button: GamepadButton) -> Option<(GamepadAxis, AxisDirection)> {
    use AxisDirection::{Negative, Positive};
    use GamepadAxis as Axis;
    use GamepadButton as Button;
    Some(match button {
        Button::LeftStickLeft => (Axis::LeftStickX, Negative),
        Button::LeftStickRight => (Axis::LeftStickX, Positive),
        Button::LeftStickDown => (Axis::LeftStickY, Negative),
        Button::LeftStickUp => (Axis::LeftStickY, Positive),
        Button::RightStickLeft => (Axis::RightStickX, Negative),
        Button::RightStickRight => (Axis::RightStickX, Positive),
        Button::RightStickDown => (Axis::RightStickY, Negative),
        Button::RightStickUp => (Axis::RightStickY, Positive),
        Button::LeftTrigger => (Axis::LeftTrigger, Positive),
        Button::RightTrigger => (Axis::RightTrigger, Positive),
        _ => return None,
    })
}

fn same_source(default: ControlBinding, candidate: ControlBinding) -> bool {
    match (default, candidate) {
        (ControlBinding::Key(_), ControlBinding::Key(_))
        | (ControlBinding::Gamepad(_), ControlBinding::Gamepad(_)) => true,
        (ControlBinding::GamepadDevice(a, _), ControlBinding::GamepadDevice(b, _)) => a == b,
        _ => false,
    }
}

fn pad_source(binding: ControlBinding) -> Option<(Option<GamepadId>, GamepadButton)> {
    match binding {
        ControlBinding::Key(_) => None,
        ControlBinding::Gamepad(button) => Some((None, button)),
        ControlBinding::GamepadDevice(id, button) => Some((Some(id), button)),
    }
}

fn overlaps(left: ControlBinding, right: ControlBinding) -> bool {
    if let (ControlBinding::Key(a), ControlBinding::Key(b)) = (left, right) {
        return a == b;
    }
    match (pad_source(left), pad_source(right)) {
        (Some((a, button_a)), Some((b, button_b))) => {
            button_a == button_b && (devices_overlap(a, b))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
