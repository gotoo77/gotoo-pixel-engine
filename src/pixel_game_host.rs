use crate::{Frame, Framebuffer, Game, GameResult, PixelPresentation, Rect, Size, Viewport, present_pixel_surface};

/// Provisional P6 helper for hosting one low-resolution `Game` inside a
/// high-resolution parent frame.
///
/// This intentionally solves only the Native/keyboard+gamepad case proven by
/// Arcade. Pointer, wheel and touch input are suppressed for the child rather
/// than forwarded with incorrect host-space coordinates. A mapped pointer/touch
/// contract remains a separate P6/P7 concern.
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

    /// Updates the child with keyboard/gamepad/text input and presents the
    /// resulting low-resolution framebuffer into `bounds` using integer-nearest
    /// composition.
    ///
    /// Pointer, wheel and touch state are intentionally hidden from the child
    /// until the mapped-input contract is introduced.
    pub fn update_and_present(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelPresentation>) {
        let child_input = host.input.without_pointer_and_touch();
        let result = {
            let mut child = Frame {
                framebuffer: &mut self.framebuffer,
                input: &child_input,
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
