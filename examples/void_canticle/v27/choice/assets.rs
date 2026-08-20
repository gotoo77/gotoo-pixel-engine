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

    const fn hover_sound(self) -> Option<SoundId> {
        self.hover_sound
    }

    const fn confirm_sound(self) -> Option<SoundId> {
        self.confirm_sound
    }

    fn render(
        self,
        framebuffer: &mut Framebuffer,
        x: i32,
        y: i32,
        selected: bool,
        time: f32,
    ) {
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
    const fn new(
        label: &'static str,
        category: &'static str,
        accent: Pixel,
        assets: Vc27ChoiceAssets<'a>,
    ) -> Self {
        Self {
            label,
            category,
            accent,
            assets,
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

    const fn hover_sound(self) -> Option<SoundId> {
        self.assets.hover_sound()
    }

    const fn confirm_sound(self) -> Option<SoundId> {
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
        self.assets.render(framebuffer, x, y, selected, time);
    }
}

fn vc27_register_choice_sounds(sounds: &mut SoundBank) {
    sounds
        .insert_wav(
            VC27_CHOICE_HOVER_SOUND,
            synthesize_chirp(620.0, 880.0, 0.035, 0.035),
        )
        .expect("VC2.7 choice hover sound id should be unique");
    sounds
        .insert_wav(
            VC27_CHOICE_CONFIRM_SOUND,
            synthesize_chirp(420.0, 1_180.0, 0.07, 0.055),
        )
        .expect("VC2.7 choice confirm sound id should be unique");
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
    }
}
