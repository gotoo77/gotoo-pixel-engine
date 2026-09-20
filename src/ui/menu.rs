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


/// Pages for the minimal pause/settings/audio reference menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPage {
    Pause,
    Settings,
    Audio,
}

/// Domain intent returned to the session owner; the UI never resumes or exits a game itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsIntent {
    None,
    Resume,
    Exit,
}

/// Consumer-owned audio values. Nothing is applied to an audio backend until
/// the session owner explicitly does so.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioSettings {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 1.0,
            sfx: 1.0,
        }
    }
}

/// Reusable reference composition. Call it once per frame while the session is
/// paused; the caller owns simulation suspension, input adaptation and exit.
pub struct PauseSettingsMenu {
    pages: MenuStack<SettingsPage>,
    pub audio: AudioSettings,
}

impl Default for PauseSettingsMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl PauseSettingsMenu {
    pub fn new() -> Self {
        Self {
            pages: MenuStack::new(SettingsPage::Pause),
            audio: AudioSettings::default(),
        }
    }

    pub fn current(&self) -> SettingsPage {
        *self.pages.current()
    }

    pub fn update(
        &mut self,
        framebuffer: &mut crate::Framebuffer,
        nav: super::experimental::UiNavInput,
        theme: super::UiTheme,
    ) -> SettingsIntent {
        use super::experimental::run;
        let mut intent = SettingsIntent::None;
        let mut next = None;
        let mut back = nav.cancel;
        match self.current() {
            SettingsPage::Pause => {
                let (output, (resume, settings, quit)) =
                    run(framebuffer, self.pages.ui_state_mut(), nav, theme, |ui| {
                        ui.text("PAUSED");
                        (
                            ui.keyed("resume", |ui| ui.button("RESUME")),
                            ui.keyed("settings", |ui| ui.button("SETTINGS")),
                            ui.keyed("quit", |ui| ui.button("QUIT")),
                        )
                    });
                if output.activated(resume) {
                    intent = SettingsIntent::Resume;
                } else if output.activated(settings) {
                    next = Some(SettingsPage::Settings);
                } else if output.activated(quit) {
                    intent = SettingsIntent::Exit;
                }
                back |= output.cancel_requested();
            }
            SettingsPage::Settings => {
                let (output, (audio, previous)) =
                    run(framebuffer, self.pages.ui_state_mut(), nav, theme, |ui| {
                        ui.text("SETTINGS");
                        (
                            ui.keyed("audio", |ui| ui.button("AUDIO")),
                            ui.keyed("back", |ui| ui.button("BACK")),
                        )
                    });
                if output.activated(audio) {
                    next = Some(SettingsPage::Audio);
                } else if output.activated(previous) {
                    back = true;
                }
                back |= output.cancel_requested();
            }
            SettingsPage::Audio => {
                let (output, (master, music, sfx, previous)) =
                    run(framebuffer, self.pages.ui_state_mut(), nav, theme, |ui| {
                        ui.text("AUDIO");
                        (
                            ui.keyed("master", |ui| {
                                ui.slider_f32("MASTER", self.audio.master, 0.0..=1.0, 0.05)
                            }),
                            ui.keyed("music", |ui| {
                                ui.slider_f32("MUSIC", self.audio.music, 0.0..=1.0, 0.05)
                            }),
                            ui.keyed("sfx", |ui| {
                                ui.slider_f32("SFX", self.audio.sfx, 0.0..=1.0, 0.05)
                            }),
                            ui.keyed("back", |ui| ui.button("BACK")),
                        )
                    });
                if let Some(value) = output.changed(master) {
                    self.audio.master = value;
                }
                if let Some(value) = output.changed(music) {
                    self.audio.music = value;
                }
                if let Some(value) = output.changed(sfx) {
                    self.audio.sfx = value;
                }
                back |= output.activated(previous) || output.cancel_requested();
            }
        }
        if back {
            if self.pages.back() == MenuBack::CloseRequested {
                return SettingsIntent::Resume;
            }
        } else if let Some(page) = next {
            self.pages.push(page);
        }
        intent
    }

    /// Apply values explicitly: errors are reported to the caller rather than
    /// silently falling back when a backend lacks volume control.
    pub fn apply_audio(&self, audio: &mut dyn crate::Audio) -> Result<(), crate::AudioError> {
        audio.set_master_volume(self.audio.master)?;
        audio.set_bus_volume(crate::AudioBus::Music, self.audio.music)?;
        audio.set_bus_volume(crate::AudioBus::Sfx, self.audio.sfx)?;
        Ok(())
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
