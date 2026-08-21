const CHOICE_HOVER_SOUND: SoundId = SoundId::new("void_canticle.ui.choice_hover");
const CHOICE_CONFIRM_SOUND: SoundId = SoundId::new("void_canticle.ui.choice_confirm");

type ChoiceArtRenderer = fn(&mut Framebuffer, i32, i32, bool, f32);

#[derive(Clone, Copy)]
pub(crate) enum ChoiceArt<'a> {
    Procedural(ChoiceArtRenderer),
    Sprite(&'a Sprite),
}

impl ChoiceArt<'_> {
    fn render(
        self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        selected: bool,
        time: f32,
    ) {
        match self {
            Self::Procedural(renderer) => renderer(framebuffer, x, y, selected, time),
            Self::Sprite(sprite) => sprite.draw_centered(framebuffer, x, y),
        }
    }
}

#[derive(Clone, Copy)]
struct ChoiceAssets<'a> {
    art: ChoiceArt<'a>,
    catalog_id: Option<ChoiceArtId>,
    hover_sound: Option<SoundId>,
    confirm_sound: Option<SoundId>,
}

impl<'a> ChoiceAssets<'a> {
    const fn new(
        art: ChoiceArt<'a>,
        hover_sound: Option<SoundId>,
        confirm_sound: Option<SoundId>,
    ) -> Self {
        Self {
            art,
            catalog_id: None,
            hover_sound,
            confirm_sound,
        }
    }

    const fn procedural(renderer: ChoiceArtRenderer) -> Self {
        Self::new(
            ChoiceArt::Procedural(renderer),
            Some(CHOICE_HOVER_SOUND),
            Some(CHOICE_CONFIRM_SOUND),
        )
    }

    const fn with_catalog_id(mut self, catalog_id: Option<ChoiceArtId>) -> Self {
        self.catalog_id = catalog_id;
        self
    }

    fn hover_sound(self) -> Option<SoundId> {
        self.catalog_id
            .and_then(|catalog_id| choice_catalog().hover_sound(catalog_id))
            .or(self.hover_sound)
    }

    fn confirm_sound(self) -> Option<SoundId> {
        self.catalog_id
            .and_then(|catalog_id| choice_catalog().confirm_sound(catalog_id))
            .or(self.confirm_sound)
    }

    fn catalog_sprite(self) -> Option<&'static Sprite> {
        self.catalog_id
            .and_then(|catalog_id| choice_catalog().sprite(catalog_id))
    }

    fn render(
        self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        selected: bool,
        time: f32,
    ) {
        if let Some(sprite) = self.catalog_sprite() {
            sprite.draw_centered(framebuffer, x, y);
            return;
        }
        self.art.render(framebuffer, x, y, selected, time);
    }
}

#[derive(Clone, Copy)]
struct ChoiceProfile<'a> {
    label: &'static str,
    category: &'static str,
    accent: Pixel,
    assets: ChoiceAssets<'a>,
}

impl<'a> ChoiceProfile<'a> {
    fn new(
        label: &'static str,
        category: &'static str,
        accent: Pixel,
        assets: ChoiceAssets<'a>,
    ) -> Self {
        Self {
            label,
            category,
            accent,
            assets: assets.with_catalog_id(ChoiceArtId::from_label(label)),
        }
    }

    const fn label(self) -> &'static str {
        self.label
    }

    const fn category(self) -> &'static str {
        self.category
    }

    const fn accent(self) -> Pixel {
        self.accent
    }

    const fn assets(self) -> ChoiceAssets<'a> {
        self.assets
    }

    fn hover_sound(self) -> Option<SoundId> {
        self.assets.hover_sound()
    }

    fn confirm_sound(self) -> Option<SoundId> {
        self.assets.confirm_sound()
    }

    fn render_art(
        self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        selected: bool,
        time: f32,
    ) {
        if selected && let Some(sprite) = self.assets.catalog_sprite() {
            let half_extent = sprite.width().max(sprite.height()) / 2;
            let radius = half_extent.saturating_add(4);
            let pulse = ((time * 5.0).sin().abs() * 2.0).round() as u32;
            framebuffer.draw_circle(x, y, radius.saturating_add(pulse), self.accent);
        }
        self.assets.render(framebuffer, x, y, selected, time);
    }
}

fn register_choice_sounds(sounds: &mut SoundBank) {
    sounds
        .insert_wav(
            CHOICE_HOVER_SOUND,
            synthesize_chirp(620.0, 880.0, 0.035, 0.035),
        )
        .expect("choice hover sound id should be unique");
    sounds
        .insert_wav(
            CHOICE_CONFIRM_SOUND,
            synthesize_chirp(420.0, 1_180.0, 0.07, 0.055),
        )
        .expect("choice confirm sound id should be unique");
    choice_catalog()
        .register_sounds(sounds)
        .expect("choice override sound ids should be unique");
}

#[cfg(test)]
mod choice_asset_tests {
    use super::*;

    fn test_art(
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        _selected: bool,
        _time: f32,
    ) {
        framebuffer.draw(x, y, Pixel::WHITE);
    }

    #[test]
    fn procedural_choice_assets_own_hover_and_confirm_audio_metadata() {
        let assets = ChoiceAssets::procedural(test_art);
        assert_eq!(assets.hover_sound(), Some(CHOICE_HOVER_SOUND));
        assert_eq!(assets.confirm_sound(), Some(CHOICE_CONFIRM_SOUND));
    }

    #[test]
    fn choice_art_can_be_replaced_by_a_sprite_without_changing_card_layout() {
        let sprite = Sprite::new(1, 1, vec![Pixel::WHITE]).expect("valid test sprite");
        let assets = ChoiceAssets::new(ChoiceArt::Sprite(&sprite), None, None);
        let mut framebuffer = Framebuffer::new(3, 3);
        framebuffer.clear(Pixel::BLACK);
        assets.render(&mut framebuffer, 1, 1, false, 0.0);
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::WHITE));
    }

    #[test]
    fn choice_profile_keeps_identity_and_assets_together() {
        let profile = ChoiceProfile::new(
            "TEST CHOICE",
            "TEST FAMILY",
            Pixel::WHITE,
            ChoiceAssets::procedural(test_art),
        );
        assert_eq!(profile.label(), "TEST CHOICE");
        assert_eq!(profile.category(), "TEST FAMILY");
        assert_eq!(profile.accent(), Pixel::WHITE);
        assert_eq!(profile.hover_sound(), Some(CHOICE_HOVER_SOUND));
        assert_eq!(profile.confirm_sound(), Some(CHOICE_CONFIRM_SOUND));
        assert_eq!(profile.assets().catalog_id, None);
    }

    #[test]
    fn known_choice_profile_binds_to_stable_catalog_id() {
        let profile = ChoiceProfile::new(
            "DEATH NOVA",
            "DEATH FIELD",
            DANGER,
            ChoiceAssets::procedural(test_art),
        );
        assert_eq!(profile.assets().catalog_id, Some(ChoiceArtId::DeathNova));
    }

    #[test]
    fn checked_in_authored_icons_decode() {
        for (name, bytes) in [
            (
                "bulwark",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/bulwark.png"
                )) as &[u8],
            ),
            (
                "pilgrim",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/pilgrim.png"
                )),
            ),
            (
                "wraith",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/wraith.png"
                )),
            ),
            (
                "death_nova",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/assets/void_canticle/ui/choice/death_nova.png"
                )),
            ),
        ] {
            let sprite = decode_choice_png_sprite(bytes)
                .unwrap_or_else(|error| panic!("{name} authored icon should decode: {error}"));
            assert!(sprite.width() > 0 && sprite.height() > 0, "{name}");
        }
    }
}
