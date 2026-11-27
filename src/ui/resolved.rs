//! Discrete controls with caller-resolved framebuffer geometry and free painting.
//!
//! All target rectangles, including disabled controls, must be nonempty and
//! disjoint (touching edges are allowed). The caller supplies visible geometry,
//! owns painting, and keeps IDs stable across layout changes. Do not reuse an ID
//! for a different action during a gesture. Overlays and z-order are outside
//! this surface's contract. Spatial ranking and focus repair belong exclusively
//! to the existing interaction kernel.
//!
//! Confirm is suppressed during directional navigation, pointer/touch events,
//! ongoing captures, or repair of a removed/disabled focus. Cancel dominates all
//! input. Multiple distinct pointer activations in one frame are ambiguous and
//! yield no activation; repeated activation of one ID is deduplicated by the
//! kernel. `active` means pointer/touch capture, never keyboard held animation.

use std::{error::Error, fmt};

use crate::{ActionId, Rect};

pub use super::experimental::{UiId, UiInput, UiNavInput, UiPointerInput};
use super::kernel::{
    UiInteractionState, UiNavigationPolicy, UiResolvedTarget, run_interaction_pass,
};

/// One discrete control. Use this same `rect` when painting and hit-testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTarget {
    pub id: UiId,
    pub rect: Rect,
    pub action: ActionId,
    pub enabled: bool,
}

impl UiResolvedTarget for ResolvedTarget {
    fn ui_id(&self) -> UiId {
        self.id
    }

    fn ui_rect(&self) -> Rect {
        self.rect
    }
}

/// Opaque state for one surface. The kernel is its sole interaction authority.
#[derive(Debug, Default)]
pub struct ResolvedState {
    interaction: UiInteractionState,
    awaiting_neutral: bool,
}

impl ResolvedState {
    /// Immediately discards all captures and withholds input until an entirely
    /// neutral semantic snapshot has been consumed. That snapshot also cannot
    /// activate. The next update repairs focus against its enabled targets.
    ///
    /// Call before updating a surface being suspended or resumed. This is an
    /// explicit lifecycle operation, not an automatic OS focus-loss hook.
    /// `UiInput` contains events, not held-device state: callers needing a
    /// physical release barrier must additionally gate their semantic snapshots.
    pub fn suspend(&mut self) {
        self.interaction.set_pointer_capture(None);
        let touches: Vec<_> = self.interaction.touch_captures().keys().copied().collect();
        for touch in touches {
            self.interaction.clear_touch_capture(touch);
        }
        self.awaiting_neutral = true;
    }
}

/// At most one ID/action pair per frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activation {
    pub id: UiId,
    pub action: ActionId,
}

/// Frame-local facts for painting and dispatch. Availability stays in the target.
#[derive(Debug, Clone)]
pub struct ResolvedOutput {
    pub focused: Option<UiId>,
    pub hovered: Option<UiId>,
    pub activation: Option<Activation>,
    pub cancelled: bool,
    active: Vec<UiId>,
}

impl ResolvedOutput {
    /// True only while the kernel captures a mouse or touch gesture for `id`.
    pub fn active(&self, id: UiId) -> bool {
        self.active.contains(&id)
    }
}

/// Deterministic validation failure. No interaction pass or action is delivered
/// for an invalid frame; captures are discarded and neutral reentry is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedError {
    RootId { index: usize },
    DuplicateId { id: UiId },
    EmptyRect { id: UiId },
    Overlap { first: UiId, second: UiId },
}

impl fmt::Display for ResolvedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid resolved controls: {self:?}")
    }
}

impl Error for ResolvedError {}

/// Updates one surface without painting or accessing a framebuffer.
///
/// All rectangles are validated before disabled targets are filtered. Geometry
/// uses the existing overflow-safe `Rect::intersects` (i64 edges). The kernel
/// owns hit-testing, spatial selection, focus fallback, captures and ID dedup.
pub fn update(
    state: &mut ResolvedState,
    mut input: UiInput<'_>,
    targets: &[ResolvedTarget],
) -> Result<ResolvedOutput, ResolvedError> {
    if let Err(error) = validate(targets) {
        state.suspend();
        return Err(error);
    }
    let enabled: Vec<_> = targets.iter().copied().filter(|t| t.enabled).collect();
    let cancelled = input.nav.cancel;
    if cancelled {
        state.suspend();
    }
    if state.awaiting_neutral {
        // Position alone is neutral: keep hover available for rendering.
        state.awaiting_neutral = input.nav != UiNavInput::default()
            || input.pointer.pressed
            || input.pointer.released
            || !input.touches.is_empty();
        input = UiInput {
            pointer: UiPointerInput {
                position: input.pointer.position,
                ..Default::default()
            },
            ..Default::default()
        };
    } else {
        let lost_focus = state
            .interaction
            .focused_id()
            .is_some_and(|id| !enabled.iter().any(|t| t.id == id));
        let directional = input.nav.up || input.nav.down || input.nav.left || input.nav.right;
        let gesture = state.interaction.pointer_capture_id().is_some()
            || state.interaction.touch_capture_count() != 0
            || input.pointer.pressed
            || input.pointer.released
            || !input.touches.is_empty();
        input.nav.confirm &= !lost_focus && !directional && !gesture;
    }
    let output = run_interaction_pass(
        &mut state.interaction,
        input,
        &enabled,
        UiNavigationPolicy::Spatial,
        |id| enabled.iter().find(|t| t.id == id).map(|t| t.action),
    );
    let activation = if output.activation_count() == 1 {
        // Never select an arbitrary element from a multi-ID HashSet.
        output.activated_ids().iter().next().and_then(|id| {
            enabled.iter().find(|t| t.id == *id).map(|t| Activation {
                id: t.id,
                action: t.action,
            })
        })
    } else {
        None
    };
    Ok(ResolvedOutput {
        focused: output.focused_id(),
        hovered: output.hovered_id(),
        activation,
        cancelled,
        active: enabled
            .iter()
            .filter(|t| state.interaction.is_active(t.id))
            .map(|t| t.id)
            .collect(),
    })
}

fn validate(targets: &[ResolvedTarget]) -> Result<(), ResolvedError> {
    for (index, target) in targets.iter().enumerate() {
        if target.id == UiId::ROOT {
            return Err(ResolvedError::RootId { index });
        }
        if targets[..index].iter().any(|t| t.id == target.id) {
            return Err(ResolvedError::DuplicateId { id: target.id });
        }
        if target.rect.width == 0 || target.rect.height == 0 {
            return Err(ResolvedError::EmptyRect { id: target.id });
        }
        if let Some(first) = targets[..index]
            .iter()
            .find(|t| t.rect.intersects(target.rect))
        {
            return Err(ResolvedError::Overlap {
                first: first.id,
                second: target.id,
            });
        }
    }
    Ok(())
}
