from pathlib import Path
import re

path = Path("examples/snake/game.rs")
text = path.read_text()


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    text = text.replace(old, new, 1)


replace_once(
    """use gotoo_pixel_engine::{
    ActionId, Audio, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key,
    LocalStorage, MouseButton, Pixel, Rect, Size, SoundBank, SoundId, Touch, TouchPhase,
};
""",
    """use gotoo_pixel_engine::{
    ActionId, Audio, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton, Key,
    LocalStorage, MouseButton, Pixel, Rect, Size, SoundBank, SoundId, Touch, TouchPhase,
    ui::{VirtualButton, VirtualPad, VirtualPadUpdate},
};
""",
    "imports",
)

replace_once(
    """    }
}

#[derive(Debug)]
pub struct SnakeGame {
""",
    """    }
}

fn virtual_pad_for_mode(mode: SnakeInteractionMode) -> Option<VirtualPad> {
    let d_pad = SnakeLayout::for_mode(mode).d_pad?;
    Some(VirtualPad::new([
        VirtualButton::new(CONTROL_UP, d_pad.up),
        VirtualButton::new(CONTROL_RIGHT, d_pad.right),
        VirtualButton::new(CONTROL_DOWN, d_pad.down),
        VirtualButton::new(CONTROL_LEFT, d_pad.left),
    ]))
}

#[derive(Debug)]
pub struct SnakeGame {
""",
    "virtual pad factory",
)

replace_once(
    """    interaction_mode: SnakeInteractionMode,
    touch_controls: TouchControls,
    controls: ControlMap,
""",
    """    interaction_mode: SnakeInteractionMode,
    virtual_pad: Option<VirtualPad>,
    #[cfg(test)]
    touch_controls: TouchControls,
    controls: ControlMap,
""",
    "SnakeGame fields",
)

replace_once(
    """    pub fn new(interaction_mode: SnakeInteractionMode) -> Self {
        Self {
""",
    """    pub fn new(interaction_mode: SnakeInteractionMode) -> Self {
        let virtual_pad = virtual_pad_for_mode(interaction_mode);
        Self {
""",
    "constructor prelude",
)

replace_once(
    """            interaction_mode,
            touch_controls: TouchControls::default(),
            controls: default_controls(),
""",
    """            interaction_mode,
            virtual_pad,
            #[cfg(test)]
            touch_controls: TouchControls::default(),
            controls: default_controls(),
""",
    "constructor fields",
)

replace_once(
    """            if self.world.phase() != Phase::Running {
                self.touch_controls.reset_contact();
                break;
            }
""",
    """            if self.world.phase() != Phase::Running {
                self.reset_virtual_pad();
                #[cfg(test)]
                self.touch_controls.reset_contact();
                break;
            }
""",
    "game over input cleanup",
)

replace_once(
    """    fn restart(&mut self) {
        self.world.restart();
        self.accumulator = Duration::ZERO;
        self.touch_controls.reset_contact();
    }
""",
    """    fn restart(&mut self) {
        self.world.restart();
        self.accumulator = Duration::ZERO;
        self.reset_virtual_pad();
        #[cfg(test)]
        self.touch_controls.reset_contact();
    }

    fn reset_virtual_pad(&mut self) {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.reset(&mut self.controls);
        }
    }
""",
    "restart/reset",
)

replace_once(
    """        } else if self.touch_controls.visible() && layout.d_pad.is_some() {
            draw_d_pad(framebuffer, layout);
        }
""",
    """        } else if self
            .virtual_pad
            .as_ref()
            .is_some_and(VirtualPad::visible)
            && layout.d_pad.is_some()
        {
            draw_d_pad(framebuffer, layout);
        }
""",
    "touch pad visibility",
)

old_game_impl = """impl Game for SnakeGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.load_best_score_once(frame.storage);
        self.controls.update(frame.input);

        if self.controls.action(CONTROL_EXIT).pressed() {
            return GameResult::Exit;
        }

        let layout = self.layout();
        let controls = SnakeControls::from_frame(
            frame,
            self.world.phase(),
            &mut self.touch_controls,
            layout,
            &self.controls,
        );
        let events = self.update_logic(frame.delta_time, controls);
        self.play_sounds(frame.audio, events);
        self.persist_best_score_if_needed(frame.storage);
        self.draw(frame);

        GameResult::Continue
    }
}
"""
new_game_impl = """impl Game for SnakeGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.load_best_score_once(frame.storage);

        let phase = self.world.phase();
        let virtual_update = match phase {
            Phase::Running => self
                .virtual_pad
                .as_mut()
                .map(|virtual_pad| virtual_pad.update(frame.input, &mut self.controls)),
            Phase::GameOver => {
                self.reset_virtual_pad();
                None
            }
        };
        self.controls.update(frame.input);

        if self.controls.action(CONTROL_EXIT).pressed() {
            return GameResult::Exit;
        }

        let layout = self.layout();
        let controls = SnakeControls::from_frame(
            frame,
            phase,
            virtual_update.as_ref(),
            layout,
            &self.controls,
        );
        let events = self.update_logic(frame.delta_time, controls);
        self.play_sounds(frame.audio, events);
        self.persist_best_score_if_needed(frame.storage);
        self.draw(frame);

        GameResult::Continue
    }
}
"""
replace_once(old_game_impl, new_game_impl, "Game implementation")

start = text.index("impl SnakeControls {")
end = text.index("#[cfg(test)]\nfn direction_for_key", start)
new_snake_controls = """impl SnakeControls {
    fn none() -> Self {
        Self {
            directions: Vec::new(),
            restart: false,
        }
    }

    fn from_frame(
        frame: &Frame<'_>,
        phase: Phase,
        virtual_update: Option<&VirtualPadUpdate>,
        layout: SnakeLayout,
        controls: &ControlMap,
    ) -> Self {
        match phase {
            Phase::Running => Self::from_running_inputs(controls, virtual_update),
            Phase::GameOver => Self {
                directions: Vec::new(),
                restart: replay_requested(
                    controls.action(CONTROL_RESTART).pressed(),
                    frame.input.mouse_button(MouseButton::Left).pressed(),
                    frame.input.mouse_position(),
                    frame.input.touches(),
                    layout,
                ),
            },
        }
    }

    fn from_running_inputs(
        controls: &ControlMap,
        virtual_update: Option<&VirtualPadUpdate>,
    ) -> Self {
        let mut result = Self::none();
        let virtual_actions = virtual_update
            .map(VirtualPadUpdate::pressed_actions)
            .unwrap_or(&[]);

        for action in virtual_actions.iter().copied() {
            if let Some(direction) = direction_for_action(action) {
                result.directions.push(direction);
            }
        }

        for (action, direction) in [
            (CONTROL_UP, Direction::Up),
            (CONTROL_RIGHT, Direction::Right),
            (CONTROL_DOWN, Direction::Down),
            (CONTROL_LEFT, Direction::Left),
        ] {
            if controls.action(action).pressed() && !virtual_actions.contains(&action) {
                result.directions.push(direction);
            }
        }

        result.restart = controls.action(CONTROL_RESTART).pressed();
        result
    }

    #[cfg(test)]
    fn from_game_over_inputs(
        keyboard_restart_pressed: bool,
        mouse_left_pressed: bool,
        mouse_position: Option<(i32, i32)>,
        touches: &[Touch],
        touch_controls: &mut TouchControls,
    ) -> Self {
        Self::from_game_over_inputs_in_layout(
            keyboard_restart_pressed,
            mouse_left_pressed,
            mouse_position,
            touches,
            touch_controls,
            SnakeLayout::touch(),
        )
    }

    #[cfg(test)]
    fn from_game_over_inputs_in_layout(
        keyboard_restart_pressed: bool,
        mouse_left_pressed: bool,
        mouse_position: Option<(i32, i32)>,
        touches: &[Touch],
        touch_controls: &mut TouchControls,
        layout: SnakeLayout,
    ) -> Self {
        touch_controls.reset_contact();

        Self {
            directions: Vec::new(),
            restart: replay_requested(
                keyboard_restart_pressed,
                mouse_left_pressed,
                mouse_position,
                touches,
                layout,
            ),
        }
    }

    #[cfg(test)]
    fn with_directions(directions: impl IntoIterator<Item = Direction>) -> Self {
        Self {
            directions: directions.into_iter().collect(),
            restart: false,
        }
    }

    #[cfg(test)]
    fn restart() -> Self {
        Self {
            directions: Vec::new(),
            restart: true,
        }
    }
}

fn direction_for_action(action: ActionId) -> Option<Direction> {
    if action == CONTROL_UP {
        Some(Direction::Up)
    } else if action == CONTROL_RIGHT {
        Some(Direction::Right)
    } else if action == CONTROL_DOWN {
        Some(Direction::Down)
    } else if action == CONTROL_LEFT {
        Some(Direction::Left)
    } else {
        None
    }
}

"""
text = text[:start] + new_snake_controls + text[end:]

for old, new, label in [
    (
        "#[derive(Debug, Default, Clone, PartialEq, Eq)]\nstruct TouchControls",
        "#[cfg(test)]\n#[derive(Debug, Default, Clone, PartialEq, Eq)]\nstruct TouchControls",
        "TouchControls cfg",
    ),
    ("impl TouchControls", "#[cfg(test)]\nimpl TouchControls", "TouchControls impl cfg"),
    (
        "#[derive(Debug, Default, Clone, PartialEq, Eq)]\nstruct DPadTracker",
        "#[cfg(test)]\n#[derive(Debug, Default, Clone, PartialEq, Eq)]\nstruct DPadTracker",
        "DPadTracker cfg",
    ),
    ("impl DPadTracker", "#[cfg(test)]\nimpl DPadTracker", "DPadTracker impl cfg"),
    (
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nstruct DPadZone",
        "#[cfg(test)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nstruct DPadZone",
        "DPadZone cfg",
    ),
    (
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nstruct DPadContact",
        "#[cfg(test)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nstruct DPadContact",
        "DPadContact cfg",
    ),
    (
        "fn d_pad_zone_at_in_layout(position: (i32, i32), layout: SnakeLayout) -> Option<DPadZone>",
        "#[cfg(test)]\nfn d_pad_zone_at_in_layout(position: (i32, i32), layout: SnakeLayout) -> Option<DPadZone>",
        "d_pad_zone_at_in_layout cfg",
    ),
]:
    replace_once(old, new, label)

path.write_text(text)
