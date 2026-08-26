use std::sync::{Mutex, OnceLock};

use crate::{Image, ImageError};

static WINDOW_ICON: OnceLock<Mutex<Option<Image>>> = OnceLock::new();

fn window_icon_slot() -> &'static Mutex<Option<Image>> {
    WINDOW_ICON.get_or_init(|| Mutex::new(None))
}

/// Sets the process-wide native window icon used by subsequent GPE windows.
///
/// GPE currently runs a single application window per process, so this keeps
/// branding configuration out of `EngineConfig` without breaking existing
/// struct literals. Web builds ignore this setting; browser branding belongs
/// to the page manifest/favicon instead.
pub fn set_window_icon(image: Image) {
    *window_icon_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(image);
}

/// Decodes a PNG and installs it as the native GPE window icon.
pub fn set_window_icon_png(bytes: &[u8]) -> Result<(), ImageError> {
    let image = Image::decode_png(bytes)?;
    set_window_icon(image);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn configured_window_icon() -> Option<Image> {
    window_icon_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn setter_installs_image() {
        let image = Image::from_rgba8(1, 1, vec![12, 34, 56, 255]).expect("valid test image");
        set_window_icon(image.clone());
        assert_eq!(configured_window_icon(), Some(image));
    }
}
