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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_input_is_idle_and_touch_free() {
        let input = UiInput::default();

        assert_eq!(input.nav, UiNavInput::default());
        assert_eq!(input.pointer, UiPointerInput::default());
        assert!(input.touches.is_empty());
    }
}
