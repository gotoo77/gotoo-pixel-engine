use std::collections::{HashMap, HashSet};

use crate::{ActionId, Rect, Touch};

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

/// Frame-local resolved target contract consumed by generic hit-testing and
/// spatial navigation.
///
/// The target itself may be a widget/layout-specific record such as the
/// MFE-001B `CardLayout`; the interaction kernel only requires stable identity
/// and final integer geometry.
pub(crate) trait UiResolvedTarget {
    fn ui_id(&self) -> UiId;
    fn ui_rect(&self) -> Rect;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiNavDirection {
    Up,
    Down,
    Left,
    Right,
}

impl UiNavDirection {
    pub(crate) fn from_nav(nav: UiNavInput) -> Option<Self> {
        match (nav.up, nav.down, nav.left, nav.right) {
            (true, false, false, false) => Some(Self::Up),
            (false, true, false, false) => Some(Self::Down),
            (false, false, true, false) => Some(Self::Left),
            (false, false, false, true) => Some(Self::Right),
            _ => None,
        }
    }
}

/// Returns the top-most resolved target at `position` using stable resolved
/// order as the z-order tiebreaker, matching the MFE-001B reverse hit-test.
pub(crate) fn hit_test_resolved<T: UiResolvedTarget>(
    position: (i32, i32),
    targets: &[T],
) -> Option<UiId> {
    targets
        .iter()
        .rev()
        .find(|target| rect_contains(target.ui_rect(), position))
        .map(UiResolvedTarget::ui_id)
}

/// Selects the deterministic spatial-navigation candidate proven by MFE-001B.
///
/// Ranking is intentionally unchanged:
///
/// 1. requested direction filter;
/// 2. primary-axis distance;
/// 3. secondary-axis distance;
/// 4. stable resolved order.
pub(crate) fn spatial_candidate<T: UiResolvedTarget>(
    focused: UiId,
    direction: UiNavDirection,
    targets: &[T],
) -> Option<UiId> {
    let focused_target = targets.iter().find(|target| target.ui_id() == focused)?;
    let (fx, fy) = rect_center(focused_target.ui_rect());

    targets
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.ui_id() != focused)
        .filter_map(|(index, candidate)| {
            let (cx, cy) = rect_center(candidate.ui_rect());
            let (primary, secondary) = match direction {
                UiNavDirection::Up if cy < fy => (fy - cy, (cx - fx).abs()),
                UiNavDirection::Down if cy > fy => (cy - fy, (cx - fx).abs()),
                UiNavDirection::Left if cx < fx => (fx - cx, (cy - fy).abs()),
                UiNavDirection::Right if cx > fx => (cx - fx, (cy - fy).abs()),
                _ => return None,
            };
            Some(((primary, secondary, index), candidate.ui_id()))
        })
        .min_by_key(|(score, _)| *score)
        .map(|(_, id)| id)
}

fn rect_contains(rect: Rect, position: (i32, i32)) -> bool {
    if rect.width == 0 || rect.height == 0 {
        return false;
    }

    let px = i64::from(position.0);
    let py = i64::from(position.1);
    let left = i64::from(rect.x);
    let top = i64::from(rect.y);
    let right = left + i64::from(rect.width);
    let bottom = top + i64::from(rect.height);
    px >= left && px < right && py >= top && py < bottom
}

fn rect_center(rect: Rect) -> (i64, i64) {
    (
        i64::from(rect.x) + i64::from(rect.width) / 2,
        i64::from(rect.y) + i64::from(rect.height) / 2,
    )
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

/// Canonical frame-local semantic interaction result.
///
/// Widget/layout adapters may attach their own geometry and debug data, but
/// focus, hover, activation, semantic actions and cancellation live here so
/// they cannot diverge into Card-specific or widget-specific authorities.
#[derive(Debug, Clone, Default)]
pub(crate) struct UiInteractionOutput {
    focused: Option<UiId>,
    hovered: Option<UiId>,
    activated: HashSet<UiId>,
    actions: Vec<ActionId>,
    cancelled: bool,
}

impl UiInteractionOutput {
    pub(crate) fn new(focused: Option<UiId>, hovered: Option<UiId>, cancelled: bool) -> Self {
        Self {
            focused,
            hovered,
            activated: HashSet::new(),
            actions: Vec::new(),
            cancelled,
        }
    }

    pub(crate) const fn focused_id(&self) -> Option<UiId> {
        self.focused
    }

    pub(crate) fn set_focused_id(&mut self, focused: Option<UiId>) {
        self.focused = focused;
    }

    pub(crate) const fn hovered_id(&self) -> Option<UiId> {
        self.hovered
    }

    pub(crate) fn activated(&self, id: UiId) -> bool {
        self.activated.contains(&id)
    }

    pub(crate) fn activated_ids(&self) -> &HashSet<UiId> {
        &self.activated
    }

    pub(crate) fn activate(&mut self, id: UiId, action: Option<ActionId>) {
        if !self.activated.insert(id) {
            return;
        }
        if let Some(action) = action {
            self.actions.push(action);
        }
    }

    pub(crate) fn action_pressed(&self, action: ActionId) -> bool {
        self.actions.contains(&action)
    }

    pub(crate) const fn cancel_requested(&self) -> bool {
        self.cancelled
    }

    pub(crate) fn activation_count(&self) -> usize {
        self.activated.len()
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

    #[derive(Debug, Clone, Copy)]
    struct TestTarget {
        id: UiId,
        rect: Rect,
    }

    impl UiResolvedTarget for TestTarget {
        fn ui_id(&self) -> UiId {
            self.id
        }

        fn ui_rect(&self) -> Rect {
            self.rect
        }
    }

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
    fn nav_direction_requires_exactly_one_spatial_direction() {
        assert_eq!(
            UiNavDirection::from_nav(UiNavInput {
                right: true,
                ..UiNavInput::default()
            }),
            Some(UiNavDirection::Right)
        );
        assert_eq!(
            UiNavDirection::from_nav(UiNavInput {
                right: true,
                down: true,
                ..UiNavInput::default()
            }),
            None
        );
    }

    #[test]
    fn resolved_hit_test_prefers_last_overlapping_target() {
        let ids = ids();
        let targets = [
            TestTarget {
                id: ids[0],
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 20,
                    height: 20,
                },
            },
            TestTarget {
                id: ids[1],
                rect: Rect {
                    x: 5,
                    y: 5,
                    width: 20,
                    height: 20,
                },
            },
        ];

        assert_eq!(hit_test_resolved((10, 10), &targets), Some(ids[1]));
        assert_eq!(hit_test_resolved((30, 30), &targets), None);
    }

    #[test]
    fn resolved_spatial_ranking_preserves_primary_secondary_then_order() {
        let ids = ids();
        let focused = TestTarget {
            id: ids[0],
            rect: Rect {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        };
        let farther_primary = TestTarget {
            id: ids[1],
            rect: Rect {
                x: 0,
                y: 30,
                width: 10,
                height: 10,
            },
        };
        let nearer_primary = TestTarget {
            id: ids[2],
            rect: Rect {
                x: 20,
                y: 20,
                width: 10,
                height: 10,
            },
        };
        let targets = [focused, farther_primary, nearer_primary];

        assert_eq!(
            spatial_candidate(ids[0], UiNavDirection::Down, &targets),
            Some(ids[2])
        );
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

    #[test]
    fn interaction_output_owns_semantic_result_without_duplicate_actions() {
        let ids = ids();
        let mut output = UiInteractionOutput::new(Some(ids[0]), Some(ids[1]), true);

        output.activate(ids[1], Some(ACTION));
        output.activate(ids[1], Some(ACTION));
        output.set_focused_id(Some(ids[1]));

        assert_eq!(output.focused_id(), Some(ids[1]));
        assert_eq!(output.hovered_id(), Some(ids[1]));
        assert!(output.activated(ids[1]));
        assert!(output.action_pressed(ACTION));
        assert!(output.cancel_requested());
        assert_eq!(output.activation_count(), 1);
        assert_eq!(output.activated_ids().len(), 1);
    }
}
