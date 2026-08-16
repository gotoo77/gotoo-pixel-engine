use std::time::Duration;

use gotoo_pixel_engine::{
    Audio, AudioError, Frame, Framebuffer, Game, GameResult, GamepadButton, Image, ImageRegion,
    Key, Pixel, PlaybackId, SoundBank, SoundId,
};
use include_dir::{Dir, include_dir};

#[allow(dead_code)]
#[path = "../smart_boy_hero/world.rs"]
mod world;
use world::{
    BoulderState, Cell, Direction, EnemyIntent, GRID_HEIGHT, GRID_WIDTH, Phase, PlayerAction,
    ROCK_HEARING_RADIUS, SmartBoyWorld, WorldEvent,
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
const ORIGIN_X: i32 = 260;
const ORIGIN_Y: i32 = 66;
const INITIAL_SEED: u32 = 0x150_512CE;
const SFX_ASSET_PREFIX: &str = "assets/smart_boy_hero/";
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
const SHOUT_SOUND: SoundId = SoundId::new("smart_boy_hero.shout");
const ROCK_IMPACT_SOUND: SoundId = SoundId::new("smart_boy_hero.rock_impact");
const ENEMY_KILL_SOUND: SoundId = SoundId::new("smart_boy_hero.enemy_kill");
const ENEMY_ALERT_SOUND: SoundId = SoundId::new("smart_boy_hero.enemy_alert");
const BOULDER_RELEASE_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_release");
const BOULDER_ROLL_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_roll");
const BOULDER_CRUSH_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_crush");
const BOULDER_STOP_SOUND: SoundId = SoundId::new("smart_boy_hero.boulder_stop");
const DEATH_SOUND: SoundId = SoundId::new("smart_boy_hero.death");
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

pub struct SmartBoyHeroIsoGame {
    world: SmartBoyWorld,
    sprites: Image,
    sounds: SoundBank,
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
    screen_shake: Duration,
    boulder_roll_loop: Option<PlaybackId>,
    feedback_text: Option<(&'static str, Duration)>,
}

impl SmartBoyHeroIsoGame {
    pub fn new() -> Self {
        Self {
            world: SmartBoyWorld::iso_slice(INITIAL_SEED),
            sprites: Image::decode_png(include_bytes!(
                "../../assets/smart_boy_hero/iso/sprites.png"
            ))
            .expect("checked-in SBH iso sprite sheet should decode"),
            sounds: smart_boy_sound_bank().expect("checked-in SBH SFX config should load"),
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
            screen_shake: Duration::ZERO,
            boulder_roll_loop: None,
            feedback_text: None,
        }
    }

    fn restart(&mut self) {
        self.world = SmartBoyWorld::iso_slice(INITIAL_SEED);
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
        self.screen_shake = Duration::ZERO;
        self.feedback_text = None;
    }

    fn restart_with_audio(&mut self, audio: &mut dyn Audio) {
        self.stop_boulder_roll(audio);
        self.restart();
    }

    fn update_running(&mut self, frame: &mut Frame<'_>) -> GameResult {
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
        if let Some(target) = touch_rock_target(frame, &self.world) {
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
        play_sounds(&mut self.sounds, frame.audio, &events);
        self.advance_transients(frame.delta_time);

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

        if let Some(cell) = touch_cell(frame) {
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
            if next.is_inside() {
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
                WorldEvent::HeroDied => self.feedback_text = Some(("OUCH!", Duration::ZERO)),
                WorldEvent::Won => self.feedback_text = Some(("CLEAR!", Duration::ZERO)),
                _ => {}
            }
        }
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
        self.screen_shake = self.screen_shake.saturating_sub(delta);
    }

    fn draw(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        let shake = self.shake_offset();
        draw_hud(
            framebuffer,
            &self.world,
            self.feedback_text,
            self.smart_flash,
        );
        draw_room(framebuffer, &self.world, &self.sprites, self, shake);
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
    } else if pressed(frame, Key::Space, GamepadButton::South) {
        Some(PlayerAction::Wait)
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

fn touch_rock_target(frame: &Frame<'_>, world: &SmartBoyWorld) -> Option<Cell> {
    touch_cell(frame).filter(|&cell| world.can_throw_rock_to(cell))
}

fn touch_cell(frame: &Frame<'_>) -> Option<Cell> {
    frame
        .input
        .touches()
        .iter()
        .filter(|touch| matches!(touch.phase, gotoo_pixel_engine::TouchPhase::Started))
        .find_map(|touch| touch.position.and_then(cell_from_screen))
}

fn cell_from_screen((x, y): (i32, i32)) -> Option<Cell> {
    let iso_x = (x - ORIGIN_X) as f32 / (TILE_WIDTH as f32 / 2.0);
    let iso_y = (y - ORIGIN_Y) as f32 / (TILE_HEIGHT as f32 / 2.0);
    let cell = Cell::new(
        ((iso_x + iso_y) / 2.0).round() as i32,
        ((iso_y - iso_x) / 2.0).round() as i32,
    );
    cell.is_inside().then_some(cell)
}

fn pressed(frame: &Frame<'_>, key: Key, button: GamepadButton) -> bool {
    frame.input.key(key).pressed() || frame.input.gamepad_button_any(button).pressed()
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
    framebuffer.draw_text(344, 8, "MOVE WASD/ARROWS", MUTED);
    framebuffer.draw_text(344, 20, "SHOUT E  ROCK F", MUTED);
    framebuffer.draw_text(344, 32, "WAIT SPACE  R/ESC", MUTED);

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
    shake: (i32, i32),
) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            draw_sprite_at_cell(
                framebuffer,
                sprites,
                SpriteFrame::Floor,
                Cell::new(x, y),
                shake,
            );
        }
    }

    draw_low_objects(framebuffer, world, sprites, shake);

    let mut drawables = Vec::new();
    for &wall in world.walls() {
        drawables.push(Drawable::Wall(wall));
    }
    for (index, door) in world.doors().iter().enumerate() {
        drawables.push(Drawable::Door {
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
                draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Wall, cell, shake)
            }
            Drawable::Door { cell, open } => draw_sprite_at_cell(
                framebuffer,
                sprites,
                if open {
                    SpriteFrame::DoorOpen
                } else {
                    SpriteFrame::DoorClosed
                },
                cell,
                shake,
            ),
            Drawable::Enemy { index, cell } => {
                draw_enemy(framebuffer, world, sprites, game, index, cell, shake)
            }
            Drawable::Boulder { index, cell } => {
                draw_boulder(framebuffer, world, sprites, game, index, cell, shake)
            }
            Drawable::Hero(cell) => draw_hero(framebuffer, sprites, game, cell, shake),
        }
    }

    draw_vfx(framebuffer, sprites, game, shake);
}

fn draw_low_objects(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    shake: (i32, i32),
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
            shake,
        );
    }
    for lever in world.levers() {
        draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Plate, lever.cell, shake);
    }
}

fn draw_enemy(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    index: usize,
    cell: Cell,
    shake: (i32, i32),
) {
    let Some(enemy) = world.enemies().get(index) else {
        return;
    };
    let point = motion_point(
        cell,
        game.enemy_motions.get(index).and_then(|motion| *motion),
        shake,
    );
    let frame = match enemy.intent {
        EnemyIntent::Patrol => SpriteFrame::WalkerPatrol,
        EnemyIntent::Investigate { .. } | EnemyIntent::ChaseHero { .. } => SpriteFrame::WalkerAlert,
    };
    draw_sprite_at_point(framebuffer, sprites, frame, point);
    draw_centered_text(framebuffer, point, &enemy.power.to_string(), POWER, -34);
}

fn draw_boulder(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    index: usize,
    cell: Cell,
    shake: (i32, i32),
) {
    let Some(boulder) = world.boulders().get(index) else {
        return;
    };
    let point = motion_point(
        cell,
        game.boulder_motions.get(index).and_then(|motion| *motion),
        shake,
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
}

fn draw_hero(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    cell: Cell,
    shake: (i32, i32),
) {
    let point = motion_point(cell, game.hero_motion, shake);
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

fn draw_vfx(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    shake: (i32, i32),
) {
    if let Some(targeting) = game.targeting {
        draw_targeting_overlay(framebuffer, &game.world, targeting, shake);
    }

    if let Some(pulse) = game.shout_pulse {
        let point = project_cell(pulse.cell, shake);
        let t = pulse.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let radius = (14.0 + t * 46.0) as u32;
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 18, radius, SHOUT_COLOR);
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 18, radius / 2, SHOUT_COLOR);
    }

    if let Some(flight) = game.rock_flight {
        let t = (flight.elapsed.as_secs_f32() / ROCK_FLIGHT_DURATION.as_secs_f32()).clamp(0.0, 1.0);
        let from = project_cell(flight.from, shake);
        let to = project_cell(flight.to, shake);
        let arc = (1.0 - (2.0 * t - 1.0).abs()) * 28.0;
        let x = from.x + (to.x - from.x) * t;
        let y = from.y + (to.y - from.y) * t - arc;
        framebuffer.fill_circle(x as i32, y as i32 - 16, 4, ROCK_COLOR);
        framebuffer.draw_circle(x as i32, y as i32 - 16, 6, ROCK_COLOR);
    }

    if let Some(impact) = game.rock_impact {
        let point = project_cell(impact.cell, shake);
        let t = impact.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let radius = (8.0 + t * 34.0) as u32;
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 12, radius, ROCK_COLOR);
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 12, radius / 2, ROCK_VALID);
    }

    for burst in &game.kill_bursts {
        let point = project_cell(burst.cell, shake);
        let t = burst.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let offset = (t * 18.0) as i32;
        draw_sprite_at_cell(
            framebuffer,
            sprites,
            SpriteFrame::TrapTriggered,
            burst.cell,
            shake,
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

fn draw_targeting_overlay(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    targeting: TargetingState,
    shake: (i32, i32),
) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = Cell::new(x, y);
            if world.can_throw_rock_to(cell) {
                draw_cell_diamond(framebuffer, cell, shake, MUTED);
            }
        }
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = Cell::new(x, y);
            if manhattan(cell, targeting.target) <= ROCK_HEARING_RADIUS {
                draw_cell_diamond(framebuffer, cell, shake, ROCK_COLOR);
            }
        }
    }

    for enemy in world.enemies() {
        if manhattan(enemy.cell, targeting.target) <= ROCK_HEARING_RADIUS {
            let point = project_cell(enemy.cell, shake);
            framebuffer.draw_circle(point.x as i32, point.y as i32 - 22, 15, ROCK_VALID);
        }
    }

    draw_cell_diamond(
        framebuffer,
        targeting.target,
        shake,
        if world.can_throw_rock_to(targeting.target) {
            ROCK_VALID
        } else {
            ROCK_INVALID
        },
    );
}

fn draw_cell_diamond(framebuffer: &mut Framebuffer, cell: Cell, shake: (i32, i32), pixel: Pixel) {
    let point = project_cell(cell, shake);
    let x = point.x as i32;
    let y = point.y as i32;
    framebuffer.draw_line(x, y - TILE_HEIGHT / 2, x + TILE_WIDTH / 2, y, pixel);
    framebuffer.draw_line(x + TILE_WIDTH / 2, y, x, y + TILE_HEIGHT / 2, pixel);
    framebuffer.draw_line(x, y + TILE_HEIGHT / 2, x - TILE_WIDTH / 2, y, pixel);
    framebuffer.draw_line(x - TILE_WIDTH / 2, y, x, y - TILE_HEIGHT / 2, pixel);
}

fn manhattan(a: Cell, b: Cell) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn draw_sprite_at_cell(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    cell: Cell,
    shake: (i32, i32),
) {
    draw_sprite_at_point(framebuffer, sprites, frame, project_cell(cell, shake));
}

fn draw_sprite_at_point(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    point: ScreenPoint,
) {
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

fn motion_point(cell: Cell, motion: Option<Motion>, shake: (i32, i32)) -> ScreenPoint {
    if let Some(motion) = motion {
        let t = (motion.elapsed.as_secs_f32() / FIXED_STEP.as_secs_f32()).clamp(0.0, 1.0);
        let from = project_cell(motion.from, shake);
        let to = project_cell(motion.to, shake);
        ScreenPoint {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
        }
    } else {
        project_cell(cell, shake)
    }
}

fn project_cell(cell: Cell, shake: (i32, i32)) -> ScreenPoint {
    ScreenPoint {
        x: (ORIGIN_X + shake.0 + (cell.x - cell.y) * TILE_WIDTH / 2) as f32,
        y: (ORIGIN_Y + shake.1 + (cell.x + cell.y) * TILE_HEIGHT / 2) as f32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drawable {
    Wall(Cell),
    Door { cell: Cell, open: bool },
    Enemy { index: usize, cell: Cell },
    Boulder { index: usize, cell: Cell },
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

fn play_sounds(sounds: &mut SoundBank, audio: &mut dyn Audio, events: &[WorldEvent]) {
    for sound in sounds_for_events(events) {
        if let Err(error) = sounds.play(audio, sound) {
            eprintln!("Smart Boy Hero ISO audio error: {error}");
        }
    }
}

fn sounds_for_events(events: &[WorldEvent]) -> Vec<SoundId> {
    events
        .iter()
        .filter_map(|event| match *event {
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
            WorldEvent::Shouted { .. } => Some(SHOUT_SOUND),
            WorldEvent::RockImpacted { .. } => Some(ROCK_IMPACT_SOUND),
            WorldEvent::EnemyKilled { .. } => Some(ENEMY_KILL_SOUND),
            WorldEvent::WalkerSpottedHero => Some(ENEMY_ALERT_SOUND),
            WorldEvent::HeroDied => Some(DEATH_SOUND),
            WorldEvent::Won => Some(VICTORY_SOUND),
            _ => None,
        })
        .collect()
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
        key: "shout",
        sound: SHOUT_SOUND,
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
        key: "death",
        sound: DEATH_SOUND,
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
        let path = value.as_str().ok_or_else(|| {
            AudioError::new(format!(
                "SBH SFX entry '{}' must be a file path string",
                binding.key
            ))
        })?;
        if !path.to_ascii_lowercase().ends_with(".wav") {
            return Err(AudioError::new(format!(
                "unsupported audio format for SBH SFX '{}': '{}'",
                binding.key, path
            )));
        }
        let asset_path = path.strip_prefix(SFX_ASSET_PREFIX).ok_or_else(|| {
            AudioError::new(format!(
                "SBH SFX path must start with '{}': '{}'",
                SFX_ASSET_PREFIX, path
            ))
        })?;
        let file = SMART_BOY_ASSETS.get_file(asset_path).ok_or_else(|| {
            AudioError::new(format!(
                "SBH SFX file not found for '{}': '{}'",
                binding.key, path
            ))
        })?;
        bank.insert_wav(binding.sound, file.contents().to_vec())?;
    }

    Ok(bank)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotoo_pixel_engine::NoopAudio;

    #[test]
    fn iso_projection_maps_grid_axes_to_diagonals() {
        let origin = project_cell(Cell::new(0, 0), (0, 0));
        let right = project_cell(Cell::new(1, 0), (0, 0));
        let down = project_cell(Cell::new(0, 1), (0, 0));

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
    fn checked_in_sprite_sheet_decodes() {
        let image = Image::decode_png(include_bytes!(
            "../../assets/smart_boy_hero/iso/sprites.png"
        ))
        .expect("sprite sheet should decode");

        assert_eq!(image.width(), SPRITE_SIZE * 17);
        assert_eq!(image.height(), SPRITE_HEIGHT);
    }

    #[test]
    fn iso_slice_contains_sandbox_walkers_boulder_and_core_props() {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);

        assert_eq!(world.enemies().len(), 6);
        assert_eq!(world.doors().len(), 0);
        assert_eq!(world.levers().len(), 1);
        assert_eq!(world.traps().len(), 3);
        assert_eq!(world.boulders().len(), 1);
        assert!(world.semi_continuous());
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
        ]);

        assert_eq!(
            sounds,
            vec![
                ENEMY_KILL_SOUND,
                ENEMY_ALERT_SOUND,
                ROCK_IMPACT_SOUND,
                BOULDER_RELEASE_SOUND,
                BOULDER_CRUSH_SOUND,
                BOULDER_STOP_SOUND
            ]
        );
        assert!(!sounds.contains(&BOULDER_ROLL_SOUND));
    }

    #[test]
    fn rock_flight_emits_noise_only_after_landing() {
        let mut game = SmartBoyHeroIsoGame::new();
        let target = Cell::new(6, 4);

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
                heard: 3,
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
