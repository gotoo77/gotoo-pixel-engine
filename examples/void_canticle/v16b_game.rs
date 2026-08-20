const VC16B_VERSION: &str = "VC1.6b";

const VOID_REVEAL_DURATION: f32 = 1.45;
const VOID_TELEGRAPH_DURATION: f32 = 0.78;
const BOSS_PHASE_BANNER_DURATION: f32 = 0.95;
const PICKUP_SNAP_RADIUS: f32 = 13.0;
const CANTICLE_READY_FLASH_DURATION: f32 = 0.72;
const BOSS_HIT_FLASH_DURATION: f32 = 0.11;
const BOSS_HIT_SOUND_COOLDOWN: f32 = 0.055;

const CANTICLE_READY_SOUND: SoundId = SoundId::new("void_canticle.canticle_ready");
const BOSS_HIT_SOUND: SoundId = SoundId::new("void_canticle.boss_hit");

const JUICE_TRAIL_SCALE: f32 = 0.045;
const JUICE_PARTICLE_GRAVITY: f32 = 22.0;

#[derive(Debug, Clone, Copy)]
struct JuiceParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: Pixel,
}

struct VoidCanticleV16B {
    combat: VoidCanticleV16,
    pressure_reveal_timer: f32,
    pressure_reveal: VoidPressure,
    pending_seen: bool,
    boss_phase_banner_timer: f32,
    boss_phase_banner: Option<BellPhase>,
    last_boss_phase: Option<BellPhase>,
    last_boss_hp: Option<u32>,
    boss_hit_flash_timer: f32,
    boss_hit_sound_cooldown: f32,
    canticle_ready_flash_timer: f32,
    canticle_was_ready: bool,
    particles: Vec<JuiceParticle>,
}

impl VoidCanticleV16B {
    fn new() -> Self {
        let mut game = Self {
            combat: VoidCanticleV16::new(),
            pressure_reveal_timer: 0.0,
            pressure_reveal: VoidPressure::Dormant,
            pending_seen: false,
            boss_phase_banner_timer: 0.0,
            boss_phase_banner: None,
            last_boss_phase: None,
            last_boss_hp: None,
            boss_hit_flash_timer: 0.0,
            boss_hit_sound_cooldown: 0.0,
            canticle_ready_flash_timer: 0.0,
            canticle_was_ready: false,
            particles: Vec::new(),
        };
        game.base_mut()
            .sounds
            .insert_wav(
                CANTICLE_READY_SOUND,
                synthesize_chirp(560.0, 1040.0, 0.14, 0.16),
            )
            .expect("VC Canticle ready sound id should be unique");
        game.base_mut()
            .sounds
            .insert_wav(
                BOSS_HIT_SOUND,
                synthesize_noise_burst(0.055, 0.16, 0xB055_0170),
            )
            .expect("VC boss hit sound id should be unique");
        game
    }

    fn reset_polish_layer(&mut self) {
        self.pressure_reveal_timer = 0.0;
        self.pressure_reveal = VoidPressure::Dormant;
        self.pending_seen = false;
        self.boss_phase_banner_timer = 0.0;
        self.boss_phase_banner = None;
        self.last_boss_phase = None;
        self.last_boss_hp = None;
        self.boss_hit_flash_timer = 0.0;
        self.boss_hit_sound_cooldown = 0.0;
        self.canticle_ready_flash_timer = 0.0;
        self.canticle_was_ready = false;
        self.particles.clear();
    }

    fn reset_run(&mut self) {
        self.combat.reset_run();
        self.reset_polish_layer();
    }

    fn base(&self) -> &VoidCanticleGame {
        self.combat.base()
    }

    fn base_mut(&mut self) -> &mut VoidCanticleGame {
        &mut self
            .combat
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

    fn v09_mut(&mut self) -> &mut VoidCanticleV09 {
        &mut self
            .combat
            .combat
            .combat
            .progression
            .combat
            .combat
            .ui
            .inner
    }

    fn progression_mut(&mut self) -> &mut VoidCanticleV13 {
        &mut self.combat.combat.combat.progression
    }

    fn apply_pacing_before_update(&mut self, dt: f32) {
        {
            let progression = self.progression_mut();
            progression.xp_next = paced_xp_requirement(progression.level);
        }

        if !self.combat.combat.gameplay_is_running() {
            return;
        }

        {
            let v09 = self.v09_mut();
            if v09.inner.base.encounter_phase == EncounterPhase::Waves
                && v09.wave_index < V09_WAVES.len()
                && v09.intermission <= 0.0
            {
                let rate = wave_clock_rate(v09.wave_index);
                let compensation = dt * (1.0 - rate);
                v09.wave_time = (v09.wave_time - compensation).max(0.0);
            }
        }

        self.snap_close_pickups();
    }

    fn snap_close_pickups(&mut self) {
        let (player_x, player_y) = {
            let base = self.base();
            (base.player_x, base.player_y)
        };

        {
            let progression = self.progression_mut();
            for orb in &mut progression.xp_orbs {
                if orb.alive && pickup_should_snap(orb.x, orb.y, player_x, player_y) {
                    orb.x = player_x;
                    orb.y = player_y;
                    orb.vx = 0.0;
                    orb.vy = 0.0;
                }
            }
        }

        let base = self.base_mut();
        for cinder in &mut base.cinders {
            if cinder.alive && pickup_should_snap(cinder.x, cinder.y, player_x, player_y) {
                cinder.x = player_x;
                cinder.y = player_y;
            }
        }
    }

    fn observe_pressure_transition(&mut self, previous: VoidPressure) {
        if self.combat.pressure == previous {
            return;
        }

        self.pressure_reveal = self.combat.pressure;
        self.pressure_reveal_timer = VOID_REVEAL_DURATION;
        self.combat.pending_attack = None;
        self.combat.warning_timer = 0.0;
        self.combat.attack_timer = self.combat.attack_timer.max(1.55);
        self.pending_seen = false;
    }

    fn extend_new_void_telegraph(&mut self) {
        if self.combat.pending_attack.is_some() {
            if !self.pending_seen {
                self.combat.warning_timer = self.combat.warning_timer.max(VOID_TELEGRAPH_DURATION);
                self.pending_seen = true;
            }
        } else {
            self.pending_seen = false;
        }
    }

    fn observe_boss_phase(&mut self) {
        let current = if self.base().encounter_phase == EncounterPhase::BossFight {
            self.base().boss.map(Bellkeeper::phase)
        } else {
            None
        };

        if current != self.last_boss_phase {
            if let Some(phase) = current {
                self.boss_phase_banner = Some(phase);
                self.boss_phase_banner_timer = BOSS_PHASE_BANNER_DURATION;
            }
            self.last_boss_phase = current;
        }
    }

    fn observe_boss_damage(&mut self, frame: &mut Frame<'_>) {
        let current = self.base().boss.map(|boss| (boss.x, boss.y, boss.hp));

        if let (Some(previous_hp), Some((x, y, hp))) = (self.last_boss_hp, current) {
            if hp < previous_hp {
                self.boss_hit_flash_timer = BOSS_HIT_FLASH_DURATION;
                let hit_count = (previous_hp - hp).min(6) as usize;
                let particle_count = 4 + hit_count * 2;
                for index in 0..particle_count {
                    let angle = index as f32 * std::f32::consts::TAU / particle_count as f32;
                    let speed = 20.0 + (index % 3) as f32 * 8.0;
                    self.particles.push(JuiceParticle {
                        x,
                        y: y + 4.0,
                        vx: angle.cos() * speed,
                        vy: angle.sin() * speed,
                        life: 0.16 + (index % 2) as f32 * 0.05,
                        color: BELL_LIGHT,
                    });
                }

                if self.boss_hit_sound_cooldown <= 0.0 {
                    let _ = self.base_mut().sounds.play(frame.audio, BOSS_HIT_SOUND);
                    self.boss_hit_sound_cooldown = BOSS_HIT_SOUND_COOLDOWN;
                }
            }
        }

        self.last_boss_hp = current.map(|(_, _, hp)| hp);
    }

    fn observe_canticle_ready(&mut self, frame: &mut Frame<'_>) {
        let core_charge = self.base().core_charge;
        let ready = core_charge >= CORE_MAX;
        if canticle_ready_crossed(self.canticle_was_ready, core_charge) {
            self.canticle_ready_flash_timer = CANTICLE_READY_FLASH_DURATION;
            let _ = self
                .base_mut()
                .sounds
                .play(frame.audio, CANTICLE_READY_SOUND);
        }
        self.canticle_was_ready = ready;
    }

    fn update_particles(&mut self, dt: f32) {
        for particle in &mut self.particles {
            particle.life = (particle.life - dt).max(0.0);
            particle.vy += JUICE_PARTICLE_GRAVITY * dt;
            particle.x += particle.vx * dt;
            particle.y += particle.vy * dt;
            let damping = (1.0 - 2.2 * dt).max(0.0);
            particle.vx *= damping;
            particle.vy *= damping;
        }
        self.particles.retain(|particle| particle.life > 0.0);
    }

    fn spawn_particles_from_new_bursts(&mut self) {
        let bursts: Vec<Burst> = self
            .base()
            .bursts
            .iter()
            .copied()
            .filter(|burst| {
                (burst.remaining - burst.duration).abs() <= 0.0001 && burst.duration >= 0.18
            })
            .collect();

        for burst in bursts {
            let count = if burst.duration >= 0.50 {
                12
            } else if burst.duration >= 0.34 {
                8
            } else {
                5
            };
            let seed = burst.x * 0.071 + burst.y * 0.113 + burst.duration * 3.7;
            for index in 0..count {
                let angle = seed
                    + index as f32 * std::f32::consts::TAU / count as f32
                    + (index as f32 * 0.37).sin() * 0.18;
                let speed = 24.0 + (index % 4) as f32 * 10.0;
                self.particles.push(JuiceParticle {
                    x: burst.x,
                    y: burst.y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    life: 0.24 + (index % 3) as f32 * 0.055,
                    color: burst.color,
                });
            }
        }
    }

    fn render_projectile_juice(&self, framebuffer: &mut Framebuffer) {
        for bullet in &self.base().enemy_bullets {
            let x = bullet.x.round() as i32;
            let y = bullet.y.round() as i32;
            let tail_x = (bullet.x - bullet.vx * JUICE_TRAIL_SCALE).round() as i32;
            let tail_y = (bullet.y - bullet.vy * JUICE_TRAIL_SCALE).round() as i32;
            let color = if bullet.alternate {
                ENEMY_SHOT_ALT
            } else {
                ENEMY_SHOT
            };
            framebuffer.draw_line(tail_x, tail_y, x, y, color);
            framebuffer.draw(x, y, VOID_LIGHT);
        }

        let game = &self
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
            let tail_x = (shot.x - shot.vx * 0.032).round() as i32;
            let tail_y = (shot.y - shot.vy * 0.032).round() as i32;
            framebuffer.draw_line(tail_x, tail_y, x, y, POWER_RELIC_LIGHT);
            framebuffer.draw(x, y, SHOT);
        }

        for orb in &self.combat.combat.combat.progression.xp_orbs {
            let x = orb.x.round() as i32;
            let y = orb.y.round() as i32;
            let tail_x = (orb.x - orb.vx * 0.040).round() as i32;
            let tail_y = (orb.y - orb.vy * 0.040).round() as i32;
            framebuffer.draw_line(tail_x, tail_y, x, y, XP_ORB);
            framebuffer.draw(x, y, XP_ORB_CORE);
        }
    }

    fn render_particles(&self, framebuffer: &mut Framebuffer) {
        for particle in &self.particles {
            let x = particle.x.round() as i32;
            let y = particle.y.round() as i32;
            let tail_x = (particle.x - particle.vx * 0.035).round() as i32;
            let tail_y = (particle.y - particle.vy * 0.035).round() as i32;
            framebuffer.draw_line(tail_x, tail_y, x, y, particle.color);
            framebuffer.draw(x, y, particle.color);
        }
    }

    fn render_boss_hit_feedback(&self, framebuffer: &mut Framebuffer) {
        if self.boss_hit_flash_timer <= 0.0 {
            return;
        }
        if let Some(boss) = self.base().boss {
            if self.base().encounter_phase != EncounterPhase::Cleared {
                let x = boss.x.round() as i32;
                let y = boss.y.round() as i32;
                framebuffer.draw_circle(x, y, 28, BELL_LIGHT);
                framebuffer.draw_circle(x, y, 31, CANTICLE_COLOR);
            }
        }
    }

    fn render_polish(&self, framebuffer: &mut Framebuffer) {
        self.render_projectile_juice(framebuffer);
        self.render_particles(framebuffer);
        self.render_boss_hit_feedback(framebuffer);
    }
}

impl Game for VoidCanticleV16B {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let dt = frame.delta_time.as_secs_f32().min(0.05);
        let previous_pressure = self.combat.pressure;
        let was_game_over = self.base().game_over;

        self.apply_pacing_before_update(dt);
        let result = self.combat.update(frame);
        if result == GameResult::Exit {
            return result;
        }

        let game_over = self.base().game_over;
        if was_game_over && !game_over {
            self.reset_polish_layer();
        }

        self.pressure_reveal_timer = (self.pressure_reveal_timer - dt).max(0.0);
        self.boss_phase_banner_timer = (self.boss_phase_banner_timer - dt).max(0.0);
        self.boss_hit_flash_timer = (self.boss_hit_flash_timer - dt).max(0.0);
        self.boss_hit_sound_cooldown = (self.boss_hit_sound_cooldown - dt).max(0.0);
        self.canticle_ready_flash_timer = (self.canticle_ready_flash_timer - dt).max(0.0);

        self.observe_pressure_transition(previous_pressure);
        self.extend_new_void_telegraph();
        self.observe_boss_phase();
        self.observe_boss_damage(frame);
        self.observe_canticle_ready(frame);

        self.update_particles(dt);
        self.spawn_particles_from_new_bursts();
        self.render_polish(frame.framebuffer);
        GameResult::Continue
    }
}

fn paced_xp_requirement(level: u32) -> u32 {
    match level {
        0 | 1 => 30,
        2 => 44,
        3 => 68,
        _ => {
            let mut required = 68_u32;
            for _ in 3..level {
                required = required.saturating_mul(148).saturating_add(50) / 100;
            }
            required
        }
    }
}

fn wave_clock_rate(wave_index: usize) -> f32 {
    match wave_index {
        0..=2 => 0.42,
        3..=7 => 0.22,
        _ => 0.13,
    }
}

fn pickup_should_snap(x: f32, y: f32, player_x: f32, player_y: f32) -> bool {
    point_near(x, y, player_x, player_y, PICKUP_SNAP_RADIUS)
}

fn canticle_ready_crossed(was_ready: bool, core_charge: u32) -> bool {
    !was_ready && core_charge >= CORE_MAX
}

fn pressure_transition_copy(pressure: VoidPressure) -> (&'static str, &'static str) {
    match pressure {
        VoidPressure::Dormant => ("VOID DORMANT", ""),
        VoidPressure::Stirring => ("THE VOID STIRS", "TWIN GATES AWAKEN"),
        VoidPressure::Awake => ("THE VOID IS AWAKE", "BLACK SUN MANIFESTS"),
        VoidPressure::Hostile => ("THE VOID TURNS HOSTILE", "CHOIR RAIN AWAKENS"),
        VoidPressure::Cataclysmic => ("VOID CATACLYSMIC", "ALL PATTERNS UNBOUND"),
    }
}

fn void_pressure_short_name(pressure: VoidPressure) -> &'static str {
    match pressure {
        VoidPressure::Dormant => "DORM",
        VoidPressure::Stirring => "STIR",
        VoidPressure::Awake => "AWAKE",
        VoidPressure::Hostile => "HOST",
        VoidPressure::Cataclysmic => "CATA",
    }
}

fn void_attack_name(kind: VoidAttackKind) -> &'static str {
    match kind {
        VoidAttackKind::TwinGates => "TWIN GATES",
        VoidAttackKind::BlackSun => "BLACK SUN",
        VoidAttackKind::ChoirRain => "CHOIR RAIN",
    }
}

fn bell_phase_name(phase: BellPhase) -> &'static str {
    match phase {
        BellPhase::Procession => "PROCESSION",
        BellPhase::Resonance => "RESONANCE",
        BellPhase::FinalToll => "FINAL TOLL",
    }
}

fn draw_centered_text(framebuffer: &mut Framebuffer, y: i32, text: &str, color: Pixel) {
    let width = text.len() as i32 * 6;
    let x = ((FRAMEBUFFER_WIDTH as i32 - width) / 2).max(4);
    framebuffer.draw_text(x, y, text, color);
}

struct VoidCanticlePauseV16B {
    game: VoidCanticleV16B,
    state: VcPauseState,
    menu: gotoo_pixel_engine::ui::MenuState,
    controls: ControlMap,
}

impl VoidCanticlePauseV16B {
    fn new(game: VoidCanticleV16B) -> Self {
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
        framebuffer.draw_text(20, 97, &format!("VERSION {VC16B_VERSION}"), TEXT);
        framebuffer.draw_text(20, 117, &format!("BUILD {BUILD_ID}"), TEXT);
        framebuffer.draw_text(
            20,
            137,
            &format!("SYNERGIES {}", self.game.combat.combat.active_synergy_count()),
            SYNERGY_LIGHT,
        );
        framebuffer.draw_text(
            20,
            157,
            &format!("VOID {}", void_pressure_name(self.game.combat.pressure)),
            void_pressure_color(self.game.combat.pressure),
        );
        framebuffer.draw_text(20, 177, "STAGE GRAVE ORBIT", TEXT);
        framebuffer.draw_text(31, 225, "ESC START  BACK", WRECK_LIGHT);
    }
}

impl Game for VoidCanticlePauseV16B {
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

pub fn run_v16b_with_obs_mirror() -> Result<(), EngineError> {
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
            VoidCanticlePauseV16B::new(VoidCanticleV16B::new()),
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        ),
    )
}

#[cfg(test)]
mod v16b_tests {
    use super::*;

    #[test]
    fn paced_xp_curve_keeps_fast_opening_then_stretches() {
        assert_eq!(paced_xp_requirement(1), 30);
        assert_eq!(paced_xp_requirement(2), 44);
        assert_eq!(paced_xp_requirement(3), 68);
        assert!(paced_xp_requirement(6) > xp_requirement(6));
        assert!(paced_xp_requirement(8) > paced_xp_requirement(6) * 2);
    }

    #[test]
    fn later_waves_breathe_longer_than_opening_waves() {
        assert!(wave_clock_rate(4) < wave_clock_rate(1));
        assert!(wave_clock_rate(10) < wave_clock_rate(4));
    }

    #[test]
    fn close_pickups_snap_before_magnet_motion_can_overshoot() {
        assert!(PICKUP_SNAP_RADIUS < BASE_MAGNET_RADIUS);
        assert!(pickup_should_snap(10.0, 10.0, 20.0, 10.0));
        assert!(!pickup_should_snap(10.0, 10.0, 30.0, 10.0));
    }

    #[test]
    fn canticle_ready_feedback_is_edge_triggered() {
        assert!(canticle_ready_crossed(false, CORE_MAX));
        assert!(!canticle_ready_crossed(true, CORE_MAX));
        assert!(!canticle_ready_crossed(false, CORE_MAX - 1));
    }

    #[test]
    fn pressure_copy_names_new_counter_pattern() {
        assert_eq!(
            pressure_transition_copy(VoidPressure::Stirring),
            ("THE VOID STIRS", "TWIN GATES AWAKEN")
        );
        assert_eq!(
            pressure_transition_copy(VoidPressure::Awake),
            ("THE VOID IS AWAKE", "BLACK SUN MANIFESTS")
        );
    }

    #[test]
    fn vc16b_version_is_explicit() {
        assert_eq!(VC16B_VERSION, "VC1.6b");
    }
}