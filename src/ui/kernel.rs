use std::collections::{HashMap, HashSet};

use crate::{ActionId, Rect, Touch, TouchPhase};

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

/// Frame-local resolved target contract consumed by generic interaction.
///
/// Targets are transient resolved geometry, not retained widgets. Focusability
/// is represented by membership in the target slice. Activation is separate:
/// e.g. a Slider can accept focus without treating Confirm/click as activation.
pub(crate) trait UiResolvedTarget {
    fn ui_id(&self) -> UiId;
    fn ui_rect(&self) -> Rect;

    fn ui_accepts_activation(&self) -> bool {
        true
    }
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

/// Navigation policy applied by the shared interaction pass.
///
/// This is deliberately a policy choice over one focus authority rather than
/// a second state/runtime model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiNavigationPolicy {
    Linear,
    Spatial,
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

/// Runs one generic interaction transaction over already-resolved targets.
///
/// Layout/widget adapters provide the transient target geometry and semantic
/// `ActionId` lookup. The kernel owns focus repair, navigation, hover,
/// pointer/touch capture, activation, cancel and order finalization.
pub(crate) fn run_interaction_pass<T, F>(
    state: &mut UiInteractionState,
    input: UiInput<'_>,
    targets: &[T],
    navigation: UiNavigationPolicy,
    action_for: F,
) -> UiInteractionOutput
where
    T: UiResolvedTarget,
    F: Fn(UiId) -> Option<ActionId>,
{
    let current_order = targets
        .iter()
        .map(UiResolvedTarget::ui_id)
        .collect::<Vec<_>>();
    state.repair_for_current_order(&current_order);

    match navigation {
        UiNavigationPolicy::Linear => state.navigate_linear(input.nav, &current_order),
        UiNavigationPolicy::Spatial => {
            if let Some(direction) = UiNavDirection::from_nav(input.nav)
                && let Some(focused) = state.focused_id()
                && let Some(next) = spatial_candidate(focused, direction, targets)
            {
                state.set_focused_id(Some(next));
            }
        }
    }

    let hovered = input
        .pointer
        .position
        .and_then(|position| hit_test_resolved(position, targets));
    let mut output = UiInteractionOutput::new(None, hovered, input.nav.cancel);

    if input.pointer.pressed {
        state.set_pointer_capture(hovered);
        if hovered.is_some() {
            state.set_focused_id(hovered);
        }
    }

    if input.pointer.released {
        if let Some(captured) = state.pointer_capture_id()
            && hovered == Some(captured)
        {
            activate_target(captured, targets, &action_for, &mut output);
        }
        state.set_pointer_capture(None);
    }

    for touch in input.touches {
        match touch.phase {
            TouchPhase::Started => {
                if let Some(position) = touch.position
                    && let Some(target) = hit_test_resolved(position, targets)
                {
                    state.set_touch_capture(touch.id, target);
                    state.set_focused_id(Some(target));
                }
            }
            TouchPhase::Moved => {
                let Some(captured) = state.touch_capture_id(touch.id) else {
                    continue;
                };
                if !touch
                    .position
                    .and_then(|position| hit_test_resolved(position, targets))
                    .is_some_and(|target| target == captured)
                {
                    state.clear_touch_capture(touch.id);
                }
            }
            TouchPhase::Ended => {
                if let Some(captured) = state.remove_touch_capture(touch.id)
                    && touch
                        .position
                        .and_then(|position| hit_test_resolved(position, targets))
                        == Some(captured)
                {
                    activate_target(captured, targets, &action_for, &mut output);
                }
            }
            TouchPhase::Cancelled => {
                state.clear_touch_capture(touch.id);
            }
        }
    }

    if input.nav.confirm
        && let Some(focused) = state.focused_id()
    {
        activate_target(focused, targets, &action_for, &mut output);
    }

    state.commit_order(&current_order);
    output.set_focused_id(state.focused_id());
    output
}

fn activate_target<T, F>(
    id: UiId,
    targets: &[T],
    action_for: &F,
    output: &mut UiInteractionOutput,
) where
    T: UiResolvedTarget,
    F: Fn(UiId) -> Option<ActionId>,
{
    if targets
        .iter()
        .find(|target| target.ui_id() == id)
        .is_some_and(UiResolvedTarget::ui_accepts_activation)
    {
        output.activate(id, action_for(id));
    }
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
        self.repair_for_current_order_with(current_order, |_| false);
    }

    /// Transactional compatibility variant that treats IDs whose widget kind
    /// changed this generation as invalid for state preservation. Fallback
    /// selection intentionally still uses the previous stable index, matching
    /// MFE-001A behavior even when that resolves to the same ID.
    pub(crate) fn repair_for_current_order_excluding(
        &mut self,
        current_order: &[UiId],
        invalid: &HashSet<UiId>,
    ) {
        self.repair_for_current_order_with(current_order, |id| invalid.contains(&id));
    }

    fn repair_for_current_order_with(
        &mut self,
        current_order: &[UiId],
        is_invalid: impl Fn(UiId) -> bool,
    ) {
        if current_order.is_empty() {
            self.focused = None;
            self.pointer_capture = None;
            self.touch_capture.clear();
            self.previous_order.clear();
            return;
        }

        let current = current_order.iter().copied().collect::<HashSet<_>>();
        let is_current_valid = |id: UiId| current.contains(&id) && !is_invalid(id);
        self.pointer_capture = self.pointer_capture.filter(|id| is_current_valid(*id));
        self.touch_capture.retain(|_, id| is_current_valid(*id));

        if self.focused.is_some_and(is_current_valid) {
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

    pub(crate) fn navigate_linear(&mut self, nav: UiNavInput, current_order: &[UiId]) {
        if current_order.is_empty() {
            self.focused = None;
            return;
        }

        let current = self
            .focused
            .and_then(|focused| current_order.iter().position(|id| *id == focused))
            .unwrap_or(0);

        self.focused = match (nav.up, nav.down) {
            (true, false) => Some(
                current_order[if current == 0 {
                    current_order.len() - 1
                } else {
                    current - 1
                }],
            ),
            (false, true) => Some(current_order[(current + 1) % current_order.len()]),
            _ => Some(current_order[current]),
        };
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

    #[derive(Debug, Clone, Copy)]
    struct PassiveTarget(TestTarget);

    impl UiResolvedTarget for PassiveTarget {
        fn ui_id(&self) -> UiId {
            self.0.id
        }

        fn ui_rect(&self) -> Rect {
            self.0.rect
        }

        fn ui_accepts_activation(&self) -> bool {
            false
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

    fn row_targets(ids: &[UiId]) -> Vec<TestTarget> {
        ids.iter()
            .enumerate()
            .map(|(index, id)| TestTarget {
                id: *id,
                rect: Rect {
                    x: (index as i32) * 30,
                    y: 0,
                    width: 20,
                    height: 20,
                },
            })
            .collect()
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
    fn interaction_pass_linear_navigation_uses_shared_state() {
        let ids = ids();
        let targets = row_targets(&ids);
        let mut state = UiInteractionState::default();

        let output = run_interaction_pass(
            &mut state,
            UiInput {
                nav: UiNavInput {
                    down: true,
                    ..UiNavInput::default()
                },
                ..UiInput::default()
            },
            &targets,
            UiNavigationPolicy::Linear,
            |_| None,
        );

        assert_eq!(output.focused_id(), Some(ids[1]));
    }

    #[test]
    fn interaction_pass_pointer_release_activates_captured_target() {
        let ids = ids();
        let targets = row_targets(&ids);
        let mut state = UiInteractionState::default();

        let pressed = run_interaction_pass(
            &mut state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some((35, 5)),
                    pressed: true,
                    released: false,
                },
                ..UiInput::default()
            },
            &targets,
            UiNavigationPolicy::Linear,
            |_| Some(ACTION),
        );
        assert_eq!(pressed.focused_id(), Some(ids[1]));
        assert_eq!(state.pointer_capture_id(), Some(ids[1]));
        assert!(!pressed.activated(ids[1]));

        let released = run_interaction_pass(
            &mut state,
            UiInput {
                pointer: UiPointerInput {
                    position: Some((35, 5)),
                    pressed: false,
                    released: true,
                },
                ..UiInput::default()
            },
            &targets,
            UiNavigationPolicy::Linear,
            |_| Some(ACTION),
        );
        assert!(released.activated(ids[1]));
        assert!(released.action_pressed(ACTION));
        assert_eq!(state.pointer_capture_id(), None);
    }

    #[test]
    fn interaction_pass_touch_move_off_cancels_capture() {
        let ids = ids();
        let targets = row_targets(&ids);
        let mut state = UiInteractionState::default();
        let started = [Touch {
            id: 7,
            phase: TouchPhase::Started,
            position: Some((5, 5)),
        }];
        let moved = [Touch {
            id: 7,
            phase: TouchPhase::Moved,
            position: Some((200, 200)),
        }];

        let _ = run_interaction_pass(
            &mut state,
            UiInput {
                touches: &started,
                ..UiInput::default()
            },
            &targets,
            UiNavigationPolicy::Linear,
            |_| Some(ACTION),
        );
        assert_eq!(state.touch_capture_id(7), Some(ids[0]));

        let output = run_interaction_pass(
            &mut state,
            UiInput {
                touches: &moved,
                ..UiInput::default()
            },
            &targets,
            UiNavigationPolicy::Linear,
            |_| Some(ACTION),
        );
        assert_eq!(state.touch_capture_id(7), None);
        assert!(!output.activated(ids[0]));
    }

    #[test]
    fn interaction_pass_does_not_activate_passive_target() {
        let ids = ids();
        let targets = [PassiveTarget(TestTarget {
            id: ids[0],
            rect: Rect {
                x: 0,
                y: 0,
                width: 20,
                height: 20,
            },
        })];
        let mut state = UiInteractionState::default();

        let output = run_interaction_pass(
            &mut state,
            UiInput {
                nav: UiNavInput {
                    confirm: true,
                    ..UiNavInput::default()
                },
                ..UiInput::default()
            },
            &targets,
            UiNavigationPolicy::Linear,
            |_| Some(ACTION),
        );

        assert_eq!(output.focused_id(), Some(ids[0]));
        assert!(!output.activated(ids[0]));
        assert!(!output.action_pressed(ACTION));
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
    fn invalid_current_target_resets_capture_but_preserves_fallback_index() {
        let ids = ids();
        let mut state = UiInteractionState::default();
        state.repair_for_current_order(&ids);
        state.set_focused_id(Some(ids[1]));
        state.set_pointer_capture(Some(ids[1]));
        state.commit_order(&ids);
        let invalid = HashSet::from([ids[1]]);

        state.repair_for_current_order_excluding(&ids, &invalid);

        assert_eq!(state.pointer_capture_id(), None);
        assert_eq!(state.focused_id(), Some(ids[1]));
    }

    #[test]
    fn linear_navigation_uses_shared_focus_authority_and_wraps() {
        let ids = ids();
        let mut state = UiInteractionState::default();
        state.repair_for_current_order(&ids);

        state.navigate_linear(
            UiNavInput {
                up: true,
                ..UiNavInput::default()
            },
            &ids,
        );
        assert_eq!(state.focused_id(), Some(ids[2]));

        state.navigate_linear(
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
            &ids,
        );
        assert_eq!(state.focused_id(), Some(ids[0]));
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
