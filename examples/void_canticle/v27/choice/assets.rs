const VC27_CHOICE_HOVER_SOUND: SoundId = SoundId::new("void_canticle.ui.choice_hover");
const VC27_CHOICE_CONFIRM_SOUND: SoundId = SoundId::new("void_canticle.ui.choice_confirm");

type Vc27ChoiceArtRenderer = fn(&mut Framebuffer, i32, i32, bool, f32);

#[derive(Clone, Copy)]
pub(crate) enum Vc27ChoiceArt<'a> {
    Procedural(Vc27ChoiceArtRenderer),
    Sprite(&'a Sprite),
}

impl Vc27ChoiceArt<'_> {
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
struct Vc27ChoiceAssets<'a> {
    art: Vc27ChoiceArt<'a>,
    catalog_id: Option<Vc27ChoiceArtId>,
    hover_sound: Option<SoundId>,
    confirm_sound: Option<SoundId>,
}

impl<'a> Vc27ChoiceAssets<'a> {
    const fn new(
        art: Vc27ChoiceArt<'a>,
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

    const fn procedural(renderer: Vc27ChoiceArtRenderer) -> Self {
        Self::new(
            Vc27ChoiceArt::Procedural(renderer),
            Some(VC27_CHOICE_HOVER_SOUND),
            Some(VC27_CHOICE_CONFIRM_SOUND),
        )
    }

    const fn with_catalog_id(mut self, catalog_id: Option<Vc27ChoiceArtId>) -> Self {
        self.catalog_id = catalog_id;
        self
    }

    fn hover_sound(self) -> Option<SoundId> {
        self.catalog_id
            .and_then(|catalog_id| vc27_choice_catalog().hover_sound(catalog_id))
            .or(self.hover_sound)
    }

    fn confirm_sound(self) -> Option<SoundId> {
        self.catalog_id
            .and_then(|catalog_id| vc27_choice_catalog().confirm_sound(catalog_id))
            .or(self.confirm_sound)
    }

    fn catalog_sprite(self) -> Option<&'static Sprite> {
        self.catalog_id
            .and_then(|catalog_id| vc27_choice_catalog().sprite(catalog_id))
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
struct Vc27ChoiceProfile<'a> {
    label: &'static str,
    category: &'static str,
    accent: Pixel,
    assets: Vc27ChoiceAssets<'a>,
}

impl<'a> Vc27ChoiceProfile<'a> {
    fn new(
        label: &'static str,
        category: &'static str,
        accent: Pixel,
        assets: Vc27ChoiceAssets<'a>,
    ) -> Self {
        Self {
            label,
            category,
            accent,
            assets: assets.with_catalog_id(Vc27ChoiceArtId::from_label(label)),
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

    const fn assets(self) -> Vc27ChoiceAssets<'a> {
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
            let radius = i32::try_from(half_extent).unwrap_or(i32::MAX).saturating_add(4);
            let pulse = ((time * 5.0).sin().abs() * 2.0).round() as i32;
            framebuffer.draw_circle(x, y, radius.saturating_add(pulse), self.accent);
        }
        self.assets.render(framebuffer, x, y, selected, time);
    }
}

fn vc27_register_choice_sounds(sounds: &mut SoundBank) {
    sounds
        .insert_wav(
            VC27_CHOICE_HOVER_SOUND,
            synthesize_chirp(620.0, 880.0, 0.035, 0.035),
        )
        .expect("VC3.0 choice hover sound id should be unique");
    sounds
        .insert_wav(
            VC27_CHOICE_CONFIRM_SOUND,
            synthesize_chirp(420.0, 1_180.0, 0.07, 0.055),
        )
        .expect("VC3.0 choice confirm sound id should be unique");
    vc27_choice_catalog()
        .register_sounds(sounds)
        .expect("VC3.0 choice override sound ids should be unique");
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
        let assets = Vc27ChoiceAssets::procedural(test_art);
        assert_eq!(assets.hover_sound(), Some(VC27_CHOICE_HOVER_SOUND));
        assert_eq!(assets.confirm_sound(), Some(VC27_CHOICE_CONFIRM_SOUND));
    }

    #[test]
    fn choice_art_can_be_replaced_by_a_sprite_without_changing_card_layout() {
        let sprite = Sprite::new(1, 1, vec![Pixel::WHITE]).expect("valid test sprite");
        let assets = Vc27ChoiceAssets::new(Vc27ChoiceArt::Sprite(&sprite), None, None);
        let mut framebuffer = Framebuffer::new(3, 3);
        framebuffer.clear(Pixel::BLACK);
        assets.render(&mut framebuffer, 1, 1, false, 0.0);
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::WHITE));
    }

    #[test]
    fn choice_profile_keeps_identity_and_assets_together() {
        let profile = Vc27ChoiceProfile::new(
            "TEST CHOICE",
            "TEST FAMILY",
            Pixel::WHITE,
            Vc27ChoiceAssets::procedural(test_art),
        );
        assert_eq!(profile.label(), "TEST CHOICE");
        assert_eq!(profile.category(), "TEST FAMILY");
        assert_eq!(profile.accent(), Pixel::WHITE);
        assert_eq!(profile.hover_sound(), Some(VC27_CHOICE_HOVER_SOUND));
        assert_eq!(profile.confirm_sound(), Some(VC27_CHOICE_CONFIRM_SOUND));
        assert_eq!(profile.assets().catalog_id, None);
    }

    #[test]
    fn known_choice_profile_binds_to_stable_catalog_id() {
        let profile = Vc27ChoiceProfile::new(
            "DEATH NOVA",
            "DEATH FIELD",
            DANGER,
            Vc27ChoiceAssets::procedural(test_art),
        );
        assert_eq!(profile.assets().catalog_id, Some(Vc27ChoiceArtId::DeathNova));
    }

    #[test]
    fn vc30_checked_in_authored_icons_decode() {
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
            let sprite = vc27_decode_png_sprite(bytes)
                .unwrap_or_else(|error| panic!("{name} authored icon should decode: {error}"));
            assert!(sprite.width() > 0 && sprite.height() > 0, "{name}");
        }
    }
}
