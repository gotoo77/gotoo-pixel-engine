use std::{cell::RefCell, time::Duration};

use gotoo_pixel_engine::{
    Framebuffer, Input, Pixel, Rect,
    ui::{TextInputEnterHint, TextInputOptions, Ui, UiState, UiTheme},
};
use wasm_bindgen::prelude::*;

const WIDTH: u32 = 480;
const HEIGHT: u32 = 270;

thread_local! {
    static PROBE: RefCell<TextInputProbe> = RefCell::new(TextInputProbe::new());
}

struct TextInputProbe {
    framebuffer: Framebuffer,
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
            framebuffer: Framebuffer::new(WIDTH, HEIGHT),
            ui_state: UiState::default(),
            search: String::new(),
            player_name: String::new(),
            search_editing: false,
            search_submits: 0,
            player_submits: 0,
        }
    }

    fn tick(&mut self) {
        self.framebuffer.clear(Pixel::rgb(8, 12, 18));
        self.framebuffer.draw_text_scaled(
            24,
            20,
            "GPE.UI TEXT INPUT",
            2,
            Pixel::rgb(105, 238, 184),
        );
        self.framebuffer
            .draw_text_scaled(24, 70, "SEARCH", 1, Pixel::WHITE);
        self.framebuffer
            .draw_text_scaled(24, 150, "PLAYER NAME", 1, Pixel::WHITE);

        let input = Input::default();
        let (search_response, player_response) = {
            let mut ui = Ui::new(
                &mut self.framebuffer,
                &input,
                Duration::ZERO,
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
    }

    fn snapshot(&self) -> String {
        format!(
            "search={};player={};search_submits={};player_submits={}",
            self.search, self.player_name, self.search_submits, self.player_submits
        )
    }
}

#[wasm_bindgen]
pub fn gpe_ui_text_input_probe_tick() {
    PROBE.with(|probe| probe.borrow_mut().tick());
}

#[wasm_bindgen]
pub fn gpe_ui_text_input_probe_snapshot() -> String {
    PROBE.with(|probe| probe.borrow().snapshot())
}

#[wasm_bindgen(start)]
pub fn start() {
    gpe_ui_text_input_probe_tick();
}
