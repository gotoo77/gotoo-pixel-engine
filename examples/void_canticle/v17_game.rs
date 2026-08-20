const VC17_VERSION: &str = "VC1.7";

const V17_LEVEL_SOUND: SoundId = SoundId::new("void_canticle.v17.level");
const V17_MUTATION_SOUND: SoundId = SoundId::new("void_canticle.v17.mutation");
const V17_SYNERGY_SOUND: SoundId = SoundId::new("void_canticle.v17.synergy");
const V17_VOID_SOUND: SoundId = SoundId::new("void_canticle.v17.void");
const V17_BOSS_PHASE_SOUND: SoundId = SoundId::new("void_canticle.v17.boss_phase");
const V17_IMPACT_SOUND: SoundId = SoundId::new("void_canticle.v17.impact");
const V17_DETONATION_SOUND: SoundId = SoundId::new("void_canticle.v17.detonation");

const BOLT_CORE: Pixel = Pixel::rgb(255, 247, 216);
const BOLT_EDGE: Pixel = Pixel::rgb(255, 190, 92);
const BOLT_RELIC: Pixel = Pixel::rgb(207, 126, 255);
const BOLT_SHADOW: Pixel = Pixel::rgb(77, 52, 105);
const HOSTILE_CORE: Pixel = Pixel::rgb(255, 215, 232);
const HOSTILE_EDGE: Pixel = Pixel::rgb(244, 78, 142);
const HOSTILE_ALT_CORE: Pixel = Pixel::rgb(238, 218, 255);
const HOSTILE_ALT_EDGE: Pixel = Pixel::rgb(174, 108, 240);
const XP_SHARD_EDGE: Pixel = Pixel::rgb(96, 184, 255);
const XP_SHARD_CORE: Pixel = Pixel::rgb(230, 250, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum V17ParticleKind {
    Spark,
    Shard,
}

#[derive(Debug, Clone, Copy)]
struct V17Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    color: Pixel,
    kind: V17ParticleKind,
}

struct V17ProjectileVisuals {
    bolt: Sprite,
    heavy_bolt: Sprite,
    xp_shard: Sprite,
    orbital: Sprite,
}

impl V17ProjectileVisuals {
    fn new() -> Self {
        let bolt_palette = [
            ('L', BOLT_CORE),
            ('C', BOLT_EDGE),
            ('V', BOLT_RELIC),
            ('D', BOLT_SHADOW),
        ];
        let xp_palette = [('L', XP_SHARD_CORE), ('C', XP_SHARD_EDGE)];
        let orbital_palette = [
            ('G', POWER_RELIC_LIGHT),
            ('V', PILGRIM_VIOLET),
            ('L', BOLT_CORE),
            ('C', BOLT_EDGE),
        ];

        Self {
            bolt: sprite_from_ascii(
                &["..L..", ".CCC.", "..L..", "..C..", "..C..", "..V..", "..D.."],
                &bolt_palette,
            ),
            heavy_bolt: sprite_from_ascii(
                &[
                    "...L...",
                    "..CCC..",
                    ".CCLCC.",
                    "..CCC..",
                    "...C...",
                    "..VVV..",
                    "...V...",
                    "...D...",
                ],
                &bolt_palette,
            ),
            xp_shard: sprite_from_ascii(
                &["...L...", "..CCC..", ".CCLCC.", "..CCC..", "...C..."],
                &xp_palette,
            ),
            orbital: sprite_from_ascii(
                &[
                    "....G....",
                    "...GGG...",
                    "..GVVVG..",
                    ".GVLCLVG.",
                    "..GVVVG..",
                    "...GGG...",
                    "....G....",
                ],
                &orbital_palette,
            ),
        }
    }
}

struct VoidCanticleV17 {
    combat: VoidCanticleV16B,
    visuals: V17ProjectileVisuals,
    particles: Vec<V17Particle>,
    muzzle_timer: f32,
    impact_sound_timer: f32,
}

impl VoidCanticleV17 {
    fn new() -> Self {
        let mut combat = VoidCanticleV16B::new();
        install_v17_sounds(&mut combat);
        Self {
            combat,
            visuals: V17ProjectileVisuals::new(),
            particles: Vec::new(),
            muzzle_timer: 0.0,
            impact_sound_timer: 0.0,
        }
    }

    fn reset_run(&mut self) {
        self.combat.reset_run();
        self.particles.clear();
        self.muzzle_timer = 0.0;
        self.impact_sound_timer = 0.0;
    }

    fn base(&self) -> &VoidCanticleGame {
        self.combat.base()
    }

    fn v15(&self) -> &VoidCanticleV15 {
        &self.combat.combat.combat
    }

    fn v14(&self) -> &VoidCanticleV14 {
        &self.combat.combat.combat.combat
    }

    fn progression(&self) -> &VoidCanticleV13 {
        &self.v14().progression
    }

    fn mutation_stack_count(&self) -> u32 {
        let mutations = self.v14().mutations;
        mutations
            .piercing_lance
            .saturating_add(mutations.split_volley)
            .saturating_add(mutations.death_nova)
            .saturating_add(mutations.orbitals)
    }

    fn boss_phase(&self) -> Option<BellPhase> {
        if self.base().encounter_phase == EncounterPhase::BossFight {
            self.base().boss.map(Bellkeeper::phase)
        } else {
            None
        }
    }

    fn play_v17_sound(&mut self, id: SoundId, frame: &mut Frame<'_>) {
        let _ = self.combat.base_mut().sounds.play(frame.audio, id);
    }

    fn update_particles(&mut self, dt: f32) {
        for particle in &mut self.particles {
            particle.life = (particle.life - dt).max(0.0);
            match particle.kind {
                V17ParticleKind::Spark => {
                    particle.vx *= (1.0 - 4.6 * dt).max(0.0);
                    particle.vy *= (1.0 - 4.6 * dt).max(0.0);
                }
                V17ParticleKind::Shard => {
                    particle.vy += 34.0 * dt;
                    particle.vx *= (1.0 - 1.8 * dt).max(0.0);
                }
            }
            particle.x += particle.vx * dt;
            particle.y += particle.vy * dt;
        }
        self.particles.retain(|particle| particle.life > 0.0);
    }

    fn spawn_radial_fx(
        &mut self,
        x: f32,
        y: f32,
        color: Pixel,
        count: usize,
        speed: f32,
        life: f32,
        kind: V17ParticleKind,
    ) {
        let seed = x * 0.083 + y * 0.047 + count as f32 * 0.31;
        for index in 0..count {
            let angle = seed + index as f32 * std::f32::consts::TAU / count as f32;
            let velocity = speed * (0.72 + (index % 4) as f32 * 0.12);
            self.particles.push(V17Particle {
                x,
                y,
                vx: angle.cos() * velocity,
                vy: angle.sin() * velocity,
                life,
                max_life: life,
                color,
                kind,
            });
        }
    }

    fn spawn_muzzle_fx(&mut self, dt: f32) {
        self.muzzle_timer = (self.muzzle_timer - dt).max(0.0);
        if !self.v15().gameplay_is_running() || !self.v15().fire_held() {
            self.muzzle_timer = 0.0;
            return;
        }
        if self.muzzle_timer > 0.0 {
            return;
        }

        let (x, y) = (self.base().player_x, self.base().player_y - 12.0);
        for direction in [-1.0_f32, 1.0] {
            self.particles.push(V17Particle {
                x: x + direction * 3.0,
                y,
                vx: direction * 18.0,
                vy: -72.0,
                life: 0.10,
                max_life: 0.10,
                color: BOLT_EDGE,
                kind: V17ParticleKind::Spark,
            });
        }
        self.particles.push(V17Particle {
            x,
            y: y - 1.0,
            vx: 0.0,
            vy: -92.0,
            life: 0.11,
            max_life: 0.11,
            color: BOLT_CORE,
            kind: V17ParticleKind::Spark,
        });
        self.muzzle_timer = 0.075;
    }

    fn process_fresh_impacts(&mut self, frame: &mut Frame<'_>) {
        let bursts: Vec<Burst> = self
            .base()
            .bursts
            .iter()
            .copied()
            .filter(|burst| (burst.remaining - burst.duration).abs() <= 0.0001)
            .collect();
        if bursts.is_empty() {
            return;
        }

        let mut heavy = false;
        let mut impact = false;
        for burst in bursts {
            if burst.duration >= 0.34 {
                heavy = true;
                self.spawn_radial_fx(
                    burst.x,
                    burst.y,
                    burst.color,
                    9,
                    78.0,
                    0.34,
                    V17ParticleKind::Shard,
                );
                self.spawn_radial_fx(
                    burst.x,
                    burst.y,
                    BOLT_CORE,
                    5,
                    104.0,
                    0.18,
                    V17ParticleKind::Spark,
                );
            } else if burst.duration >= 0.10 {
                impact = true;
                self.spawn_radial_fx(
                    burst.x,
                    burst.y,
                    burst.color,
                    4,
                    62.0,
                    0.16,
                    V17ParticleKind::Spark,
                );
            }
        }

        if self.impact_sound_timer <= 0.0 {
            if heavy {
                self.play_v17_sound(V17_DETONATION_SOUND, frame);
                self.impact_sound_timer = 0.10;
            } else if impact {
                self.play_v17_sound(V17_IMPACT_SOUND, frame);
                self.impact_sound_timer = 0.055;
            }
        }
    }

    fn process_major_events(
        &mut self,
        level_before: u32,
        mutation_before: u32,
        synergies_before: u32,
        pressure_before: VoidPressure,
        boss_phase_before: Option<BellPhase>,
        frame: &mut Frame<'_>,
    ) {
        let level_after = self.progression().level;
        let mutation_after = self.mutation_stack_count();
        let synergies_after = self.v15().active_synergy_count();
        let pressure_after = self.combat.combat.pressure;
        let boss_phase_after = self.boss_phase();
        let player = (self.base().player_x, self.base().player_y - 5.0);

        if synergies_after > synergies_before {
            self.play_v17_sound(V17_SYNERGY_SOUND, frame);
            self.spawn_radial_fx(
                player.0,
                player.1,
                SYNERGY_LIGHT,
                18,
                112.0,
                0.46,
                V17ParticleKind::Shard,
            );
        } else if mutation_after > mutation_before {
            self.play_v17_sound(V17_MUTATION_SOUND, frame);
            self.spawn_radial_fx(
                player.0,
                player.1,
                MUTATION_LIGHT,
                13,
                92.0,
                0.38,
                V17ParticleKind::Shard,
            );
        } else if level_after > level_before {
            self.play_v17_sound(V17_LEVEL_SOUND, frame);
            self.spawn_radial_fx(
                player.0,
                player.1,
                XP_SHARD_CORE,
                10,
                76.0,
                0.31,
                V17ParticleKind::Spark,
            );
        }

        if pressure_after > pressure_before {
            self.play_v17_sound(V17_VOID_SOUND, frame);
            self.spawn_radial_fx(
                FRAMEBUFFER_WIDTH as f32 / 2.0,
                82.0,
                void_pressure_color(pressure_after),
                16,
                86.0,
                0.52,
                V17ParticleKind::Shard,
            );
        }

        if boss_phase_after != boss_phase_before && boss_phase_after.is_some() {
            self.play_v17_sound(V17_BOSS_PHASE_SOUND, frame);
            if let Some(boss) = self.base().boss {
                self.spawn_radial_fx(
                    boss.x,
                    boss.y,
                    BELL_LIGHT,
                    14,
                    74.0,
                    0.42,
                    V17ParticleKind::Shard,
                );
            }
        }
    }

    fn render_enemy_missile_heads(&self, framebuffer: &mut Framebuffer) {
        for bullet in &self.base().enemy_bullets {
            let speed = (bullet.vx * bullet.vx + bullet.vy * bullet.vy).sqrt().max(0.001);
            let nx = bullet.vx / speed;
            let ny = bullet.vy / speed;
            let px = -ny;
            let py = nx;
            let x = bullet.x;
            let y = bullet.y;
            let tip_x = (x + nx * 2.5).round() as i32;
            let tip_y = (y + ny * 2.5).round() as i32;
            let neck_x = (x - nx * 1.0).round() as i32;
            let neck_y = (y - ny * 1.0).round() as i32;
            let tail_x = (x - nx * 5.5).round() as i32;
            let tail_y = (y - ny * 5.5).round() as i32;
            let left_x = (x - nx + px * 2.3).round() as i32;
            let left_y = (y - ny + py * 2.3).round() as i32;
            let right_x = (x - nx - px * 2.3).round() as i32;
            let right_y = (y - ny - py * 2.3).round() as i32;
            let (edge, core) = if bullet.alternate {
                (HOSTILE_ALT_EDGE, HOSTILE_ALT_CORE)
            } else {
                (HOSTILE_EDGE, HOSTILE_CORE)
            };

            framebuffer.draw_line(tail_x, tail_y, tip_x, tip_y, edge);
            framebuffer.draw_line(left_x, left_y, right_x, right_y, edge);
            framebuffer.draw_line(neck_x, neck_y, tip_x, tip_y, core);
            framebuffer.draw(tip_x, tip_y, core);
        }
    }

    fn render_player_bolts(&self, framebuffer: &mut Framebuffer) {
        let game = &self
            .combat
            .combat
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner;
        for shot in &game.power_shots {
            let x = shot.x.round() as i32;
            let y = shot.y.round() as i32;
            if shot.radius >= 2 || shot.damage > 1 {
                self.visuals.heavy_bolt.draw_centered(framebuffer, x, y);
            } else {
                self.visuals.bolt.draw_centered(framebuffer, x, y);
            }

            if shot.vx.abs() > 6.0 {
                let wing = if shot.vx > 0.0 { -3 } else { 3 };
                framebuffer.draw_line(x, y + 2, x + wing, y + 5, BOLT_RELIC);
            }
        }
    }

    fn render_xp_shards(&self, framebuffer: &mut Framebuffer) {
        for orb in &self.progression().xp_orbs {
            self.visuals
                .xp_shard
                .draw_centered(framebuffer, orb.x.round() as i32, orb.y.round() as i32);
        }
    }

    fn render_orbitals(&self, framebuffer: &mut Framebuffer) {
        for (x, y) in self.v14().orbital_positions() {
            self.visuals
                .orbital
                .draw_centered(framebuffer, x.round() as i32, y.round() as i32);
        }
    }

    fn render_particles(&self, framebuffer: &mut Framebuffer) {
        for particle in &self.particles {
            let x = particle.x.round() as i32;
            let y = particle.y.round() as i32;
            let life_ratio = if particle.max_life > 0.0 {
                particle.life / particle.max_life
            } else {
                0.0
            };
            match particle.kind {
                V17ParticleKind::Spark => {
                    let tail_scale = 0.025 + life_ratio * 0.035;
                    let tail_x = (particle.x - particle.vx * tail_scale).round() as i32;
                    let tail_y = (particle.y - particle.vy * tail_scale).round() as i32;
                    framebuffer.draw_line(tail_x, tail_y, x, y, particle.color);
                    framebuffer.draw(x, y, BOLT_CORE);
                }
                V17ParticleKind::Shard => {
                    let tail_x = (particle.x - particle.vx * 0.025).round() as i32;
                    let tail_y = (particle.y - particle.vy * 0.025).round() as i32;
                    framebuffer.draw_line(tail_x, tail_y, x, y, particle.color);
                    framebuffer.draw_line(x - 1, y, x + 1, y, particle.color);
                    framebuffer.draw_line(x, y - 1, x, y + 1, particle.color);
                }
            }
        }
    }

    fn render_v17_polish(&self, framebuffer: &mut Framebuffer) {
        self.render_particles(framebuffer);
        self.render_enemy_missile_heads(framebuffer);
        self.render_player_bolts(framebuffer);
        self.render_xp_shards(framebuffer);
        self.render_orbitals(framebuffer);
    }
}

impl Game for VoidCanticleV17 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let level_before = self.progression().level;
        let mutation_before = self.mutation_stack_count();
        let synergies_before = self.v15().active_synergy_count();
        let pressure_before = self.combat.combat.pressure;
        let boss_phase_before = self.boss_phase();
        let was_game_over = self.base().game_over;

        self.impact_sound_timer = (self.impact_sound_timer - dt).max(0.0);
        self.update_particles(dt);
        let result = self.combat.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let game_over = self.base().game_over;
        if was_game_over && !game_over {
            self.particles.clear();
            self.muzzle_timer = 0.0;
            self.impact_sound_timer = 0.0;
        }

        self.spawn_muzzle_fx(dt);
        self.process_fresh_impacts(frame);
        self.process_major_events(
            level_before,
            mutation_before,
            synergies_before,
            pressure_before,
            boss_phase_before,
            frame,
        );
        self.render_v17_polish(frame.framebuffer);
        GameResult::Continue
    }
}

fn install_v17_sounds(game: &mut VoidCanticleV16B) {
    let sounds = &mut game.base_mut().sounds;
    for (id, wav) in [
        (V17_LEVEL_SOUND, synthesize_chirp(620.0, 1_260.0, 0.16, 0.16)),
        (V17_MUTATION_SOUND, synthesize_chirp(410.0, 1_020.0, 0.22, 0.19)),
        (V17_SYNERGY_SOUND, synthesize_chirp(520.0, 1_680.0, 0.29, 0.22)),
        (V17_VOID_SOUND, synthesize_chirp(185.0, 72.0, 0.32, 0.24)),
        (V17_BOSS_PHASE_SOUND, synthesize_chirp(330.0, 118.0, 0.26, 0.20)),
        (
            V17_IMPACT_SOUND,
            synthesize_noise_burst(0.055, 0.18, 0x17A1_C710),
        ),
        (
            V17_DETONATION_SOUND,
            synthesize_noise_burst(0.115, 0.29, 0x17DE_7001),
        ),
    ] {
        sounds
            .insert_wav(id, wav)
            .expect("VC1.7 sound ids should be unique");
    }
}

struct VoidCanticlePauseV17 {
    game: VoidCanticleV17,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV17 {
    fn new(game: VoidCanticleV17) -> Self {
        let mut controls = gotoo_pixel_engine::ui::standard_menu_controls(
            VC_PAUSE_UP,
            VC_PAUSE_DOWN,
            VC_PAUSE_CONFIRM,
        );
        controls
            .bind_key(VC_PAUSE_TOGGLE, Key::Escape)
            .bind_gamepad(VC_PAUSE_TOGGLE, GamepadButton::Start);
        Self {
            game,
            state: VcPauseState::Running,
            menu: gotoo_pixel_engine::ui::MenuState::new(5),
            controls,
        }
    }

    fn pause_input_held(&self) -> bool {
        [VC_PAUSE_TOGGLE, VC_PAUSE_UP, VC_PAUSE_DOWN, VC_PAUSE_CONFIRM]
            .into_iter()
            .any(|action| self.controls.action(action).held())
    }

    fn render_menu(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 18,
                y: 48,
                width: 144,
                height: 220,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(60, 62, "PAUSED", POWER_RELIC_LIGHT);
        for (index, (y, label)) in [
            (90, "RESUME"),
            (118, "RESTART"),
            (146, "CONTROLS"),
            (174, "BUILD INFO"),
            (202, "QUIT"),
        ]
        .into_iter()
        .enumerate()
        {
            gotoo_pixel_engine::ui::draw_menu_item(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 34,
                    y,
                    width: 112,
                    height: 18,
                },
                label,
                self.menu.selected() == Some(index),
                1,
                TEXT,
                POWER_RELIC_LIGHT,
            );
        }
    }

    fn render_controls(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 10,
                y: 46,
                width: 160,
                height: 224,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(52, 59, "CONTROLS", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 86, "MOVE  ARROWS WASD", TEXT);
        framebuffer.draw_text(20, 106, "FIRE  SPACE SOUTH", TEXT);
        framebuffer.draw_text(20, 126, "FOCUS SHIFT LB", TEXT);
        framebuffer.draw_text(20, 146, "CANTICLE X EAST", TEXT);
        framebuffer.draw_text(20, 166, "PAUSE ESC START", TEXT);
        framebuffer.draw_text(31, 231, "ESC START  BACK", WRECK_LIGHT);
    }

    fn render_build_info(&self, framebuffer: &mut Framebuffer) {
        gotoo_pixel_engine::ui::draw_panel(
            framebuffer,
            gotoo_pixel_engine::Rect {
                x: 10,
                y: 54,
                width: 160,
                height: 206,
            },
            Pixel::rgb(9, 8, 15),
            PILGRIM_VIOLET,
        );
        framebuffer.draw_text(46, 68, "BUILD INFO", POWER_RELIC_LIGHT);
        framebuffer.draw_text(20, 97, &format!("VERSION {VC17_VERSION}"), TEXT);
        framebuffer.draw_text(20, 117, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(
            20,
            137,
            &format!("SYNERGIES {}", self.game.v15().active_synergy_count()),
            SYNERGY_LIGHT,
        );
        framebuffer.draw_text(
            20,
            157,
            &format!("VOID {}", void_pressure_name(self.game.combat.combat.pressure)),
            void_pressure_color(self.game.combat.combat.pressure),
        );
        framebuffer.draw_text(20, 177, "FX PROJECTILE PASS", BOLT_EDGE);
        framebuffer.draw_text(31, 225, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV17 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.controls.update(frame.input);
        match self.state {
            VcPauseState::Running => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::Menu;
                    self.menu = gotoo_pixel_engine::ui::MenuState::new(5);
                    self.render_menu(frame.framebuffer);
                    GameResult::Continue
                } else {
                    self.game.update(frame)
                }
            }
            VcPauseState::Menu => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed() {
                    self.state = VcPauseState::ResumeGate;
                } else {
                    if self.controls.action(VC_PAUSE_UP).pressed() {
                        self.menu.select_previous();
                    }
                    if self.controls.action(VC_PAUSE_DOWN).pressed() {
                        self.menu.select_next();
                    }
                    if self.controls.action(VC_PAUSE_CONFIRM).pressed() {
                        match self.menu.selected() {
                            Some(0) => self.state = VcPauseState::ResumeGate,
                            Some(1) => {
                                self.game.reset_run();
                                self.state = VcPauseState::ResumeGate;
                            }
                            Some(2) => self.state = VcPauseState::Controls,
                            Some(3) => self.state = VcPauseState::BuildInfo,
                            Some(4) => return GameResult::Exit,
                            _ => {}
                        }
                    }
                }
                match self.state {
                    VcPauseState::Controls => self.render_controls(frame.framebuffer),
                    VcPauseState::BuildInfo => self.render_build_info(frame.framebuffer),
                    _ => self.render_menu(frame.framebuffer),
                }
                GameResult::Continue
            }
            VcPauseState::Controls => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_controls(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::BuildInfo => {
                if self.controls.action(VC_PAUSE_TOGGLE).pressed()
                    || self.controls.action(VC_PAUSE_CONFIRM).pressed()
                {
                    self.state = VcPauseState::Menu;
                    self.render_menu(frame.framebuffer);
                } else {
                    self.render_build_info(frame.framebuffer);
                }
                GameResult::Continue
            }
            VcPauseState::ResumeGate => {
                if self.pause_input_held() {
                    self.render_menu(frame.framebuffer);
                } else {
                    self.state = VcPauseState::Running;
                }
                GameResult::Continue
            }
        }
    }
}

pub fn run_v17_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePauseV17::new(VoidCanticleV17::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v17_tests {
    use super::*;

    #[test]
    fn projectile_visuals_construct_from_authored_sprites() {
        let _ = V17ProjectileVisuals::new();
    }

    #[test]
    fn mutation_stack_count_tracks_all_mutation_families() {
        let mut game = VoidCanticleV17::new();
        game.combat.combat.combat.combat.mutations = MutationBuild {
            piercing_lance: 1,
            split_volley: 2,
            death_nova: 1,
            orbitals: 3,
        };
        assert_eq!(game.mutation_stack_count(), 7);
    }

    #[test]
    fn v17_version_is_explicit() {
        assert_eq!(VC17_VERSION, "VC1.7");
    }
}
