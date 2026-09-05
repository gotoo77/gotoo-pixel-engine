use crate::{
    Frame, Framebuffer, Game, GameResult, PixelPresentation, Rect, Size, Viewport,
    present_pixel_surface,
};

/// Provisional P6 helper for hosting one low-resolution `Game` inside a
/// high-resolution parent frame.
///
/// This intentionally solves only the Native keyboard/gamepad case proven by
/// Arcade. The child currently receives the parent's `Input` snapshot unchanged,
/// so pointer/touch coordinates remain host-space and are therefore outside this
/// helper's validated contract. A mapped pointer/touch contract remains a
/// separate P6/P7 concern.
pub struct PixelGameHost {
    game: Box<dyn Game>,
    framebuffer: Framebuffer,
    size: Size,
}

impl PixelGameHost {
    pub fn from_boxed(game: Box<dyn Game>, size: Size) -> Self {
        Self {
            game,
            framebuffer: Framebuffer::new(size.width, size.height),
            size,
        }
    }

    pub fn new<G: Game + 'static>(game: G, size: Size) -> Self {
        Self::from_boxed(Box::new(game), size)
    }

    pub const fn size(&self) -> Size {
        self.size
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// Updates a keyboard/gamepad-oriented child and presents its low-resolution
    /// framebuffer into `bounds` using integer-nearest composition.
    ///
    /// Pointer/touch consumers must not use this provisional method until the
    /// mapped-input contract is implemented.
    pub fn update_and_present_shared_input(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelPresentation>) {
        let result = {
            let mut child = Frame {
                framebuffer: &mut self.framebuffer,
                input: host.input,
                delta_time: host.delta_time,
                storage: &mut *host.storage,
                audio: &mut *host.audio,
                surface_size: self.size,
                viewport: Viewport::new(self.size, self.size),
            };
            self.game.update(&mut child)
        };

        let presentation = present_pixel_surface(host.framebuffer, &self.framebuffer, bounds);
        (result, presentation)
    }

    pub fn present(&self, host: &mut Framebuffer, bounds: Rect) -> Option<PixelPresentation> {
        present_pixel_surface(host, &self.framebuffer, bounds)
    }
}
