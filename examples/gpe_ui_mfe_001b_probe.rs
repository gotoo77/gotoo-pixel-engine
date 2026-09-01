#![deny(warnings)]

use gotoo_pixel_engine::{
    ActionId, EngineConfig, Frame, Game, GameResult, GamepadButton, Image, Key, MouseButton, Pixel,
    Rect, Size, TextRenderer, run,
    ui::{
        UiTheme,
        experimental::{self, UiId, UiNavInput, UiStateStore},
        experimental_spatial::{
            CardLayout, CardPainter, CardVisualState, DefaultCardPainter, GridSpec, PointerInput,
            SpatialCard, SpatialInput, SpatialState, run_card_grid,
        },
    },
};

const WIDTH: u32 = 480;
const HEIGHT: u32 = 270;

const LAUNCH_A: ActionId = ActionId::new("mfe001b.launch.a");
const LAUNCH_B: ActionId = ActionId::new("mfe001b.launch.b");
const LAUNCH_C: ActionId = ActionId::new("mfe001b.launch.c");
const LAUNCH_D: ActionId = ActionId::new("mfe001b.launch.d");
const LAUNCH_E: ActionId = ActionId::new("mfe001b.launch.e");
const LAUNCH_F: ActionId = ActionId::new("mfe001b.launch.f");

#[derive(Debug, Clone, Copy)]
struct Entry {
    key: &'static str,
    title: &'static str,
    subtitle: &'static str,
    action: ActionId,
}

const ENTRIES: [Entry; 6] = [
    Entry {
        key: "alpha",
        title: "ALPHA",
        subtitle: "ACTION",
        action: LAUNCH_A,
    },
    Entry {
        key: "bravo",
        title: "BRAVO",
        subtitle: "PUZZLE",
        action: LAUNCH_B,
    },
    Entry {
        key: "charlie",
        title: "CHARLIE",
        subtitle: "ARCADE",
        action: LAUNCH_C,
    },
    Entry {
        key: "delta",
        title: "DELTA",
        subtitle: "TACTIC",
        action: LAUNCH_D,
    },
    Entry {
        key: "echo",
        title: "ECHO",
        subtitle: "SHMUP",
        action: LAUNCH_E,
    },
    Entry {
        key: "foxtrot",
        title: "FOXTROT",
        subtitle: "CUSTOM",
        action: LAUNCH_F,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WidthMode {
    Small,
    Medium,
    Wide,
}

impl WidthMode {
    fn next(self) -> Self {
        match self {
            Self::Small => Self::Medium,
            Self::Medium => Self::Wide,
            Self::Wide => Self::Small,
        }
    }

    const fn width(self) -> u32 {
        match self {
            Self::Small => 190,
            Self::Medium => 330,
            Self::Wide => 464,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Small => "SMALL",
            Self::Medium => "MEDIUM",
            Self::Wide => "WIDE",
        }
    }
}

struct ProbeCardPainter;

impl CardPainter for ProbeCardPainter {
    fn paint(
        &self,
        framebuffer: &mut gotoo_pixel_engine::Framebuffer,
        card: &SpatialCard<'_>,
        layout: CardLayout,
        visual: CardVisualState,
        theme: UiTheme,
    ) {
        const COMPACT_HEIGHT: u32 = 44;

        if layout.rect.height >= COMPACT_HEIGHT {
            DefaultCardPainter.paint(framebuffer, card, layout, visual, theme);
        } else {
            let border = if visual.focused || visual.hovered {
                theme.accent
            } else {
                theme.border
            };
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                layout.rect.width,
                layout.rect.height,
                theme.control_background,
            );
            framebuffer.draw_rect(
                layout.rect.x,
                layout.rect.y,
                layout.rect.width,
                layout.rect.height,
                border,
            );

            let text = TextRenderer::new(theme.font);
            let (text_width, text_height) = text.text_size(card.title, theme.text_scale.max(1));
            let text_x = layout.rect.x.saturating_add(
                i32::try_from(layout.rect.width.saturating_sub(text_width) / 2).unwrap_or(i32::MAX),
            );
            let text_y = layout.rect.y.saturating_add(
                i32::try_from(layout.rect.height.saturating_sub(text_height) / 2)
                    .unwrap_or(i32::MAX),
            );
            text.draw_scaled(
                framebuffer,
                text_x,
                text_y,
                card.title,
                theme.text_scale.max(1),
                theme.text,
            );
        }

        if visual.focused {
            framebuffer.fill_rect(
                layout.rect.x,
                layout.rect.y,
                4_u32.min(layout.rect.width),
                layout.rect.height,
                theme.accent,
            );
        }
    }
}

struct Mfe001bProbe {
    ids: Vec<UiId>,
    images: Vec<Image>,
    spatial_state: SpatialState,
    filtered: bool,
    reversed: bool,
    width_mode: WidthMode,
    last_action: Option<&'static str>,
}

impl Mfe001bProbe {
    fn new() -> Self {
        Self {
            ids: stable_ids(),
            images: probe_images(),
            spatial_state: SpatialState::default(),
            filtered: false,
            reversed: false,
            width_mode: WidthMode::Wide,
            last_action: None,
        }
    }

    fn nav(frame: &Frame<'_>) -> UiNavInput {
        UiNavInput {
            up: frame.input.key(Key::Up).pressed()
                || frame.input.key(Key::W).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::DPadUp)
                    .pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::LeftStickUp)
                    .pressed(),
            down: frame.input.key(Key::Down).pressed()
                || frame.input.key(Key::S).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::DPadDown)
                    .pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::LeftStickDown)
                    .pressed(),
            left: frame.input.key(Key::Left).pressed()
                || frame.input.key(Key::A).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::DPadLeft)
                    .pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::LeftStickLeft)
                    .pressed(),
            right: frame.input.key(Key::Right).pressed()
                || frame.input.key(Key::D).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::DPadRight)
                    .pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::LeftStickRight)
                    .pressed(),
            confirm: frame.input.key(Key::Space).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::South)
                    .pressed(),
            cancel: frame.input.key(Key::Escape).pressed()
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::East)
                    .pressed(),
        }
    }
}

impl Game for Mfe001bProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::East)
                .pressed()
        {
            return GameResult::Exit;
        }
        if frame.input.key(Key::F).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::West)
                .pressed()
        {
            self.filtered = !self.filtered;
        }
        if frame.input.key(Key::R).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::North)
                .pressed()
        {
            self.reversed = !self.reversed;
        }
        if frame.input.key(Key::L).pressed()
            || frame
                .input
                .gamepad_button_any(GamepadButton::LeftShoulder)
                .pressed()
        {
            self.width_mode = self.width_mode.next();
        }

        frame.framebuffer.clear(Pixel::rgb(6, 9, 14));

        let mut indices = (0..ENTRIES.len())
            .filter(|index| !self.filtered || index % 2 == 0)
            .collect::<Vec<_>>();
        if self.reversed {
            indices.reverse();
        }

        let ids = &self.ids;
        let images = &self.images;
        let cards = indices
            .iter()
            .copied()
            .map(|index| SpatialCard {
                id: ids[index],
                title: ENTRIES[index].title,
                subtitle: ENTRIES[index].subtitle,
                image: Some(&images[index]),
                action: ENTRIES[index].action,
            })
            .collect::<Vec<_>>();

        let grid_width = self.width_mode.width();
        let grid_x = ((WIDTH.saturating_sub(grid_width)) / 2) as i32;
        let bounds = Rect {
            x: grid_x,
            y: 48,
            width: grid_width,
            height: 174,
        };
        let input = SpatialInput {
            nav: Self::nav(frame),
            pointer: PointerInput {
                position: frame.input.mouse_position(),
                pressed: frame.input.mouse_button(MouseButton::Left).pressed(),
                released: frame.input.mouse_button(MouseButton::Left).released(),
            },
            touches: frame.input.touches(),
        };
        let spec = GridSpec {
            min_cell_width: 118,
            preferred_cell_height: 78,
            gap: 8,
            padding: 6,
        };
        let output = run_card_grid(
            frame.framebuffer,
            bounds,
            &mut self.spatial_state,
            input,
            spec,
            UiTheme::default(),
            &cards,
            &ProbeCardPainter,
        );

        for entry in ENTRIES {
            if output.action_pressed(entry.action) {
                self.last_action = Some(entry.title);
            }
        }

        let text = TextRenderer::default();
        let normal = Pixel::rgb(190, 205, 225);
        let info = Pixel::rgb(130, 200, 235);
        let accent = Pixel::rgb(245, 195, 90);

        text.draw(
            frame.framebuffer,
            8,
            6,
            "GPE.UI MFE-001B - RESPONSIVE / SPATIAL / CUSTOM CARD",
            normal,
        );
        text.draw(
            frame.framebuffer,
            8,
            18,
            "ARROWS/WASD + PAD: FOCUS   SPACE/SOUTH: ACTIVATE   MOUSE/TOUCH: CARD",
            normal,
        );
        text.draw(
            frame.framebuffer,
            8,
            30,
            "L/LB: WIDTH   F/WEST: FILTER   R/NORTH: REORDER   ESC/EAST: EXIT",
            normal,
        );
        text.draw(
            frame.framebuffer,
            8,
            232,
            &format!(
                "WIDTH={} {}PX  GRID={} COL x {} ROW  CARDS={}  FILTER={}  REVERSE={}",
                self.width_mode.label(),
                grid_width,
                output.columns(),
                output.rows(),
                cards.len(),
                self.filtered,
                self.reversed
            ),
            info,
        );
        text.draw(
            frame.framebuffer,
            8,
            244,
            &format!(
                "FOCUS={:?}  HOVER={:?}",
                output.focused_id().map(UiId::raw),
                output.hovered_id().map(UiId::raw)
            ),
            info,
        );
        text.draw(
            frame.framebuffer,
            8,
            256,
            match self.last_action {
                Some(title) => title,
                None => "ACTIVATE A CARD - RESULT APPEARS HERE",
            },
            accent,
        );

        GameResult::Continue
    }
}

fn stable_ids() -> Vec<UiId> {
    let mut state = UiStateStore::default();
    let (_, ids) = experimental::run_headless(
        Size {
            width: WIDTH,
            height: HEIGHT,
        },
        &mut state,
        UiNavInput::default(),
        UiTheme::default(),
        |ui| {
            ENTRIES
                .iter()
                .map(|entry| ui.keyed(entry.key, |ui| ui.button(entry.title).id()))
                .collect::<Vec<_>>()
        },
    );
    ids
}

fn probe_images() -> Vec<Image> {
    [
        (55, 120, 210),
        (170, 70, 180),
        (45, 160, 125),
        (200, 105, 45),
        (70, 150, 200),
        (180, 65, 90),
    ]
    .into_iter()
    .map(|(r, g, b)| checker_image(r, g, b))
    .collect()
}

fn checker_image(r: u8, g: u8, b: u8) -> Image {
    let width = 16_u32;
    let height = 10_u32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let boost = if (x / 2 + y / 2) % 2 == 0 { 20 } else { 0 };
            rgba.extend_from_slice(&[
                r.saturating_add(boost),
                g.saturating_add(boost),
                b.saturating_add(boost),
                255,
            ]);
        }
    }
    Image::from_rgba8(width, height, rgba).expect("probe image dimensions are valid")
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    run(
        EngineConfig {
            title: "GPE.UI MFE-001B - Spatial Card Grid Probe".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH * 2,
            window_height: HEIGHT * 2,
        },
        Mfe001bProbe::new(),
    )
}
