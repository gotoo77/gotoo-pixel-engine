use std::collections::{HashMap, HashSet};

use crate::Touch;

pub(crate) use super::experimental::{UiId, UiNavInput};

/// Pointer snapshot consumed by the frame-local UI interaction pass.
///
/// This is deliberately semantic at the UI boundary: platform event handling
/// remains owned by GPE and callers decide how physical devices map into the
/// snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiPointerInput {
    pub position: Option<(i32, i32)>,
    pub pressed: bool,
    pub released: bool,
}

/// Canonical frame-local input snapshot for the productionized GPE.UI kernel.
///
/// `UiNavInput` carries semantic navigation while pointer/touch data carries
/// spatial interaction facts. Existing MFE APIs may temporarily expose aliases
/// to this type while the two experimental runtimes converge.
#[derive(Debug, Clone, Copy, Default)]
pub struct UiInput<'a> {
    pub nav: UiNavInput,
    pub pointer: UiPointerInput,
    pub touches: &'a [Touch],
}

/// Shared persistent interaction state for one UI transaction surface.
///
/// This is an internal productionization primitive, not a stabilized public
/// API. It centralizes identity-keyed focus and capture semantics so the
/// transactional and spatial MFE paths can converge without keeping two
/// independent interaction authorities.
#[derive(Debug, Default)]
pub(crate) struct UiInteractionState {
    focused: Option<UiId>,
    pointer_capture: Option<UiId>,
    touch_capture: HashMap<u64, UiId>,
    previous_order: Vec<UiId>,
}

impl UiInteractionState {
    pub(crate) const fn focused_id(&self) -> Option<UiId> {
        self.focused
    }

    pub(crate) fn set_focused_id(&mut self, focused: Option<UiId>) {
        self.focused = focused;
    }

    pub(crate) const fn pointer_capture_id(&self) -> Option<UiId> {
        self.pointer_capture
    }

    pub(crate) fn set_pointer_capture(&mut self, captured: Option<UiId>) {
        self.pointer_capture = captured;
    }

    pub(crate) fn touch_capture_count(&self) -> usize {
        self.touch_capture.len()
    }

    pub(crate) fn touch_capture_id(&self, touch_id: u64) -> Option<UiId> {
        self.touch_capture.get(&touch_id).copied()
    }

    pub(crate) fn set_touch_capture(&mut self, touch_id: u64, target: UiId) {
        self.touch_capture.insert(touch_id, target);
    }

    pub(crate) fn remove_touch_capture(&mut self, touch_id: u64) -> Option<UiId> {
        self.touch_capture.remove(&touch_id)
    }

    pub(crate) fn clear_touch_capture(&mut self, touch_id: u64) {
        self.touch_capture.remove(&touch_id);
    }

    pub(crate) fn is_active(&self, id: UiId) -> bool {
        self.pointer_capture == Some(id) || self.touch_capture.values().any(|target| *target == id)
    }

    pub(crate) fn touch_captures(&self) -> &HashMap<u64, UiId> {
        &self.touch_capture
    }

    /// Repairs focus and captures against the targets present in the current
    /// transaction while preserving the deterministic fallback behavior proven
    /// by MFE-001B.
    pub(crate) fn repair_for_current_order(&mut self, current_order: &[UiId]) {
        if current_order.is_empty() {
            self.focused = None;
            self.pointer_capture = None;
            self.touch_capture.clear();
            self.previous_order.clear();
            return;
        }

        let current = current_order.iter().copied().collect::<HashSet<_>>();
        self.pointer_capture = self.pointer_capture.filter(|id| current.contains(id));
        self.touch_capture.retain(|_, id| current.contains(id));

        if self.focused.is_some_and(|id| current.contains(&id)) {
            return;
        }

        let fallback_index = self
            .focused
            .and_then(|id| {
                self.previous_order
                    .iter()
                    .position(|previous| *previous == id)
            })
            .unwrap_or(0)
            .min(current_order.len() - 1);
        self.focused = Some(current_order[fallback_index]);
    }

    pub(crate) fn commit_order(&mut self, current_order: &[UiId]) {
        self.previous_order.clear();
        self.previous_order.extend_from_slice(current_order);
    }
}

#[cfg(test)]
mod tests {
    use crate::{ActionId, Size};

    use super::*;
    use crate::ui::{
        UiTheme,
        experimental::{UiStateStore, run_headless},
    };

    const ACTION: ActionId = ActionId::new("kernel.interaction.test");

    fn ids() -> Vec<UiId> {
        let mut state = UiStateStore::default();
        let (_, ids) = run_headless(
            Size {
                width: 320,
                height: 200,
            },
            &mut state,
            UiNavInput::default(),
            UiTheme::default(),
            |ui| {
                ["A", "B", "C"]
                    .iter()
                    .map(|key| ui.keyed(key, |ui| ui.button_action(*key, ACTION).id()))
                    .collect::<Vec<_>>()
            },
        );
        ids
    }

    #[test]
    fn default_input_is_idle_and_touch_free() {
        let input = UiInput::default();

        assert_eq!(input.nav, UiNavInput::default());
        assert_eq!(input.pointer, UiPointerInput::default());
        assert!(input.touches.is_empty());
    }

    #[test]
    fn interaction_state_preserves_keyed_focus_across_reorder() {
        let ids = ids();
        let mut state = UiInteractionState::default();
        state.repair_for_current_order(&ids);
        state.set_focused_id(Some(ids[1]));
        state.commit_order(&ids);

        let reordered = [ids[2], ids[0], ids[1]];
        state.repair_for_current_order(&reordered);

        assert_eq!(state.focused_id(), Some(ids[1]));
    }

    #[test]
    fn interaction_state_repairs_removed_focus_by_previous_index() {
        let ids = ids();
        let mut state = UiInteractionState::default();
        state.repair_for_current_order(&ids);
        state.set_focused_id(Some(ids[1]));
        state.commit_order(&ids);

        let without_focused = [ids[0], ids[2]];
        state.repair_for_current_order(&without_focused);

        assert_eq!(state.focused_id(), Some(ids[2]));
    }

    #[test]
    fn interaction_state_prunes_disappearing_pointer_and_touch_capture() {
        let ids = ids();
        let mut state = UiInteractionState::default();
        state.repair_for_current_order(&ids);
        state.set_pointer_capture(Some(ids[1]));
        state.set_touch_capture(7, ids[1]);
        state.set_touch_capture(9, ids[2]);
        state.commit_order(&ids);

        let remaining = [ids[0], ids[2]];
        state.repair_for_current_order(&remaining);

        assert_eq!(state.pointer_capture_id(), None);
        assert_eq!(state.touch_capture_id(7), None);
        assert_eq!(state.touch_capture_id(9), Some(ids[2]));
        assert_eq!(state.touch_capture_count(), 1);
        assert!(state.is_active(ids[2]));
    }

    #[test]
    fn empty_target_set_clears_all_interaction_state() {
        let ids = ids();
        let mut state = UiInteractionState::default();
        state.repair_for_current_order(&ids);
        state.set_focused_id(Some(ids[0]));
        state.set_pointer_capture(Some(ids[0]));
        state.set_touch_capture(3, ids[0]);
        state.commit_order(&ids);

        state.repair_for_current_order(&[]);

        assert_eq!(state.focused_id(), None);
        assert_eq!(state.pointer_capture_id(), None);
        assert_eq!(state.touch_capture_count(), 0);
    }

    #[test]
    fn touch_capture_mutation_is_explicit_and_deterministic() {
        let ids = ids();
        let mut state = UiInteractionState::default();

        state.set_touch_capture(11, ids[0]);
        assert_eq!(state.touch_capture_id(11), Some(ids[0]));
        assert_eq!(state.remove_touch_capture(11), Some(ids[0]));
        assert_eq!(state.touch_capture_id(11), None);

        state.set_touch_capture(11, ids[1]);
        state.clear_touch_capture(11);
        assert_eq!(state.touch_capture_id(11), None);
        assert!(state.touch_captures().is_empty());
    }
}
