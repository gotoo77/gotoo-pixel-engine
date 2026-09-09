//! Curated UI font assets validated by the optional P4 typography work.
//!
//! The bytes remain owned by GPE so consumers do not need to duplicate font
//! binaries. This module is available only with the `outline-fonts` feature.
//!
//! The bundled fonts come from Google Fonts and are distributed under the
//! SIL Open Font License; provenance and original license files live under
//! `assets/fonts/p4/`.

use crate::outline_text::OutlineFont;

/// Figtree: neutral, readable UI/body face suitable for game interfaces.
pub const FIGTREE: &[u8] = include_bytes!("../../assets/fonts/p4/figtree/font.ttf");

/// Exo 2: readable UI/body face for controls, labels and metadata.
pub const EXO_2: &[u8] = include_bytes!("../../assets/fonts/p4/exo2/font.ttf");

/// Unbounded: display face for product/launcher headings.
pub const UNBOUNDED: &[u8] = include_bytes!("../../assets/fonts/p4/unbounded/font.ttf");

/// Curated outline fonts bundled with GPE.
///
/// Games can depend on this catalog instead of duplicating font binaries in
/// each standalone repository. Keep this list intentionally small: adding a
/// face here makes it part of GPE's public asset contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinOutlineFont {
    Figtree,
    Exo2,
    Unbounded,
}

impl BuiltinOutlineFont {
    pub const ALL: [Self; 3] = [Self::Figtree, Self::Exo2, Self::Unbounded];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Figtree => "Figtree",
            Self::Exo2 => "Exo 2",
            Self::Unbounded => "Unbounded",
        }
    }

    pub const fn bytes(self) -> &'static [u8] {
        match self {
            Self::Figtree => FIGTREE,
            Self::Exo2 => EXO_2,
            Self::Unbounded => UNBOUNDED,
        }
    }

    pub fn load(self) -> Result<OutlineFont, &'static str> {
        OutlineFont::from_bytes(self.bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_outline_font_loads() {
        for font in BuiltinOutlineFont::ALL {
            assert!(font.load().is_ok(), "{} must load", font.name());
        }
    }

    #[test]
    fn figtree_supports_initial_minoku_languages() {
        let font = BuiltinOutlineFont::Figtree.load().expect("Figtree must load");
        assert!(font.supports_text("Français Deutsch Italiano English"));
    }
}
