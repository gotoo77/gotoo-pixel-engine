use std::io::Cursor;

const VC19_VERSION: &str = "VC1.9";
const V19_CELL_SIZE: u32 = 32;
const V19_SPRITE_COUNT: u32 = 7;
const V19_SHEET_WIDTH: u32 = V19_CELL_SIZE * V19_SPRITE_COUNT;
const V19_SHEET_HEIGHT: u32 = V19_CELL_SIZE;
const V19_EMBEDDED_SHEET: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/void_canticle/grave_orbit_enemies.png"
));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum V19SpriteSlot {
    Carrion = 0,
    GraveKnight = 1,
    BellWraith = 2,
    RelicCarrier = 3,
    ChoirNode = 4,
    VoidLeech = 5,
    Bellkeeper = 6,
}

impl V19SpriteSlot {
    const ALL: [Self; V19_SPRITE_COUNT as usize] = [
        Self::Carrion,
        Self::GraveKnight,
        Self::BellWraith,
        Self::RelicCarrier,
        Self::ChoirNode,
        Self::VoidLeech,
        Self::Bellkeeper,
    ];

    const fn index(self) -> u32 {
        self as u32
    }
}

struct V19DecodedSheet {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

struct V19AssetVisuals {
    carrion: Sprite,
    grave_knight: Sprite,
    bell_wraith: Sprite,
    relic_carrier: Sprite,
    choir_node: Sprite,
    void_leech: Sprite,
    bellkeeper: Sprite,
}

impl V19AssetVisuals {
    fn new() -> Self {
        let bytes = v19_asset_bytes();
        Self::from_bytes(&bytes).unwrap_or_else(|error| {
            panic!("unable to load VC1.9 Grave Orbit sprite sheet: {error}")
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let sheet = decode_v19_sheet(bytes)?;
        validate_v19_sheet(&sheet)?;

        Ok(Self {
            carrion: sprite_from_v19_sheet(&sheet, V19SpriteSlot::Carrion)?,
            grave_knight: sprite_from_v19_sheet(&sheet, V19SpriteSlot::GraveKnight)?,
            bell_wraith: sprite_from_v19_sheet(&sheet, V19SpriteSlot::BellWraith)?,
            relic_carrier: sprite_from_v19_sheet(&sheet, V19SpriteSlot::RelicCarrier)?,
            choir_node: sprite_from_v19_sheet(&sheet, V19SpriteSlot::ChoirNode)?,
            void_leech: sprite_from_v19_sheet(&sheet, V19SpriteSlot::VoidLeech)?,
            bellkeeper: sprite_from_v19_sheet(&sheet, V19SpriteSlot::Bellkeeper)?,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn v19_asset_bytes() -> Vec<u8> {
    if let Some(path) = std::env::var_os("GPE_VC_ART_SHEET") {
        let bytes = std::fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "GPE_VC_ART_SHEET could not read {}: {error}",
                std::path::Path::new(&path).display()
            )
        });
        eprintln!(
            "Void Canticle VC1.9: using external art sheet {}",
            std::path::Path::new(&path).display()
        );
        return bytes;
    }

    V19_EMBEDDED_SHEET.to_vec()
}

#[cfg(target_arch = "wasm32")]
fn v19_asset_bytes() -> Vec<u8> {
    V19_EMBEDDED_SHEET.to_vec()
}

fn decode_v19_sheet(bytes: &[u8]) -> Result<V19DecodedSheet, String> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("PNG header: {error}"))?;

    let (color_type, bit_depth) = reader.output_color_type();
    if color_type != png::ColorType::Rgba || bit_depth != png::BitDepth::Eight {
        return Err(format!(
            "expected 8-bit RGBA PNG, got {color_type:?} {bit_depth:?}"
        ));
    }

    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| "PNG output buffer is too large".to_string())?;
    let mut raw = vec![0_u8; buffer_size];
    let info = reader
        .next_frame(&mut raw)
        .map_err(|error| format!("PNG frame: {error}"))?;
    let used = info.buffer_size();

    let expected = (info.width as usize)
        .checked_mul(info.height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| "PNG dimensions overflow".to_string())?;
    if used != expected {
        return Err(format!(
            "expected tightly packed RGBA data ({expected} bytes), got {used}"
        ));
    }

    let pixels = raw[..used]
        .chunks_exact(4)
        .map(|rgba| Pixel::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
        .collect();

    Ok(V19DecodedSheet {
        width: info.width,
        height: info.height,
        pixels,
    })
}

fn validate_v19_sheet(sheet: &V19DecodedSheet) -> Result<(), String> {
    if sheet.width != V19_SHEET_WIDTH || sheet.height != V19_SHEET_HEIGHT {
        return Err(format!(
            "expected {}x{} sheet ({} cells of {}x{}), got {}x{}",
            V19_SHEET_WIDTH,
            V19_SHEET_HEIGHT,
            V19_SPRITE_COUNT,
            V19_CELL_SIZE,
            V19_CELL_SIZE,
            sheet.width,
            sheet.height
        ));
    }
    Ok(())
}

fn sprite_from_v19_sheet(
    sheet: &V19DecodedSheet,
    slot: V19SpriteSlot,
) -> Result<Sprite, String> {
    let source_x = slot.index() * V19_CELL_SIZE;
    let mut pixels = Vec::with_capacity((V19_CELL_SIZE * V19_CELL_SIZE) as usize);

    for y in 0..V19_CELL_SIZE {
        for x in 0..V19_CELL_SIZE {
            let source_index = (y * sheet.width + source_x + x) as usize;
            pixels.push(sheet.pixels[source_index]);
        }
    }

    Sprite::new(V19_CELL_SIZE, V19_CELL_SIZE, pixels).map_err(|error| error.to_string())
}

struct VoidCanticleV19 {
    ui: VoidCanticlePauseV17,
    art: V19AssetVisuals,
}

impl VoidCanticleV19 {
    fn new() -> Self {
        Self {
            ui: VoidCanticlePauseV17::new(VoidCanticleV17::new()),
            art: V19AssetVisuals::new(),
        }
    }

    fn base(&self) -> &VoidCanticleGame {
        self.ui.game.base()
    }

    fn v14(&self) -> &VoidCanticleV14 {
        self.ui.game.v14()
    }

    fn v12(&self) -> &VoidCanticleV12 {
        &self.v14().progression.combat
    }

    fn art_can_overlay_game(&self) -> bool {
        if !matches!(&self.ui.state, VcPauseState::Running) || self.base().game_over {
            return false;
        }

        self.v14().progression.level_choice.is_none() && self.v14().mutation_choice.is_none()
    }

    fn render_carrion_art(&self, framebuffer: &mut Framebuffer) {
        for enemy in &self.base().enemies {
            self.art.carrion.draw_centered(
                framebuffer,
                enemy.x.round() as i32,
                enemy.y.round() as i32,
            );
        }
    }

    fn render_special_art(&self, framebuffer: &mut Framebuffer) {
        for enemy in &self.v12().combat.specials {
            let x = enemy.x.round() as i32;
            let y = enemy.y.round() as i32;
            match enemy.kind {
                SpecialKind::GraveKnight => {
                    if enemy.age >= 1.45 {
                        framebuffer.draw_line(
                            x - 5,
                            y + 10,
                            x - 5,
                            y + 17,
                            PILGRIM_THRUSTER,
                        );
                        framebuffer.draw_line(
                            x + 5,
                            y + 10,
                            x + 5,
                            y + 17,
                            PILGRIM_THRUSTER,
                        );
                    }
                    self.art.grave_knight.draw_centered(framebuffer, x, y);
                }
                SpecialKind::BellWraith => {
                    let halo = 11 + ((enemy.age * 4.8).sin().abs() * 3.0) as u32;
                    framebuffer.draw_circle(x, y, halo, ART_VOID);
                    self.art.bell_wraith.draw_centered(framebuffer, x, y);
                }
                SpecialKind::RelicCarrier => {
                    let wake = if enemy.direction >= 0.0 { -1 } else { 1 };
                    framebuffer.draw_line(x + wake * 10, y, x + wake * 19, y, ART_GOLD);
                    framebuffer.draw_line(
                        x + wake * 11,
                        y - 3,
                        x + wake * 17,
                        y - 3,
                        ART_METAL,
                    );
                    self.art.relic_carrier.draw_centered(framebuffer, x, y);
                }
            }
        }
    }

    fn render_threat_art(&self, framebuffer: &mut Framebuffer) {
        for threat in &self.v12().threats {
            let x = threat.x.round() as i32;
            let y = threat.y.round() as i32;
            match threat.kind {
                ThreatKind::ChoirNode => {
                    let halo = 12 + ((threat.age * 3.4).sin().abs() * 4.0) as u32;
                    framebuffer.draw_circle(x, y, halo, ART_CYAN);
                    framebuffer.draw_line(x - 15, y, x + 15, y, ART_CYAN);
                    framebuffer.draw_line(x, y - 15, x, y + 15, ART_CYAN);
                    self.art.choir_node.draw_centered(framebuffer, x, y);
                }
                ThreatKind::VoidLeech => {
                    let halo = 13 + threat.charge.min(5) * 2;
                    framebuffer.draw_circle(x, y, halo, ART_VOID);
                    self.art.void_leech.draw_centered(framebuffer, x, y);
                    let pips = threat.charge.min(LEECH_PULSE_CHARGE);
                    for index in 0..pips {
                        framebuffer.draw(x - 6 + index as i32 * 3, y + 14, ART_CYAN_LIGHT);
                    }
                }
            }
        }
    }

    fn render_bellkeeper_art(&self, framebuffer: &mut Framebuffer) {
        if self.base().encounter_phase == EncounterPhase::Cleared {
            return;
        }
        let Some(boss) = self.base().boss else {
            return;
        };

        let x = boss.x.round() as i32;
        let y = boss.y.round() as i32;
        let pulse = ((self.base().animation_time * 5.0).sin().abs() * 3.0) as u32;
        match boss.phase() {
            BellPhase::Procession => {
                framebuffer.draw_circle(x, y, 20 + pulse, BELL_METAL);
            }
            BellPhase::Resonance => {
                framebuffer.draw_circle(x, y, 22 + pulse, BELL_LIGHT);
                framebuffer.draw_circle(x, y, 27 + pulse, ART_VOID);
            }
            BellPhase::FinalToll => {
                framebuffer.draw_circle(x, y, 24 + pulse, DANGER);
                framebuffer.draw_circle(x, y, 30 + pulse, ART_GOLD);
                framebuffer.draw_line(x - 30, y, x + 30, y, DANGER);
                framebuffer.draw_line(x, y - 28, x, y + 28, DANGER);
            }
        }

        self.art.bellkeeper.draw_centered(framebuffer, x, y);
        framebuffer.fill_circle(x, y + 2, 2 + pulse / 2, CANTICLE_COLOR);
    }

    fn render_art_overlay(&self, framebuffer: &mut Framebuffer) {
        self.render_carrion_art(framebuffer);
        self.render_special_art(framebuffer);
        self.render_threat_art(framebuffer);
        self.render_bellkeeper_art(framebuffer);
    }

    fn render_build_info_overlay(&self, framebuffer: &mut Framebuffer) {
        framebuffer.fill_rect(17, 92, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 97, &format!("VERSION {VC19_VERSION}"), TEXT);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 177, "PNG ASSET SHEET", ART_GOLD);
    }
}

impl Game for VoidCanticleV19 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.ui.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        if self.art_can_overlay_game() {
            self.render_art_overlay(frame.framebuffer);
        } else if matches!(&self.ui.state, VcPauseState::BuildInfo) {
            self.render_build_info_overlay(frame.framebuffer);
        }

        GameResult::Continue
    }
}

pub fn run_v19_with_obs_mirror() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Void Canticle - Gotoo Pixel Engine".to_string(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        gotoo_pixel_engine::ObsMirrorGame::from_env(
            VoidCanticleV19::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v19_tests {
    use super::*;

    #[test]
    fn embedded_sheet_is_expected_rgba_grid() {
        let sheet = decode_v19_sheet(V19_EMBEDDED_SHEET).expect("embedded VC1.9 sheet should decode");
        assert_eq!(sheet.width, V19_SHEET_WIDTH);
        assert_eq!(sheet.height, V19_SHEET_HEIGHT);
        assert_eq!(
            sheet.pixels.len(),
            (V19_SHEET_WIDTH * V19_SHEET_HEIGHT) as usize
        );
    }

    #[test]
    fn every_authored_slot_contains_visible_pixels() {
        let sheet = decode_v19_sheet(V19_EMBEDDED_SHEET).expect("embedded VC1.9 sheet should decode");
        for slot in V19SpriteSlot::ALL {
            let sprite =
                sprite_from_v19_sheet(&sheet, slot).expect("VC1.9 sheet slot should be valid");
            assert!(
                sprite.pixels().iter().any(|pixel| pixel.a != 0),
                "{slot:?} must contain visible pixels"
            );
        }
    }

    #[test]
    fn asset_visuals_extract_seven_cells() {
        let art = V19AssetVisuals::from_bytes(V19_EMBEDDED_SHEET)
            .expect("embedded VC1.9 sheet should construct visuals");
        for sprite in [
            &art.carrion,
            &art.grave_knight,
            &art.bell_wraith,
            &art.relic_carrier,
            &art.choir_node,
            &art.void_leech,
            &art.bellkeeper,
        ] {
            assert_eq!(sprite.width(), V19_CELL_SIZE);
            assert_eq!(sprite.height(), V19_CELL_SIZE);
        }
    }

    #[test]
    fn vc19_version_is_explicit() {
        assert_eq!(VC19_VERSION, "VC1.9");
    }
}
