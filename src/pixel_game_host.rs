use crate::{
    Frame, Framebuffer, Game, GameResult, Input, PixelFitPresentation, PixelPresentation, Rect,
    Size, ToolFrame, ToolWindowConfig, Viewport, present_pixel_surface, present_pixel_surface_fit,
    presentation::fit_pixel_surface_presentation,
};

/// Hosts one low-resolution `Game` inside a high-resolution parent frame.
///
/// Keyboard/gamepad state is forwarded unchanged. The adaptive-fit path maps
/// mouse and touch positions from host space into the presented child surface,
/// while preserving touch ids/phases and button transitions.
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

    pub fn tool_window_config(&self) -> Option<ToolWindowConfig> {
        self.game.tool_window_config()
    }

    pub fn update_tool_window(&mut self, frame: &mut ToolFrame<'_>) {
        self.game.update_tool_window(frame);
    }

    pub fn tool_window_closed(&mut self) {
        self.game.tool_window_closed();
    }

    fn update_child(&mut self, host: &mut Frame<'_>) -> GameResult {
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
    }

    fn update_child_with_input(&mut self, host: &mut Frame<'_>, input: &Input) -> GameResult {
        let mut child = Frame {
            framebuffer: &mut self.framebuffer,
            input,
            delta_time: host.delta_time,
            storage: &mut *host.storage,
            audio: &mut *host.audio,
            surface_size: self.size,
            viewport: Viewport::new(self.size, self.size),
        };
        self.game.update(&mut child)
    }

    /// Updates a keyboard/gamepad-oriented child and presents its low-resolution
    /// framebuffer into `bounds` using integer-nearest composition.
    ///
    /// Pointer/touch consumers must not use this provisional method until the
    /// mapped-input contract is implemented.
    pub fn update_and_present(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelPresentation>) {
        let result = self.update_child(host);
        let presentation = present_pixel_surface(host.framebuffer, &self.framebuffer, bounds);
        (result, presentation)
    }

    /// Updates a child and fits its framebuffer into `bounds` using
    /// aspect-ratio preserving nearest-neighbour sampling at any scale.
    ///
    /// Pointer and touch positions are mapped through the exact presentation
    /// rectangle before the child update. Events outside the child surface keep
    /// their lifecycle/id but carry no position, allowing touch consumers to
    /// release contacts without interpreting host-space coordinates.
    pub fn update_and_present_fit(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelFitPresentation>) {
        let input_presentation = fit_pixel_surface_presentation(self.size, bounds);
        let mapped_input = input_presentation.map(|presentation| {
            host.input
                .map_pointer_positions(|point| presentation.map_point(point))
        });

        let result = if let Some(input) = mapped_input.as_ref() {
            self.update_child_with_input(host, input)
        } else {
            self.update_child(host)
        };
        let presentation = present_pixel_surface_fit(host.framebuffer, &self.framebuffer, bounds);
        (result, presentation)
    }

    /// Compatibility alias kept during the P6 API-shape audit.
    pub fn update_and_present_shared_input(
        &mut self,
        host: &mut Frame<'_>,
        bounds: Rect,
    ) -> (GameResult, Option<PixelPresentation>) {
        self.update_and_present(host, bounds)
    }

    pub fn present(&self, host: &mut Framebuffer, bounds: Rect) -> Option<PixelPresentation> {
        present_pixel_surface(host, &self.framebuffer, bounds)
    }

    pub fn present_fit(
        &self,
        host: &mut Framebuffer,
        bounds: Rect,
    ) -> Option<PixelFitPresentation> {
        present_pixel_surface_fit(host, &self.framebuffer, bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MouseButton, Touch, TouchPhase};

    #[test]
    fn adaptive_fit_maps_host_pointer_and_touch_into_child_space() {
        let mut input = Input::default();
        input.press_mouse_button(MouseButton::Left);
        input.set_mouse_position(Some((640, 480)));
        input.push_touch(Touch {
            id: 11,
            phase: TouchPhase::Started,
            position: Some((640, 480)),
        });
        input.push_touch(Touch {
            id: 12,
            phase: TouchPhase::Ended,
            position: Some((100, 480)),
        });

        let presentation = fit_pixel_surface_presentation(
            Size {
                width: 540,
                height: 960,
            },
            Rect {
                x: 0,
                y: 0,
                width: 1280,
                height: 960,
            },
        )
        .expect("portrait child should fit");
        let mapped = input.map_pointer_positions(|point| presentation.map_point(point));

        assert_eq!(mapped.mouse_position(), Some((270, 480)));
        assert_eq!(
            mapped.touches(),
            &[
                Touch {
                    id: 11,
                    phase: TouchPhase::Started,
                    position: Some((270, 480)),
                },
                Touch {
                    id: 12,
                    phase: TouchPhase::Ended,
                    position: None,
                },
            ]
        );
    }
}
