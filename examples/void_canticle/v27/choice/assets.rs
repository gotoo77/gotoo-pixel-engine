const VC27_CHOICE_HOVER_SOUND: SoundId = SoundId::new("void_canticle.ui.choice_hover");

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
}

impl<'a> Vc27ChoiceAssets<'a> {
    const fn new(art: Vc27ChoiceArt<'a>, hover_sound: Option<SoundId>) -> Self {
        Self { art, hover_sound }
    }

    const fn procedural(renderer: Vc27ChoiceArtRenderer) -> Self {
        Self::new(
            Vc27ChoiceArt::Procedural(renderer),
            Some(VC27_CHOICE_HOVER_SOUND),
        )
    }

    const fn hover_sound(self) -> Option<SoundId> {
        self.hover_sound
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

fn vc27_register_choice_hover_sound(sounds: &mut SoundBank) {
    sounds
        .insert_wav(
            VC27_CHOICE_HOVER_SOUND,
            synthesize_chirp(620.0, 880.0, 0.035, 0.035),
        )
        .expect("VC2.7 choice hover sound id should be unique");
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
    fn procedural_choice_assets_can_own_hover_audio_metadata() {
        let assets = Vc27ChoiceAssets::procedural(test_art);
        assert_eq!(assets.hover_sound(), Some(VC27_CHOICE_HOVER_SOUND));
    }

    #[test]
    fn choice_art_can_be_replaced_by_a_sprite_without_changing_card_layout() {
        let sprite = Sprite::new(1, 1, vec![Pixel::WHITE]).expect("valid test sprite");
        let assets = Vc27ChoiceAssets::new(Vc27ChoiceArt::Sprite(&sprite), None);
        let mut framebuffer = Framebuffer::new(3, 3);
        framebuffer.clear(Pixel::BLACK);
        assets.render(&mut framebuffer, 1, 1, false, 0.0);
        assert_eq!(framebuffer.pixel(1, 1), Some(Pixel::WHITE));
    }
}
