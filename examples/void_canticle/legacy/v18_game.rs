const VC18_VERSION: &str = "VC1.8";
const V18_ART_FAMILY_COUNT: usize = 7;

const ART_METAL: Pixel = Pixel::rgb(112, 121, 142);
const ART_METAL_LIGHT: Pixel = Pixel::rgb(202, 211, 221);
const ART_BONE: Pixel = Pixel::rgb(222, 211, 183);
const ART_SHADOW: Pixel = Pixel::rgb(28, 24, 35);
const ART_RUST: Pixel = Pixel::rgb(154, 68, 78);
const ART_VOID: Pixel = Pixel::rgb(151, 80, 201);
const ART_GOLD: Pixel = Pixel::rgb(226, 176, 83);
const ART_CYAN: Pixel = Pixel::rgb(92, 183, 218);
const ART_CYAN_LIGHT: Pixel = Pixel::rgb(219, 248, 255);

struct V18ArtVisuals {
    carrion: Sprite,
    grave_knight: Sprite,
    bell_wraith: Sprite,
    relic_carrier: Sprite,
    choir_node: Sprite,
    void_leech: Sprite,
    bellkeeper: Sprite,
}

impl V18ArtVisuals {
    fn new() -> Self {
        let palette = [
            ('M', ART_METAL),
            ('L', ART_METAL_LIGHT),
            ('B', ART_BONE),
            ('D', ART_SHADOW),
            ('E', ART_RUST),
            ('R', ENEMY_EYE),
            ('V', ART_VOID),
            ('G', ART_GOLD),
            ('C', ART_CYAN),
            ('W', ART_CYAN_LIGHT),
            ('K', BELL_DARK),
        ];

        Self {
            carrion: art_sprite(
                17,
                &[
                    "E...............E",
                    ".E.............E.",
                    "..E...DDDDD...E..",
                    "...E.DMMMMMD.E...",
                    "...DDMBBRBBMDD...",
                    "..EMMBBRRRBBMME..",
                    "...DDMBBRBBMDD...",
                    "...E.DMMMMMD.E...",
                    "..E...D...D...E..",
                    ".E....D...D....E.",
                    "E.....D...D.....E",
                    "......V...V......",
                    ".....VV...VV.....",
                ],
                &palette,
            ),
            grave_knight: art_sprite(
                17,
                &[
                    "GGG",
                    "GLG",
                    "GLLLG",
                    "GMMMMMG",
                    "GMMDDMMMG",
                    "GMMDDDDMMMG",
                    "GMMDDLRLDDMMG",
                    "GMMDDRRRDDMMG",
                    "GMMDDLRLDDMMG",
                    "GMMDDDDDMMG",
                    "GMMMMMMMG",
                    "GMMMMMG",
                    "DMM.MMD",
                    "DDM...MDD",
                    "GDM.....MDG",
                    "GDM.....MDG",
                    "D.......D",
                    "DD.......DD",
                    "DDD.......DDD",
                    "V.........V",
                    "VVV.......VVV",
                ],
                &palette,
            ),
            bell_wraith: art_sprite(
                19,
                &[
                    "V.V",
                    "VVVVV",
                    "VVLLLVV",
                    "VLLWWWLLV",
                    "VLLWDDDWLLV",
                    "VLWDDDDDWLV",
                    "VLWDDRDDWLV",
                    "VLWDRRRDWLV",
                    "VLWDDRDDWLV",
                    "VLLWDDDWLLV",
                    "VLLWWWLLV",
                    "VVLLLVV",
                    "V.D.V",
                    "V..D..V",
                    "V...D...V",
                    "V....D....V",
                    "V.....D.....V",
                ],
                &palette,
            ),
            relic_carrier: art_sprite(
                19,
                &[
                    "G.......G",
                    "GG.....GG",
                    "GGG...GGG",
                    "DGGG.GGGD",
                    "DMGGGGGMD",
                    "DMMGLGMM D".replace(' ', "").as_str(),
                ],
                &palette,
            ),
            choir_node: art_sprite(
                19,
                &[
                    "C...C",
                    "CC.CC",
                    "CWCWC",
                    "CCMMMCC",
                    "CMMWMMC",
                    "CMWWWWMC",
                    "CMWWRWWMC",
                    "CMWWWWMC",
                    "CMMWMMC",
                    "CCMMMCC",
                    "CWCWC",
                    "CC.CC",
                    "C...C",
                ],
                &palette,
            ),
            void_leech: art_sprite(
                21,
                &[
                    "V.........V",
                    "VV.......VV",
                    "VVD.....DVV",
                    "VVDD...DDVV",
                    "VDDMMMMMDDV",
                    "DDMMVVVMMDD",
                    "DMMVDRDVMMD",
                    "DMMVRRRVMMD",
                    "DMMVDRDVMMD",
                    "DDMMVVVMMDD",
                    "VDDMMMMMDDV",
                    "VVDD...DDVV",
                    "VVD.....DVV",
                    "VV.......VV",
                    "V.........V",
                ],
                &palette,
            ),
            bellkeeper: art_sprite(
                31,
                &[
                    "GGG",
                    "GGGLLLGGG",
                    "GGLLMMMMMLLGG",
                    "GLLMMKKKKKMMLLG",
                    "LLMMKBBBBBKMMLL",
                    "LMMKBBBLBBBK MML".replace(' ', "").as_str(),
                ],
                &palette,
            ),
        }
    }
}

fn art_sprite(width: usize, rows: &[&str], palette: &[(char, Pixel)]) -> Sprite {
    let mut pixels = Vec::with_capacity(width * rows.len());
    for row in rows {
        let cells: Vec<char> = row.chars().collect();
        assert!(cells.len() <= width, "VC1.8 authored sprite row exceeds width");
        let left = (width - cells.len()) / 2;
        let right = width - cells.len() - left;
        pixels.extend(std::iter::repeat_n(Pixel::TRANSPARENT, left));
        for cell in cells {
            if cell == '.' {
                pixels.push(Pixel::TRANSPARENT);
                continue;
            }
            let color = palette
                .iter()
                .find_map(|(key, color)| (*key == cell).then_some(*color))
                .unwrap_or_else(|| panic!("unknown VC1.8 art palette key: {cell}"));
            pixels.push(color);
        }
        pixels.extend(std::iter::repeat_n(Pixel::TRANSPARENT, right));
    }

    Sprite::new(width as u32, rows.len() as u32, pixels)
        .expect("VC1.8 authored sprite dimensions must match pixels")
}

struct VoidCanticleV18 {
    ui: VoidCanticlePauseV17,
    art: V18ArtVisuals,
}

impl VoidCanticleV18 {
    fn new() -> Self {
        Self {
            ui: VoidCanticlePauseV17::new(VoidCanticleV17::new()),
            art: V18ArtVisuals::new(),
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
            let x = enemy.x.round() as i32;
            let y = enemy.y.round() as i32;
            let wing = 9 + ((enemy.age * 7.0).sin().abs() * 3.0) as i32;
            framebuffer.draw_line(x - wing, y - 1, x - 5, y + 2, ART_RUST);
            framebuffer.draw_line(x + 5, y + 2, x + wing, y - 1, ART_RUST);
            self.art.carrion.draw_centered(framebuffer, x, y);
        }
    }

    fn render_special_art(&self, framebuffer: &mut Framebuffer) {
        for enemy in &self.v12().combat.specials {
            let x = enemy.x.round() as i32;
            let y = enemy.y.round() as i32;
            match enemy.kind {
                SpecialKind::GraveKnight => {
                    framebuffer.draw_line(x - 8, y + 11, x + 8, y + 11, ART_SHADOW);
                    self.art.grave_knight.draw_centered(framebuffer, x, y);
                    if enemy.age >= 1.45 {
                        framebuffer.draw_line(x - 5, y + 11, x - 5, y + 17, PILGRIM_THRUSTER);
                        framebuffer.draw_line(x + 5, y + 11, x + 5, y + 17, PILGRIM_THRUSTER);
                    }
                }
                SpecialKind::BellWraith => {
                    let halo = 11 + ((enemy.age * 4.8).sin().abs() * 3.0) as u32;
                    framebuffer.draw_circle(x, y, halo, ART_VOID);
                    self.art.bell_wraith.draw_centered(framebuffer, x, y);
                }
                SpecialKind::RelicCarrier => {
                    let wake = if enemy.direction >= 0.0 { -1 } else { 1 };
                    framebuffer.draw_line(x + wake * 9, y, x + wake * 17, y, ART_GOLD);
                    framebuffer.draw_line(x + wake * 10, y - 3, x + wake * 15, y - 3, ART_METAL);
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
                    framebuffer.draw_line(x - 14, y, x + 14, y, ART_CYAN);
                    framebuffer.draw_line(x, y - 14, x, y + 14, ART_CYAN);
                    self.art.choir_node.draw_centered(framebuffer, x, y);
                }
                ThreatKind::VoidLeech => {
                    let halo = 13 + threat.charge.min(5) * 2;
                    framebuffer.draw_circle(x, y, halo, ART_VOID);
                    self.art.void_leech.draw_centered(framebuffer, x, y);
                    let pips = threat.charge.min(LEECH_PULSE_CHARGE);
                    for index in 0..pips {
                        framebuffer.draw(x - 6 + index as i32 * 3, y + 12, ART_CYAN_LIGHT);
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
                framebuffer.draw_line(x - 18, y - 4, x - 28, y + 14, BELL_DARK);
                framebuffer.draw_line(x + 18, y - 4, x + 28, y + 14, BELL_DARK);
                framebuffer.draw_circle(x, y, 20 + pulse, BELL_METAL);
            }
            BellPhase::Resonance => {
                framebuffer.draw_circle(x, y, 22 + pulse, BELL_LIGHT);
                framebuffer.draw_circle(x, y, 27 + pulse, ART_VOID);
                framebuffer.draw_line(x - 27, y, x + 27, y, ART_VOID);
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
        framebuffer.draw_text(20, 97, &format!("VERSION {VC18_VERSION}"), TEXT);
        framebuffer.fill_rect(17, 172, 146, 14, Pixel::rgb(9, 8, 15));
        framebuffer.draw_text(20, 177, "ART SILHOUETTES", ART_GOLD);
    }
}

impl Game for VoidCanticleV18 {
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

pub fn run_v18_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticleV18::new(),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v18_tests {
    use super::*;

    #[test]
    fn authored_art_constructs_all_enemy_families() {
        let art = V18ArtVisuals::new();
        assert_eq!(V18_ART_FAMILY_COUNT, 7);
        assert_eq!(art.carrion.width(), 17);
        assert_eq!(art.grave_knight.width(), 17);
        assert_eq!(art.bell_wraith.width(), 19);
        assert_eq!(art.relic_carrier.width(), 19);
        assert_eq!(art.choir_node.width(), 19);
        assert_eq!(art.void_leech.width(), 21);
        assert_eq!(art.bellkeeper.width(), 31);
    }

    #[test]
    fn silhouettes_are_not_color_only_identifiers() {
        let art = V18ArtVisuals::new();
        assert_ne!(art.grave_knight.height(), art.choir_node.height());
        assert_ne!(art.bell_wraith.width(), art.void_leech.width());
        assert!(art.bellkeeper.width() > art.carrion.width());
    }

    #[test]
    fn vc18_version_is_explicit() {
        assert_eq!(VC18_VERSION, "VC1.8");
    }
}
