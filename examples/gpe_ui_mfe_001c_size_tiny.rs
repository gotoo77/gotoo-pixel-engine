#![deny(warnings)]

use std::hint::black_box;

use gotoo_pixel_engine::{
    Size,
    ui::{
        UiTheme,
        experimental::{self, UiNavInput, UiStateStore},
    },
};

fn main() {
    let mut state = UiStateStore::default();
    let (output, ()) = experimental::run_headless(
        Size {
            width: 320,
            height: 180,
        },
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| {
            ui.panel(|ui| {
                ui.text("PAUSED");
                let _ = ui.button("RESUME");
            });
        },
    );

    black_box(output.metrics());
}
