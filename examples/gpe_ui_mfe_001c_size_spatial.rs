#![deny(warnings)]

use std::hint::black_box;

use gotoo_pixel_engine::{
    ActionId, Rect, Size,
    ui::{
        UiTheme,
        experimental::{self, UiId, UiNavInput, UiStateStore},
        experimental_spatial::{GridSpec, SpatialCard, SpatialInput, SpatialState, run_card_grid_headless},
    },
};

const ACTION: ActionId = ActionId::new("mfe001c.size");

fn main() {
    let ids = stable_ids();
    let cards = ids.map(|id| SpatialCard {
        id,
        title: "CARD",
        subtitle: "PROBE",
        image: None,
        action: ACTION,
    });

    let mut state = SpatialState::default();
    let output = run_card_grid_headless(
        Rect {
            x: 0,
            y: 0,
            width: 464,
            height: 174,
        },
        &mut state,
        SpatialInput::default(),
        GridSpec {
            min_cell_width: 118,
            preferred_cell_height: 78,
            gap: 8,
            padding: 6,
        },
        &cards,
    );

    black_box(output.layouts().len());
}

fn stable_ids() -> [UiId; 6] {
    let mut state = UiStateStore::default();
    let (_, ids) = experimental::run_headless(
        Size {
            width: 320,
            height: 180,
        },
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| {
            ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot"]
                .map(|key| ui.keyed(key, |ui| ui.button(key).id()))
        },
    );
    ids
}
