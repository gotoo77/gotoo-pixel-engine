const VC16_VERSION: &str = "VC1.6";

const VOID_STIRRING_AT: u32 = 7;
const VOID_AWAKE_AT: u32 = 15;
const VOID_HOSTILE_AT: u32 = 24;
const VOID_CATACLYSMIC_AT: u32 = 34;
const VOID_PRESSURE_BANNER: f32 = 2.0;
const VOID_RETALIATION_FLASH: f32 = 0.42;
const VOID_WARNING_TIME: f32 = 0.46;

const VOID_DIM: Pixel = Pixel::rgb(80, 66, 118);
const VOID_GLOW: Pixel = Pixel::rgb(172, 86, 238);
const VOID_DANGER: Pixel = Pixel::rgb(248, 70, 126);
const VOID_CATA: Pixel = Pixel::rgb(255, 208, 104);
const VOID_LIGHT: Pixel = Pixel::rgb(235, 224, 255);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum VoidPressure {
    Dormant,
    Stirring,
    Awake,
    Hostile,
    Cataclysmic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoidAttackKind {
    TwinGates,
    BlackSun,
    ChoirRain,
}

#[derive(Debug, Clone, Copy)]
struct PendingVoidAttack {
    kind: VoidAttackKind,
    target_x: f32,
    target_y: f32,
}

struct VoidCanticleV16 {
    combat: VoidCanticleV15,
    pressure: VoidPressure,
    pressure_banner_timer: f32,
    retaliation_flash: f32,
    attack_timer: f32,
    warning_timer: f32,
    pending_attack: Option<PendingVoidAttack>,
    attack_step: u32,
}

impl VoidCanticleV16 {
    fn new() -> Self {
        Self {
            combat: VoidCanticleV15::new(),
            pressure: VoidPressure::Dormant,
            pressure_banner_timer: 0.0,
            retaliation_flash: 0.0,
            attack_timer: 99.0,
            warning_timer: 0.0,
            pending_attack: None,
            attack_step: 0,
        }
    }

    fn reset_pressure_layer(&mut self) {
        self.pressure = VoidPressure::Dormant;
        self.pressure_banner_timer = 0.0;
        self.retaliation_flash = 0.0;
        self.attack_timer = 99.0;
        self.warning_timer = 0.0;
        self.pending_attack = None;
        self.attack_step = 0;
    }

    fn reset_run(&mut self) {
        self.combat.reset_run();
        self.reset_pressure_layer();
    }

    fn base(&self) -> &VoidCanticle {
        &self
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base
    }

    fn desired_pressure(&self) -> VoidPressure {
        void_pressure(
            self.combat.build(),
            self.combat.mutations(),
            self.combat.combat.progression.level,
        )
    }

    fn refresh_pressure(&mut self, frame: &mut Frame<'_>) {
        let desired = self.desired_pressure();
        if desired <= self.pressure {
            return;
        }

        self.pressure = desired;
        self.pressure_banner_timer = VOID_PRESSURE_BANNER;
        self.retaliation_flash = VOID_RETALIATION_FLASH;
        self.pending_attack = None;
        self.warning_timer = 0.0;
        self.attack_timer = (void_attack_interval(self.pressure) * 0.65).max(0.8);

        let color = void_pressure_color(self.pressure);
        let base = &mut self
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base;
        base.bursts
            .push(Burst::new(FRAMEBUFFER_WIDTH as f32 / 2.0, 58.0, 0.58, color));
        let _ = base.sounds.play(frame.audio, BELL_SOUND);
    }

    fn pressure_attack_window(&self) -> bool {
        if !self.combat.gameplay_is_running() || self.pressure == VoidPressure::Dormant {
            return false;
        }

        let phase = self.base().encounter_phase;
        match phase {
            EncounterPhase::Waves => {
                self.combat
                    .combat
                    .progression
                    .combat
                    .combat
                    .ui
                    .inner
                    .intermission
                    <= 0.0
            }
            EncounterPhase::BossFight => true,
            EncounterPhase::BossIntro | EncounterPhase::Cleared => false,
        }
    }

    fn schedule_void_attack(&mut self) {
        let kind = void_attack_for_step(self.pressure, self.attack_step);
        let base = self.base();
        self.pending_attack = Some(PendingVoidAttack {
            kind,
            target_x: base.player_x,
            target_y: base.player_y,
        });
        self.warning_timer = VOID_WARNING_TIME;
        self.attack_step = self.attack_step.wrapping_add(1);
    }

    fn update_void_attacks(&mut self, dt: f32, frame: &mut Frame<'_>) {
        if !self.pressure_attack_window() {
            return;
        }

        if self.pending_attack.is_some() {
            self.warning_timer = (self.warning_timer - dt).max(0.0);
            if self.warning_timer <= 0.0
                && let Some(attack) = self.pending_attack.take()
            {
                self.fire_void_attack(attack, frame);
                self.attack_timer = void_attack_interval(self.pressure);
            }
            return;
        }

        self.attack_timer = (self.attack_timer - dt).max(0.0);
        if self.attack_timer <= 0.0 {
            self.schedule_void_attack();
        }
    }

    fn fire_void_attack(&mut self, attack: PendingVoidAttack, frame: &mut Frame<'_>) {
        let mut bullets = Vec::new();
        match attack.kind {
            VoidAttackKind::TwinGates => {
                let fan_count = match self.pressure {
                    VoidPressure::Dormant | VoidPressure::Stirring | VoidPressure::Awake => 3,
                    VoidPressure::Hostile | VoidPressure::Cataclysmic => 5,
                };
                let speed = match self.pressure {
                    VoidPressure::Dormant | VoidPressure::Stirring => 76.0,
                    VoidPressure::Awake => 86.0,
                    VoidPressure::Hostile => 96.0,
                    VoidPressure::Cataclysmic => 110.0,
                };
                spawn_void_aimed_fan(
                    &mut bullets,
                    28.0,
                    48.0,
                    attack.target_x,
                    attack.target_y,
                    fan_count,
                    0.13,
                    speed,
                );
                spawn_void_aimed_fan(
                    &mut bullets,
                    FRAMEBUFFER_WIDTH as f32 - 28.0,
                    48.0,
                    attack.target_x,
                    attack.target_y,
                    fan_count,
                    0.13,
                    speed,
                );
            }
            VoidAttackKind::BlackSun => {
                let count = match self.pressure {
                    VoidPressure::Dormant | VoidPressure::Stirring => 8,
                    VoidPressure::Awake => 10,
                    VoidPressure::Hostile => 12,
                    VoidPressure::Cataclysmic => 16,
                };
                let speed = match self.pressure {
                    VoidPressure::Dormant | VoidPressure::Stirring => 62.0,
                    VoidPressure::Awake => 70.0,
                    VoidPressure::Hostile => 78.0,
                    VoidPressure::Cataclysmic => 90.0,
                };
                let rotation = self.attack_step as f32 * 0.37;
                spawn_ring(
                    &mut bullets,
                    FRAMEBUFFER_WIDTH as f32 / 2.0,
                    72.0,
                    count,
                    rotation,
                    speed,
                );
            }
            VoidAttackKind::ChoirRain => {
                let lane_count = if self.pressure == VoidPressure::Cataclysmic {
                    11
                } else {
                    9
                };
                let gap = distant_gap_lane(attack.target_x, lane_count);
                let speed = if self.pressure == VoidPressure::Cataclysmic {
                    132.0
                } else {
                    108.0
                };
                spawn_choir_rain(&mut bullets, lane_count, gap, speed, self.attack_step);
            }
        }

        if bullets.is_empty() {
            return;
        }

        let color = void_pressure_color(self.pressure);
        let base = &mut self
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
            .inner
            .base;
        base.enemy_bullets.extend(bullets);
        match attack.kind {
            VoidAttackKind::TwinGates => {
                base.bursts.push(Burst::new(28.0, 48.0, 0.25, color));
                base.bursts.push(Burst::new(
                    FRAMEBUFFER_WIDTH as f32 - 28.0,
                    48.0,
                    0.25,
                    color,
                ));
            }
            VoidAttackKind::BlackSun => {
                base.bursts.push(Burst::new(
                    FRAMEBUFFER_WIDTH as f32 / 2.0,
                    72.0,
                    0.33,
                    color,
                ));
            }
            VoidAttackKind::ChoirRain => {
                base.bursts.push(Burst::new(
                    FRAMEBUFFER_WIDTH as f32 / 2.0,
                    24.0,
                    0.27,
                    color,
                ));
            }
        }
        let _ = base.sounds.play(frame.audio, ENEMY_FIRE_SOUND);
    }

    fn render_pressure_fx(&self, framebuffer: &mut Framebuffer) {
        if self.retaliation_flash > 0.0 {
            let color = void_pressure_color(self.pressure);
            framebuffer.draw_rect(2, 20, FRAMEBUFFER_WIDTH - 4, FRAMEBUFFER_HEIGHT - 24, color);
        }

        if let Some(attack) = self.pending_attack {
            let pulse = ((self.warning_timer * 20.0).floor() as i32 & 1) == 0;
            let color = if pulse { VOID_LIGHT } else { void_pressure_color(self.pressure) };
            match attack.kind {
                VoidAttackKind::TwinGates => {
                    let tx = attack.target_x.round() as i32;
                    let ty = attack.target_y.round() as i32;
                    for x in [28_i32, FRAMEBUFFER_WIDTH as i32 - 28] {
                        framebuffer.draw_circle(x, 48, 7, color);
                        framebuffer.draw_line(x, 48, tx, ty, VOID_DIM);
                    }
                }
                VoidAttackKind::BlackSun => {
                    framebuffer.draw_circle(90, 72, 9, color);
                    framebuffer.draw_circle(90, 72, 13, VOID_DIM);
                    framebuffer.draw(90, 72, VOID_CATA);
                }
                VoidAttackKind::ChoirRain => {
                    let lane_count = if self.pressure == VoidPressure::Cataclysmic {
                        11
                    } else {
                        9
                    };
                    let gap = distant_gap_lane(attack.target_x, lane_count);
                    for lane in 0..lane_count {
                        let x = rain_lane_x(lane, lane_count).round() as i32;
                        if lane == gap {
                            framebuffer.draw_line(x, 21, x, 34, VOID_LIGHT);
                        } else {
                            framebuffer.draw_line(x, 21, x, 29, color);
                        }
                    }
                }
            }
        }

        if self.pressure_banner_timer > 0.0 && self.pressure != VoidPressure::Dormant {
            gotoo_pixel_engine::ui::draw_panel(
                framebuffer,
                gotoo_pixel_engine::Rect {
                    x: 23,
                    y: 74,
                    width: 134,
                    height: 31,
                },
                Pixel::rgb(8, 7, 18),
                void_pressure_color(self.pressure),
            );
            framebuffer.draw_text(61, 81, "VOID", VOID_LIGHT);
            let name = void_pressure_name(self.pressure);
            let text_width = name.len() as i32 * 6;
            framebuffer.draw_text(
                ((FRAMEBUFFER_WIDTH as i32 - text_width) / 2).max(28),
                94,
                name,
                void_pressure_color(self.pressure),
            );
        }
    }
}

impl Game for VoidCanticleV16 {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let was_game_over = self.base().game_over;
        let synergy_count_before = self.combat.active_synergy_count();
        let dt = frame.delta_time.as_secs_f32().min(0.05);

        let result = self.combat.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let game_over = self.base().game_over;
        if was_game_over && !game_over {
            self.reset_pressure_layer();
        }

        self.refresh_pressure(frame);
        let synergy_count_after = self.combat.active_synergy_count();
        if synergy_count_after > synergy_count_before && self.pressure > VoidPressure::Dormant {
            self.retaliation_flash = VOID_RETALIATION_FLASH;
            self.attack_timer = self.attack_timer.min(0.72);
        }

        self.pressure_banner_timer = (self.pressure_banner_timer - dt).max(0.0);
        self.retaliation_flash = (self.retaliation_flash - dt).max(0.0);
        self.update_void_attacks(dt, frame);
        self.render_pressure_fx(frame.framebuffer);
        GameResult::Continue
    }
}

fn void_pressure(build: BuildState, mutations: MutationBuild, level: u32) -> VoidPressure {
    void_pressure_from_score(void_pressure_score(build, mutations, level))
}

fn void_pressure_score(build: BuildState, mutations: MutationBuild, level: u32) -> u32 {
    let build_score = build
        .rapid_fire
        .saturating_mul(2)
        .saturating_add(build.stellar_power.saturating_mul(2))
        .saturating_add(build.magnet_field)
        .saturating_add(build.xp_hunger)
        .saturating_add(build.vital_spark)
        .saturating_add(build.core_surge);
    let mutation_score = mutations
        .piercing_lance
        .saturating_add(mutations.split_volley)
        .saturating_add(mutations.death_nova)
        .saturating_add(mutations.orbitals)
        .saturating_mul(3);
    let synergy_score = synergy_mask(build, mutations).count_ones().saturating_mul(4);

    level
        .saturating_sub(1)
        .saturating_add(build_score)
        .saturating_add(mutation_score)
        .saturating_add(synergy_score)
}

fn void_pressure_from_score(score: u32) -> VoidPressure {
    if score >= VOID_CATACLYSMIC_AT {
        VoidPressure::Cataclysmic
    } else if score >= VOID_HOSTILE_AT {
        VoidPressure::Hostile
    } else if score >= VOID_AWAKE_AT {
        VoidPressure::Awake
    } else if score >= VOID_STIRRING_AT {
        VoidPressure::Stirring
    } else {
        VoidPressure::Dormant
    }
}

fn void_pressure_name(pressure: VoidPressure) -> &'static str {
    match pressure {
        VoidPressure::Dormant => "DORMANT",
        VoidPressure::Stirring => "STIRRING",
        VoidPressure::Awake => "AWAKE",
        VoidPressure::Hostile => "HOSTILE",
        VoidPressure::Cataclysmic => "CATACLYSMIC",
    }
}

fn void_pressure_color(pressure: VoidPressure) -> Pixel {
    match pressure {
        VoidPressure::Dormant => VOID_DIM,
        VoidPressure::Stirring => VOID_GLOW,
        VoidPressure::Awake => Pixel::rgb(220, 76, 200),
        VoidPressure::Hostile => VOID_DANGER,
        VoidPressure::Cataclysmic => VOID_CATA,
    }
}

fn void_attack_interval(pressure: VoidPressure) -> f32 {
    match pressure {
        VoidPressure::Dormant => 99.0,
        VoidPressure::Stirring => 3.6,
        VoidPressure::Awake => 2.9,
        VoidPressure::Hostile => 2.25,
        VoidPressure::Cataclysmic => 1.65,
    }
}

fn void_attack_for_step(pressure: VoidPressure, step: u32) -> VoidAttackKind {
    match pressure {
        VoidPressure::Dormant | VoidPressure::Stirring => VoidAttackKind::TwinGates,
        VoidPressure::Awake => {
            if step.is_multiple_of(2) {
                VoidAttackKind::TwinGates
            } else {
                VoidAttackKind::BlackSun
            }
        }
        VoidPressure::Hostile => match step % 3 {
            0 => VoidAttackKind::TwinGates,
            1 => VoidAttackKind::ChoirRain,
            _ => VoidAttackKind::BlackSun,
        },
        VoidPressure::Cataclysmic => match step % 4 {
            0 => VoidAttackKind::ChoirRain,
            1 => VoidAttackKind::TwinGates,
            2 => VoidAttackKind::BlackSun,
            _ => VoidAttackKind::TwinGates,
        },
    }
}

fn spawn_void_aimed_fan(
    output: &mut Vec<Bullet>,
    x: f32,
    y: f32,
    target_x: f32,
    target_y: f32,
    count: usize,
    spread: f32,
    speed: f32,
) {
    let base_angle = (target_y - y).atan2(target_x - x);
    let center = (count.saturating_sub(1)) as f32 / 2.0;
    for index in 0..count {
        let angle = base_angle + (index as f32 - center) * spread;
        output.push(Bullet {
            x,
            y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            alive: true,
            alternate: index % 2 == 1,
        });
    }
}

fn distant_gap_lane(target_x: f32, lane_count: usize) -> usize {
    let lane_count = lane_count.max(1);
    let normalized = (target_x / FRAMEBUFFER_WIDTH as f32).clamp(0.0, 0.999_9);
    let player_lane = (normalized * lane_count as f32) as usize;
    (player_lane + lane_count / 2) % lane_count
}

fn rain_lane_x(lane: usize, lane_count: usize) -> f32 {
    let spacing = FRAMEBUFFER_WIDTH as f32 / (lane_count as f32 + 1.0);
    spacing * (lane as f32 + 1.0)
}

fn spawn_choir_rain(
    output: &mut Vec<Bullet>,
    lane_count: usize,
    gap: usize,
    speed: f32,
    step: u32,
) {
    for lane in 0..lane_count {
        if lane == gap {
            continue;
        }
        let x = rain_lane_x(lane, lane_count);
        let drift = if (lane as u32 + step).is_multiple_of(2) {
            9.0
        } else {
            -9.0
        };
        output.push(Bullet {
            x,
            y: 22.0,
            vx: drift,
            vy: speed,
            alive: true,
            alternate: lane % 2 == 1,
        });
    }
}

struct VoidCanticlePauseV16 {
    game: VoidCanticleV16,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV16 {
    fn new(game: VoidCanticleV16) -> Self {
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
        framebuffer.draw_text(20, 97, &format!("VERSION {VC16_VERSION}"), TEXT);
        framebuffer.draw_text(20, 117, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(
            20,
            137,
            &format!("SYNERGIES {}", self.game.combat.active_synergy_count()),
            SYNERGY_LIGHT,
        );
        framebuffer.draw_text(
            20,
            157,
            &format!("VOID {}", void_pressure_name(self.game.pressure)),
            void_pressure_color(self.game.pressure),
        );
        framebuffer.draw_text(20, 177, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(31, 225, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV16 {
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

pub fn run_v16_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePauseV16::new(VoidCanticleV16::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v16_tests {
    use super::*;

    #[test]
    fn pressure_thresholds_follow_documented_order() {
        assert_eq!(void_pressure_from_score(0), VoidPressure::Dormant);
        assert_eq!(void_pressure_from_score(VOID_STIRRING_AT), VoidPressure::Stirring);
        assert_eq!(void_pressure_from_score(VOID_AWAKE_AT), VoidPressure::Awake);
        assert_eq!(void_pressure_from_score(VOID_HOSTILE_AT), VoidPressure::Hostile);
        assert_eq!(
            void_pressure_from_score(VOID_CATACLYSMIC_AT),
            VoidPressure::Cataclysmic
        );
    }

    #[test]
    fn coherent_build_wakes_void_faster_than_isolated_upgrade() {
        let isolated = BuildState {
            rapid_fire: 1,
            ..BuildState::default()
        };
        let coherent = isolated;
        let split = MutationBuild {
            split_volley: 1,
            ..MutationBuild::default()
        };

        assert_eq!(void_pressure(isolated, MutationBuild::default(), 2), VoidPressure::Dormant);
        assert!(void_pressure_score(coherent, split, 2) > void_pressure_score(isolated, MutationBuild::default(), 2));
        assert_eq!(void_pressure(coherent, split, 2), VoidPressure::Stirring);
    }

    #[test]
    fn pressure_increases_attack_frequency() {
        assert!(void_attack_interval(VoidPressure::Awake) < void_attack_interval(VoidPressure::Stirring));
        assert!(void_attack_interval(VoidPressure::Hostile) < void_attack_interval(VoidPressure::Awake));
        assert!(
            void_attack_interval(VoidPressure::Cataclysmic)
                < void_attack_interval(VoidPressure::Hostile)
        );
    }

    #[test]
    fn hostile_pressure_rotates_three_distinct_counter_patterns() {
        let attacks = [
            void_attack_for_step(VoidPressure::Hostile, 0),
            void_attack_for_step(VoidPressure::Hostile, 1),
            void_attack_for_step(VoidPressure::Hostile, 2),
        ];
        assert!(attacks.contains(&VoidAttackKind::TwinGates));
        assert!(attacks.contains(&VoidAttackKind::BlackSun));
        assert!(attacks.contains(&VoidAttackKind::ChoirRain));
    }

    #[test]
    fn choir_rain_keeps_one_readable_escape_lane() {
        let mut bullets = Vec::new();
        spawn_choir_rain(&mut bullets, 9, 4, 100.0, 0);
        assert_eq!(bullets.len(), 8);
        let gap_x = rain_lane_x(4, 9);
        assert!(bullets.iter().all(|bullet| (bullet.x - gap_x).abs() > 0.01));
    }

    #[test]
    fn vc16_version_is_explicit() {
        assert_eq!(VC16_VERSION, "VC1.6");
    }
}
