use std::time::Duration;

use gotoo_pixel_engine::{
    Audio, AudioError, Frame, Framebuffer, Game, GameResult, GamepadButton, Image, ImageRegion,
    Key, Pixel, PlaybackId, SoundBank, SoundId, TouchPhase,
};
use include_dir::{Dir, include_dir};

#[allow(dead_code)]
#[path = "../smart_boy_hero/world.rs"]
mod world;
use world::{
    BoulderState, Cell, Direction, EnemyIntent, Phase, PlayerAction, ROCK_HEARING_RADIUS,
    SmartBoyWorld, WorldEvent,
};

pub const FRAMEBUFFER_WIDTH: u32 = 520;
pub const FRAMEBUFFER_HEIGHT: u32 = 320;

const FIXED_STEP: Duration = Duration::from_millis(420);
const FEEDBACK_DURATION: Duration = Duration::from_millis(620);
const ROCK_FLIGHT_DURATION: Duration = Duration::from_millis(280);
const TILE_WIDTH: i32 = 32;
const TILE_HEIGHT: i32 = 16;
const SPRITE_SIZE: u32 = 32;
const SPRITE_HEIGHT: u32 = 40;
const VIEWPORT_TOP: i32 = 48;
const VIEWPORT_WIDTH: i32 = FRAMEBUFFER_WIDTH as i32;
const VIEWPORT_HEIGHT: i32 = FRAMEBUFFER_HEIGHT as i32 - VIEWPORT_TOP;
const CAMERA_DEAD_ZONE_MARGIN_X: f32 = 130.0;
const CAMERA_DEAD_ZONE_MARGIN_Y: f32 = 68.0;
const CAMERA_SMOOTH_DURATION: Duration = Duration::from_millis(180);
const DEATH_ANIMATION_DURATION: Duration = Duration::from_millis(760);
const INITIAL_SEED: u32 = 0x150_512CE;
const SFX_ASSET_PREFIX: &str = "assets/smart_boy_hero/";
const SHOUT_SFX_KEY: &str = "shout";
const DEATH_SFX_KEY: &str = "death";
const SFX_CONFIG_JSON: &str = include_str!("../../assets/smart_boy_hero/sfx.json");
static SMART_BOY_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/smart_boy_hero");

const COMBAT_SOUND: SoundId = SoundId::new("smart_boy_hero.combat");
const PRESSURE_PLATE_ON_SOUND: SoundId = SoundId::new("smart_boy_hero.pressure_plate_on");
const PRESSURE_PLATE_OFF_SOUND: SoundId = SoundId::new("smart_boy_hero.pressure_plate_off");
const DOOR_OPEN_SOUND: SoundId = SoundId::new("smart_boy_hero.door_open");
const DOOR_CLOSE_SOUND: SoundId = SoundId::new("smart_boy_hero.door_close");
const TRAP_ARM_SOUND: SoundId = SoundId::new("smart_boy_hero.trap_arm");
const TRAP_DISARM_SOUND: SoundId = SoundId::new("smart_boy_hero.trap_disarm");
const TRAP_TRIGGER_SOUND: SoundId = SoundId::new("smart_boy_hero.trap_trigger");
const SHOUT_SOUND_1: SoundId = SoundId::new("smart_boy_hero.shout_1");
const SHOUT_SOUND_2: SoundId = SoundId::new("smart_boy_hero.shout_2");
const SHOUT_SOUND_3: SoundId = SoundId::new("smart_boy_hero.shout_3");
const SHOUT_SOUNDS: [SoundId; 3] = [SHOUT_SOUND_1, SHOUT_SOUND_2, SHOUT_SOUND_3];
const ROCK_IMPACT_SOUND: SoundId = SoundId::new("smart_boy_hero.rock_impact");
const ENEMY_KILL_SOUND: SoundId = SoundId::new("smart_boy_hero.enemy_kill");
const ENEMY_ALERT_SOUND: SoundId = SoundId::new("smart_boy_hero.enemy_alert");
const BOULDER_RELEASE_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_release");
const BOULDER_ROLL_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_roll");
const BOULDER_CRUSH_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_crush");
const BOULDER_STOP_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_stop");
const KEY_PICKUP_SOUND: SoundId = SoundId::new("smart_boy_hero.key_pickup");
const KEY_UNLOCK_SOUND: SoundId = SoundId::new("smart_boy_hero.key_unlock");
const DEATH_SOUND_1: SoundId = SoundId::new("smart_boy_hero.death_1");
const DEATH_SOUND_2: SoundId = SoundId::new("smart_boy_hero.death_2");
const DEATH_SOUND_3: SoundId = SoundId::new("smart_boy_hero.death_3");
const DEATH_SOUNDS: [SoundId; 3] = [DEATH_SOUND_1, DEATH_SOUND_2, DEATH_SOUND_3];
const VICTORY_SOUND: SoundId = SoundId::new("smart_boy_hero.victory");

const BG: Pixel = Pixel::rgb(9, 11, 15);
const PANEL: Pixel = Pixel::rgb(18, 20, 25);
const TEXT: Pixel = Pixel::rgb(244, 239, 217);
const MUTED: Pixel = Pixel::rgb(132, 144, 152);
const GOLD: Pixel = Pixel::rgb(255, 218, 76);
const POWER: Pixel = Pixel::rgb(255, 246, 104);
const SHOUT_COLOR: Pixel = Pixel::rgb(217, 91, 255);
const ROCK_COLOR: Pixel = Pixel::rgb(190, 222, 214);
const ROCK_INVALID: Pixel = Pixel::rgb(255, 82, 82);
const ROCK_VALID: Pixel = Pixel::rgb(91, 214, 255);
const DANGER: Pixel = Pixel::rgb(255, 56, 68);
const SMART: Pixel = Pixel::rgb(86, 240, 185);
const BRONZE: Pixel = Pixel::rgb(218, 139, 61);
const LOCKED: Pixel = Pixel::rgb(255, 196, 82);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpriteFrame {
    Floor = 0,
    Wall = 1,
    HeroIdle = 2,
    HeroWalk = 3,
    HeroShout = 4,
    WalkerPatrol = 5,
    WalkerAlert = 6,
    WalkerDeath = 7,
    TrapInactive = 8,
    TrapArmed = 9,
    TrapTriggered = 10,
    Plate = 11,
    DoorClosed = 12,
    DoorOpen = 13,
    Boulder0 = 14,
    Boulder1 = 15,
    Boulder2 = 16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScreenPoint {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Motion {
    from: Cell,
    to: Cell,
    elapsed: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimedCell {
    cell: Cell,
    elapsed: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TargetingState {
    target: Cell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RockFlight {
    from: Cell,
    to: Cell,
    elapsed: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Camera {
    offset_x: f32,
    offset_y: f32,
}

impl Camera {
    fn centered_for_world(world: &SmartBoyWorld) -> Self {
        let (min_x, max_x, min_y, max_y) = raw_map_bounds(world);
        let hero = raw_project_cell(world.hero());
        let target_x = hero.x - VIEWPORT_WIDTH as f32 / 2.0;
        let target_y = hero.y - VIEWPORT_HEIGHT as f32 / 2.0;
        let offset_x = clamp_camera_axis(target_x, min_x, max_x, VIEWPORT_WIDTH as f32);
        let offset_y = clamp_camera_axis(target_y, min_y, max_y, VIEWPORT_HEIGHT as f32);

        Self { offset_x, offset_y }
    }

    fn target_for_world(world: &SmartBoyWorld, current: Self) -> Self {
        let (min_x, max_x, min_y, max_y) = raw_map_bounds(world);
        let hero = project_cell(world.hero(), current);
        let left = CAMERA_DEAD_ZONE_MARGIN_X;
        let right = VIEWPORT_WIDTH as f32 - CAMERA_DEAD_ZONE_MARGIN_X;
        let top = VIEWPORT_TOP as f32 + CAMERA_DEAD_ZONE_MARGIN_Y;
        let bottom = FRAMEBUFFER_HEIGHT as f32 - CAMERA_DEAD_ZONE_MARGIN_Y;

        let mut offset_x = current.offset_x;
        let mut offset_y = current.offset_y;
        if hero.x < left {
            offset_x -= left - hero.x;
        } else if hero.x > right {
            offset_x += hero.x - right;
        }
        if hero.y < top {
            offset_y -= top - hero.y;
        } else if hero.y > bottom {
            offset_y += hero.y - bottom;
        }

        Self {
            offset_x: clamp_camera_axis(offset_x, min_x, max_x, VIEWPORT_WIDTH as f32),
            offset_y: clamp_camera_axis(offset_y, min_y, max_y, VIEWPORT_HEIGHT as f32),
        }
    }

    fn move_toward(self, target: Self, delta: Duration) -> Self {
        let t = (delta.as_secs_f32() / CAMERA_SMOOTH_DURATION.as_secs_f32()).clamp(0.0, 1.0);
        Self {
            offset_x: self.offset_x + (target.offset_x - self.offset_x) * t,
            offset_y: self.offset_y + (target.offset_y - self.offset_y) * t,
        }
    }

    fn with_shake(self, shake: (i32, i32)) -> Self {
        Self {
            offset_x: self.offset_x - shake.0 as f32,
            offset_y: self.offset_y - shake.1 as f32,
        }
    }
}

pub struct SmartBoyHeroIsoGame {
    world: SmartBoyWorld,
    sprites: Image,
    sounds: SoundBank,
    camera: Camera,
    camera_target: Camera,
    simulation_accumulator: Duration,
    hero_motion: Option<Motion>,
    enemy_motions: Vec<Option<Motion>>,
    boulder_motions: Vec<Option<Motion>>,
    targeting: Option<TargetingState>,
    rock_flight: Option<RockFlight>,
    rock_impact: Option<TimedCell>,
    shout_pulse: Option<TimedCell>,
    kill_bursts: Vec<TimedCell>,
    smart_flash: Option<(usize, Duration)>,
    key_elapsed: Duration,
    screen_shake: Duration,
    boulder_roll_loop: Option<PlaybackId>,
    sfx_rng: u32,
    feedback_text: Option<(&'static str, Duration)>,
    death_elapsed: Option<Duration>,
    game_over: bool,
}

impl SmartBoyHeroIsoGame {
    pub fn new() -> Self {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let camera = Camera::centered_for_world(&world);
        Self {
            world,
            sprites: Image::decode_png(include_bytes!(
                "../../assets/smart_boy_hero/iso/sprites.png"
            ))
            .expect("checked-in SBH iso sprite sheet should decode"),
            sounds: smart_boy_sound_bank().expect("checked-in SBH SFX config should load"),
            camera,
            camera_target: camera,
            simulation_accumulator: Duration::ZERO,
            hero_motion: None,
            enemy_motions: Vec::new(),
            boulder_motions: Vec::new(),
            targeting: None,
            rock_flight: None,
            rock_impact: None,
            shout_pulse: None,
            kill_bursts: Vec::new(),
            smart_flash: None,
            key_elapsed: Duration::ZERO,
            screen_shake: Duration::ZERO,
            boulder_roll_loop: None,
            sfx_rng: INITIAL_SEED ^ 0x0A11_D150,
            feedback_text: None,
            death_elapsed: None,
            game_over: false,
        }
    }

    fn restart(&mut self) {
        self.world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        self.camera = Camera::centered_for_world(&self.world);
        self.camera_target = self.camera;
        self.simulation_accumulator = Duration::ZERO;
        self.hero_motion = None;
        self.enemy_motions.clear();
        self.boulder_motions.clear();
        self.targeting = None;
        self.rock_flight = None;
        self.rock_impact = None;
        self.shout_pulse = None;
        self.kill_bursts.clear();
        self.smart_flash = None;
        self.key_elapsed = Duration::ZERO;
        self.screen_shake = Duration::ZERO;
        self.feedback_text = None;
        self.death_elapsed = None;
        self.game_over = false;
    }

    fn restart_with_audio(&mut self, audio: &mut dyn Audio) {
        self.stop_boulder_roll(audio);
        self.restart();
    }

    fn update_running(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.game_over || self.death_elapsed.is_some() {
            return self.update_death_or_game_over(frame);
        }

        if pressed(frame, Key::R, GamepadButton::West) {
            self.restart_with_audio(frame.audio);
            return GameResult::Continue;
        }

        if self.targeting.is_some() {
            self.update_targeting(frame);
            self.advance_transients(frame.delta_time);
            return GameResult::Continue;
        }

        if pressed(frame, Key::Escape, GamepadButton::Start) {
            self.stop_boulder_roll(frame.audio);
            return GameResult::Exit;
        }

        let mut events = Vec::new();
        if let Some(target) = touch_rock_target(frame, &self.world, self.camera) {
            self.start_rock_flight(target);
        } else if pressed(frame, Key::F, GamepadButton::RightShoulder) {
            self.start_targeting();
        } else if self.rock_flight.is_none()
            && let Some(action) = requested_action(frame)
        {
            events.extend(self.capture_player_action(action));
        }

        events.extend(self.update_rock_flight(frame.delta_time));

        self.simulation_accumulator += frame.delta_time;
        while self.simulation_accumulator >= FIXED_STEP && self.world.phase() == Phase::Running {
            self.simulation_accumulator -= FIXED_STEP;
            let previous_enemies = self
                .world
                .enemies()
                .iter()
                .map(|enemy| enemy.cell)
                .collect::<Vec<_>>();
            let previous_boulders = self
                .world
                .boulders()
                .iter()
                .map(|boulder| boulder.cell)
                .collect::<Vec<_>>();
            let report = self.world.update_tick();
            self.capture_enemy_motions(&previous_enemies);
            self.capture_boulder_motions(&previous_boulders);
            events.extend(report.events);
        }

        self.capture_feedback(&events);
        self.update_boulder_roll_audio(frame.audio, &events);
        self.play_sounds(frame.audio, &events);
        if self.world.phase() == Phase::Dead {
            self.stop_boulder_roll(frame.audio);
        }
        self.advance_camera(frame.delta_time);
        self.advance_transients(frame.delta_time);

        GameResult::Continue
    }

    fn update_death_or_game_over(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.stop_boulder_roll(frame.audio);
        if let Some(elapsed) = self.death_elapsed {
            let elapsed = elapsed + frame.delta_time;
            self.death_elapsed = Some(elapsed);
            self.advance_transients(frame.delta_time);
            if elapsed >= DEATH_ANIMATION_DURATION {
                self.death_elapsed = None;
                self.game_over = true;
                self.feedback_text = None;
            }
            return GameResult::Continue;
        }

        if game_over_retry_requested(frame) {
            self.restart_with_audio(frame.audio);
            return GameResult::Continue;
        }
        if game_over_quit_requested(frame) {
            return GameResult::Exit;
        }
        GameResult::Continue
    }

    fn start_targeting(&mut self) {
        self.targeting = Some(TargetingState {
            target: initial_rock_target(&self.world),
        });
        self.feedback_text = Some(("ROCK?", Duration::ZERO));
    }

    fn update_targeting(&mut self, frame: &Frame<'_>) {
        if pressed(frame, Key::Escape, GamepadButton::East) {
            self.targeting = None;
            self.feedback_text = Some(("CANCEL", Duration::ZERO));
            return;
        }

        if let Some(cell) = touch_cell(frame, &self.world, self.camera) {
            if self.world.can_throw_rock_to(cell) {
                self.targeting = None;
                self.start_rock_flight(cell);
                return;
            }
            if let Some(targeting) = &mut self.targeting {
                targeting.target = cell;
            }
        }

        if let Some(direction) = target_direction(frame)
            && let Some(targeting) = &mut self.targeting
        {
            let next = targeting.target.step(direction);
            if world_cell_in_bounds(&self.world, next) {
                targeting.target = next;
            }
        }

        if pressed(frame, Key::Space, GamepadButton::South)
            && let Some(targeting) = self.targeting
        {
            if self.world.can_throw_rock_to(targeting.target) {
                self.targeting = None;
                self.start_rock_flight(targeting.target);
            } else {
                self.feedback_text = Some(("NOPE", Duration::ZERO));
            }
        }
    }

    fn start_rock_flight(&mut self, target: Cell) {
        self.rock_flight = Some(RockFlight {
            from: self.world.hero(),
            to: target,
            elapsed: Duration::ZERO,
        });
        self.feedback_text = Some(("THROW", Duration::ZERO));
    }

    fn update_rock_flight(&mut self, delta: Duration) -> Vec<WorldEvent> {
        let Some(mut flight) = self.rock_flight else {
            return Vec::new();
        };
        flight.elapsed += delta;
        if flight.elapsed < ROCK_FLIGHT_DURATION {
            self.rock_flight = Some(flight);
            return Vec::new();
        }

        self.rock_flight = None;
        let report = self.world.apply(PlayerAction::ThrowRock(flight.to));
        report.events
    }

    fn update_boulder_roll_audio(&mut self, audio: &mut dyn Audio, events: &[WorldEvent]) {
        for event in events {
            match event {
                WorldEvent::BoulderReleased { .. } => self.start_boulder_roll(audio),
                WorldEvent::BoulderStopped { .. } => self.stop_boulder_roll(audio),
                _ => {}
            }
        }
    }

    fn play_sounds(&mut self, audio: &mut dyn Audio, events: &[WorldEvent]) {
        for sound in self.sounds_for_events(events) {
            if let Err(error) = self.sounds.play(audio, sound) {
                eprintln!("Smart Boy Hero ISO audio error: {error}");
            }
        }
    }

    fn sounds_for_events(&mut self, events: &[WorldEvent]) -> Vec<SoundId> {
        events
            .iter()
            .filter_map(|event| {
                let (shout_sound, death_sound) = match event {
                    WorldEvent::Shouted { .. } => (self.next_variant_sound(SHOUT_SOUNDS), None),
                    WorldEvent::HeroDied => {
                        (SHOUT_SOUNDS[0], Some(self.next_variant_sound(DEATH_SOUNDS)))
                    }
                    _ => (SHOUT_SOUNDS[0], None),
                };
                sound_for_event(event, shout_sound, death_sound.unwrap_or(DEATH_SOUNDS[0]))
            })
            .collect()
    }

    fn next_variant_sound(&mut self, sounds: [SoundId; 3]) -> SoundId {
        self.sfx_rng ^= self.sfx_rng << 13;
        self.sfx_rng ^= self.sfx_rng >> 17;
        self.sfx_rng ^= self.sfx_rng << 5;
        sounds[self.sfx_rng as usize % sounds.len()]
    }

    fn start_boulder_roll(&mut self, audio: &mut dyn Audio) {
        if self.boulder_roll_loop.is_some() {
            return;
        }
        match self.sounds.start_loop(audio, BOULDER_ROLL_SOUND) {
            Ok(playback) => self.boulder_roll_loop = Some(playback),
            Err(error) => eprintln!("Smart Boy Hero ISO audio error: {error}"),
        }
    }

    fn stop_boulder_roll(&mut self, audio: &mut dyn Audio) {
        let Some(playback) = self.boulder_roll_loop.take() else {
            return;
        };
        if let Err(error) = self.sounds.stop_loop(audio, playback) {
            eprintln!("Smart Boy Hero ISO audio error: {error}");
        }
    }

    fn capture_player_action(&mut self, action: PlayerAction) -> Vec<WorldEvent> {
        let hero_before = self.world.hero();
        let report = self.world.apply(action);
        if self.world.hero() != hero_before {
            self.hero_motion = Some(Motion {
                from: hero_before,
                to: self.world.hero(),
                elapsed: Duration::ZERO,
            });
        }
        report.events
    }

    fn capture_enemy_motions(&mut self, previous_enemies: &[Cell]) {
        self.enemy_motions = self
            .world
            .enemies()
            .iter()
            .enumerate()
            .map(|(index, enemy)| {
                let from = previous_enemies.get(index).copied().unwrap_or(enemy.cell);
                (from != enemy.cell).then_some(Motion {
                    from,
                    to: enemy.cell,
                    elapsed: Duration::ZERO,
                })
            })
            .collect();
    }

    fn capture_boulder_motions(&mut self, previous_boulders: &[Cell]) {
        self.boulder_motions = self
            .world
            .boulders()
            .iter()
            .enumerate()
            .map(|(index, boulder)| {
                let from = previous_boulders
                    .get(index)
                    .copied()
                    .unwrap_or(boulder.cell);
                (from != boulder.cell).then_some(Motion {
                    from,
                    to: boulder.cell,
                    elapsed: Duration::ZERO,
                })
            })
            .collect();
    }

    fn capture_feedback(&mut self, events: &[WorldEvent]) {
        for event in events {
            match *event {
                WorldEvent::Shouted { cell, heard } => {
                    self.shout_pulse = Some(TimedCell {
                        cell,
                        elapsed: Duration::ZERO,
                    });
                    self.feedback_text =
                        Some((if heard == 0 { "SHOUT" } else { "HEARD!" }, Duration::ZERO));
                }
                WorldEvent::RockImpacted { cell, heard } => {
                    self.rock_impact = Some(TimedCell {
                        cell,
                        elapsed: Duration::ZERO,
                    });
                    self.feedback_text =
                        Some((if heard == 0 { "CLACK" } else { "LURED!" }, Duration::ZERO));
                }
                WorldEvent::EnemyKilled { cell, .. } => self.kill_bursts.push(TimedCell {
                    cell,
                    elapsed: Duration::ZERO,
                }),
                WorldEvent::SmartChain { count } => {
                    self.smart_flash = Some((count, Duration::ZERO));
                    self.feedback_text = Some(("SMART!", Duration::ZERO));
                }
                WorldEvent::BoulderReleased { .. } => {
                    self.feedback_text = Some(("RUN!", Duration::ZERO));
                    self.screen_shake = Duration::from_millis(120);
                }
                WorldEvent::BoulderCrushedEnemy { chain, .. } => {
                    self.feedback_text = Some(("CRUSH!", Duration::ZERO));
                    self.screen_shake = Duration::from_millis(if chain >= 3 { 360 } else { 220 });
                }
                WorldEvent::BoulderStopped { .. } => {
                    self.feedback_text = Some(("THUD!", Duration::ZERO));
                    self.screen_shake = self.screen_shake.max(Duration::from_millis(160));
                }
                WorldEvent::BoulderSmartChain { count } => {
                    self.smart_flash = Some((count, Duration::ZERO));
                    self.feedback_text = Some(if count >= 4 {
                        ("GENIUS!", Duration::ZERO)
                    } else {
                        ("SMART!", Duration::ZERO)
                    });
                }
                WorldEvent::TrapTriggered => self.feedback_text = Some(("SNAP!", Duration::ZERO)),
                WorldEvent::WalkerSpottedHero => {
                    self.feedback_text = Some(("SPOTTED!", Duration::ZERO));
                }
                WorldEvent::CoreKeyDropped { .. } => {
                    self.feedback_text = Some(("KEY DROP", Duration::ZERO));
                }
                WorldEvent::CoreKeyAcquired => {
                    self.feedback_text = Some(("KEY!", Duration::ZERO));
                }
                WorldEvent::LockedGateBlocked => {
                    self.feedback_text = Some(("LOCKED", Duration::ZERO));
                }
                WorldEvent::CoreGateUnlocked => {
                    self.feedback_text = Some(("OPEN!", Duration::ZERO));
                }
                WorldEvent::HeroDied => {
                    self.feedback_text = Some(("OUCH!", Duration::ZERO));
                    self.death_elapsed = Some(Duration::ZERO);
                    self.targeting = None;
                    self.rock_flight = None;
                }
                WorldEvent::Won => self.feedback_text = Some(("CLEAR!", Duration::ZERO)),
                _ => {}
            }
        }
    }

    fn advance_camera(&mut self, delta: Duration) {
        self.camera_target = Camera::target_for_world(&self.world, self.camera_target);
        self.camera = self.camera.move_toward(self.camera_target, delta);
    }

    fn advance_transients(&mut self, delta: Duration) {
        if let Some(motion) = &mut self.hero_motion {
            motion.elapsed += delta;
            if motion.elapsed >= FIXED_STEP {
                self.hero_motion = None;
            }
        }
        for motion_slot in &mut self.enemy_motions {
            if let Some(motion) = motion_slot.as_mut() {
                motion.elapsed += delta;
                if motion.elapsed >= FIXED_STEP {
                    *motion_slot = None;
                }
            }
        }
        for motion_slot in &mut self.boulder_motions {
            if let Some(motion) = motion_slot.as_mut() {
                motion.elapsed += delta;
                if motion.elapsed >= FIXED_STEP {
                    *motion_slot = None;
                }
            }
        }
        if let Some(pulse) = &mut self.shout_pulse {
            pulse.elapsed += delta;
            if pulse.elapsed >= FEEDBACK_DURATION {
                self.shout_pulse = None;
            }
        }
        if let Some(impact) = &mut self.rock_impact {
            impact.elapsed += delta;
            if impact.elapsed >= FEEDBACK_DURATION {
                self.rock_impact = None;
            }
        }
        for burst in &mut self.kill_bursts {
            burst.elapsed += delta;
        }
        self.kill_bursts
            .retain(|burst| burst.elapsed < FEEDBACK_DURATION);
        if let Some((_, elapsed)) = &mut self.smart_flash {
            *elapsed += delta;
            if *elapsed >= FEEDBACK_DURATION {
                self.smart_flash = None;
            }
        }
        if let Some((_, elapsed)) = &mut self.feedback_text {
            *elapsed += delta;
            if *elapsed >= FEEDBACK_DURATION {
                self.feedback_text = None;
            }
        }
        self.key_elapsed += delta;
        self.screen_shake = self.screen_shake.saturating_sub(delta);
    }

    fn draw(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        let camera = self.camera.with_shake(self.shake_offset());
        draw_hud(
            framebuffer,
            &self.world,
            self.feedback_text,
            self.smart_flash,
        );
        draw_room(framebuffer, &self.world, &self.sprites, self, camera);
        if self.game_over {
            draw_game_over(framebuffer);
        }
    }

    fn shake_offset(&self) -> (i32, i32) {
        if self.screen_shake.is_zero() {
            return (0, 0);
        }
        let ticks = (self.screen_shake.as_millis() / 28) as i32;
        let amplitude = if self.screen_shake > Duration::from_millis(220) {
            4
        } else {
            2
        };
        let x = match ticks.rem_euclid(4) {
            0 => amplitude,
            1 => -amplitude,
            2 => amplitude / 2,
            _ => -amplitude / 2,
        };
        let y = if ticks % 2 == 0 {
            amplitude / 2
        } else {
            -amplitude / 2
        };
        (x, y)
    }
}

impl Game for SmartBoyHeroIsoGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.update_running(frame);
        self.draw(frame.framebuffer);
        result
    }
}

impl Default for SmartBoyHeroIsoGame {
    fn default() -> Self {
        Self::new()
    }
}

fn requested_action(frame: &Frame<'_>) -> Option<PlayerAction> {
    if pressed(frame, Key::Up, GamepadButton::DPadUp) || frame.input.key(Key::W).pressed() {
        Some(PlayerAction::Move(Direction::Up))
    } else if pressed(frame, Key::Down, GamepadButton::DPadDown)
        || frame.input.key(Key::S).pressed()
    {
        Some(PlayerAction::Move(Direction::Down))
    } else if pressed(frame, Key::Left, GamepadButton::DPadLeft)
        || frame.input.key(Key::A).pressed()
    {
        Some(PlayerAction::Move(Direction::Left))
    } else if pressed(frame, Key::Right, GamepadButton::DPadRight)
        || frame.input.key(Key::D).pressed()
    {
        Some(PlayerAction::Move(Direction::Right))
    } else if pressed(frame, Key::E, GamepadButton::North) {
        Some(PlayerAction::Shout)
    } else {
        None
    }
}

fn target_direction(frame: &Frame<'_>) -> Option<Direction> {
    if pressed(frame, Key::Up, GamepadButton::DPadUp) || frame.input.key(Key::W).pressed() {
        Some(Direction::Up)
    } else if pressed(frame, Key::Down, GamepadButton::DPadDown)
        || frame.input.key(Key::S).pressed()
    {
        Some(Direction::Down)
    } else if pressed(frame, Key::Left, GamepadButton::DPadLeft)
        || frame.input.key(Key::A).pressed()
    {
        Some(Direction::Left)
    } else if pressed(frame, Key::Right, GamepadButton::DPadRight)
        || frame.input.key(Key::D).pressed()
    {
        Some(Direction::Right)
    } else {
        None
    }
}

fn initial_rock_target(world: &SmartBoyWorld) -> Cell {
    [
        Direction::Right,
        Direction::Up,
        Direction::Down,
        Direction::Left,
    ]
    .into_iter()
    .map(|direction| world.hero().step(direction))
    .find(|&cell| world.can_throw_rock_to(cell))
    .unwrap_or(world.hero())
}

fn touch_rock_target(frame: &Frame<'_>, world: &SmartBoyWorld, camera: Camera) -> Option<Cell> {
    touch_cell(frame, world, camera).filter(|&cell| world.can_throw_rock_to(cell))
}

fn touch_cell(frame: &Frame<'_>, _world: &SmartBoyWorld, camera: Camera) -> Option<Cell> {
    frame
        .input
        .touches()
        .iter()
        .filter(|touch| matches!(touch.phase, TouchPhase::Started))
        .find_map(|touch| {
            touch
                .position
                .and_then(|position| cell_from_screen(position, camera))
        })
}

fn cell_from_screen((x, y): (i32, i32), camera: Camera) -> Option<Cell> {
    let world_x = x as f32 + camera.offset_x;
    let world_y = (y - VIEWPORT_TOP) as f32 + camera.offset_y;
    let iso_x = world_x / (TILE_WIDTH as f32 / 2.0);
    let iso_y = world_y / (TILE_HEIGHT as f32 / 2.0);
    let cell = Cell::new(
        ((iso_x + iso_y) / 2.0).round() as i32,
        ((iso_y - iso_x) / 2.0).round() as i32,
    );
    Some(cell)
}

fn world_cell_in_bounds(world: &SmartBoyWorld, cell: Cell) -> bool {
    cell.x >= 0 && cell.y >= 0 && cell.x < world.grid_width() && cell.y < world.grid_height()
}

fn pressed(frame: &Frame<'_>, key: Key, button: GamepadButton) -> bool {
    frame.input.key(key).pressed() || frame.input.gamepad_button_any(button).pressed()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ButtonRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl ButtonRect {
    fn contains(self, point: (i32, i32)) -> bool {
        point.0 >= self.x
            && point.1 >= self.y
            && point.0 < self.x + self.width
            && point.1 < self.y + self.height
    }
}

const GAME_OVER_RETRY: ButtonRect = ButtonRect {
    x: 178,
    y: 174,
    width: 164,
    height: 32,
};
const GAME_OVER_QUIT: ButtonRect = ButtonRect {
    x: 178,
    y: 214,
    width: 164,
    height: 32,
};

fn game_over_retry_requested(frame: &Frame<'_>) -> bool {
    pressed(frame, Key::R, GamepadButton::West)
        || pressed(frame, Key::Space, GamepadButton::South)
        || touch_started_in(frame, GAME_OVER_RETRY)
}

fn game_over_quit_requested(frame: &Frame<'_>) -> bool {
    pressed(frame, Key::Escape, GamepadButton::Start)
        || frame
            .input
            .gamepad_button_any(GamepadButton::East)
            .pressed()
        || touch_started_in(frame, GAME_OVER_QUIT)
}

fn touch_started_in(frame: &Frame<'_>, rect: ButtonRect) -> bool {
    frame.input.touches().iter().any(|touch| {
        matches!(touch.phase, TouchPhase::Started)
            && touch.position.is_some_and(|pos| rect.contains(pos))
    })
}

fn draw_hud(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    feedback_text: Option<(&'static str, Duration)>,
    smart_flash: Option<(usize, Duration)>,
) {
    framebuffer.fill_rect(0, 0, FRAMEBUFFER_WIDTH, 48, PANEL);
    framebuffer.draw_text_scaled(10, 8, "SMART BOY HERO ISO", 2, GOLD);
    framebuffer.draw_text(
        10,
        30,
        &format!(
            "{}  POWER {}  TICK {}",
            world.level_name(),
            world.hero_power(),
            world.turn_count()
        ),
        TEXT,
    );
    framebuffer.draw_text(
        318,
        8,
        if world.has_core_key() {
            "KEY ACQUIRED"
        } else {
            "OBJ FIND CORE KEY"
        },
        MUTED,
    );
    framebuffer.draw_text(338, 20, "MOVE WASD/ARROWS", MUTED);
    framebuffer.draw_text(338, 32, "SHOUT E  ROCK F  R/ESC", MUTED);

    if let Some((count, _)) = smart_flash {
        framebuffer.draw_text_scaled(192, 52, &format!("SMART x{count}"), 3, SMART);
    } else if let Some((text, _)) = feedback_text {
        framebuffer.draw_text_scaled(216, 52, text, 2, POWER);
    }
}

fn draw_room(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    camera: Camera,
) {
    for y in 0..world.grid_height() {
        for x in 0..world.grid_width() {
            draw_sprite_at_cell(
                framebuffer,
                sprites,
                SpriteFrame::Floor,
                Cell::new(x, y),
                camera,
            );
        }
    }

    draw_low_objects(framebuffer, world, sprites, camera);

    let mut drawables = Vec::new();
    for &wall in world.walls() {
        drawables.push(Drawable::Wall(wall));
    }
    for (index, door) in world.doors().iter().enumerate() {
        drawables.push(Drawable::Door {
            index,
            cell: door.cell,
            open: world.door_open(index),
        });
    }
    for (index, enemy) in world.enemies().iter().enumerate() {
        drawables.push(Drawable::Enemy {
            index,
            cell: enemy.cell,
        });
    }
    for (index, boulder) in world.boulders().iter().enumerate() {
        drawables.push(Drawable::Boulder {
            index,
            cell: boulder.cell,
        });
    }
    drawables.push(Drawable::Hero(world.hero()));
    drawables.sort_by_key(|drawable| drawable.depth_key());

    for drawable in drawables {
        match drawable {
            Drawable::Wall(cell) => {
                draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Wall, cell, camera)
            }
            Drawable::Door { index, cell, open } => {
                draw_sprite_at_cell(
                    framebuffer,
                    sprites,
                    if open {
                        SpriteFrame::DoorOpen
                    } else {
                        SpriteFrame::DoorClosed
                    },
                    cell,
                    camera,
                );
                if world.door_is_core_gate(index) && !open {
                    draw_locked_gate_marker(framebuffer, cell, camera);
                }
            }
            Drawable::Enemy { index, cell } => {
                draw_enemy(framebuffer, world, sprites, game, index, cell, camera)
            }
            Drawable::Boulder { index, cell } => {
                draw_boulder(framebuffer, world, sprites, game, index, cell, camera)
            }
            Drawable::Hero(cell) => draw_hero(framebuffer, sprites, game, cell, camera),
        }
    }

    draw_vfx(framebuffer, sprites, game, camera);
    if let Some(cell) = world.core_key_cell() {
        draw_key_marker(framebuffer, cell, camera, game.key_elapsed);
    }
}

fn draw_low_objects(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    camera: Camera,
) {
    for (index, trap) in world.traps().iter().enumerate() {
        draw_sprite_at_cell(
            framebuffer,
            sprites,
            if world.trap_active(index) {
                SpriteFrame::TrapArmed
            } else {
                SpriteFrame::TrapInactive
            },
            trap.cell,
            camera,
        );
        let point = project_cell(trap.cell, camera);
        if world.trap_active(index) {
            framebuffer.draw_circle(point.x as i32, point.y as i32 - 4, 14, DANGER);
        }
    }
    for (index, lever) in world.levers().iter().enumerate() {
        draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Plate, lever.cell, camera);
        draw_actuator_marker(framebuffer, world.lever_actuator(index), lever.cell, camera);
    }
}

fn draw_enemy(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    index: usize,
    cell: Cell,
    camera: Camera,
) {
    let Some(enemy) = world.enemies().get(index) else {
        return;
    };
    let point = motion_point(
        cell,
        game.enemy_motions.get(index).and_then(|motion| *motion),
        camera,
    );
    let frame = match enemy.intent {
        EnemyIntent::Patrol => SpriteFrame::WalkerPatrol,
        EnemyIntent::Investigate { .. } | EnemyIntent::ChaseHero { .. } => SpriteFrame::WalkerAlert,
    };
    draw_sprite_at_point(framebuffer, sprites, frame, point);
    if world.enemy_is_key_warden(index) {
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 22, 20, LOCKED);
        framebuffer.draw_text(point.x as i32 - 6, point.y as i32 - 54, "KEY", LOCKED);
    }
    draw_centered_text(framebuffer, point, &enemy.power.to_string(), POWER, -34);
}

fn draw_boulder(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    index: usize,
    cell: Cell,
    camera: Camera,
) {
    let Some(boulder) = world.boulders().get(index) else {
        return;
    };
    let point = motion_point(
        cell,
        game.boulder_motions.get(index).and_then(|motion| *motion),
        camera,
    );
    let frame = match boulder.state {
        BoulderState::Ready | BoulderState::Stopped => SpriteFrame::Boulder0,
        BoulderState::Rolling { .. } => match world.turn_count() % 3 {
            0 => SpriteFrame::Boulder0,
            1 => SpriteFrame::Boulder1,
            _ => SpriteFrame::Boulder2,
        },
    };
    draw_sprite_at_point(framebuffer, sprites, frame, point);
    if matches!(boulder.state, BoulderState::Ready) {
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 16, 18, BRONZE);
    }
}

fn draw_hero(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    cell: Cell,
    camera: Camera,
) {
    let point = motion_point(cell, game.hero_motion, camera);
    if game.world.phase() == Phase::Dead {
        draw_dead_hero(framebuffer, point, game.death_elapsed);
        return;
    }

    let frame = if game.shout_pulse.is_some() {
        SpriteFrame::HeroShout
    } else if game.hero_motion.is_some() {
        SpriteFrame::HeroWalk
    } else {
        SpriteFrame::HeroIdle
    };
    draw_sprite_at_point(framebuffer, sprites, frame, point);
    draw_centered_text(
        framebuffer,
        point,
        &game.world.hero_power().to_string(),
        POWER,
        -41,
    );
}

fn draw_actuator_marker(
    framebuffer: &mut Framebuffer,
    actuator: world::ActuatorKind,
    cell: Cell,
    camera: Camera,
) {
    let point = project_cell(cell, camera);
    match actuator {
        world::ActuatorKind::Boulder => {
            framebuffer.draw_circle(point.x as i32, point.y as i32 - 6, 15, BRONZE);
            framebuffer.draw_line(
                point.x as i32 - 8,
                point.y as i32 - 6,
                point.x as i32 + 8,
                point.y as i32 - 6,
                BRONZE,
            );
        }
        world::ActuatorKind::Trap => {
            framebuffer.draw_circle(point.x as i32, point.y as i32 - 6, 14, DANGER);
            framebuffer.draw_line(
                point.x as i32,
                point.y as i32 - 16,
                point.x as i32,
                point.y as i32 + 2,
                DANGER,
            );
        }
        world::ActuatorKind::Door => {
            framebuffer.draw_circle(point.x as i32, point.y as i32 - 6, 13, LOCKED);
        }
    }
}

fn draw_key_marker(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera, elapsed: Duration) {
    let point = project_cell(cell, camera);
    let x = point.x as i32;
    let y = point.y as i32;
    let pulse = (elapsed.as_secs_f32() * 7.0).sin();
    let bob = if pulse > 0.0 { -2 } else { 0 };
    let outer = 18 + if pulse > 0.45 { 2 } else { 0 };
    draw_iso_diamond_at(framebuffer, x, y, outer, outer / 2, BRONZE);
    draw_iso_diamond_at(framebuffer, x, y, 13, 6, LOCKED);
    framebuffer.fill_circle(x, y, 4, LOCKED);

    let key_y = y - 14 + bob;
    framebuffer.fill_circle(x - 7, key_y, 5, LOCKED);
    framebuffer.draw_circle(x - 7, key_y, 8, GOLD);
    framebuffer.draw_line(x - 1, key_y, x + 15, key_y, LOCKED);
    framebuffer.draw_line(x + 9, key_y, x + 9, key_y + 7, LOCKED);
    framebuffer.draw_line(x + 14, key_y, x + 14, key_y + 5, LOCKED);
}

fn draw_locked_gate_marker(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera) {
    let point = project_cell(cell, camera);
    framebuffer.draw_circle(point.x as i32, point.y as i32 - 32, 15, LOCKED);
    framebuffer.fill_rect(point.x as i32 - 8, point.y as i32 - 31, 16, 14, LOCKED);
    framebuffer.draw_text(point.x as i32 - 8, point.y as i32 - 48, "KEY", LOCKED);
}

fn draw_dead_hero(
    framebuffer: &mut Framebuffer,
    point: ScreenPoint,
    death_elapsed: Option<Duration>,
) {
    let t = death_elapsed
        .map(|elapsed| elapsed.as_secs_f32() / DEATH_ANIMATION_DURATION.as_secs_f32())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    let slump = (t * 12.0) as i32;
    framebuffer.fill_circle(point.x as i32, point.y as i32 - 13 + slump, 8, DANGER);
    framebuffer.draw_line(
        point.x as i32 - 12,
        point.y as i32 - 26 + slump,
        point.x as i32 + 12,
        point.y as i32 - 6 + slump,
        TEXT,
    );
    framebuffer.draw_line(
        point.x as i32 + 12,
        point.y as i32 - 26 + slump,
        point.x as i32 - 12,
        point.y as i32 - 6 + slump,
        TEXT,
    );
    framebuffer.draw_line(
        point.x as i32 - 10,
        point.y as i32 + slump,
        point.x as i32 + 10,
        point.y as i32 + slump,
        DANGER,
    );
}

fn draw_vfx(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    camera: Camera,
) {
    if let Some(targeting) = game.targeting {
        draw_targeting_overlay(framebuffer, &game.world, targeting, camera);
    }

    if let Some(pulse) = game.shout_pulse {
        let point = project_cell(pulse.cell, camera);
        let t = pulse.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let radius = (14.0 + t * 46.0) as u32;
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 18, radius, SHOUT_COLOR);
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 18, radius / 2, SHOUT_COLOR);
    }

    if let Some(flight) = game.rock_flight {
        let t = (flight.elapsed.as_secs_f32() / ROCK_FLIGHT_DURATION.as_secs_f32()).clamp(0.0, 1.0);
        let from = project_cell(flight.from, camera);
        let to = project_cell(flight.to, camera);
        let arc = (1.0 - (2.0 * t - 1.0).abs()) * 28.0;
        let x = from.x + (to.x - from.x) * t;
        let y = from.y + (to.y - from.y) * t - arc;
        framebuffer.fill_circle(x as i32, y as i32 - 16, 4, ROCK_COLOR);
        framebuffer.draw_circle(x as i32, y as i32 - 16, 6, ROCK_COLOR);
    }

    if let Some(impact) = game.rock_impact {
        let point = project_cell(impact.cell, camera);
        let t = impact.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let radius = (8.0 + t * 34.0) as u32;
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 12, radius, ROCK_COLOR);
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 12, radius / 2, ROCK_VALID);
    }

    for burst in &game.kill_bursts {
        let point = project_cell(burst.cell, camera);
        let t = burst.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let offset = (t * 18.0) as i32;
        draw_sprite_at_cell(
            framebuffer,
            sprites,
            SpriteFrame::TrapTriggered,
            burst.cell,
            camera,
        );
        draw_sprite_at_point(
            framebuffer,
            sprites,
            SpriteFrame::WalkerDeath,
            ScreenPoint {
                x: point.x,
                y: point.y - offset as f32,
            },
        );
        framebuffer.draw_circle(
            point.x as i32,
            point.y as i32 - 18,
            18 + offset as u32,
            DANGER,
        );
    }
}

fn draw_game_over(framebuffer: &mut Framebuffer) {
    framebuffer.fill_rect(
        0,
        0,
        FRAMEBUFFER_WIDTH,
        FRAMEBUFFER_HEIGHT,
        Pixel::rgba(0, 0, 0, 180),
    );
    framebuffer.fill_rect(148, 98, 224, 162, PANEL);
    framebuffer.draw_rect(148, 98, 224, 162, DANGER);
    framebuffer.draw_text_scaled(184, 116, "GAME OVER", 3, DANGER);
    draw_button(framebuffer, GAME_OVER_RETRY, "RETRY  R / SPACE", LOCKED);
    draw_button(framebuffer, GAME_OVER_QUIT, "QUIT  ESC", MUTED);
}

fn draw_button(framebuffer: &mut Framebuffer, rect: ButtonRect, label: &str, color: Pixel) {
    framebuffer.fill_rect(
        rect.x,
        rect.y,
        rect.width as u32,
        rect.height as u32,
        Pixel::rgb(28, 30, 36),
    );
    framebuffer.draw_rect(rect.x, rect.y, rect.width as u32, rect.height as u32, color);
    let (text_width, text_height) = Framebuffer::text_size(label, 1);
    framebuffer.draw_text(
        rect.x + (rect.width - text_width as i32) / 2,
        rect.y + (rect.height - text_height as i32) / 2,
        label,
        TEXT,
    );
}

fn draw_targeting_overlay(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    targeting: TargetingState,
    camera: Camera,
) {
    for y in 0..world.grid_height() {
        for x in 0..world.grid_width() {
            let cell = Cell::new(x, y);
            if world.can_throw_rock_to(cell) {
                draw_cell_diamond(framebuffer, cell, camera, MUTED);
            }
        }
    }

    for y in 0..world.grid_height() {
        for x in 0..world.grid_width() {
            let cell = Cell::new(x, y);
            if manhattan(cell, targeting.target) <= ROCK_HEARING_RADIUS {
                draw_cell_diamond(framebuffer, cell, camera, ROCK_COLOR);
            }
        }
    }

    for enemy in world.enemies() {
        if manhattan(enemy.cell, targeting.target) <= ROCK_HEARING_RADIUS {
            let point = project_cell(enemy.cell, camera);
            framebuffer.draw_circle(point.x as i32, point.y as i32 - 22, 15, ROCK_VALID);
        }
    }

    draw_cell_diamond(
        framebuffer,
        targeting.target,
        camera,
        if world.can_throw_rock_to(targeting.target) {
            ROCK_VALID
        } else {
            ROCK_INVALID
        },
    );
}

fn draw_cell_diamond(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera, pixel: Pixel) {
    let point = project_cell(cell, camera);
    draw_iso_diamond_at(
        framebuffer,
        point.x as i32,
        point.y as i32,
        TILE_WIDTH / 2,
        TILE_HEIGHT / 2,
        pixel,
    );
}

fn draw_iso_diamond_at(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    half_width: i32,
    half_height: i32,
    pixel: Pixel,
) {
    framebuffer.draw_line(x, y - half_height, x + half_width, y, pixel);
    framebuffer.draw_line(x + half_width, y, x, y + half_height, pixel);
    framebuffer.draw_line(x, y + half_height, x - half_width, y, pixel);
    framebuffer.draw_line(x - half_width, y, x, y - half_height, pixel);
}

fn manhattan(a: Cell, b: Cell) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn draw_sprite_at_cell(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    cell: Cell,
    camera: Camera,
) {
    draw_sprite_at_point(framebuffer, sprites, frame, project_cell(cell, camera));
}

fn draw_sprite_at_point(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    point: ScreenPoint,
) {
    if point.x < -(SPRITE_SIZE as f32)
        || point.x > FRAMEBUFFER_WIDTH as f32 + SPRITE_SIZE as f32
        || point.y < -(SPRITE_HEIGHT as f32)
        || point.y > FRAMEBUFFER_HEIGHT as f32 + SPRITE_HEIGHT as f32
    {
        return;
    }
    framebuffer.draw_image_region(
        point.x as i32 - SPRITE_SIZE as i32 / 2,
        point.y as i32 - 28,
        sprites,
        ImageRegion::new(frame as u32 * SPRITE_SIZE, 0, SPRITE_SIZE, SPRITE_HEIGHT),
    );
}

fn draw_centered_text(
    framebuffer: &mut Framebuffer,
    point: ScreenPoint,
    text: &str,
    pixel: Pixel,
    y_offset: i32,
) {
    let (width, _) = Framebuffer::text_size(text, 1);
    framebuffer.draw_text(
        point.x as i32 - width as i32 / 2,
        point.y as i32 + y_offset,
        text,
        pixel,
    );
}

fn motion_point(cell: Cell, motion: Option<Motion>, camera: Camera) -> ScreenPoint {
    if let Some(motion) = motion {
        let t = (motion.elapsed.as_secs_f32() / FIXED_STEP.as_secs_f32()).clamp(0.0, 1.0);
        let from = project_cell(motion.from, camera);
        let to = project_cell(motion.to, camera);
        ScreenPoint {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
        }
    } else {
        project_cell(cell, camera)
    }
}

fn project_cell(cell: Cell, camera: Camera) -> ScreenPoint {
    let raw = raw_project_cell(cell);
    ScreenPoint {
        x: raw.x - camera.offset_x,
        y: VIEWPORT_TOP as f32 + raw.y - camera.offset_y,
    }
}

fn raw_project_cell(cell: Cell) -> ScreenPoint {
    ScreenPoint {
        x: ((cell.x - cell.y) * TILE_WIDTH / 2) as f32,
        y: ((cell.x + cell.y) * TILE_HEIGHT / 2) as f32,
    }
}

fn raw_map_bounds(world: &SmartBoyWorld) -> (f32, f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for y in 0..world.grid_height() {
        for x in 0..world.grid_width() {
            let point = raw_project_cell(Cell::new(x, y));
            min_x = min_x.min(point.x - TILE_WIDTH as f32);
            max_x = max_x.max(point.x + TILE_WIDTH as f32);
            min_y = min_y.min(point.y - TILE_HEIGHT as f32);
            max_y = max_y.max(point.y + SPRITE_HEIGHT as f32);
        }
    }

    (min_x, max_x, min_y, max_y)
}

fn clamp_camera_axis(target: f32, min: f32, max: f32, viewport: f32) -> f32 {
    let span = max - min;
    if span <= viewport {
        return min - (viewport - span) / 2.0;
    }
    target.clamp(min, max - viewport)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drawable {
    Wall(Cell),
    Door {
        index: usize,
        cell: Cell,
        open: bool,
    },
    Enemy {
        index: usize,
        cell: Cell,
    },
    Boulder {
        index: usize,
        cell: Cell,
    },
    Hero(Cell),
}

impl Drawable {
    fn depth_key(self) -> (i32, i32) {
        let cell = match self {
            Self::Wall(cell)
            | Self::Door { cell, .. }
            | Self::Enemy { cell, .. }
            | Self::Boulder { cell, .. }
            | Self::Hero(cell) => cell,
        };
        (cell.x + cell.y, cell.y)
    }
}

#[cfg(test)]
fn sounds_for_events(events: &[WorldEvent]) -> Vec<SoundId> {
    events
        .iter()
        .filter_map(|event| sound_for_event(event, SHOUT_SOUNDS[0], DEATH_SOUNDS[0]))
        .collect()
}

fn sound_for_event(
    event: &WorldEvent,
    shout_sound: SoundId,
    death_sound: SoundId,
) -> Option<SoundId> {
    match *event {
        WorldEvent::CombatWon { .. } | WorldEvent::WalkerDestroyed { .. } => Some(COMBAT_SOUND),
        WorldEvent::PressurePlateOn => Some(PRESSURE_PLATE_ON_SOUND),
        WorldEvent::PressurePlateOff => Some(PRESSURE_PLATE_OFF_SOUND),
        WorldEvent::DoorOpened => Some(DOOR_OPEN_SOUND),
        WorldEvent::DoorClosed => Some(DOOR_CLOSE_SOUND),
        WorldEvent::TrapArmed => Some(TRAP_ARM_SOUND),
        WorldEvent::TrapDisarmed => Some(TRAP_DISARM_SOUND),
        WorldEvent::TrapTriggered => Some(TRAP_TRIGGER_SOUND),
        WorldEvent::BoulderReleased { .. } => Some(BOULDER_RELEASE_SOUND),
        WorldEvent::BoulderCrushedEnemy { .. } => Some(BOULDER_CRUSH_SOUND),
        WorldEvent::BoulderStopped { .. } => Some(BOULDER_STOP_SOUND),
        WorldEvent::Shouted { .. } => Some(shout_sound),
        WorldEvent::RockImpacted { .. } => Some(ROCK_IMPACT_SOUND),
        WorldEvent::EnemyKilled { .. } => Some(ENEMY_KILL_SOUND),
        WorldEvent::CoreKeyAcquired => Some(KEY_PICKUP_SOUND),
        WorldEvent::CoreGateUnlocked => Some(KEY_UNLOCK_SOUND),
        WorldEvent::WalkerSpottedHero => Some(ENEMY_ALERT_SOUND),
        WorldEvent::HeroDied => Some(death_sound),
        WorldEvent::Won => Some(VICTORY_SOUND),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SfxBinding {
    key: &'static str,
    sound: SoundId,
}

const REQUIRED_SFX: [SfxBinding; 18] = [
    SfxBinding {
        key: "combat",
        sound: COMBAT_SOUND,
    },
    SfxBinding {
        key: "pressure_plate_on",
        sound: PRESSURE_PLATE_ON_SOUND,
    },
    SfxBinding {
        key: "pressure_plate_off",
        sound: PRESSURE_PLATE_OFF_SOUND,
    },
    SfxBinding {
        key: "door_open",
        sound: DOOR_OPEN_SOUND,
    },
    SfxBinding {
        key: "door_close",
        sound: DOOR_CLOSE_SOUND,
    },
    SfxBinding {
        key: "trap_arm",
        sound: TRAP_ARM_SOUND,
    },
    SfxBinding {
        key: "trap_disarm",
        sound: TRAP_DISARM_SOUND,
    },
    SfxBinding {
        key: "trap_trigger",
        sound: TRAP_TRIGGER_SOUND,
    },
    SfxBinding {
        key: "rock_impact",
        sound: ROCK_IMPACT_SOUND,
    },
    SfxBinding {
        key: "enemy_kill",
        sound: ENEMY_KILL_SOUND,
    },
    SfxBinding {
        key: "enemy_alert",
        sound: ENEMY_ALERT_SOUND,
    },
    SfxBinding {
        key: "boulder_release",
        sound: BOULDER_RELEASE_SOUND,
    },
    SfxBinding {
        key: "boulder_roll",
        sound: BOULDER_ROLL_SOUND,
    },
    SfxBinding {
        key: "boulder_crush",
        sound: BOULDER_CRUSH_SOUND,
    },
    SfxBinding {
        key: "boulder_stop",
        sound: BOULDER_STOP_SOUND,
    },
    SfxBinding {
        key: "key_pickup",
        sound: KEY_PICKUP_SOUND,
    },
    SfxBinding {
        key: "key_unlock",
        sound: KEY_UNLOCK_SOUND,
    },
    SfxBinding {
        key: "victory",
        sound: VICTORY_SOUND,
    },
];

fn smart_boy_sound_bank() -> Result<SoundBank, AudioError> {
    let value: serde_json::Value = serde_json::from_str(SFX_CONFIG_JSON)
        .map_err(|err| AudioError::new(format!("invalid SBH SFX JSON: {err}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| AudioError::new("invalid SBH SFX JSON: root must be an object"))?;
    let mut bank = SoundBank::new();

    for binding in REQUIRED_SFX {
        let value = object.get(binding.key).ok_or_else(|| {
            AudioError::new(format!("missing required SBH SFX entry '{}'", binding.key))
        })?;
        let path = parse_sfx_path(binding.key, value)?;
        insert_configured_wav(&mut bank, binding.key, path, binding.sound)?;
    }
    let shout_value = object.get(SHOUT_SFX_KEY).ok_or_else(|| {
        AudioError::new(format!(
            "missing required SBH SFX entry '{}'",
            SHOUT_SFX_KEY
        ))
    })?;
    let shout_paths = parse_sfx_path_list(SHOUT_SFX_KEY, shout_value)?;
    if shout_paths.len() != SHOUT_SOUNDS.len() {
        return Err(AudioError::new(format!(
            "SBH SFX entry '{}' must contain exactly {} file paths",
            SHOUT_SFX_KEY,
            SHOUT_SOUNDS.len()
        )));
    }
    for (path, sound) in shout_paths.into_iter().zip(SHOUT_SOUNDS) {
        insert_configured_wav(&mut bank, SHOUT_SFX_KEY, path, sound)?;
    }
    let death_value = object.get(DEATH_SFX_KEY).ok_or_else(|| {
        AudioError::new(format!(
            "missing required SBH SFX entry '{}'",
            DEATH_SFX_KEY
        ))
    })?;
    let death_paths = parse_sfx_path_list(DEATH_SFX_KEY, death_value)?;
    if death_paths.len() != DEATH_SOUNDS.len() {
        return Err(AudioError::new(format!(
            "SBH SFX entry '{}' must contain exactly {} file paths",
            DEATH_SFX_KEY,
            DEATH_SOUNDS.len()
        )));
    }
    for (path, sound) in death_paths.into_iter().zip(DEATH_SOUNDS) {
        insert_configured_wav(&mut bank, DEATH_SFX_KEY, path, sound)?;
    }

    Ok(bank)
}

fn parse_sfx_path<'a>(key: &str, value: &'a serde_json::Value) -> Result<&'a str, AudioError> {
    let path = value.as_str().ok_or_else(|| {
        AudioError::new(format!(
            "SBH SFX entry '{}' must be a file path string",
            key
        ))
    })?;
    validate_sfx_path_format(key, path)?;
    Ok(path)
}

fn parse_sfx_path_list<'a>(
    key: &str,
    value: &'a serde_json::Value,
) -> Result<Vec<&'a str>, AudioError> {
    let values = value.as_array().ok_or_else(|| {
        AudioError::new(format!(
            "SBH SFX entry '{}' must be an array of file path strings",
            key
        ))
    })?;
    values
        .iter()
        .map(|value| parse_sfx_path(key, value))
        .collect()
}

fn validate_sfx_path_format(key: &str, path: &str) -> Result<(), AudioError> {
    if !path.to_ascii_lowercase().ends_with(".wav") {
        return Err(AudioError::new(format!(
            "unsupported audio format for SBH SFX '{}': '{}'",
            key, path
        )));
    }
    Ok(())
}

fn insert_configured_wav(
    bank: &mut SoundBank,
    key: &str,
    path: &str,
    sound: SoundId,
) -> Result<(), AudioError> {
    let asset_path = path.strip_prefix(SFX_ASSET_PREFIX).ok_or_else(|| {
        AudioError::new(format!(
            "SBH SFX path must start with '{}': '{}'",
            SFX_ASSET_PREFIX, path
        ))
    })?;
    let file = SMART_BOY_ASSETS.get_file(asset_path).ok_or_else(|| {
        AudioError::new(format!("SBH SFX file not found for '{}': '{}'", key, path))
    })?;
    bank.insert_wav(sound, file.contents().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotoo_pixel_engine::{Input, NoopAudio, NoopStorage, Size, Viewport};

    fn update_with_default_frame(game: &mut SmartBoyHeroIsoGame, delta: Duration) -> GameResult {
        let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);
        let input = Input::default();
        let mut storage = NoopStorage;
        let mut audio = NoopAudio::default();
        let surface_size = Size {
            width: FRAMEBUFFER_WIDTH,
            height: FRAMEBUFFER_HEIGHT,
        };
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time: delta,
            storage: &mut storage,
            audio: &mut audio,
            surface_size,
            viewport: Viewport::new(surface_size, surface_size),
        };
        game.update_running(&mut frame)
    }

    fn kill_hero_on_trap(game: &mut SmartBoyHeroIsoGame) {
        game.world = SmartBoyWorld::for_level(13, INITIAL_SEED);
        game.camera = Camera::centered_for_world(&game.world);
        game.camera_target = game.camera;

        let first = game.capture_player_action(PlayerAction::Move(Direction::Right));
        game.capture_feedback(&first);
        let second = game.capture_player_action(PlayerAction::Move(Direction::Right));
        game.capture_feedback(&second);
    }

    fn walk_to_iso_boulder_plate(world: &mut SmartBoyWorld) {
        world.apply(PlayerAction::Move(Direction::Up));
        for _ in 0..12 {
            world.apply(PlayerAction::Move(Direction::Right));
        }
        for action in [
            PlayerAction::Move(Direction::Down),
            PlayerAction::Move(Direction::Down),
            PlayerAction::Move(Direction::Down),
            PlayerAction::Move(Direction::Down),
            PlayerAction::Move(Direction::Right),
        ] {
            world.apply(action);
        }
    }

    fn drop_iso_core_key_with_boulder(game: &mut SmartBoyHeroIsoGame) -> Cell {
        walk_to_iso_boulder_plate(&mut game.world);
        let report = game.world.update_tick();
        game.capture_feedback(&report.events);
        let key_cell = game
            .world
            .core_key_cell()
            .expect("Clockwork Keep Warden should drop a key");
        game.camera = Camera::target_for_world(&game.world, game.camera);
        game.camera_target = game.camera;
        key_cell
    }

    fn framebuffer_has_pixel_near(
        framebuffer: &Framebuffer,
        point: ScreenPoint,
        radius: i32,
        pixel: Pixel,
    ) -> bool {
        let x = point.x.round() as i32;
        let y = point.y.round() as i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius
                    && framebuffer.pixel(x + dx, y + dy) == Some(pixel)
                {
                    return true;
                }
            }
        }
        false
    }

    #[test]
    fn iso_projection_maps_grid_axes_to_diagonals() {
        let origin = raw_project_cell(Cell::new(0, 0));
        let right = raw_project_cell(Cell::new(1, 0));
        let down = raw_project_cell(Cell::new(0, 1));

        assert!(right.x > origin.x);
        assert!(right.y > origin.y);
        assert!(down.x < origin.x);
        assert!(down.y > origin.y);
    }

    #[test]
    fn depth_key_sorts_lower_iso_cells_later() {
        let mut drawables = [
            Drawable::Hero(Cell::new(2, 2)),
            Drawable::Wall(Cell::new(4, 4)),
            Drawable::Enemy {
                index: 0,
                cell: Cell::new(1, 1),
            },
            Drawable::Boulder {
                index: 0,
                cell: Cell::new(3, 3),
            },
        ];

        drawables.sort_by_key(|drawable| drawable.depth_key());

        assert_eq!(
            drawables[0],
            Drawable::Enemy {
                index: 0,
                cell: Cell::new(1, 1)
            }
        );
        assert_eq!(drawables[3], Drawable::Wall(Cell::new(4, 4)));
    }

    #[test]
    fn camera_dead_zone_keeps_camera_immobile_for_small_hero_moves() {
        let mut world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let camera = Camera::centered_for_world(&world);

        world.apply(PlayerAction::Move(Direction::Right));

        assert_eq!(Camera::target_for_world(&world, camera), camera);
    }

    #[test]
    fn camera_crossing_right_threshold_moves_target() {
        let mut world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let camera = Camera::centered_for_world(&world);

        for action in [
            PlayerAction::Move(Direction::Up),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
        ] {
            world.apply(action);
        }

        let target = Camera::target_for_world(&world, camera);
        assert!(target.offset_x > camera.offset_x);
    }

    #[test]
    fn camera_does_not_recentre_when_hero_returns_inside_dead_zone() {
        let mut world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let mut camera = Camera::centered_for_world(&world);

        for _ in 0..12 {
            world.apply(PlayerAction::Move(Direction::Right));
        }
        camera = Camera::target_for_world(&world, camera);
        for _ in 0..4 {
            world.apply(PlayerAction::Move(Direction::Left));
        }

        assert_eq!(Camera::target_for_world(&world, camera), camera);
    }

    #[test]
    fn camera_target_clamps_to_map_bounds() {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let camera = Camera::target_for_world(
            &world,
            Camera {
                offset_x: 99_999.0,
                offset_y: 99_999.0,
            },
        );
        let (min_x, max_x, min_y, max_y) = raw_map_bounds(&world);

        assert!(camera.offset_x >= min_x);
        assert!(camera.offset_x <= max_x - VIEWPORT_WIDTH as f32);
        assert!(camera.offset_y >= min_y);
        assert!(camera.offset_y <= max_y - VIEWPORT_HEIGHT as f32);
    }

    #[test]
    fn checked_in_sprite_sheet_decodes() {
        let image = Image::decode_png(include_bytes!(
            "../../assets/smart_boy_hero/iso/sprites.png"
        ))
        .expect("sprite sheet should decode");

        assert_eq!(image.width(), SPRITE_SIZE * 17);
        assert_eq!(image.height(), SPRITE_HEIGHT);
    }

    #[test]
    fn iso_slice_contains_clockwork_keep_props() {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);

        assert_eq!(world.level_name(), "THE CLOCKWORK KEEP");
        assert_eq!(world.grid_width(), 26);
        assert_eq!(world.grid_height(), 18);
        assert_eq!(world.enemies().len(), 14);
        assert_eq!(world.doors().len(), 2);
        assert_eq!(world.levers().len(), 3);
        assert_eq!(world.traps().len(), 4);
        assert_eq!(world.boulders().len(), 1);
        assert!(
            world
                .enemies()
                .iter()
                .any(|enemy| matches!(enemy.role, world::EnemyRole::KeyWarden))
        );
        assert!(world.semi_continuous());
    }

    #[test]
    fn iso_actuator_presentation_distinguishes_trap_and_boulder_triggers() {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);

        assert_eq!(world.lever_actuator(1), world::ActuatorKind::Trap);
        assert_eq!(world.lever_actuator(2), world::ActuatorKind::Boulder);
    }

    #[test]
    fn camera_projection_round_trips_world_cell_after_scroll() {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let camera = Camera {
            offset_x: 180.0,
            offset_y: 64.0,
        };
        let cell = Cell::new(18, 12);
        let screen = project_cell(cell, camera);

        assert_eq!(
            cell_from_screen((screen.x.round() as i32, screen.y.round() as i32), camera),
            Some(cell)
        );
        assert!(world.can_throw_rock_to(Cell::new(5, 9)));
    }

    #[test]
    fn core_key_marker_is_centered_on_logical_key_cell_despite_death_burst() {
        let mut game = SmartBoyHeroIsoGame::new();
        let key_cell = drop_iso_core_key_with_boulder(&mut game);
        let camera = game.camera.with_shake(game.shake_offset());
        let point = project_cell(key_cell, camera);
        let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);

        assert!(
            !game
                .world
                .enemies()
                .iter()
                .any(|enemy| matches!(enemy.role, world::EnemyRole::KeyWarden))
        );
        assert_eq!(game.world.hero(), Cell::new(15, 12));
        assert_eq!(key_cell, Cell::new(18, 8));
        assert!(
            game.kill_bursts
                .iter()
                .any(|burst| burst.cell == Cell::new(17, 8))
        );

        game.draw(&mut framebuffer);

        assert_eq!(game.world.core_key_cell(), Some(key_cell));
        assert!(
            framebuffer_has_pixel_near(&framebuffer, point, 4, LOCKED),
            "key center should be drawn at projected core_key_cell {key_cell:?}"
        );
        assert!(
            framebuffer_has_pixel_near(&framebuffer, point, 20, BRONZE),
            "key tile highlight should surround projected core_key_cell {key_cell:?}"
        );
    }

    #[test]
    fn camera_smoothing_moves_toward_target_without_snapping() {
        let camera = Camera {
            offset_x: 10.0,
            offset_y: 20.0,
        };
        let target = Camera {
            offset_x: 190.0,
            offset_y: 20.0,
        };

        let smoothed = camera.move_toward(target, Duration::from_millis(90));

        assert!(smoothed.offset_x > camera.offset_x);
        assert!(smoothed.offset_x < target.offset_x);
        assert_eq!(smoothed.offset_y, target.offset_y);
    }

    #[test]
    fn camera_tracks_hero_without_changing_world_logic() {
        let mut world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        let target = Cell::new(5, 9);
        let initial_camera = Camera::centered_for_world(&world);
        let accepted_before = world.can_throw_rock_to(target);

        for action in [
            PlayerAction::Move(Direction::Up),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
            PlayerAction::Move(Direction::Right),
        ] {
            world.apply(action);
        }

        let moved_camera = Camera::target_for_world(&world, initial_camera);
        assert!(moved_camera.offset_x > initial_camera.offset_x);
        assert_eq!(
            accepted_before,
            SmartBoyWorld::iso_slice(INITIAL_SEED).can_throw_rock_to(target)
        );
    }

    #[test]
    fn iso_slice_draws_nonblank_scene() {
        let game = SmartBoyHeroIsoGame::new();
        let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);

        game.draw(&mut framebuffer);

        assert!(
            framebuffer
                .as_rgba8()
                .chunks_exact(4)
                .any(|pixel| pixel != BG.to_rgba8())
        );
    }

    #[test]
    fn iso_world_events_map_kill_and_alert_sfx_once() {
        let sounds = sounds_for_events(&[
            WorldEvent::EnemyKilled {
                cell: Cell::new(4, 4),
                power: 9,
            },
            WorldEvent::WalkerSpottedHero,
            WorldEvent::RockImpacted {
                cell: Cell::new(6, 4),
                heard: 3,
            },
            WorldEvent::Shouted {
                cell: Cell::new(6, 4),
                heard: 2,
            },
            WorldEvent::BoulderReleased {
                cell: Cell::new(2, 4),
                direction: Direction::Right,
            },
            WorldEvent::BoulderCrushedEnemy {
                cell: Cell::new(4, 4),
                power: 9,
                chain: 2,
            },
            WorldEvent::BoulderStopped {
                cell: Cell::new(9, 4),
            },
            WorldEvent::CoreKeyAcquired,
            WorldEvent::CoreGateUnlocked,
            WorldEvent::HeroDied,
        ]);

        assert_eq!(
            sounds,
            vec![
                ENEMY_KILL_SOUND,
                ENEMY_ALERT_SOUND,
                ROCK_IMPACT_SOUND,
                SHOUT_SOUNDS[0],
                BOULDER_RELEASE_SOUND,
                BOULDER_CRUSH_SOUND,
                BOULDER_STOP_SOUND,
                KEY_PICKUP_SOUND,
                KEY_UNLOCK_SOUND,
                DEATH_SOUNDS[0],
            ]
        );
        assert!(!sounds.contains(&BOULDER_ROLL_SOUND));
    }

    #[test]
    fn runtime_sfx_variants_do_not_repeat_one_fixed_sound() {
        let mut game = SmartBoyHeroIsoGame::new();
        let mut death_sounds = Vec::new();
        let mut shout_sounds = Vec::new();

        for _ in 0..8 {
            let death = game.sounds_for_events(&[WorldEvent::HeroDied]);
            if !death_sounds.contains(&death[0]) {
                death_sounds.push(death[0]);
            }
            let shout = game.sounds_for_events(&[WorldEvent::Shouted {
                cell: Cell::new(1, 1),
                heard: 1,
            }]);
            if !shout_sounds.contains(&shout[0]) {
                shout_sounds.push(shout[0]);
            }
        }

        assert!(death_sounds.len() > 1);
        assert!(shout_sounds.len() > 1);
    }

    #[test]
    fn rock_flight_emits_noise_only_after_landing() {
        let mut game = SmartBoyHeroIsoGame::new();
        let target = Cell::new(5, 9);

        game.start_rock_flight(target);
        let early = game.update_rock_flight(ROCK_FLIGHT_DURATION / 2);
        assert!(early.is_empty());
        assert!(game.rock_flight.is_some());

        let landed = game.update_rock_flight(ROCK_FLIGHT_DURATION / 2);
        assert_eq!(
            landed
                .iter()
                .find(|event| matches!(event, WorldEvent::RockImpacted { .. })),
            Some(&WorldEvent::RockImpacted {
                cell: target,
                heard: 2,
            })
        );
        assert!(game.rock_flight.is_none());
    }

    #[test]
    fn boulder_roll_loop_starts_once_and_stops() {
        let mut game = SmartBoyHeroIsoGame::new();
        let mut audio = NoopAudio::default();

        game.start_boulder_roll(&mut audio);
        let first = game.boulder_roll_loop;
        assert!(first.is_some());

        game.start_boulder_roll(&mut audio);
        assert_eq!(game.boulder_roll_loop, first);

        game.stop_boulder_roll(&mut audio);
        assert!(game.boulder_roll_loop.is_none());
    }

    #[test]
    fn hero_death_enters_death_animation_state() {
        let mut game = SmartBoyHeroIsoGame::new();

        kill_hero_on_trap(&mut game);

        assert_eq!(game.world.phase(), Phase::Dead);
        assert_eq!(game.death_elapsed, Some(Duration::ZERO));
        assert!(!game.game_over);
    }

    #[test]
    fn death_animation_reaches_game_over() {
        let mut game = SmartBoyHeroIsoGame::new();
        kill_hero_on_trap(&mut game);

        let result = update_with_default_frame(&mut game, DEATH_ANIMATION_DURATION);

        assert_eq!(result, GameResult::Continue);
        assert_eq!(game.death_elapsed, None);
        assert!(game.game_over);
    }

    #[test]
    fn game_over_draws_panel_after_death() {
        let mut game = SmartBoyHeroIsoGame::new();
        game.game_over = true;
        let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);

        game.draw(&mut framebuffer);

        assert!(
            framebuffer
                .as_rgba8()
                .chunks_exact(4)
                .any(|pixel| pixel == DANGER.to_rgba8())
        );
    }

    #[test]
    fn game_over_frame_does_not_advance_gameplay_without_menu_action() {
        let mut game = SmartBoyHeroIsoGame::new();
        game.game_over = true;
        let initial = game.world.clone();

        update_with_default_frame(&mut game, FIXED_STEP * 3);

        assert_eq!(game.world, initial);
        assert!(game.game_over);
    }

    #[test]
    fn retry_restores_warden_key_and_gate_state() {
        let mut game = SmartBoyHeroIsoGame::new();
        let mut audio = NoopAudio::default();

        game.world.apply(PlayerAction::Move(Direction::Right));
        game.restart_with_audio(&mut audio);

        assert_eq!(game.world.core_key_cell(), None);
        assert!(!game.world.has_core_key());
        assert!(!game.world.door_open(1));
        assert!(
            game.world
                .enemies()
                .iter()
                .any(|enemy| matches!(enemy.role, world::EnemyRole::KeyWarden))
        );
        assert!(!game.game_over);
        assert_eq!(game.death_elapsed, None);
    }

    #[test]
    fn restart_stops_active_boulder_roll_loop() {
        let mut game = SmartBoyHeroIsoGame::new();
        let mut audio = NoopAudio::default();

        game.start_boulder_roll(&mut audio);
        assert!(game.boulder_roll_loop.is_some());

        game.restart_with_audio(&mut audio);

        assert!(game.boulder_roll_loop.is_none());
        assert!(game.targeting.is_none());
        assert!(game.rock_flight.is_none());
        assert!(game.rock_impact.is_none());
    }
}
