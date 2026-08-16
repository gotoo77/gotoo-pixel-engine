use std::time::Duration;

use gotoo_pixel_engine::{
    Audio, AudioError, Frame, Framebuffer, Game, GameResult, GamepadButton, Image, ImageRegion,
    Key, Pixel, SoundBank, SoundId,
};
use include_dir::{Dir, include_dir};

#[allow(dead_code)]
#[path = "../smart_boy_hero/world.rs"]
mod world;
use world::{
    Cell, Direction, EnemyIntent, GRID_HEIGHT, GRID_WIDTH, Phase, PlayerAction, SmartBoyWorld,
    WorldEvent,
};

pub const FRAMEBUFFER_WIDTH: u32 = 520;
pub const FRAMEBUFFER_HEIGHT: u32 = 320;

const FIXED_STEP: Duration = Duration::from_millis(420);
const FEEDBACK_DURATION: Duration = Duration::from_millis(620);
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
const ENEMY_KILL_SOUND: SoundId = SoundId::new("smart_boy_hero.enemy_kill");
const ENEMY_ALERT_SOUND: SoundId = SoundId::new("smart_boy_hero.enemy_alert");
const DEATH_SOUND: SoundId = SoundId::new("smart_boy_hero.death");
const VICTORY_SOUND: SoundId = SoundId::new("smart_boy_hero.victory");

const BG: Pixel = Pixel::rgb(9, 11, 15);
const PANEL: Pixel = Pixel::rgb(18, 20, 25);
const TEXT: Pixel = Pixel::rgb(244, 239, 217);
const MUTED: Pixel = Pixel::rgb(132, 144, 152);
const GOLD: Pixel = Pixel::rgb(255, 218, 76);
const POWER: Pixel = Pixel::rgb(255, 246, 104);
const SHOUT_COLOR: Pixel = Pixel::rgb(217, 91, 255);
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

pub struct SmartBoyHeroIsoGame {
    world: SmartBoyWorld,
    sprites: Image,
    sounds: SoundBank,
    simulation_accumulator: Duration,
    hero_motion: Option<Motion>,
    enemy_motions: Vec<Option<Motion>>,
    shout_pulse: Option<TimedCell>,
    kill_bursts: Vec<TimedCell>,
    smart_flash: Option<(usize, Duration)>,
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
            shout_pulse: None,
            kill_bursts: Vec::new(),
            smart_flash: None,
            feedback_text: None,
        }
    }

    fn restart(&mut self) {
        self.world = SmartBoyWorld::iso_slice(INITIAL_SEED);
        self.simulation_accumulator = Duration::ZERO;
        self.hero_motion = None;
        self.enemy_motions.clear();
        self.shout_pulse = None;
        self.kill_bursts.clear();
        self.smart_flash = None;
        self.feedback_text = None;
    }

    fn update_running(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if pressed(frame, Key::Escape, GamepadButton::Start) {
            return GameResult::Exit;
        }
        if pressed(frame, Key::R, GamepadButton::West) {
            self.restart();
            return GameResult::Continue;
        }

        let mut events = Vec::new();
        if let Some(action) = requested_action(frame) {
            let hero_before = self.world.hero();
            let report = self.world.apply(action);
            if self.world.hero() != hero_before {
                self.hero_motion = Some(Motion {
                    from: hero_before,
                    to: self.world.hero(),
                    elapsed: Duration::ZERO,
                });
            }
            events.extend(report.events);
        }

        self.simulation_accumulator += frame.delta_time;
        while self.simulation_accumulator >= FIXED_STEP && self.world.phase() == Phase::Running {
            self.simulation_accumulator -= FIXED_STEP;
            let previous_enemies = self
                .world
                .enemies()
                .iter()
                .map(|enemy| enemy.cell)
                .collect::<Vec<_>>();
            let report = self.world.update_tick();
            self.capture_enemy_motions(&previous_enemies);
            events.extend(report.events);
        }

        self.capture_feedback(&events);
        play_sounds(&mut self.sounds, frame.audio, &events);
        self.advance_transients(frame.delta_time);

        GameResult::Continue
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
                WorldEvent::EnemyKilled { cell, .. } => self.kill_bursts.push(TimedCell {
                    cell,
                    elapsed: Duration::ZERO,
                }),
                WorldEvent::SmartChain { count } => {
                    self.smart_flash = Some((count, Duration::ZERO));
                    self.feedback_text = Some(("SMART!", Duration::ZERO));
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
        if let Some(pulse) = &mut self.shout_pulse {
            pulse.elapsed += delta;
            if pulse.elapsed >= FEEDBACK_DURATION {
                self.shout_pulse = None;
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
    }

    fn draw(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        draw_hud(
            framebuffer,
            &self.world,
            self.feedback_text,
            self.smart_flash,
        );
        draw_room(framebuffer, &self.world, &self.sprites, self);
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
    framebuffer.draw_text(344, 20, "SHOUT E  WAIT SPACE", MUTED);
    framebuffer.draw_text(344, 32, "RETRY R  EXIT ESC", MUTED);

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
) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Floor, Cell::new(x, y));
        }
    }

    draw_low_objects(framebuffer, world, sprites);

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
    drawables.push(Drawable::Hero(world.hero()));
    drawables.sort_by_key(|drawable| drawable.depth_key());

    for drawable in drawables {
        match drawable {
            Drawable::Wall(cell) => {
                draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Wall, cell)
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
            ),
            Drawable::Enemy { index, cell } => {
                draw_enemy(framebuffer, world, sprites, game, index, cell)
            }
            Drawable::Hero(cell) => draw_hero(framebuffer, sprites, game, cell),
        }
    }

    draw_vfx(framebuffer, sprites, game);
}

fn draw_low_objects(framebuffer: &mut Framebuffer, world: &SmartBoyWorld, sprites: &Image) {
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
        );
    }
    for lever in world.levers() {
        draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Plate, lever.cell);
    }
}

fn draw_enemy(
    framebuffer: &mut Framebuffer,
    world: &SmartBoyWorld,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    index: usize,
    cell: Cell,
) {
    let Some(enemy) = world.enemies().get(index) else {
        return;
    };
    let point = motion_point(
        cell,
        game.enemy_motions.get(index).and_then(|motion| *motion),
    );
    let frame = match enemy.intent {
        EnemyIntent::Patrol => SpriteFrame::WalkerPatrol,
        EnemyIntent::Investigate { .. } | EnemyIntent::ChaseHero { .. } => SpriteFrame::WalkerAlert,
    };
    draw_sprite_at_point(framebuffer, sprites, frame, point);
    draw_centered_text(framebuffer, point, &enemy.power.to_string(), POWER, -34);
}

fn draw_hero(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    game: &SmartBoyHeroIsoGame,
    cell: Cell,
) {
    let point = motion_point(cell, game.hero_motion);
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

fn draw_vfx(framebuffer: &mut Framebuffer, sprites: &Image, game: &SmartBoyHeroIsoGame) {
    if let Some(pulse) = game.shout_pulse {
        let point = project_cell(pulse.cell);
        let t = pulse.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let radius = (14.0 + t * 46.0) as u32;
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 18, radius, SHOUT_COLOR);
        framebuffer.draw_circle(point.x as i32, point.y as i32 - 18, radius / 2, SHOUT_COLOR);
    }

    for burst in &game.kill_bursts {
        let point = project_cell(burst.cell);
        let t = burst.elapsed.as_secs_f32() / FEEDBACK_DURATION.as_secs_f32();
        let offset = (t * 18.0) as i32;
        draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::TrapTriggered, burst.cell);
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

fn draw_sprite_at_cell(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    cell: Cell,
) {
    draw_sprite_at_point(framebuffer, sprites, frame, project_cell(cell));
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

fn motion_point(cell: Cell, motion: Option<Motion>) -> ScreenPoint {
    if let Some(motion) = motion {
        let t = (motion.elapsed.as_secs_f32() / FIXED_STEP.as_secs_f32()).clamp(0.0, 1.0);
        let from = project_cell(motion.from);
        let to = project_cell(motion.to);
        ScreenPoint {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
        }
    } else {
        project_cell(cell)
    }
}

fn project_cell(cell: Cell) -> ScreenPoint {
    ScreenPoint {
        x: (ORIGIN_X + (cell.x - cell.y) * TILE_WIDTH / 2) as f32,
        y: (ORIGIN_Y + (cell.x + cell.y) * TILE_HEIGHT / 2) as f32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drawable {
    Wall(Cell),
    Door { cell: Cell, open: bool },
    Enemy { index: usize, cell: Cell },
    Hero(Cell),
}

impl Drawable {
    fn depth_key(self) -> (i32, i32) {
        let cell = match self {
            Self::Wall(cell)
            | Self::Door { cell, .. }
            | Self::Enemy { cell, .. }
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
            WorldEvent::Shouted { .. } => Some(SHOUT_SOUND),
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

const REQUIRED_SFX: [SfxBinding; 13] = [
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
        key: "enemy_kill",
        sound: ENEMY_KILL_SOUND,
    },
    SfxBinding {
        key: "enemy_alert",
        sound: ENEMY_ALERT_SOUND,
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

    #[test]
    fn iso_projection_maps_grid_axes_to_diagonals() {
        let origin = project_cell(Cell::new(0, 0));
        let right = project_cell(Cell::new(1, 0));
        let down = project_cell(Cell::new(0, 1));

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
        ];

        drawables.sort_by_key(|drawable| drawable.depth_key());

        assert_eq!(
            drawables[0],
            Drawable::Enemy {
                index: 0,
                cell: Cell::new(1, 1)
            }
        );
        assert_eq!(drawables[2], Drawable::Wall(Cell::new(4, 4)));
    }

    #[test]
    fn checked_in_sprite_sheet_decodes() {
        let image = Image::decode_png(include_bytes!(
            "../../assets/smart_boy_hero/iso/sprites.png"
        ))
        .expect("sprite sheet should decode");

        assert_eq!(image.width(), SPRITE_SIZE * 14);
        assert_eq!(image.height(), SPRITE_HEIGHT);
    }

    #[test]
    fn iso_slice_contains_three_walkers_and_core_props() {
        let world = SmartBoyWorld::iso_slice(INITIAL_SEED);

        assert_eq!(world.enemies().len(), 3);
        assert_eq!(world.doors().len(), 1);
        assert_eq!(world.levers().len(), 1);
        assert_eq!(world.traps().len(), 3);
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
        ]);

        assert_eq!(sounds, vec![ENEMY_KILL_SOUND, ENEMY_ALERT_SOUND]);
    }
}
