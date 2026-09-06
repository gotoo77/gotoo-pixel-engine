//! Curated UI font assets validated by the optional P4 typography work.
//!
//! The bytes remain owned by GPE so consumers do not need to duplicate font
//! binaries. This module is available only with the `outline-fonts` feature.
//!
//! The bundled fonts come from Google Fonts and are distributed under the
//! SIL Open Font License; provenance and original license files live under
//! `assets/fonts/p4/`.

/// Exo 2: readable UI/body face for controls, labels and metadata.
pub const EXO_2: &[u8] = include_bytes!("../../assets/fonts/p4/exo2/font.ttf");

/// Unbounded: display face for product/launcher headings.
pub const UNBOUNDED: &[u8] = include_bytes!("../../assets/fonts/p4/unbounded/font.ttf");
