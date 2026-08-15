use crate::{
    ActionId, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, GamepadId,
    GamepadProfile, Input, Key, Pixel, Rect, Size,
};

use super::{MenuState, VirtualButton, VirtualPad, draw_menu_item, draw_panel, draw_text_centered};

const PAUSE_TOGGLE: ActionId = ActionId::new("ui.pause.toggle");
const PAUSE_UP: ActionId = ActionId::new("ui.pause.up");
const PAUSE_DOWN: ActionId = ActionId::new("ui.pause.down");
const PAUSE_CONFIRM: ActionId = ActionId::new("ui.pause.confirm");
const TOUCH_RESUME: ActionId = ActionId::new("ui.pause.touch.resume");
const TOUCH_QUIT: ActionId = ActionId::new("ui.pause.touch.quit");

const BG: Pixel = Pixel::rgb(7, 10, 14);
const PANEL: Pixel = Pixel::rgb(12, 18, 24);
const FG: Pixel = Pixel::rgb(224, 234, 220);
const BORDER: Pixel = Pixel::rgb(80, 150, 220);
const ACCENT: Pixel = Pixel::rgb(120, 235, 180);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseConfig {
    surface_size: Size,
    touch_button: Option<Rect>,
}

impl PauseConfig {
    pub const fn new(surface_size: Size) -> Self {
        Self {
            surface_size,
            touch_button: None,
        }
    }

    pub const fn with_touch_button(mut self, rect: Rect) -> Self {
        self.touch_button = Some(rect);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseState {
    Running,
    Paused,
    ResumeGate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PauseLayout {
    panel: Rect,
    title: Rect,
    resume: Rect,
    quit: Rect,
}

impl PauseLayout {
    fn new(size: Size) -> Self {
        let panel_width = size.width.min(192);
        let panel_height = size.height.min(116);
        let panel = Rect {
            x: (size.width.saturating_sub(panel_width) / 2) as i32,
            y: (size.height.saturating_sub(panel_height) / 2) as i32,
            width: panel_width,
            height: panel_height,
        };
        let item_width = panel.width.saturating_sub(32);

        Self {
            panel,
            title: Rect {
                x: panel.x + 16,
                y: panel.y + 12,
                width: item_width,
                height: 22,
            },
            resume: Rect {
                x: panel.x + 16,
                y: panel.y + 48,
                width: item_width,
                height: 22,
            },
            quit: Rect {
                x: panel.x + 16,
                y: panel.y + 78,
                width: item_width,
                height: 22,
            },
        }
    }
}

pub struct PauseGame<G> {
    game: G,
    config: PauseConfig,
    layout: PauseLayout,
    state: PauseState,
    menu: MenuState,
    controls: ControlMap,
    trigger_pad: Option<VirtualPad>,
    menu_pad: Option<VirtualPad>,
}

impl<G> PauseGame<G> {
    pub fn new(game: G, config: PauseConfig) -> Self {
        let layout = PauseLayout::new(config.surface_size);
        let trigger_pad = config
            .touch_button
            .map(|rect| VirtualPad::new([VirtualButton::new(PAUSE_TOGGLE, rect)]));
        let menu_pad = config.touch_button.map(|_| {
            VirtualPad::new([
                VirtualButton::new(TOUCH_RESUME, layout.resume),
                VirtualButton::new(TOUCH_QUIT, layout.quit),
            ])
        });

        Self {
            game,
            config,
            layout,
            state: PauseState::Running,
            menu: MenuState::new(2),
            controls: pause_controls(),
            trigger_pad,
            menu_pad,
        }
    }

    fn update_running(&mut self, frame: &mut Frame<'_>) -> GameResult
    where
        G: Game,
    {
        let trigger_touch = self.update_trigger(frame.input);
        self.controls.update(frame.input);

        if trigger_touch || self.controls.action(PAUSE_TOGGLE).pressed() {
            self.state = PauseState::Paused;
            self.menu = MenuState::new(2);
            self.render_pause(frame.framebuffer);
            return GameResult::Continue;
        }

        let result = self.game.update(frame);
        if result == GameResult::Continue {
            self.render_pause_button(frame.framebuffer);
        }
        result
    }

    fn update_paused(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let trigger_touch = self.update_trigger(frame.input);
        let (touch_resume, touch_quit) = self.update_menu_pad(frame.input);
        self.controls.update(frame.input);

        if trigger_touch || self.controls.action(PAUSE_TOGGLE).pressed() || touch_resume {
            self.state = PauseState::ResumeGate;
            self.render_pause(frame.framebuffer);
            return GameResult::Continue;
        }

        if touch_quit {
            return GameResult::Exit;
        }

        if self.controls.action(PAUSE_UP).pressed() {
            self.menu.select_previous();
        }
        if self.controls.action(PAUSE_DOWN).pressed() {
            self.menu.select_next();
        }
        if self.controls.action(PAUSE_CONFIRM).pressed() {
            match self.menu.selected() {
                Some(0) => {
                    self.state = PauseState::ResumeGate;
                    self.render_pause(frame.framebuffer);
                    return GameResult::Continue;
                }
                Some(1) => return GameResult::Exit,
                _ => {}
            }
        }

        self.render_pause(frame.framebuffer);
        GameResult::Continue
    }

    fn update_resume_gate(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.update_trigger(frame.input);
        self.update_menu_pad(frame.input);
        self.controls.update(frame.input);

        if self.pause_input_held() {
            self.render_pause(frame.framebuffer);
            return GameResult::Continue;
        }

        if let Some(trigger_pad) = &mut self.trigger_pad {
            trigger_pad.reset(&mut self.controls);
        }
        if let Some(menu_pad) = &mut self.menu_pad {
            menu_pad.reset(&mut self.controls);
        }
        self.state = PauseState::Running;

        // Deliberately do not update the child on the release frame. This keeps
        // the input used to resume from leaking into the resumed game.
        GameResult::Continue
    }

    fn update_trigger(&mut self, input: &Input) -> bool {
        self.trigger_pad
            .as_mut()
            .map(|pad| {
                pad.update(input, &mut self.controls)
                    .pressed_actions()
                    .contains(&PAUSE_TOGGLE)
            })
            .unwrap_or(false)
    }

    fn update_menu_pad(&mut self, input: &Input) -> (bool, bool) {
        let Some(menu_pad) = &mut self.menu_pad else {
            return (false, false);
        };
        let update = menu_pad.update(input, &mut self.controls);
        (
            update.pressed_actions().contains(&TOUCH_RESUME),
            update.pressed_actions().contains(&TOUCH_QUIT),
        )
    }

    fn pause_input_held(&self) -> bool {
        [
            PAUSE_TOGGLE,
            PAUSE_UP,
            PAUSE_DOWN,
            PAUSE_CONFIRM,
            TOUCH_RESUME,
            TOUCH_QUIT,
        ]
        .into_iter()
        .any(|action| self.controls.action(action).held())
    }

    fn render_pause_button(&self, framebuffer: &mut Framebuffer) {
        let Some(rect) = self.config.touch_button else {
            return;
        };

        draw_panel(framebuffer, rect, BG, TOUCH_ACCENT);
        draw_text_centered(framebuffer, rect, "PAUSE", 1, TOUCH_ACCENT);
    }

    fn render_pause(&self, framebuffer: &mut Framebuffer) {
        draw_panel(framebuffer, self.layout.panel, PANEL, BORDER);
        draw_text_centered(framebuffer, self.layout.title, "PAUSED", 2, ACCENT);

        for (index, (rect, label)) in [(self.layout.resume, "RESUME"), (self.layout.quit, "QUIT")]
            .into_iter()
            .enumerate()
        {
            draw_menu_item(
                framebuffer,
                rect,
                label,
                self.menu.selected() == Some(index),
                1,
                FG,
                ACCENT,
            );
        }
    }
}

impl<G: Game> Game for PauseGame<G> {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        match self.state {
            PauseState::Running => self.update_running(frame),
            PauseState::Paused => self.update_paused(frame),
            PauseState::ResumeGate => self.update_resume_gate(frame),
        }
    }

    fn gamepad_profile(&self, id: GamepadId) -> Option<GamepadProfile> {
        self.game.gamepad_profile(id)
    }
}

fn pause_controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(PAUSE_TOGGLE, Key::Escape)
        .bind_gamepad(PAUSE_TOGGLE, GamepadButton::Start)
        .bind_key(PAUSE_UP, Key::Up)
        .bind_key(PAUSE_UP, Key::W)
        .bind_gamepad(PAUSE_UP, GamepadButton::DPadUp)
        .bind_gamepad(PAUSE_UP, GamepadButton::LeftStickUp)
        .bind_key(PAUSE_DOWN, Key::Down)
        .bind_key(PAUSE_DOWN, Key::S)
        .bind_gamepad(PAUSE_DOWN, GamepadButton::DPadDown)
        .bind_gamepad(PAUSE_DOWN, GamepadButton::LeftStickDown)
        .bind_key(PAUSE_CONFIRM, Key::Space)
        .bind_gamepad(PAUSE_CONFIRM, GamepadButton::South);
    controls
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{NoopAudio, NoopStorage, Viewport};

    #[derive(Default)]
    struct CountingGame {
        updates: usize,
    }

    impl Game for CountingGame {
        fn update(&mut self, _frame: &mut Frame<'_>) -> GameResult {
            self.updates += 1;
            GameResult::Continue
        }
    }

    fn with_frame<T>(
        input: &Input,
        framebuffer: &mut Framebuffer,
        f: impl FnOnce(&mut Frame<'_>) -> T,
    ) -> T {
        let size = Size {
            width: framebuffer.width(),
            height: framebuffer.height(),
        };
        let mut storage = NoopStorage;
        let mut audio = NoopAudio::default();
        let mut frame = Frame {
            framebuffer,
            input,
            delta_time: Duration::from_millis(16),
            storage: &mut storage,
            audio: &mut audio,
            surface_size: size,
            viewport: Viewport::new(size, size),
        };
        f(&mut frame)
    }

    #[test]
    fn running_wrapper_updates_child() {
        let mut wrapper = PauseGame::new(
            CountingGame::default(),
            PauseConfig::new(Size {
                width: 320,
                height: 180,
            }),
        );
        let input = Input::default();
        let mut framebuffer = Framebuffer::new(320, 180);

        with_frame(&input, &mut framebuffer, |frame| {
            assert_eq!(wrapper.update(frame), GameResult::Continue);
        });

        assert_eq!(wrapper.game.updates, 1);
    }

    #[test]
    fn pause_stops_child_updates() {
        let mut wrapper = PauseGame::new(
            CountingGame::default(),
            PauseConfig::new(Size {
                width: 320,
                height: 180,
            }),
        );
        let mut input = Input::default();
        let mut framebuffer = Framebuffer::new(320, 180);
        input.press_key(Key::Escape);

        with_frame(&input, &mut framebuffer, |frame| {
            assert_eq!(wrapper.update(frame), GameResult::Continue);
        });
        assert_eq!(wrapper.state, PauseState::Paused);
        assert_eq!(wrapper.game.updates, 0);

        input.advance_frame();
        input.release_key(Key::Escape);
        with_frame(&input, &mut framebuffer, |frame| {
            assert_eq!(wrapper.update(frame), GameResult::Continue);
        });
        assert_eq!(wrapper.game.updates, 0);
    }

    #[test]
    fn resume_waits_for_confirm_release_before_child_updates() {
        let mut wrapper = PauseGame::new(
            CountingGame::default(),
            PauseConfig::new(Size {
                width: 320,
                height: 180,
            }),
        );
        let mut input = Input::default();
        let mut framebuffer = Framebuffer::new(320, 180);

        input.press_key(Key::Escape);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        input.advance_frame();
        input.release_key(Key::Escape);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        input.advance_frame();

        input.press_key(Key::Space);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        assert_eq!(wrapper.state, PauseState::ResumeGate);
        assert_eq!(wrapper.game.updates, 0);

        input.advance_frame();
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        assert_eq!(wrapper.state, PauseState::ResumeGate);
        assert_eq!(wrapper.game.updates, 0);

        input.release_key(Key::Space);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        assert_eq!(wrapper.state, PauseState::Running);
        assert_eq!(wrapper.game.updates, 0);

        input.advance_frame();
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        assert_eq!(wrapper.game.updates, 1);
    }

    #[test]
    fn quit_from_pause_returns_exit() {
        let mut wrapper = PauseGame::new(
            CountingGame::default(),
            PauseConfig::new(Size {
                width: 320,
                height: 180,
            }),
        );
        let mut input = Input::default();
        let mut framebuffer = Framebuffer::new(320, 180);

        input.press_key(Key::Escape);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        input.advance_frame();
        input.release_key(Key::Escape);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        input.advance_frame();

        input.press_key(Key::Down);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        input.advance_frame();
        input.release_key(Key::Down);
        with_frame(&input, &mut framebuffer, |frame| {
            wrapper.update(frame);
        });
        input.advance_frame();

        input.press_key(Key::Space);
        let result = with_frame(&input, &mut framebuffer, |frame| wrapper.update(frame));
        assert_eq!(result, GameResult::Exit);
        assert_eq!(wrapper.game.updates, 0);
    }

    #[test]
    fn touch_layout_keeps_pause_button_separate_from_overlay_actions() {
        let config = PauseConfig::new(Size {
            width: 480,
            height: 260,
        })
        .with_touch_button(Rect {
            x: 400,
            y: 228,
            width: 72,
            height: 24,
        });
        let wrapper = PauseGame::new(CountingGame::default(), config);

        let trigger = config
            .touch_button
            .expect("touch config should have trigger");
        assert!(!trigger.intersects(wrapper.layout.resume));
        assert!(!trigger.intersects(wrapper.layout.quit));
    }
}
