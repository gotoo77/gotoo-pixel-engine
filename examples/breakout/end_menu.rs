use gotoo_pixel_engine::{
    ControlMap, Input, Size,
    ui::{
        UiTheme,
        experimental::{self, UiId, UiNavInput, UiStateStore},
        menu_confirm_pressed, menu_down_pressed, menu_up_pressed,
    },
};

use super::{ACTION, MOVE_LEFT, MOVE_RIGHT};

const END_MENU_SURFACE: Size = Size {
    width: super::FRAMEBUFFER_WIDTH,
    height: super::FRAMEBUFFER_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EndMenuAction {
    Replay,
    Quit,
}

#[derive(Debug, Clone, Copy)]
struct EndMenuIds {
    replay: UiId,
    quit: UiId,
}

pub(super) struct EndMenuState {
    ui: UiStateStore,
    ids: EndMenuIds,
}

impl EndMenuState {
    pub(super) fn new() -> Self {
        Self {
            ui: UiStateStore::default(),
            ids: end_menu_ids(),
        }
    }

    pub(super) fn reset(&mut self) {
        self.ui = UiStateStore::default();
        let _ = run_end_menu(&mut self.ui, UiNavInput::default());
    }

    pub(super) fn update(&mut self, input: &Input, controls: &ControlMap) -> Option<EndMenuAction> {
        run_end_menu(
            &mut self.ui,
            UiNavInput {
                up: menu_up_pressed(input) || controls.action(MOVE_LEFT).pressed(),
                down: menu_down_pressed(input) || controls.action(MOVE_RIGHT).pressed(),
                confirm: menu_confirm_pressed(input) || controls.action(ACTION).pressed(),
                ..UiNavInput::default()
            },
        )
    }

    pub(super) fn focused(&self, action: EndMenuAction) -> bool {
        self.ui.focused_id()
            == Some(match action {
                EndMenuAction::Replay => self.ids.replay,
                EndMenuAction::Quit => self.ids.quit,
            })
    }
}

fn run_end_menu(state: &mut UiStateStore, nav: UiNavInput) -> Option<EndMenuAction> {
    let (output, (replay, quit)) =
        experimental::run_headless(END_MENU_SURFACE, state, nav, UiTheme::default(), |ui| {
            ui.column(|ui| {
                let replay = ui.keyed("replay", |ui| ui.button("REPLAY"));
                let quit = ui.keyed("quit", |ui| ui.button("QUIT"));
                (replay, quit)
            })
        });

    if output.activated(replay) {
        Some(EndMenuAction::Replay)
    } else if output.activated(quit) {
        Some(EndMenuAction::Quit)
    } else {
        None
    }
}

fn end_menu_ids() -> EndMenuIds {
    let mut state = UiStateStore::default();
    let (_, (replay, quit)) = experimental::run_headless(
        END_MENU_SURFACE,
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| {
            ui.column(|ui| {
                let replay = ui.keyed("replay", |ui| ui.button("REPLAY"));
                let quit = ui.keyed("quit", |ui| ui.button("QUIT"));
                (replay, quit)
            })
        },
    );
    EndMenuIds {
        replay: replay.id(),
        quit: quit.id(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_focus_is_replay() {
        let mut menu = EndMenuState::new();
        menu.reset();
        assert!(menu.focused(EndMenuAction::Replay));
        assert!(!menu.focused(EndMenuAction::Quit));
    }

    #[test]
    fn linear_navigation_reaches_quit_and_wraps() {
        let mut menu = EndMenuState::new();
        menu.reset();

        let _ = run_end_menu(
            &mut menu.ui,
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
        );
        assert!(menu.focused(EndMenuAction::Quit));

        let _ = run_end_menu(
            &mut menu.ui,
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
        );
        assert!(menu.focused(EndMenuAction::Replay));
    }

    #[test]
    fn confirm_activates_focused_semantic_item() {
        let mut menu = EndMenuState::new();
        menu.reset();
        assert_eq!(
            run_end_menu(
                &mut menu.ui,
                UiNavInput {
                    confirm: true,
                    ..UiNavInput::default()
                },
            ),
            Some(EndMenuAction::Replay)
        );

        let _ = run_end_menu(
            &mut menu.ui,
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
        );
        assert_eq!(
            run_end_menu(
                &mut menu.ui,
                UiNavInput {
                    confirm: true,
                    ..UiNavInput::default()
                },
            ),
            Some(EndMenuAction::Quit)
        );
    }

    #[test]
    fn keyed_identity_is_stable_across_rebuilds() {
        let first = end_menu_ids();
        let second = end_menu_ids();
        assert_eq!(first.replay, second.replay);
        assert_eq!(first.quit, second.quit);
    }
}
