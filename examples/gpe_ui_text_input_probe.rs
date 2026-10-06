use std::cell::RefCell;

use gotoo_pixel_engine::{
    EngineConfig, Frame, Game, GameResult, Pixel, Rect, run,
    ui::{TextInputEnterHint, TextInputOptions, Ui, UiState, UiTheme},
};
use wasm_bindgen::prelude::*;

const WIDTH: u32 = 480;
const HEIGHT: u32 = 270;

thread_local! {
    static SNAPSHOT: RefCell<String> = const { RefCell::new(String::new()) };
}

struct TextInputProbe {
    ui_state: UiState,
    search: String,
    player_name: String,
    search_editing: bool,
    search_submits: u32,
    player_submits: u32,
}

impl TextInputProbe {
    fn new() -> Self {
        Self {
            ui_state: UiState::default(),
            search: String::new(),
            player_name: String::new(),
            search_editing: false,
            search_submits: 0,
            player_submits: 0,
        }
    }

    fn publish_snapshot(&self) {
        SNAPSHOT.with(|snapshot| {
            *snapshot.borrow_mut() = format!(
                "search={};player={};search_submits={};player_submits={}",
                self.search, self.player_name, self.search_submits, self.player_submits
            );
        });
    }
}

impl Game for TextInputProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        frame.framebuffer.clear(Pixel::rgb(8, 12, 18));
        frame.framebuffer.draw_text_scaled(
            24,
            20,
            "GPE.UI TEXT INPUT",
            2,
            Pixel::rgb(105, 238, 184),
        );
        frame
            .framebuffer
            .draw_text_scaled(24, 70, "SEARCH", 1, Pixel::WHITE);
        frame
            .framebuffer
            .draw_text_scaled(24, 150, "PLAYER NAME", 1, Pixel::WHITE);

        let (search_response, player_response) = {
            let mut ui = Ui::new(
                frame.framebuffer,
                frame.input,
                frame.delta_time,
                &mut self.ui_state,
                UiTheme::default(),
            );

            let search = ui.text_input_at(
                Rect {
                    x: 24,
                    y: 88,
                    width: 432,
                    height: 40,
                },
                &mut self.search,
                TextInputOptions {
                    placeholder: "SEARCH",
                    aria_label: "Search games",
                    max_chars: Some(48),
                    enter_hint: TextInputEnterHint::Search,
                    editing: self.search_editing,
                },
            );

            let player = ui.text_input_at(
                Rect {
                    x: 24,
                    y: 168,
                    width: 432,
                    height: 40,
                },
                &mut self.player_name,
                TextInputOptions {
                    placeholder: "PLAYER NAME",
                    aria_label: "Player name",
                    max_chars: Some(20),
                    enter_hint: TextInputEnterHint::Done,
                    editing: true,
                },
            );

            (search, player)
        };

        if search_response.editing {
            self.search_editing = !search_response.submitted;
        }
        if search_response.submitted {
            self.search_submits = self.search_submits.saturating_add(1);
        }
        if player_response.submitted {
            self.player_submits = self.player_submits.saturating_add(1);
        }

        self.publish_snapshot();
        GameResult::Continue
    }
}

#[wasm_bindgen]
pub fn gpe_ui_text_input_probe_snapshot() -> String {
    SNAPSHOT.with(|snapshot| snapshot.borrow().clone())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    run(
        EngineConfig {
            title: "GPE.UI text input probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: 960,
            window_height: 540,
        },
        TextInputProbe::new(),
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))
}
