use super::FlagIcon;

/// Language metadata for selector UIs.
///
/// The locale and native language name are authoritative. `flag` is only a
/// secondary visual cue and remains optional because languages are not countries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageOption {
    pub locale: &'static str,
    pub native_name: &'static str,
    pub flag: Option<FlagIcon>,
}

impl LanguageOption {
    pub const fn new(
        locale: &'static str,
        native_name: &'static str,
        flag: Option<FlagIcon>,
    ) -> Self {
        Self {
            locale,
            native_name,
            flag,
        }
    }
}

pub const ENGLISH: LanguageOption =
    LanguageOption::new("en", "English", Some(FlagIcon::UnitedKingdom));
pub const FRENCH: LanguageOption = LanguageOption::new("fr", "Français", Some(FlagIcon::France));
pub const GERMAN: LanguageOption = LanguageOption::new("de", "Deutsch", Some(FlagIcon::Germany));
pub const SPANISH: LanguageOption = LanguageOption::new("es", "Español", Some(FlagIcon::Spain));
pub const ITALIAN: LanguageOption = LanguageOption::new("it", "Italiano", Some(FlagIcon::Italy));
pub const PORTUGUESE_PORTUGAL: LanguageOption =
    LanguageOption::new("pt-PT", "Português", Some(FlagIcon::Portugal));
pub const PORTUGUESE_BRAZIL: LanguageOption =
    LanguageOption::new("pt-BR", "Português (Brasil)", Some(FlagIcon::Brazil));
pub const RUSSIAN: LanguageOption = LanguageOption::new("ru", "Русский", Some(FlagIcon::Russia));
pub const JAPANESE: LanguageOption = LanguageOption::new("ja", "日本語", Some(FlagIcon::Japan));
pub const KOREAN: LanguageOption = LanguageOption::new("ko", "한국어", Some(FlagIcon::SouthKorea));
pub const CHINESE_SIMPLIFIED: LanguageOption =
    LanguageOption::new("zh-CN", "简体中文", Some(FlagIcon::China));
pub const CHINESE_TRADITIONAL: LanguageOption =
    LanguageOption::new("zh-TW", "繁體中文", Some(FlagIcon::Taiwan));

/// Conservative text-script classification used only for raster-quality policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextScriptClass {
    Other,
    Cjk,
}

/// Returns `Cjk` when the text contains Han ideographs, Hiragana, Katakana,
/// Hangul, or common CJK punctuation.
pub fn classify_text_script(text: &str) -> TextScriptClass {
    if text.chars().any(is_cjk_character) {
        TextScriptClass::Cjk
    } else {
        TextScriptClass::Other
    }
}

/// Recommended internal raster scale for small outline UI text.
///
/// This changes raster precision, not the logical size of the text. Dense CJK
/// glyphs receive 3x internal rasterization by default; other scripts keep 1x.
pub fn recommended_raster_scale(text: &str) -> u32 {
    match classify_text_script(text) {
        TextScriptClass::Other => 1,
        TextScriptClass::Cjk => 3,
    }
}

fn is_cjk_character(character: char) -> bool {
    matches!(
        character as u32,
        0x3000..=0x303F // CJK punctuation
            | 0x3040..=0x309F // Hiragana
            | 0x30A0..=0x30FF // Katakana
            | 0x31F0..=0x31FF // Katakana phonetic extensions
            | 0x3400..=0x4DBF // CJK unified ideographs extension A
            | 0x4E00..=0x9FFF // CJK unified ideographs
            | 0xAC00..=0xD7AF // Hangul syllables
            | 0xF900..=0xFAFF // CJK compatibility ideographs
            | 0x20000..=0x2FA1F // supplementary CJK ideographs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_metadata_keeps_locale_name_and_flag_separate() {
        assert_eq!(JAPANESE.locale, "ja");
        assert_eq!(JAPANESE.native_name, "日本語");
        assert_eq!(JAPANESE.flag, Some(FlagIcon::Japan));

        let neutral = LanguageOption::new("eo", "Esperanto", None);
        assert_eq!(neutral.flag, None);
    }

    #[test]
    fn cjk_policy_detects_japanese_chinese_and_korean() {
        for text in ["ゲーム開始", "日本語", "简体中文", "繁體中文", "한국어"] {
            assert_eq!(classify_text_script(text), TextScriptClass::Cjk, "{text}");
            assert_eq!(recommended_raster_scale(text), 3, "{text}");
        }
    }

    #[test]
    fn latin_and_cyrillic_keep_standard_raster_scale() {
        for text in ["English", "Français", "Deutsch", "Русский"] {
            assert_eq!(classify_text_script(text), TextScriptClass::Other, "{text}");
            assert_eq!(recommended_raster_scale(text), 1, "{text}");
        }
    }
}
