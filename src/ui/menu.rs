//! Hierarchical navigation for transactional GPE.UI menus.
//!
//! Each page owns its interaction state. Domain actions and menu presentation
//! remain owned by the consumer.

use super::experimental::UiStateStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuBack {
    ReturnedToParent,
    CloseRequested,
}

struct MenuPage<Id> {
    id: Id,
    ui_state: UiStateStore,
}

/// A root-preserving stack of menu pages.
///
/// Consumers render the active page using `ui_state_mut()`, then commit
/// navigation requests after the UI transaction has completed.
pub struct MenuStack<Id> {
    pages: Vec<MenuPage<Id>>,
}

impl<Id> MenuStack<Id> {
    pub fn new(root: Id) -> Self {
        Self {
            pages: vec![MenuPage {
                id: root,
                ui_state: UiStateStore::default(),
            }],
        }
    }

    pub fn current(&self) -> &Id {
        &self.pages.last().expect("menu root is always present").id
    }

    pub fn depth(&self) -> usize {
        self.pages.len()
    }

    pub fn ui_state_mut(&mut self) -> &mut UiStateStore {
        &mut self
            .pages
            .last_mut()
            .expect("menu root is always present")
            .ui_state
    }

    pub fn push(&mut self, id: Id) {
        self.pages.push(MenuPage {
            id,
            ui_state: UiStateStore::default(),
        });
    }

    pub fn back(&mut self) -> MenuBack {
        if self.pages.len() == 1 {
            MenuBack::CloseRequested
        } else {
            self.pages.pop();
            MenuBack::ReturnedToParent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Size,
        ui::UiTheme,
        ui::experimental::{UiNavInput, run_headless},
    };

    fn surface() -> Size {
        Size {
            width: 320,
            height: 180,
        }
    }

    #[test]
    fn navigating_back_preserves_root_and_parent_identity() {
        let mut stack = MenuStack::new("pause");
        assert_eq!(stack.current(), &"pause");
        assert_eq!(stack.back(), MenuBack::CloseRequested);
        stack.push("settings");
        stack.push("audio");
        assert_eq!(stack.depth(), 3);
        assert_eq!(stack.back(), MenuBack::ReturnedToParent);
        assert_eq!(stack.current(), &"settings");
        assert_eq!(stack.back(), MenuBack::ReturnedToParent);
        assert_eq!(stack.current(), &"pause");
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn parent_focus_is_restored_after_visiting_child_page() {
        let mut stack = MenuStack::new("pause");
        let render = |state: &mut UiStateStore, nav: UiNavInput| {
            let (output, _) = run_headless(surface(), state, nav, UiTheme::default(), |ui| {
                ui.keyed("resume", |ui| ui.button("RESUME"));
                ui.keyed("settings", |ui| ui.button("SETTINGS"));
                ui.keyed("quit", |ui| ui.button("QUIT"));
            });
            output
        };

        render(stack.ui_state_mut(), UiNavInput::default());
        let selected = render(
            stack.ui_state_mut(),
            UiNavInput {
                down: true,
                ..UiNavInput::default()
            },
        );
        let parent_focus = selected.focused_id().expect("settings should have focus");

        stack.push("settings");
        let (child_output, _) = run_headless(
            surface(),
            stack.ui_state_mut(),
            UiNavInput::default(),
            UiTheme::default(),
            |ui| {
                ui.keyed("audio", |ui| ui.button("AUDIO"));
            },
        );
        assert!(child_output.focused_id().is_some());

        assert_eq!(stack.back(), MenuBack::ReturnedToParent);
        let restored = render(stack.ui_state_mut(), UiNavInput::default());
        assert_eq!(restored.focused_id(), Some(parent_focus));
    }
}
