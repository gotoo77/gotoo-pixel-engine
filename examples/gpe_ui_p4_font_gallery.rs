use gotoo_pixel_engine::{
    EngineConfig, Frame, Framebuffer, Game, GameResult, Key, MouseButton, Pixel, Rect,
    TextInputEvent, TextRenderer,
    outline_text::OutlineFont,
    run,
    ui::{
        UiTheme,
        experimental::{
            self, UiGridSpec, UiId, UiInput, UiNavInput, UiPointerInput, UiStateStore,
        },
    },
};

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 800;
const REPEAT_INITIAL_DELAY_S: f32 = 0.300;
const REPEAT_INTERVAL_S: f32 = 0.060;

macro_rules! face {
    ($name:literal, $dir:literal) => {
        (
            $name,
            include_bytes!(concat!("../assets/fonts/p4/", $dir, "/font.ttf")) as &[u8],
        )
    };
}
const FACES: [(&str, &[u8]); 52] = [
    face!("Abel", "abel"),
    face!("ABeeZee", "abeezee"),
    face!("Abril Fatface", "abrilfatface"),
    face!("Acme", "acme"),
    face!("Alata", "alata"),
    face!("Aldrich", "aldrich"),
    face!("Alice", "alice"),
    face!("Amaranth", "amaranth"),
    face!("Amatic SC", "amaticsc"),
    face!("Architects Daughter", "architectsdaughter"),
    face!("Arvo", "arvo"),
    face!("Bangers", "bangers"),
    face!("Belleza", "belleza"),
    face!("Bitter", "bitter"),
    face!("Cabin", "cabin"),
    face!("Caveat", "caveat"),
    face!("Cinzel", "cinzel"),
    face!("Comfortaa", "comfortaa"),
    face!("Cormorant Garamond", "cormorantgaramond"),
    face!("Dancing Script", "dancingscript"),
    face!("EB Garamond", "ebgaramond"),
    face!("Exo 2", "exo2"),
    face!("Figtree", "figtree"),
    face!("Fraunces", "fraunces"),
    face!("Great Vibes", "greatvibes"),
    face!("Heebo", "heebo"),
    face!("Hind", "hind"),
    face!("Inconsolata", "inconsolata"),
    face!("Indie Flower", "indieflower"),
    face!("Josefin Sans", "josefinsans"),
    face!("Jost", "jost"),
    face!("Karla", "karla"),
    face!("Lato", "lato"),
    face!("Lora", "lora"),
    face!("Merriweather", "merriweather"),
    face!("Montserrat", "montserrat"),
    face!("Nunito", "nunito"),
    face!("Oswald", "oswald"),
    face!("Outfit", "outfit"),
    face!("Pacifico", "pacifico"),
    face!("Playfair Display", "playfairdisplay"),
    face!("Quicksand", "quicksand"),
    face!("Raleway", "raleway"),
    face!("Rubik", "rubik"),
    face!("Sacramento", "sacramento"),
    face!("Teko", "teko"),
    face!("Unbounded", "unbounded"),
    face!("Vollkorn", "vollkorn"),
    face!("Work Sans", "worksans"),
    face!("Xanh Mono", "xanhmono"),
    face!("Yanone Kaffeesatz", "yanonekaffeesatz"),
    face!("Zilla Slab", "zillaslab"),
];
const PAGE_SIZE: usize = 8;
const MENU_Y: i32 = 190;
#[path = "gpe_ui_p4_font_gallery/search.rs"]
mod search;
use search::Search;
const INK: Pixel = Pixel::rgb(232, 235, 237);
const MUTED: Pixel = Pixel::rgb(153, 163, 172);
const ACCENT: Pixel = Pixel::rgb(132, 222, 187);

#[derive(Debug, Default)]
struct HorizontalRepeat {
    direction: i8,
    elapsed_s: f32,
    next_repeat_s: f32,
}

impl HorizontalRepeat {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn pulse(
        &mut self,
        left_held: bool,
        right_held: bool,
        left_pressed: bool,
        right_pressed: bool,
        delta_s: f32,
    ) -> i8 {
        let direction = match (left_held, right_held) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };
        if direction == 0 {
            self.reset();
            return 0;
        }

        let pressed = (direction < 0 && left_pressed) || (direction > 0 && right_pressed);
        if pressed || self.direction != direction {
            self.direction = direction;
            self.elapsed_s = 0.0;
            self.next_repeat_s = REPEAT_INITIAL_DELAY_S;
            return direction;
        }

        self.elapsed_s += delta_s.max(0.0);
        if self.elapsed_s + f32::EPSILON < self.next_repeat_s {
            return 0;
        }

        self.next_repeat_s += REPEAT_INTERVAL_S;
        direction
    }
}

struct Gallery {
    search: Search,
    fonts: Vec<OutlineFont>,
    selected: usize,
    px: f32,
    state: UiStateStore,
    menu: Framebuffer,
    last_pointer: Option<(i32, i32)>,
    size_id: Option<UiId>,
    horizontal_repeat: HorizontalRepeat,
    pending_wheel_steps: i32,
    painted: bool,
}
impl Gallery {
    fn new() -> Result<Self, &'static str> {
        let fonts = FACES
            .iter()
            .map(|(_, bytes)| OutlineFont::from_bytes(bytes))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            search: Search::new(),
            fonts,
            selected: 0,
            px: 30.0,
            state: UiStateStore::default(),
            menu: Framebuffer::new(280, 560),
            last_pointer: None,
            size_id: None,
            horizontal_repeat: HorizontalRepeat::default(),
            pending_wheel_steps: 0,
            painted: false,
        })
    }
    fn advance(&mut self, delta: isize) {
        if self.search.matches.is_empty() {
            return;
        }
        let position = self
            .search
            .matches
            .iter()
            .position(|i| *i == self.selected)
            .unwrap_or(0);
        let next =
            (position as isize + delta).rem_euclid(self.search.matches.len() as isize) as usize;
        self.selected = self.search.matches[next];
    }
    fn paint(&mut self, fb: &mut Framebuffer, input: UiInput<'_>) {
        fb.clear(Pixel::rgb(17, 20, 24));
        self.menu.clear(Pixel::rgb(17, 20, 24));
        let position = self
            .search
            .matches
            .iter()
            .position(|i| *i == self.selected)
            .unwrap_or(0);
        let page = position / PAGE_SIZE;
        let visible: Vec<_> = self
            .search
            .matches
            .iter()
            .skip(page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .copied()
            .collect();
        let horizontal = isize::from(input.nav.right) - isize::from(input.nav.left);
        let theme = UiTheme {
            padding: 0,
            row_height: 34,
            row_spacing: 6,
            text_scale: 2,
            control_background: Pixel::rgb(29, 34, 40),
            border: Pixel::rgb(60, 69, 77),
            accent: ACCENT,
            ..UiTheme::default()
        };
        let (out, (items, previous, next, size)) =
            experimental::run_with_input(&mut self.menu, &mut self.state, input, theme, |ui| {
                let items = ui.grid(
                    UiGridSpec {
                        min_cell_width: 280,
                        preferred_cell_height: 34,
                        gap: 6,
                        padding: 0,
                    },
                    |ui| {
                        (0..PAGE_SIZE)
                            .map(|row| {
                                if let Some(&i) = visible.get(row) {
                                    Some(ui.keyed(FACES[i].0, |ui| ui.button("")))
                                } else {
                                    ui.text("");
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    },
                );
                let previous = ui.keyed("previous", |ui| ui.button("< PREVIOUS FONT"));
                let next = ui.keyed("next", |ui| ui.button("NEXT FONT >"));
                let size = ui.keyed("size", |ui| {
                    ui.slider_f32("SIZE", self.px, 16.0..=56.0, 1.0)
                });
                (items, previous, next, size)
            });
        self.size_id = Some(size.id());
        for (row, handle) in items.iter().enumerate() {
            let Some(handle) = handle else {
                continue;
            };
            let i = visible[row];
            if out.activated(*handle) {
                self.selected = i;
            }
            self.fonts[i].draw(
                &mut self.menu,
                FACES[i].0,
                21.0,
                Rect {
                    x: 12,
                    y: row as i32 * 40 + 3,
                    width: 254,
                    height: 29,
                },
                if i == self.selected { ACCENT } else { INK },
            );
        }
        if out.activated(previous) {
            self.advance(-1);
        }
        if out.activated(next) {
            self.advance(1);
        }
        if let Some(px) = out.changed(size) {
            self.px = px;
        }
        if self.pending_wheel_steps != 0 && out.hovered_id() == Some(size.id()) {
            let before = self.px;
            self.px = (self.px + self.pending_wheel_steps as f32).clamp(16.0, 56.0);
            if self.px != before {
                self.painted = false;
            }
        }
        self.pending_wheel_steps = 0;
        if out.focused_id() != Some(size.id()) && horizontal != 0 {
            self.advance(horizontal);
        }
        for y in 0..self.menu.height() {
            for x in 0..self.menu.width() {
                fb.draw(
                    x as i32 + 24,
                    y as i32 + MENU_Y,
                    self.menu.pixel(x as i32, y as i32).unwrap(),
                );
            }
        }
        self.fonts[4].draw(
            fb,
            "GPE.UI / Font library",
            28.0,
            Rect {
                x: 24,
                y: 22,
                width: 700,
                height: 45,
            },
            INK,
        );
        self.fonts[4].draw(
            fb,
            &format!(
                "{} families   /   {} results",
                FACES.len(),
                self.search.matches.len()
            ),
            18.0,
            Rect {
                x: 880,
                y: 32,
                width: 290,
                height: 30,
            },
            ACCENT,
        );
        self.fonts[4].draw(
            fb,
            &format!(
                "{} / {}    {}",
                if self.search.matches.is_empty() {
                    0
                } else {
                    page + 1
                },
                self.search.matches.len().div_ceil(PAGE_SIZE),
                if self.search.query.is_empty() {
                    "A - Z"
                } else {
                    "FUZZY"
                }
            ),
            17.0,
            Rect {
                x: 24,
                y: 153,
                width: 280,
                height: 32,
            },
            MUTED,
        );
        fb.fill_rect(24, 92, 280, 42, Pixel::rgb(29, 34, 40));
        fb.draw_rect(
            24,
            92,
            280,
            42,
            if self.search.active { ACCENT } else { MUTED },
        );
        let label = if self.search.query.is_empty() && !self.search.active {
            "Rechercher...".to_owned()
        } else {
            self.search.display()
        };
        self.fonts[4].draw(
            fb,
            &label,
            19.0,
            Rect {
                x: 34,
                y: 99,
                width: 226,
                height: 30,
            },
            INK,
        );
        self.fonts[4].draw(
            fb,
            "\u{d7}",
            24.0,
            Rect {
                x: 275,
                y: 97,
                width: 24,
                height: 30,
            },
            MUTED,
        );
        fb.fill_rect(328, 100, 1, 640, Pixel::rgb(57, 65, 74));
        if self.search.matches.is_empty() {
            self.fonts[4].draw(
                fb,
                "Aucune police trouv\u{e9}e",
                32.0,
                Rect {
                    x: 370,
                    y: 220,
                    width: 790,
                    height: 70,
                },
                INK,
            );
            return;
        }
        let font = &mut self.fonts[self.selected];
        let sections = [
            (110, 80, 52.0, FACES[self.selected].0, ACCENT),
            (220, 160, self.px, "Une interface, mille voix.", INK),
            (
                390,
                160,
                22.0,
                "\u{c9}t\u{e9}, for\u{ea}t, fa\u{e7}ade, c\u{153}ur : les d\u{e9}tails comptent.\nABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz",
                INK,
            ),
            (560, 70, 30.0, "0123456789   0.65   100%   + - = ? !", INK),
            (
                650,
                70,
                18.0,
                "Continuer la partie    Param\u{e8}tres    Sauvegarder\nAudio  /  Vid\u{e9}o  /  Commandes",
                MUTED,
            ),
        ];
        for (y, height, px, text, color) in sections {
            font.draw(
                fb,
                text,
                px,
                Rect {
                    x: 370,
                    y,
                    width: 790,
                    height,
                },
                color,
            );
        }
        TextRenderer::default().draw(
            fb,
            24,
            759,
            &format!(
                "P4 / OUTLINE FONTS     {} PX     SIL OPEN FONT LICENSE",
                self.px
            ),
            MUTED,
        );
    }
}
impl Game for Gallery {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let old_query = self.search.query.clone();
        let was_active = self.search.active;
        let ctrl =
            frame.input.key(Key::LeftControl).held() || frame.input.key(Key::RightControl).held();
        if ctrl && frame.input.key(Key::F).pressed() {
            self.search.active = true;
            self.search.select_all();
            self.painted = false;
        }
        if frame.input.mouse_button(MouseButton::Left).pressed() {
            if let Some((x, y)) = frame.input.mouse_position() {
                self.search.active = (24..304).contains(&x) && (92..134).contains(&y);
                if self.search.active && x >= 268 {
                    self.search.clear();
                }
            }
        }
        if frame.input.key(Key::Escape).pressed() {
            if !self.search.query.is_empty() {
                self.search.clear();
                self.search.active = true;
            } else if self.search.active {
                self.search.active = false;
            } else {
                return GameResult::Exit;
            }
            self.painted = false;
        }
        if self.search.active {
            if ctrl && frame.input.key(Key::A).pressed() {
                self.search.select_all();
            }
            if !ctrl {
                self.search.edit(frame.input.text_events());
            }
            if frame.input.key(Key::Down).pressed() {
                self.advance(1);
            }
            if frame.input.key(Key::Up).pressed() {
                self.advance(-1);
            }
            if frame.input.key(Key::Enter).pressed() || frame.input.key(Key::Tab).pressed() {
                self.search.active = false;
            }
        } else if frame.input.key(Key::Tab).pressed() {
            self.search.active = true;
        }
        if self.search.query != old_query {
            if let Some(&first) = self.search.matches.first() {
                self.selected = first;
            }
            self.state = UiStateStore::default();
            self.size_id = None;
            self.horizontal_repeat.reset();
            self.painted = false;
        }
        let pointer = UiPointerInput {
            position: frame
                .input
                .mouse_position()
                .map(|(x, y)| (x - 24, y - MENU_Y)),
            pressed: frame.input.mouse_button(MouseButton::Left).pressed(),
            released: frame.input.mouse_button(MouseButton::Left).released(),
        };
        let slider_focused = self
            .size_id
            .is_some_and(|id| self.state.focused_id() == Some(id));
        let left = frame.input.key(Key::Left);
        let right = frame.input.key(Key::Right);
        let horizontal_direction = if self.search.active {
            self.horizontal_repeat.reset();
            0
        } else if slider_focused {
            self.horizontal_repeat.pulse(
                left.held(),
                right.held(),
                left.pressed(),
                right.pressed(),
                frame.delta_time.as_secs_f32(),
            )
        } else {
            self.horizontal_repeat.reset();
            i8::from(right.pressed()) - i8::from(left.pressed())
        };
        let wheel_steps = frame.input.mouse_wheel_steps();
        self.pending_wheel_steps = wheel_steps;
        let changed = !self.painted
            || !frame.input.text_events().is_empty()
            || was_active != self.search.active
            || pointer.position != self.last_pointer
            || pointer.pressed
            || pointer.released
            || horizontal_direction != 0
            || wheel_steps != 0
            || [Key::Up, Key::Down, Key::Space, Key::Enter, Key::Tab]
                .iter()
                .any(|key| frame.input.key(*key).pressed());
        if !changed {
            return GameResult::Continue;
        }
        self.last_pointer = pointer.position;
        self.painted = true;
        let previous_selection = self.selected;
        self.paint(
            frame.framebuffer,
            UiInput {
                pointer,
                nav: UiNavInput {
                    up: !was_active && !self.search.active && frame.input.key(Key::Up).pressed(),
                    down: !was_active
                        && !self.search.active
                        && frame.input.key(Key::Down).pressed(),
                    left: horizontal_direction < 0,
                    right: horizontal_direction > 0,
                    confirm: !was_active
                        && !self.search.active
                        && (frame.input.key(Key::Space).pressed()
                            || frame.input.key(Key::Enter).pressed()),
                    ..UiNavInput::default()
                },
                touches: &[],
            },
        );
        if self.selected != previous_selection {
            self.painted = false;
        }
        GameResult::Continue
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gallery = Gallery::new()?;
    if let Some(directory) = std::env::args().nth(1) {
        std::fs::create_dir_all(&directory)?;
        for index in 0..FACES.len() {
            gallery.selected = index;
            gallery.state = UiStateStore::default();
            let mut fb = Framebuffer::new(WIDTH, HEIGHT);
            gallery.paint(&mut fb, UiInput::default());
            let path = std::path::Path::new(&directory).join(format!("font-{index:02}.png"));
            let mut encoder = png::Encoder::new(
                std::io::BufWriter::new(std::fs::File::create(path)?),
                WIDTH,
                HEIGHT,
            );
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(fb.as_rgba8())?;
        }
        return Ok(());
    }
    run(
        EngineConfig {
            title: "GPE.UI P4 / Font Finder A-Z".into(),
            framebuffer_width: WIDTH,
            framebuffer_height: HEIGHT,
            window_width: WIDTH,
            window_height: HEIGHT,
        },
        gallery,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_distinct_fonts_render_and_clip() {
        let mut signatures = std::collections::HashSet::new();
        for (name, bytes) in FACES {
            let mut font = OutlineFont::from_bytes(bytes).unwrap();
            assert!(font.has_glyph('A'), "{name}");
            let mut fb = Framebuffer::new(260, 100);
            fb.clear(Pixel::BLACK);
            let bounds = Rect {
                x: 10,
                y: 10,
                width: 230,
                height: 70,
            };
            let size = font.measure("Bonjour \u{e9}t\u{e9} 0123", 26.0, bounds);
            assert_eq!(
                font.draw(&mut fb, "Bonjour \u{e9}t\u{e9} 0123", 26.0, bounds, INK),
                size
            );
            assert!(
                fb.as_rgba8()
                    .chunks_exact(4)
                    .any(|p| p[0] > 0 && p[0] < INK.r),
                "{name}"
            );
            for y in 0..100 {
                for x in 0..260 {
                    if !(10..240).contains(&x) || !(10..80).contains(&y) {
                        assert_eq!(fb.pixel(x, y), Some(Pixel::BLACK));
                    }
                }
            }
            signatures.insert(fb.as_rgba8().to_vec());
        }
        assert_eq!(signatures.len(), FACES.len());
    }
    #[test]
    fn selection_wraps_both_directions() {
        let mut gallery = Gallery::new().unwrap();
        gallery.selected = gallery.search.matches[0];
        gallery.advance(-1);
        assert_eq!(gallery.selected, *gallery.search.matches.last().unwrap());
        gallery.advance(1);
        assert_eq!(gallery.selected, gallery.search.matches[0]);
    }

    #[test]
    fn horizontal_repeat_is_immediate_then_waits_and_repeats() {
        let mut repeat = HorizontalRepeat::default();
        assert_eq!(repeat.pulse(false, true, false, true, 0.0), 1);
        assert_eq!(repeat.pulse(false, true, false, false, 0.299), 0);
        assert_eq!(repeat.pulse(false, true, false, false, 0.001), 1);
        assert_eq!(repeat.pulse(false, true, false, false, 0.059), 0);
        assert_eq!(repeat.pulse(false, true, false, false, 0.001), 1);
        assert_eq!(repeat.pulse(false, false, false, false, 0.100), 0);
        assert_eq!(repeat.pulse(true, false, true, false, 0.0), -1);
    }

    #[test]
    fn menu_mouse_and_keyboard_reach_both_pages_and_slider() {
        let mut gallery = Gallery::new().unwrap();
        let mut fb = Framebuffer::new(WIDTH, HEIGHT);
        for _ in 0..FACES.len() {
            let before = gallery
                .search
                .matches
                .iter()
                .position(|&i| i == gallery.selected)
                .unwrap();
            for released in [false, true] {
                gallery.paint(
                    &mut fb,
                    UiInput {
                        pointer: UiPointerInput {
                            position: Some((140, 375)),
                            pressed: !released,
                            released,
                        },
                        ..UiInput::default()
                    },
                );
            }
            assert_eq!(
                gallery.selected,
                gallery.search.matches[(before + 1) % FACES.len()]
            );
        }
        gallery.selected = gallery.search.matches[0];
        gallery.paint(
            &mut fb,
            UiInput {
                nav: UiNavInput {
                    left: true,
                    ..UiNavInput::default()
                },
                ..UiInput::default()
            },
        );
        assert_eq!(gallery.selected, *gallery.search.matches.last().unwrap());
        for released in [false, true] {
            gallery.paint(
                &mut fb,
                UiInput {
                    pointer: UiPointerInput {
                        position: Some((270, 415)),
                        pressed: !released,
                        released,
                    },
                    ..UiInput::default()
                },
            );
        }
        assert!(gallery.px > 50.0);
        assert_eq!(gallery.selected, *gallery.search.matches.last().unwrap());
    }

    #[test]
    fn newlines_do_not_draw_missing_glyph_boxes() {
        for (_, bytes) in FACES {
            let mut font = OutlineFont::from_bytes(bytes).unwrap();
            let mut fb = Framebuffer::new(100, 100);
            fb.clear(Pixel::BLACK);
            font.draw(
                &mut fb,
                "\n\r\n",
                26.0,
                Rect {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                Pixel::WHITE,
            );
            assert!(fb.as_rgba8().chunks_exact(4).all(|p| p[0] == 0));
        }
    }

    #[test]
    fn largest_preview_fits_for_every_family() {
        for (name, bytes) in FACES {
            let mut font = OutlineFont::from_bytes(bytes).unwrap();
            let bounds = Rect {
                x: 0,
                y: 0,
                width: 790,
                height: 160,
            };
            let size = font.measure("Une interface, mille voix.", 56.0, bounds);
            assert!(
                size.width <= bounds.width && size.height <= bounds.height,
                "{name}: {size:?}"
            );
        }
    }
}
