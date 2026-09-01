#![deny(warnings)]

use gotoo_pixel_engine::{
    ActionId, EngineConfig, Frame, Game, GameResult, Key, Pixel, TextRenderer, run,
    ui::{
        UiTheme,
        experimental::{self, UiNavInput, UiStateStore, WidgetRef},
    },
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 224;
const RESET: ActionId = ActionId::new("mfe001a.reset");

#[derive(Debug, Clone, Copy)]
struct Settings {
    enabled: bool,
    volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 0.65,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Controls {
    enabled: WidgetRef<bool>,
    volume: WidgetRef<f32>,
}

#[derive(Debug, Clone, Copy)]
struct LastSliderTransaction {
    snapshot: f32,
    proposed: f32,
}

struct Mfe001aProbe {
    settings: Settings,
    ui_state: UiStateStore,
    last_slider_transaction: Option<LastSliderTransaction>,
}

impl Mfe001aProbe {
    fn nav(frame: &Frame<'_>) -> UiNavInput {
        UiNavInput {
            up: frame.input.key(Key::Up).pressed() || frame.input.key(Key::W).pressed(),
            down: frame.input.key(Key::Down).pressed() || frame.input.key(Key::S).pressed(),
            left: frame.input.key(Key::Left).pressed() || frame.input.key(Key::A).pressed(),
            right: frame.input.key(Key::Right).pressed() || frame.input.key(Key::D).pressed(),
            confirm: frame.input.key(Key::Space).pressed(),
            cancel: false,
        }
    }
}

impl Game for Mfe001aProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        frame.framebuffer.clear(Pixel::rgb(6, 9, 14));

        let snapshot = self.settings;
        let nav = Self::nav(frame);
        let theme = UiTheme::default();

        let (output, controls) = experimental::run(
            frame.framebuffer,
            &mut self.ui_state,
            nav,
            theme,
            |ui| {
                ui.panel(|ui| {
                    ui.text("GPE.UI MFE-001A — T1 TRANSACTION");
                    ui.text("UP/DOWN: FOCUS   LEFT/RIGHT: SLIDER   SPACE: ACTIVATE");
                    ui.text(format!(
                        "UI SNAPSHOT: ENABLED={}  VOLUME={:.0}%",
                        snapshot.enabled,
                        snapshot.volume * 100.0
                    ));

                    ui.column(|ui| {
                        let enabled = ui.keyed("enabled", |ui| {
                            ui.toggle("ENABLED", snapshot.enabled)
                        });
                        let volume = ui.keyed("volume", |ui| {
                            ui.slider_f32("VOLUME", snapshot.volume, 0.0..=1.0, 0.05)
                        });
                        ui.keyed("reset", |ui| ui.button_action("RESET", RESET));
                        Controls { enabled, volume }
                    })
                })
            },
        );

        if let Some(value) = output.changed(controls.enabled) {
            self.settings.enabled = value;
        }

        if let Some(value) = output.changed(controls.volume) {
            self.last_slider_transaction = Some(LastSliderTransaction {
                snapshot: snapshot.volume,
                proposed: value,
            });
            self.settings.volume = value;
        }

        if output.action_pressed(RESET) {
            self.settings = Settings::default();
        }

        let text = TextRenderer::default();
        let info = Pixel::rgb(160, 205, 235);
        let accent = Pixel::rgb(245, 195, 90);

        text.draw(
            frame.framebuffer,
            8,
            176,
            &format!(
                "AUTHORITATIVE AFTER RUN: ENABLED={} VOLUME={:.0}%",
                self.settings.enabled,
                self.settings.volume * 100.0
            ),
            info,
        );

        if let Some(last) = self.last_slider_transaction {
            text.draw(
                frame.framebuffer,
                8,
                190,
                &format!(
                    "LAST T1: SNAPSHOT {:.0}% -> EFFECTIVE/PROPOSED {:.0}%",
                    last.snapshot * 100.0,
                    last.proposed * 100.0
                ),
                accent,
            );
            text.draw(
                frame.framebuffer,
                8,
                202,
                "QUESTION: DOES THIS ONE-TRANSACTION MODEL FEEL OK?",
                accent,
            );
        } else {
            text.draw(
                frame.framebuffer,
                8,
                190,
                "FOCUS VOLUME, THEN PRESS LEFT/RIGHT TO CREATE A T1 PROPOSAL",
                accent,
            );
        }

        GameResult::Continue
    }
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE.UI MFE-001A — T1 Transaction Probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 3,
            window_height: HEIGHT * 3,
        },
        Mfe001aProbe {
            settings: Settings::default(),
            ui_state: UiStateStore::default(),
            last_slider_transaction: None,
        },
    )
}
