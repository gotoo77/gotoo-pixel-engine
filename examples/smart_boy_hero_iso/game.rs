use std::time::Duration;

use gotoo_pixel_engine::{
    Audio, AudioError, Font, Frame, Framebuffer, Game, GameResult, GamepadButton, Image,
    ImageRegion, Key, Pixel, PlaybackId, SoundBank, SoundId, TouchPhase,
};
use include_dir::{Dir, include_dir};

#[allow(dead_code)]
#[path = "../smart_boy_hero/world.rs"]
mod world;
use world::{
    BoulderState, Cell, Direction, EnemyIntent, EnemyKind, Phase, PlayerAction,
    ROCK_HEARING_RADIUS, SmartBoyWorld, WorldEvent,
};

pub const FRAMEBUFFER_WIDTH: u32 = 520;
pub const FRAMEBUFFER_HEIGHT: u32 = 320;

const FIXED_STEP: Duration = Duration::from_millis(420);
const FEEDBACK_DURATION: Duration = Duration::from_millis(620);
const ROCK_FLIGHT_DURATION: Duration = Duration::from_millis(280);
const MOVE_REPEAT_DELAY: Duration = Duration::from_millis(180);
const MOVE_REPEAT_PERIOD: Duration = Duration::from_millis(90);
const TILE_WIDTH: i32 = 32;
const TILE_HEIGHT: i32 = 16;
const ACTUATOR_MARKER_HALF_WIDTH: i32 = TILE_WIDTH / 2 - 4;
const ACTUATOR_MARKER_HALF_HEIGHT: i32 = TILE_HEIGHT / 2 - 2;
const SPRITE_SIZE: u32 = 32;
const SPRITE_HEIGHT: u32 = 40;
const TILE_SPRITE_Y_OFFSET: i32 = -28;
const ACTOR_SPRITE_Y_OFFSET: i32 = -36;
const TILE_VISUAL_CENTER_Y_OFFSET: f32 = -2.0;
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
const MENU_CONFIG_JSON: &str = include_str!("../../assets/smart_boy_hero/iso/menu.json");
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
const DANGER_DARK: Pixel = Pixel::rgb(74, 32, 39);
const GUARD_MARK: Pixel = Pixel::rgb(255, 196, 82);
const SMART: Pixel = Pixel::rgb(86, 240, 185);
const BRONZE: Pixel = Pixel::rgb(218, 139, 61);
const LOCKED: Pixel = Pixel::rgb(255, 196, 82);
const MENU_ACCENT: Pixel = Pixel::rgb(91, 214, 255);
const MENU_SELECTED: Pixel = Pixel::rgb(255, 246, 104);

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct DirectionRepeat {
    direction: Option<Direction>,
    held_for: Duration,
    repeat_accumulator: Duration,
}

impl DirectionRepeat {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn update(&mut self, direction: Option<Direction>, delta: Duration) -> Option<Direction> {
        let Some(direction) = direction else {
            self.reset();
            return None;
        };

        if self.direction != Some(direction) {
            self.direction = Some(direction);
            self.held_for = Duration::ZERO;
            self.repeat_accumulator = Duration::ZERO;
            return Some(direction);
        }

        self.held_for = self.held_for.saturating_add(delta);
        if self.held_for < MOVE_REPEAT_DELAY {
            return None;
        }

        self.repeat_accumulator = self.repeat_accumulator.saturating_add(delta);
        if self.repeat_accumulator >= MOVE_REPEAT_PERIOD {
            self.repeat_accumulator -= MOVE_REPEAT_PERIOD;
            Some(direction)
        } else {
            None
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveFeedback {
    key: &'static str,
    elapsed: Duration,
}

impl ActiveFeedback {
    fn new(key: &'static str) -> Self {
        Self {
            key,
            elapsed: Duration::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoMenuConfig {
    style: IsoMenuStyle,
    feedback: IsoFeedbackConfig,
    screens: Vec<IsoMenuScreen>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoMenuScreen {
    id: String,
    title: String,
    style: IsoMenuStyle,
    body: Vec<String>,
    items: Vec<IsoMenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoMenuStyle {
    font: Font,
    title_color: Pixel,
    text_color: Pixel,
    muted_color: Pixel,
    selected_color: Pixel,
    accent_color: Pixel,
    title_scale: u32,
    body_scale: u32,
    item_scale: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoMenuItem {
    label: String,
    action: IsoMenuAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IsoMenuAction {
    Resume,
    Retry,
    Quit,
    Back,
    Submenu(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IsoMenuCursor {
    screen: usize,
    selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoMenu {
    config: IsoMenuConfig,
    stack: Vec<IsoMenuCursor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IsoMenuCommand {
    Continue,
    Resume,
    Retry,
    Quit,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct IsoFeedbackConfig {
    default: IsoFeedbackPreset,
    events: Vec<IsoFeedbackEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoFeedbackEvent {
    key: String,
    preset: IsoFeedbackPreset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoFeedbackPreset {
    text: String,
    x: i32,
    y: i32,
    scale: u32,
    color: Pixel,
    shadow_color: Option<Pixel>,
    backdrop_color: Option<Pixel>,
    backdrop_padding: i32,
    font: Font,
    sound: Option<String>,
}

impl IsoMenu {
    fn new(config: IsoMenuConfig) -> Self {
        Self {
            config,
            stack: Vec::new(),
        }
    }

    fn open(&mut self, screen_id: &str) {
        if let Some(screen) = self.screen_index(screen_id) {
            self.stack.clear();
            self.stack.push(IsoMenuCursor {
                screen,
                selected: 0,
            });
        }
    }

    fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    fn close(&mut self) {
        self.stack.clear();
    }

    fn active_screen(&self) -> Option<&IsoMenuScreen> {
        self.stack
            .last()
            .and_then(|cursor| self.config.screens.get(cursor.screen))
    }

    fn selected_index(&self) -> usize {
        self.stack.last().map(|cursor| cursor.selected).unwrap_or(0)
    }

    fn select_previous(&mut self) {
        let Some(cursor) = self.stack.last_mut() else {
            return;
        };
        let item_count = self
            .config
            .screens
            .get(cursor.screen)
            .map(|screen| screen.items.len())
            .unwrap_or(0);
        if item_count == 0 {
            return;
        }
        cursor.selected = if cursor.selected == 0 {
            item_count - 1
        } else {
            cursor.selected - 1
        };
    }

    fn select_next(&mut self) {
        let Some(cursor) = self.stack.last_mut() else {
            return;
        };
        let item_count = self
            .config
            .screens
            .get(cursor.screen)
            .map(|screen| screen.items.len())
            .unwrap_or(0);
        if item_count == 0 {
            return;
        }
        cursor.selected = (cursor.selected + 1) % item_count;
    }

    fn select(&mut self, index: usize) {
        let Some(cursor) = self.stack.last_mut() else {
            return;
        };
        let item_count = self
            .config
            .screens
            .get(cursor.screen)
            .map(|screen| screen.items.len())
            .unwrap_or(0);
        if index < item_count {
            cursor.selected = index;
        }
    }

    fn activate_selected(&mut self) -> IsoMenuCommand {
        let Some(cursor) = self.stack.last().copied() else {
            return IsoMenuCommand::Continue;
        };
        let Some(action) = self
            .config
            .screens
            .get(cursor.screen)
            .and_then(|screen| screen.items.get(cursor.selected))
            .map(|item| item.action.clone())
        else {
            return IsoMenuCommand::Continue;
        };
        self.apply_action(action)
    }

    fn escape(&mut self) -> IsoMenuCommand {
        if self.stack.len() <= 1 {
            self.close();
            return IsoMenuCommand::Resume;
        }
        self.stack.pop();
        IsoMenuCommand::Continue
    }

    fn apply_action(&mut self, action: IsoMenuAction) -> IsoMenuCommand {
        match action {
            IsoMenuAction::Resume => {
                self.close();
                IsoMenuCommand::Resume
            }
            IsoMenuAction::Retry => {
                self.close();
                IsoMenuCommand::Retry
            }
            IsoMenuAction::Quit => IsoMenuCommand::Quit,
            IsoMenuAction::Back => self.escape(),
            IsoMenuAction::Submenu(target) => {
                if let Some(screen) = self.screen_index(&target) {
                    self.stack.push(IsoMenuCursor {
                        screen,
                        selected: 0,
                    });
                }
                IsoMenuCommand::Continue
            }
        }
    }

    fn screen_index(&self, screen_id: &str) -> Option<usize> {
        self.config
            .screens
            .iter()
            .position(|screen| screen.id == screen_id)
    }
}

impl IsoMenuConfig {
    fn parse(json: &str) -> Result<Self, String> {
        let value = serde_json::from_str::<serde_json::Value>(json)
            .map_err(|err| format!("invalid SBH ISO menu JSON: {err}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| "invalid SBH ISO menu JSON: root must be an object".to_string())?;
        let style = object
            .get("style")
            .map(|value| IsoMenuStyle::parse(value, IsoMenuStyle::default()))
            .transpose()?
            .unwrap_or_default();
        let feedback = object
            .get("feedback")
            .map(IsoFeedbackConfig::parse)
            .transpose()?
            .unwrap_or_default();
        let screens = object
            .get("screens")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "SBH ISO menu JSON must contain a screens array".to_string())?;

        let screens = screens
            .iter()
            .map(|value| parse_menu_screen(value, &style))
            .collect::<Result<Vec<_>, _>>()?;
        if screens.is_empty() {
            return Err("SBH ISO menu config must contain at least one screen".to_string());
        }
        if !screens.iter().any(|screen| screen.id == "pause") {
            return Err("SBH ISO menu config must contain a pause screen".to_string());
        }
        if !screens.iter().any(|screen| screen.id == "game_over") {
            return Err("SBH ISO menu config must contain a game_over screen".to_string());
        }
        if !screens.iter().any(|screen| screen.id == "victory") {
            return Err("SBH ISO menu config must contain a victory screen".to_string());
        }
        for screen in &screens {
            for item in &screen.items {
                if let IsoMenuAction::Submenu(target) = &item.action
                    && !screens.iter().any(|screen| &screen.id == target)
                {
                    return Err(format!(
                        "SBH ISO menu item '{}' targets unknown submenu '{}'",
                        item.label, target
                    ));
                }
            }
        }

        Ok(Self {
            style,
            feedback,
            screens,
        })
    }
}

impl Default for IsoMenuStyle {
    fn default() -> Self {
        Self {
            font: Font::default(),
            title_color: MENU_ACCENT,
            text_color: TEXT,
            muted_color: MUTED,
            selected_color: MENU_SELECTED,
            accent_color: MENU_ACCENT,
            title_scale: 2,
            body_scale: 1,
            item_scale: 1,
        }
    }
}

impl IsoMenuStyle {
    fn parse(value: &serde_json::Value, base: Self) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "SBH ISO menu style must be an object".to_string())?;
        Ok(Self {
            font: optional_font(object, "font")?.unwrap_or(base.font),
            title_color: optional_pixel(object, "title_color")?.unwrap_or(base.title_color),
            text_color: optional_pixel(object, "text_color")?.unwrap_or(base.text_color),
            muted_color: optional_pixel(object, "muted_color")?.unwrap_or(base.muted_color),
            selected_color: optional_pixel(object, "selected_color")?
                .unwrap_or(base.selected_color),
            accent_color: optional_pixel(object, "accent_color")?.unwrap_or(base.accent_color),
            title_scale: optional_u32(object, "title_scale")?
                .unwrap_or(base.title_scale)
                .max(1),
            body_scale: optional_u32(object, "body_scale")?
                .unwrap_or(base.body_scale)
                .max(1),
            item_scale: optional_u32(object, "item_scale")?
                .unwrap_or(base.item_scale)
                .max(1),
        })
    }
}

impl IsoFeedbackConfig {
    fn parse(value: &serde_json::Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "SBH ISO feedback config must be an object".to_string())?;
        let default = object
            .get("default")
            .map(|value| IsoFeedbackPreset::parse(value, IsoFeedbackPreset::default()))
            .transpose()?
            .unwrap_or_default();
        let events_object = object
            .get("events")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "SBH ISO feedback config must contain an events object".to_string())?;
        let events = events_object
            .iter()
            .map(|(key, value)| {
                Ok(IsoFeedbackEvent {
                    key: key.clone(),
                    preset: IsoFeedbackPreset::parse(value, default.clone())?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self { default, events })
    }

    fn preset(&self, key: &str) -> Option<&IsoFeedbackPreset> {
        self.events
            .iter()
            .find(|event| event.key == key)
            .map(|event| &event.preset)
    }
}

impl Default for IsoFeedbackPreset {
    fn default() -> Self {
        Self {
            text: String::new(),
            x: 178,
            y: 55,
            scale: 3,
            color: POWER,
            shadow_color: Some(BG),
            backdrop_color: Some(Pixel::rgba(18, 20, 25, 204)),
            backdrop_padding: 5,
            font: Font::default(),
            sound: None,
        }
    }
}

impl IsoFeedbackPreset {
    fn parse(value: &serde_json::Value, base: Self) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "SBH ISO feedback preset must be an object".to_string())?;
        Ok(Self {
            text: optional_string(object, "text")
                .unwrap_or(base.text.as_str())
                .to_string(),
            x: optional_i32(object, "x")?.unwrap_or(base.x),
            y: optional_i32(object, "y")?.unwrap_or(base.y),
            scale: optional_u32(object, "scale")?.unwrap_or(base.scale).max(1),
            color: optional_pixel(object, "color")?.unwrap_or(base.color),
            shadow_color: optional_nullable_pixel(object, "shadow_color")?
                .unwrap_or(base.shadow_color),
            backdrop_color: optional_nullable_pixel(object, "backdrop_color")?
                .unwrap_or(base.backdrop_color),
            backdrop_padding: optional_i32(object, "backdrop_padding")?
                .unwrap_or(base.backdrop_padding)
                .max(0),
            font: optional_font(object, "font")?.unwrap_or(base.font),
            sound: optional_nullable_string(object, "sound")?
                .map(|sound| sound.map(str::to_string))
                .unwrap_or(base.sound),
        })
    }
}

fn parse_menu_screen(
    value: &serde_json::Value,
    default_style: &IsoMenuStyle,
) -> Result<IsoMenuScreen, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "SBH ISO menu screen must be an object".to_string())?;
    let id = required_string(object, "id")?.to_string();
    let title = required_string(object, "title")?.to_string();
    let style = object
        .get("style")
        .map(|value| IsoMenuStyle::parse(value, default_style.clone()))
        .transpose()?
        .unwrap_or_else(|| default_style.clone());
    let body = object
        .get("body")
        .map(parse_string_array)
        .transpose()?
        .unwrap_or_default();
    let items = object
        .get("items")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("SBH ISO menu screen '{id}' must contain an items array"))?
        .iter()
        .map(parse_menu_item)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(IsoMenuScreen {
        id,
        title,
        style,
        body,
        items,
    })
}

fn parse_menu_item(value: &serde_json::Value) -> Result<IsoMenuItem, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "SBH ISO menu item must be an object".to_string())?;
    let label = required_string(object, "label")?.to_string();
    let action = object
        .get("action")
        .ok_or_else(|| format!("SBH ISO menu item '{label}' must contain an action"))?;

    Ok(IsoMenuItem {
        label,
        action: parse_menu_action(action)?,
    })
}

fn parse_menu_action(value: &serde_json::Value) -> Result<IsoMenuAction, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "SBH ISO menu action must be an object".to_string())?;
    let kind = required_string(object, "kind")?;
    match kind {
        "resume" => Ok(IsoMenuAction::Resume),
        "retry" => Ok(IsoMenuAction::Retry),
        "quit" => Ok(IsoMenuAction::Quit),
        "back" => Ok(IsoMenuAction::Back),
        "submenu" => Ok(IsoMenuAction::Submenu(
            required_string(object, "target")?.to_string(),
        )),
        _ => Err(format!("unsupported SBH ISO menu action kind '{kind}'")),
    }
}

fn required_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<&'a str, String> {
    object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("SBH ISO menu field '{key}' must be a string"))
}

fn optional_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    object.get(key).and_then(serde_json::Value::as_str)
}

fn optional_nullable_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<Option<&'a str>>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(Some(None)),
        Some(value) => value
            .as_str()
            .map(|value| Some(Some(value)))
            .ok_or_else(|| format!("SBH ISO menu field '{key}' must be a string or null")),
    }
}

fn optional_i32(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<i32>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| format!("SBH ISO menu field '{key}' must be an i32")),
    }
}

fn optional_u32(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<u32>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| format!("SBH ISO menu field '{key}' must be a u32")),
    }
}

fn optional_pixel(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<Pixel>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_str()
            .ok_or_else(|| format!("SBH ISO menu field '{key}' must be a color string"))
            .and_then(parse_color)
            .map(Some),
    }
}

fn optional_font(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<Font>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => {
            let name = value
                .as_str()
                .ok_or_else(|| format!("SBH ISO menu field '{key}' must be a font string"))?;
            Font::from_name(name)
                .map(Some)
                .ok_or_else(|| format!("unsupported SBH ISO font '{name}'"))
        }
    }
}

fn optional_nullable_pixel(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<Option<Option<Pixel>>, String> {
    match object.get(key) {
        None => Ok(None),
        Some(serde_json::Value::Null) => Ok(Some(None)),
        Some(value) => value
            .as_str()
            .ok_or_else(|| format!("SBH ISO menu field '{key}' must be a color string or null"))
            .and_then(parse_color)
            .map(|color| Some(Some(color))),
    }
}

fn parse_color(value: &str) -> Result<Pixel, String> {
    match value {
        "bg" => Ok(BG),
        "panel" => Ok(PANEL),
        "text" => Ok(TEXT),
        "muted" => Ok(MUTED),
        "gold" => Ok(GOLD),
        "power" => Ok(POWER),
        "shout" => Ok(SHOUT_COLOR),
        "rock" => Ok(ROCK_COLOR),
        "rock_invalid" => Ok(ROCK_INVALID),
        "rock_valid" => Ok(ROCK_VALID),
        "danger" => Ok(DANGER),
        "danger_dark" => Ok(DANGER_DARK),
        "guard_mark" => Ok(GUARD_MARK),
        "smart" => Ok(SMART),
        "bronze" => Ok(BRONZE),
        "locked" => Ok(LOCKED),
        "menu_accent" => Ok(MENU_ACCENT),
        "menu_selected" => Ok(MENU_SELECTED),
        hex if hex.starts_with('#') => parse_hex_color(hex),
        _ => Err(format!("unsupported SBH ISO color '{value}'")),
    }
}

fn parse_hex_color(value: &str) -> Result<Pixel, String> {
    let hex = &value[1..];
    let parse_byte = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&hex[range], 16).map_err(|_| format!("invalid SBH ISO color '{value}'"))
    };
    match hex.len() {
        6 => Ok(Pixel::rgb(
            parse_byte(0..2)?,
            parse_byte(2..4)?,
            parse_byte(4..6)?,
        )),
        8 => Ok(Pixel::rgba(
            parse_byte(0..2)?,
            parse_byte(2..4)?,
            parse_byte(4..6)?,
            parse_byte(6..8)?,
        )),
        _ => Err(format!("invalid SBH ISO color '{value}'")),
    }
}

fn parse_string_array(value: &serde_json::Value) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| "SBH ISO menu body must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "SBH ISO menu body entries must be strings".to_string())
        })
        .collect()
}

pub struct SmartBoyHeroIsoGame {
    world: SmartBoyWorld,
    sprites: Image,
    sounds: SoundBank,
    menu: IsoMenu,
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
    feedback: Option<ActiveFeedback>,
    death_elapsed: Option<Duration>,
    game_over: bool,
    move_repeat: DirectionRepeat,
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
            menu: IsoMenu::new(
                IsoMenuConfig::parse(MENU_CONFIG_JSON)
                    .expect("checked-in SBH ISO menu config should parse"),
            ),
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
            feedback: None,
            death_elapsed: None,
            game_over: false,
            move_repeat: DirectionRepeat::default(),
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
        self.menu.close();
        self.targeting = None;
        self.rock_flight = None;
        self.rock_impact = None;
        self.shout_pulse = None;
        self.kill_bursts.clear();
        self.smart_flash = None;
        self.key_elapsed = Duration::ZERO;
        self.screen_shake = Duration::ZERO;
        self.feedback = None;
        self.death_elapsed = None;
        self.game_over = false;
        self.move_repeat.reset();
    }

    fn restart_with_audio(&mut self, audio: &mut dyn Audio) {
        self.stop_boulder_roll(audio);
        self.restart();
    }

    fn update_running(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.world.phase() == Phase::Won {
            self.move_repeat.reset();
            return self.update_victory(frame);
        }

        if self.game_over || self.death_elapsed.is_some() {
            self.move_repeat.reset();
            return self.update_death_or_game_over(frame);
        }

        if self.menu.is_open() {
            self.move_repeat.reset();
            return self.update_menu(frame);
        }

        if pressed(frame, Key::R, GamepadButton::West) {
            self.restart_with_audio(frame.audio);
            return GameResult::Continue;
        }

        if self.targeting.is_some() {
            self.move_repeat.reset();
            self.update_targeting(frame);
            self.advance_transients(frame.delta_time);
            return GameResult::Continue;
        }

        if pressed(frame, Key::Escape, GamepadButton::Start) {
            self.move_repeat.reset();
            self.stop_boulder_roll(frame.audio);
            self.menu.open("pause");
            return GameResult::Continue;
        }

        let mut events = Vec::new();
        if let Some(target) = touch_rock_target(frame, &self.world, self.camera) {
            self.move_repeat.reset();
            self.start_rock_flight(target);
        } else if pressed(frame, Key::F, GamepadButton::RightShoulder) {
            self.move_repeat.reset();
            self.start_targeting();
        } else if self.rock_flight.is_none() {
            if let Some(action) = self.requested_action(frame) {
                events.extend(self.capture_player_action(action));
            }
        } else {
            self.move_repeat.reset();
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
        if self.world.phase() != Phase::Running {
            self.stop_boulder_roll(frame.audio);
        }
        self.advance_camera(frame.delta_time);
        self.advance_transients(frame.delta_time);

        GameResult::Continue
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if pressed(frame, Key::Escape, GamepadButton::Start)
            || frame
                .input
                .gamepad_button_any(GamepadButton::East)
                .pressed()
        {
            let command = self.menu.escape();
            return self.apply_menu_command(command, frame.audio);
        }

        if let Some(index) = touched_menu_item(frame, &self.menu) {
            self.menu.select(index);
            let command = self.menu.activate_selected();
            return self.apply_menu_command(command, frame.audio);
        }

        if pressed(frame, Key::Up, GamepadButton::DPadUp) || frame.input.key(Key::W).pressed() {
            self.menu.select_previous();
        }
        if pressed(frame, Key::Down, GamepadButton::DPadDown) || frame.input.key(Key::S).pressed() {
            self.menu.select_next();
        }
        if pressed(frame, Key::Space, GamepadButton::South) {
            let command = self.menu.activate_selected();
            return self.apply_menu_command(command, frame.audio);
        }

        self.advance_transients(frame.delta_time);
        GameResult::Continue
    }

    fn apply_menu_command(&mut self, command: IsoMenuCommand, audio: &mut dyn Audio) -> GameResult {
        match command {
            IsoMenuCommand::Continue | IsoMenuCommand::Resume => GameResult::Continue,
            IsoMenuCommand::Retry => {
                self.restart_with_audio(audio);
                GameResult::Continue
            }
            IsoMenuCommand::Quit => {
                self.stop_boulder_roll(audio);
                GameResult::Exit
            }
        }
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
                self.feedback = None;
                self.menu.open("game_over");
            }
            return GameResult::Continue;
        }

        if !self.menu.is_open() {
            self.menu.open("game_over");
        }
        if pressed(frame, Key::R, GamepadButton::West) {
            self.restart_with_audio(frame.audio);
            return GameResult::Continue;
        }
        let on_game_over_root = self
            .menu
            .active_screen()
            .is_some_and(|screen| screen.id == "game_over");
        if on_game_over_root
            && (pressed(frame, Key::Escape, GamepadButton::Start)
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::East)
                    .pressed())
        {
            return GameResult::Exit;
        }
        self.update_menu(frame)
    }

    fn update_victory(&mut self, frame: &mut Frame<'_>) -> GameResult {
        self.stop_boulder_roll(frame.audio);
        if !self.menu.is_open() {
            self.menu.open("victory");
        }
        if pressed(frame, Key::R, GamepadButton::West) {
            self.restart_with_audio(frame.audio);
            return GameResult::Continue;
        }
        let on_victory_root = self
            .menu
            .active_screen()
            .is_some_and(|screen| screen.id == "victory");
        if on_victory_root
            && (pressed(frame, Key::Escape, GamepadButton::Start)
                || frame
                    .input
                    .gamepad_button_any(GamepadButton::East)
                    .pressed())
        {
            return GameResult::Exit;
        }
        self.update_menu(frame)
    }

    fn start_targeting(&mut self) {
        self.targeting = Some(TargetingState {
            target: initial_rock_target(&self.world),
        });
        self.set_feedback("rock_prompt");
    }

    fn update_targeting(&mut self, frame: &Frame<'_>) {
        if pressed(frame, Key::Escape, GamepadButton::East) {
            self.targeting = None;
            self.set_feedback("target_cancel");
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
                self.set_feedback("target_invalid");
            }
        }
    }

    fn start_rock_flight(&mut self, target: Cell) {
        self.rock_flight = Some(RockFlight {
            from: self.world.hero(),
            to: target,
            elapsed: Duration::ZERO,
        });
        self.set_feedback("rock_throw");
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
                let configured_sound = feedback_key_for_event(event)
                    .and_then(|key| self.menu.config.feedback.preset(key))
                    .and_then(|preset| preset.sound.as_deref())
                    .and_then(|key| {
                        sound_id_from_config_key(
                            key,
                            shout_sound,
                            death_sound.unwrap_or(DEATH_SOUNDS[0]),
                        )
                    });
                configured_sound.or_else(|| {
                    sound_for_event(event, shout_sound, death_sound.unwrap_or(DEATH_SOUNDS[0]))
                })
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

    fn requested_action(&mut self, frame: &Frame<'_>) -> Option<PlayerAction> {
        self.move_repeat
            .update(held_direction(frame), frame.delta_time)
            .map(PlayerAction::Move)
            .or_else(|| pressed(frame, Key::E, GamepadButton::North).then_some(PlayerAction::Shout))
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
                    self.set_feedback(if heard == 0 {
                        "shout_empty"
                    } else {
                        "shout_heard"
                    });
                }
                WorldEvent::RockImpacted { cell, heard } => {
                    self.rock_impact = Some(TimedCell {
                        cell,
                        elapsed: Duration::ZERO,
                    });
                    self.set_feedback(if heard == 0 {
                        "rock_impact"
                    } else {
                        "rock_lured"
                    });
                }
                WorldEvent::EnemyKilled { cell, .. } => self.kill_bursts.push(TimedCell {
                    cell,
                    elapsed: Duration::ZERO,
                }),
                WorldEvent::SmartChain { count } => {
                    self.smart_flash = Some((count, Duration::ZERO));
                    self.set_feedback("smart_chain");
                }
                WorldEvent::BoulderReleased { .. } => {
                    self.set_feedback("boulder_release");
                    self.screen_shake = Duration::from_millis(120);
                }
                WorldEvent::BoulderCrushedEnemy { chain, .. } => {
                    self.set_feedback("boulder_crush");
                    self.screen_shake = Duration::from_millis(if chain >= 3 { 360 } else { 220 });
                }
                WorldEvent::BoulderStopped { .. } => {
                    self.set_feedback("boulder_stop");
                    self.screen_shake = self.screen_shake.max(Duration::from_millis(160));
                }
                WorldEvent::BoulderSmartChain { count } => {
                    self.smart_flash = Some((count, Duration::ZERO));
                    self.set_feedback(if count >= 4 {
                        "boulder_genius"
                    } else {
                        "smart_chain"
                    });
                }
                WorldEvent::TrapTriggered => self.set_feedback("trap_trigger"),
                WorldEvent::WalkerSpottedHero => {
                    self.set_feedback("enemy_alert");
                }
                WorldEvent::CoreKeyDropped { .. } => {
                    self.set_feedback("key_drop");
                }
                WorldEvent::CoreKeyAcquired => {
                    self.set_feedback("key_pickup");
                }
                WorldEvent::LockedGateBlocked => {
                    self.set_feedback("locked");
                }
                WorldEvent::CoreGateUnlocked => {
                    self.set_feedback("gate_open");
                }
                WorldEvent::HeroDied => {
                    self.set_feedback("hero_dead");
                    self.death_elapsed = Some(Duration::ZERO);
                    self.targeting = None;
                    self.rock_flight = None;
                }
                WorldEvent::Won => {
                    self.set_feedback("victory");
                    self.move_repeat.reset();
                    self.targeting = None;
                    self.rock_flight = None;
                    self.menu.open("victory");
                }
                _ => {}
            }
        }
    }

    fn set_feedback(&mut self, key: &'static str) {
        self.feedback = Some(ActiveFeedback::new(key));
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
        if let Some(feedback) = &mut self.feedback {
            feedback.elapsed += delta;
            if feedback.elapsed >= FEEDBACK_DURATION {
                self.feedback = None;
            }
        }
        self.key_elapsed += delta;
        self.screen_shake = self.screen_shake.saturating_sub(delta);
    }

    fn draw(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear(BG);
        let camera = self.camera.with_shake(self.shake_offset());
        draw_hud(framebuffer, &self.world);
        draw_room(framebuffer, &self.world, &self.sprites, self, camera);
        draw_feedback_overlay(framebuffer, self.feedback, &self.menu.config.feedback);
        if self.menu.is_open() {
            draw_menu_overlay(framebuffer, &self.menu);
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

fn held_direction(frame: &Frame<'_>) -> Option<Direction> {
    if held(frame, Key::Up, GamepadButton::DPadUp) || frame.input.key(Key::W).held() {
        Some(Direction::Up)
    } else if held(frame, Key::Down, GamepadButton::DPadDown) || frame.input.key(Key::S).held() {
        Some(Direction::Down)
    } else if held(frame, Key::Left, GamepadButton::DPadLeft) || frame.input.key(Key::A).held() {
        Some(Direction::Left)
    } else if held(frame, Key::Right, GamepadButton::DPadRight) || frame.input.key(Key::D).held() {
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

fn held(frame: &Frame<'_>, key: Key, button: GamepadButton) -> bool {
    frame.input.key(key).held() || frame.input.gamepad_button_any(button).held()
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

const MENU_PANEL: ButtonRect = ButtonRect {
    x: 116,
    y: 58,
    width: 288,
    height: 216,
};

fn touched_menu_item(frame: &Frame<'_>, menu: &IsoMenu) -> Option<usize> {
    let screen = menu.active_screen()?;
    frame
        .input
        .touches()
        .iter()
        .filter(|touch| matches!(touch.phase, TouchPhase::Started))
        .filter_map(|touch| touch.position)
        .find_map(|position| {
            screen
                .items
                .iter()
                .enumerate()
                .find(|(index, _)| menu_item_rect(screen, *index).contains(position))
                .map(|(index, _)| index)
        })
}

fn menu_items_start_y(screen: &IsoMenuScreen) -> i32 {
    let title_height = Framebuffer::text_size_with_font(
        screen.style.font,
        &screen.title,
        screen.style.title_scale,
    )
    .1 as i32;
    let mut y = MENU_PANEL.y + 16 + title_height + 14;
    y += screen.body.len() as i32 * (8 * screen.style.body_scale as i32 + 6);
    y + if screen.body.is_empty() { 8 } else { 10 }
}

fn menu_item_rect(screen: &IsoMenuScreen, index: usize) -> ButtonRect {
    let item_height = 9 * screen.style.item_scale as i32 + 9;
    ButtonRect {
        x: MENU_PANEL.x + 18,
        y: menu_items_start_y(screen) + index as i32 * (item_height + 5),
        width: MENU_PANEL.width - 36,
        height: item_height,
    }
}

fn draw_hud(framebuffer: &mut Framebuffer, world: &SmartBoyWorld) {
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
            "OBJ REACH EXIT"
        } else {
            "OBJ FIND CORE KEY"
        },
        MUTED,
    );
    framebuffer.draw_text(338, 20, "MOVE WASD/ARROWS", MUTED);
    framebuffer.draw_text(338, 32, "SHOUT E  ROCK F  ESC MENU", MUTED);
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
        draw_iso_trap(framebuffer, trap.cell, camera, world.trap_active(index));
    }
    for (index, lever) in world.levers().iter().enumerate() {
        draw_sprite_at_cell(framebuffer, sprites, SpriteFrame::Plate, lever.cell, camera);
        draw_actuator_marker(framebuffer, world.lever_actuator(index), lever.cell, camera);
    }
    draw_exit_marker(framebuffer, world.exit(), camera);
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
    let frame = enemy_sprite_frame(enemy.kind, enemy.intent);
    draw_actor_sprite_at_point(framebuffer, sprites, frame, point);
    if matches!(enemy.kind, EnemyKind::Guard) {
        draw_guard_marker(framebuffer, point);
    }
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
    draw_actor_sprite_at_point(framebuffer, sprites, frame, point);
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
    draw_actor_sprite_at_point(framebuffer, sprites, frame, point);
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
    let point = project_tile_center(cell, camera);
    let x = point.x as i32;
    let y = point.y as i32;
    match actuator {
        world::ActuatorKind::Boulder => {
            draw_iso_diamond_at(
                framebuffer,
                x,
                y,
                ACTUATOR_MARKER_HALF_WIDTH,
                ACTUATOR_MARKER_HALF_HEIGHT,
                BRONZE,
            );
            framebuffer.draw_line(x - 8, y, x + 8, y, BRONZE);
            framebuffer.draw_line(x - 5, y - 2, x - 5, y + 2, BRONZE);
            framebuffer.draw_line(x + 5, y - 2, x + 5, y + 2, BRONZE);
        }
        world::ActuatorKind::Trap => {
            draw_iso_diamond_at(
                framebuffer,
                x,
                y,
                ACTUATOR_MARKER_HALF_WIDTH,
                ACTUATOR_MARKER_HALF_HEIGHT,
                DANGER,
            );
            framebuffer.draw_line(x - 6, y + 2, x, y - 4, DANGER);
            framebuffer.draw_line(x, y - 4, x + 6, y + 2, DANGER);
            framebuffer.draw_line(x - 5, y + 2, x + 5, y + 2, DANGER);
        }
        world::ActuatorKind::Door => {
            draw_iso_diamond_at(
                framebuffer,
                x,
                y,
                ACTUATOR_MARKER_HALF_WIDTH,
                ACTUATOR_MARKER_HALF_HEIGHT,
                LOCKED,
            );
            framebuffer.draw_rect(x - 4, y - 3, 9, 6, LOCKED);
            framebuffer.draw_line(x, y - 5, x, y + 5, LOCKED);
        }
    }
}

fn draw_key_marker(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera, elapsed: Duration) {
    let point = project_tile_center(cell, camera);
    let x = point.x as i32;
    let y = point.y as i32;
    let pulse = (elapsed.as_secs_f32() * 7.0).sin();
    let bob = if pulse > 0.0 { -2 } else { 0 };
    let outer = 18 + if pulse > 0.45 { 2 } else { 0 };
    draw_iso_diamond_at(framebuffer, x, y, outer, outer / 2, BRONZE);
    draw_iso_diamond_at(framebuffer, x, y, 13, 6, LOCKED);
    framebuffer.fill_circle(x, y, 4, LOCKED);

    let key_y = y - 14 + bob;
    framebuffer.fill_circle(x - 10, key_y, 5, LOCKED);
    framebuffer.draw_circle(x - 10, key_y, 8, GOLD);
    framebuffer.draw_line(x - 4, key_y, x + 12, key_y, LOCKED);
    framebuffer.draw_line(x + 6, key_y, x + 6, key_y + 7, LOCKED);
    framebuffer.draw_line(x + 11, key_y, x + 11, key_y + 5, LOCKED);
}

fn draw_locked_gate_marker(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera) {
    let point = project_tile_center(cell, camera);
    framebuffer.draw_circle(point.x as i32, point.y as i32 - 32, 15, LOCKED);
    framebuffer.fill_rect(point.x as i32 - 8, point.y as i32 - 31, 16, 14, LOCKED);
    framebuffer.draw_text(point.x as i32 - 8, point.y as i32 - 48, "KEY", LOCKED);
}

fn draw_exit_marker(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera) {
    let point = project_tile_center(cell, camera);
    let x = point.x as i32;
    let y = point.y as i32;
    draw_iso_diamond_at(framebuffer, x, y, 17, 8, SMART);
    draw_iso_diamond_at(framebuffer, x, y, 10, 5, GOLD);
    framebuffer.draw_text(x - 12, y - 24, "EXIT", SMART);
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
        draw_iso_trap(framebuffer, burst.cell, camera, true);
        draw_actor_sprite_at_point(
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

fn draw_menu_overlay(framebuffer: &mut Framebuffer, menu: &IsoMenu) {
    let Some(screen) = menu.active_screen() else {
        return;
    };
    let panel = MENU_PANEL;
    framebuffer.fill_rect(
        0,
        0,
        FRAMEBUFFER_WIDTH,
        FRAMEBUFFER_HEIGHT,
        Pixel::rgba(0, 0, 0, 150),
    );
    framebuffer.fill_rect(
        panel.x,
        panel.y,
        panel.width as u32,
        panel.height as u32,
        PANEL,
    );
    framebuffer.draw_rect(
        panel.x,
        panel.y,
        panel.width as u32,
        panel.height as u32,
        screen.style.accent_color,
    );
    framebuffer.draw_text_scaled_with_font(
        screen.style.font,
        panel.x + 18,
        panel.y + 16,
        &screen.title,
        screen.style.title_scale,
        screen.style.title_color,
    );

    let title_height = Framebuffer::text_size_with_font(
        screen.style.font,
        &screen.title,
        screen.style.title_scale,
    )
    .1 as i32;
    let mut y = panel.y + 16 + title_height + 14;
    for line in &screen.body {
        framebuffer.draw_text_scaled_with_font(
            screen.style.font,
            panel.x + 18,
            y,
            line,
            screen.style.body_scale,
            screen.style.muted_color,
        );
        y += 8 * screen.style.body_scale as i32 + 6;
    }

    y = menu_items_start_y(screen);
    for (index, item) in screen.items.iter().enumerate() {
        let selected = index == menu.selected_index();
        let color = if selected {
            screen.style.selected_color
        } else {
            screen.style.text_color
        };
        let prefix = if selected { ">" } else { " " };
        let rect = menu_item_rect(screen, index);
        if selected {
            framebuffer.fill_rect(
                rect.x,
                rect.y - 3,
                rect.width as u32,
                rect.height as u32,
                Pixel::rgba(255, 255, 255, 22),
            );
            framebuffer.draw_rect(
                rect.x,
                rect.y - 3,
                rect.width as u32,
                rect.height as u32,
                screen.style.selected_color,
            );
        }
        framebuffer.draw_text_scaled_with_font(
            screen.style.font,
            rect.x + 6,
            y,
            prefix,
            screen.style.item_scale,
            color,
        );
        framebuffer.draw_text_scaled_with_font(
            screen.style.font,
            rect.x + 22,
            y,
            &item.label,
            screen.style.item_scale,
            color,
        );
        y += rect.height + 5;
    }

    framebuffer.draw_text_scaled_with_font(
        screen.style.font,
        panel.x + 18,
        panel.y + panel.height - 24,
        "SPACE OK   ESC BACK",
        screen.style.body_scale,
        screen.style.muted_color,
    );
}

fn draw_feedback_overlay(
    framebuffer: &mut Framebuffer,
    feedback: Option<ActiveFeedback>,
    config: &IsoFeedbackConfig,
) {
    let Some(feedback) = feedback else {
        return;
    };
    let Some(preset) = config.preset(feedback.key) else {
        return;
    };
    if preset.text.is_empty() {
        return;
    }
    let (text_width, text_height) =
        Framebuffer::text_size_with_font(preset.font, &preset.text, preset.scale);
    let x = preset.x;
    let y = preset.y;
    if let Some(color) = preset.backdrop_color {
        let padding = preset.backdrop_padding;
        framebuffer.fill_rect(
            x - padding,
            y - padding,
            text_width + (padding * 2) as u32,
            text_height + (padding * 2) as u32,
            color,
        );
    }
    if let Some(color) = preset.shadow_color {
        framebuffer.draw_text_scaled_with_font(
            preset.font,
            x + 2,
            y + 2,
            &preset.text,
            preset.scale,
            color,
        );
    }
    framebuffer.draw_text_scaled_with_font(
        preset.font,
        x,
        y,
        &preset.text,
        preset.scale,
        preset.color,
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
    let point = project_tile_center(cell, camera);
    draw_iso_diamond_at(
        framebuffer,
        point.x as i32,
        point.y as i32,
        TILE_WIDTH / 2,
        TILE_HEIGHT / 2,
        pixel,
    );
}

fn draw_iso_trap(framebuffer: &mut Framebuffer, cell: Cell, camera: Camera, active: bool) {
    let point = project_tile_center(cell, camera);
    let x = point.x as i32;
    let y = point.y as i32;
    let base = if active {
        DANGER_DARK
    } else {
        Pixel::rgb(31, 35, 39)
    };
    let edge = if active { DANGER } else { MUTED };

    fill_iso_diamond_at(
        framebuffer,
        x,
        y,
        TILE_WIDTH / 2 - 2,
        TILE_HEIGHT / 2 - 1,
        base,
    );
    draw_iso_diamond_at(framebuffer, x, y, TILE_WIDTH / 2 - 1, TILE_HEIGHT / 2, edge);

    if active {
        for dx in [-8, 0, 8] {
            framebuffer.draw_line(x + dx - 4, y + 1, x + dx, y - 7, DANGER);
            framebuffer.draw_line(x + dx + 4, y + 1, x + dx, y - 7, DANGER);
            framebuffer.draw_line(x + dx - 3, y + 2, x + dx + 3, y + 2, DANGER);
        }
    } else {
        framebuffer.draw_line(x - 9, y, x + 9, y, MUTED);
        framebuffer.draw_line(x, y - 4, x, y + 4, MUTED);
    }
}

fn fill_iso_diamond_at(
    framebuffer: &mut Framebuffer,
    x: i32,
    y: i32,
    half_width: i32,
    half_height: i32,
    pixel: Pixel,
) {
    for dy in -half_height..=half_height {
        let width = half_width * (half_height - dy.abs()) / half_height.max(1);
        framebuffer.draw_line(x - width, y + dy, x + width, y + dy, pixel);
    }
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

fn draw_guard_marker(framebuffer: &mut Framebuffer, point: ScreenPoint) {
    let x = point.x as i32;
    let y = point.y as i32;
    framebuffer.draw_rect(x - 7, y - 26, 14, 13, GUARD_MARK);
    framebuffer.draw_line(x - 10, y - 30, x + 10, y - 30, GUARD_MARK);
    framebuffer.draw_line(x + 10, y - 31, x + 10, y - 8, GUARD_MARK);
    framebuffer.fill_rect(x + 8, y - 10, 5, 3, GUARD_MARK);
}

fn enemy_sprite_frame(kind: EnemyKind, intent: EnemyIntent) -> SpriteFrame {
    match kind {
        EnemyKind::Guard => SpriteFrame::WalkerAlert,
        EnemyKind::Walker { .. } | EnemyKind::Rat | EnemyKind::Cat => match intent {
            EnemyIntent::Patrol => SpriteFrame::WalkerPatrol,
            EnemyIntent::Investigate { .. } | EnemyIntent::ChaseHero { .. } => {
                SpriteFrame::WalkerAlert
            }
        },
    }
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
    draw_sprite_at_point_with_y_offset(framebuffer, sprites, frame, point, TILE_SPRITE_Y_OFFSET);
}

fn draw_actor_sprite_at_point(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    point: ScreenPoint,
) {
    draw_sprite_at_point_with_y_offset(framebuffer, sprites, frame, point, ACTOR_SPRITE_Y_OFFSET);
}

fn draw_sprite_at_point_with_y_offset(
    framebuffer: &mut Framebuffer,
    sprites: &Image,
    frame: SpriteFrame,
    point: ScreenPoint,
    y_offset: i32,
) {
    if point.x < -(SPRITE_SIZE as f32)
        || point.x > FRAMEBUFFER_WIDTH as f32 + SPRITE_SIZE as f32
        || point.y < -(SPRITE_HEIGHT as f32)
        || point.y > FRAMEBUFFER_HEIGHT as f32 + SPRITE_HEIGHT as f32
    {
        return;
    }
    let (x, y) = sprite_top_left(point, y_offset);
    framebuffer.draw_image_region(
        x,
        y,
        sprites,
        ImageRegion::new(frame as u32 * SPRITE_SIZE, 0, SPRITE_SIZE, SPRITE_HEIGHT),
    );
}

fn sprite_top_left(point: ScreenPoint, y_offset: i32) -> (i32, i32) {
    (
        point.x as i32 - SPRITE_SIZE as i32 / 2,
        point.y as i32 + y_offset,
    )
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

fn project_tile_center(cell: Cell, camera: Camera) -> ScreenPoint {
    let point = project_cell(cell, camera);
    ScreenPoint {
        x: point.x,
        y: point.y + TILE_VISUAL_CENTER_Y_OFFSET,
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

fn feedback_key_for_event(event: &WorldEvent) -> Option<&'static str> {
    match *event {
        WorldEvent::Shouted { heard, .. } => Some(if heard == 0 {
            "shout_empty"
        } else {
            "shout_heard"
        }),
        WorldEvent::RockImpacted { heard, .. } => Some(if heard == 0 {
            "rock_impact"
        } else {
            "rock_lured"
        }),
        WorldEvent::SmartChain { .. } => Some("smart_chain"),
        WorldEvent::BoulderReleased { .. } => Some("boulder_release"),
        WorldEvent::BoulderCrushedEnemy { .. } => Some("boulder_crush"),
        WorldEvent::BoulderStopped { .. } => Some("boulder_stop"),
        WorldEvent::BoulderSmartChain { count } => Some(if count >= 4 {
            "boulder_genius"
        } else {
            "smart_chain"
        }),
        WorldEvent::TrapTriggered => Some("trap_trigger"),
        WorldEvent::WalkerSpottedHero => Some("enemy_alert"),
        WorldEvent::CoreKeyDropped { .. } => Some("key_drop"),
        WorldEvent::CoreKeyAcquired => Some("key_pickup"),
        WorldEvent::LockedGateBlocked => Some("locked"),
        WorldEvent::CoreGateUnlocked => Some("gate_open"),
        WorldEvent::HeroDied => Some("hero_dead"),
        WorldEvent::Won => Some("victory"),
        _ => None,
    }
}

fn sound_id_from_config_key(
    key: &str,
    shout_sound: SoundId,
    death_sound: SoundId,
) -> Option<SoundId> {
    match key {
        "combat" => Some(COMBAT_SOUND),
        "pressure_plate_on" => Some(PRESSURE_PLATE_ON_SOUND),
        "pressure_plate_off" => Some(PRESSURE_PLATE_OFF_SOUND),
        "door_open" => Some(DOOR_OPEN_SOUND),
        "door_close" => Some(DOOR_CLOSE_SOUND),
        "trap_arm" => Some(TRAP_ARM_SOUND),
        "trap_disarm" => Some(TRAP_DISARM_SOUND),
        "trap_trigger" => Some(TRAP_TRIGGER_SOUND),
        "shout" => Some(shout_sound),
        "rock_impact" => Some(ROCK_IMPACT_SOUND),
        "enemy_kill" => Some(ENEMY_KILL_SOUND),
        "enemy_alert" => Some(ENEMY_ALERT_SOUND),
        "boulder_release" => Some(BOULDER_RELEASE_SOUND),
        "boulder_crush" => Some(BOULDER_CRUSH_SOUND),
        "boulder_stop" => Some(BOULDER_STOP_SOUND),
        "key_pickup" => Some(KEY_PICKUP_SOUND),
        "key_unlock" => Some(KEY_UNLOCK_SOUND),
        "death" => Some(death_sound),
        "victory" => Some(VICTORY_SOUND),
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

    fn checked_in_menu_config() -> IsoMenuConfig {
        IsoMenuConfig::parse(MENU_CONFIG_JSON).expect("checked-in menu config should parse")
    }

    #[test]
    fn held_direction_repeats_after_initial_delay() {
        let mut repeat = DirectionRepeat::default();

        assert_eq!(
            repeat.update(Some(Direction::Right), Duration::from_millis(16)),
            Some(Direction::Right)
        );
        assert_eq!(
            repeat.update(
                Some(Direction::Right),
                MOVE_REPEAT_DELAY - Duration::from_millis(1)
            ),
            None
        );
        assert_eq!(
            repeat.update(Some(Direction::Right), Duration::from_millis(1)),
            None
        );
        assert_eq!(
            repeat.update(
                Some(Direction::Right),
                MOVE_REPEAT_PERIOD - Duration::from_millis(1)
            ),
            Some(Direction::Right)
        );
        assert_eq!(
            repeat.update(Some(Direction::Up), Duration::from_millis(16)),
            Some(Direction::Up)
        );

        assert_eq!(repeat.update(None, Duration::from_millis(16)), None);
        assert_eq!(
            repeat.update(Some(Direction::Up), Duration::from_millis(16)),
            Some(Direction::Up)
        );
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
    fn checked_in_menu_config_describes_pause_and_commands() {
        let config = checked_in_menu_config();
        assert_eq!(config.style.font, Font::Pixel5x7);

        let pause = config
            .screens
            .iter()
            .find(|screen| screen.id == "pause")
            .expect("pause screen should exist");
        assert_eq!(pause.title, "PAUSE");
        assert!(
            pause
                .items
                .iter()
                .any(|item| item.action == IsoMenuAction::Resume)
        );
        assert!(pause.items.iter().any(|item| matches!(
            item.action,
            IsoMenuAction::Submenu(ref target) if target == "commands"
        )));

        let commands = config
            .screens
            .iter()
            .find(|screen| screen.id == "commands")
            .expect("commands screen should exist");
        assert!(commands.body.iter().any(|line| line.contains("SHOUT")));
        assert!(commands.body.iter().any(|line| line.contains("ROCK")));

        let game_over = config
            .screens
            .iter()
            .find(|screen| screen.id == "game_over")
            .expect("game_over screen should exist");
        assert_eq!(game_over.title, "GAME OVER");
        assert!(
            game_over
                .items
                .iter()
                .any(|item| item.action == IsoMenuAction::Retry)
        );

        let victory = config
            .screens
            .iter()
            .find(|screen| screen.id == "victory")
            .expect("victory screen should exist");
        assert_eq!(victory.title, "CLEAR!");
        assert!(victory.body.iter().any(|line| line.contains("CHECKPOINT")));
        assert!(
            victory
                .items
                .iter()
                .any(|item| item.action == IsoMenuAction::Retry)
        );

        let trap_feedback = config
            .feedback
            .preset("trap_trigger")
            .expect("trap feedback should exist");
        assert_eq!(trap_feedback.text, "SNAP!");
        assert_eq!(trap_feedback.color, DANGER);
        assert_eq!(trap_feedback.sound.as_deref(), Some("trap_trigger"));
    }

    #[test]
    fn menu_actions_can_open_submenu_back_and_resume() {
        let mut menu = IsoMenu::new(checked_in_menu_config());
        menu.open("pause");
        assert!(menu.is_open());
        assert_eq!(
            menu.active_screen().map(|screen| screen.id.as_str()),
            Some("pause")
        );

        menu.select_next();
        assert_eq!(menu.activate_selected(), IsoMenuCommand::Continue);
        assert_eq!(
            menu.active_screen().map(|screen| screen.id.as_str()),
            Some("commands")
        );

        assert_eq!(menu.escape(), IsoMenuCommand::Continue);
        assert_eq!(
            menu.active_screen().map(|screen| screen.id.as_str()),
            Some("pause")
        );
        assert_eq!(menu.escape(), IsoMenuCommand::Resume);
        assert!(!menu.is_open());
    }

    #[test]
    fn menu_quit_action_maps_to_quit_command() {
        let mut menu = IsoMenu::new(checked_in_menu_config());
        menu.open("pause");

        menu.select_next();
        menu.select_next();

        assert_eq!(menu.activate_selected(), IsoMenuCommand::Quit);
    }

    #[test]
    fn game_over_retry_action_uses_shared_menu_command() {
        let mut menu = IsoMenu::new(checked_in_menu_config());
        menu.open("game_over");

        assert_eq!(menu.activate_selected(), IsoMenuCommand::Retry);
        assert!(!menu.is_open());
    }

    #[test]
    fn victory_event_opens_victory_menu() {
        let mut game = SmartBoyHeroIsoGame::new();

        game.capture_feedback(&[WorldEvent::Won]);

        assert_eq!(
            game.menu.active_screen().map(|screen| screen.id.as_str()),
            Some("victory")
        );
        assert_eq!(game.feedback.map(|feedback| feedback.key), Some("victory"));
    }

    #[test]
    fn open_menu_freezes_iso_simulation() {
        let mut game = SmartBoyHeroIsoGame::new();
        let turn_count = game.world.turn_count();
        let enemies = game
            .world
            .enemies()
            .iter()
            .map(|enemy| enemy.cell)
            .collect::<Vec<_>>();

        game.menu.open("pause");
        update_with_default_frame(&mut game, FIXED_STEP * 4);

        assert_eq!(game.world.turn_count(), turn_count);
        assert_eq!(
            game.world
                .enemies()
                .iter()
                .map(|enemy| enemy.cell)
                .collect::<Vec<_>>(),
            enemies
        );
    }

    #[test]
    fn actor_sprite_feet_are_anchored_on_tile_center_not_bottom_vertex() {
        let point = ScreenPoint { x: 120.0, y: 80.0 };
        let (_, tile_y) = sprite_top_left(point, TILE_SPRITE_Y_OFFSET);
        let (_, actor_y) = sprite_top_left(point, ACTOR_SPRITE_Y_OFFSET);

        assert_eq!(tile_y, 52);
        assert_eq!(actor_y, 44);
        assert_eq!(actor_y + SPRITE_HEIGHT as i32 - 4, point.y as i32);
    }

    #[test]
    fn tile_overlays_use_floor_sprite_visual_center() {
        let camera = Camera {
            offset_x: 12.0,
            offset_y: 8.0,
        };
        let cell = Cell::new(6, 4);
        let logical = project_cell(cell, camera);
        let visual = project_tile_center(cell, camera);

        assert_eq!(visual.x, logical.x);
        assert_eq!(visual.y, logical.y - 2.0);
    }

    #[test]
    fn immobile_guard_uses_different_sprite_than_patrolling_walker() {
        assert_eq!(
            enemy_sprite_frame(EnemyKind::Guard, EnemyIntent::Patrol),
            SpriteFrame::WalkerAlert
        );
        assert_eq!(
            enemy_sprite_frame(
                EnemyKind::Walker {
                    direction: Direction::Right
                },
                EnemyIntent::Patrol
            ),
            SpriteFrame::WalkerPatrol
        );
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
    fn actuator_markers_stay_centered_inside_tile_diamond_bounds() {
        let cell = Cell::new(8, 4);
        let camera = Camera {
            offset_x: 0.0,
            offset_y: 0.0,
        };
        let center = project_tile_center(cell, camera);

        for (actuator, color) in [
            (world::ActuatorKind::Boulder, BRONZE),
            (world::ActuatorKind::Trap, DANGER),
            (world::ActuatorKind::Door, LOCKED),
        ] {
            let mut framebuffer = Framebuffer::new(200, 160);

            draw_actuator_marker(&mut framebuffer, actuator, cell, camera);

            let mut bounds = None;
            for y in 0..160 {
                for x in 0..200 {
                    if framebuffer.pixel(x, y) == Some(color) {
                        let (min_x, max_x, min_y, max_y) = bounds.unwrap_or((x, x, y, y));
                        bounds = Some((min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y)));
                        assert!(
                            (x - center.x as i32).abs() <= ACTUATOR_MARKER_HALF_WIDTH,
                            "actuator marker overflowed tile width at ({x}, {y})"
                        );
                        assert!(
                            (y - center.y as i32).abs() <= ACTUATOR_MARKER_HALF_HEIGHT,
                            "actuator marker overflowed tile height at ({x}, {y})"
                        );
                    }
                }
            }
            let bounds = bounds.expect("actuator marker should draw pixels");
            assert_eq!(bounds.0, center.x as i32 - ACTUATOR_MARKER_HALF_WIDTH);
            assert_eq!(bounds.1, center.x as i32 + ACTUATOR_MARKER_HALF_WIDTH);
            assert_eq!(bounds.2, center.y as i32 - ACTUATOR_MARKER_HALF_HEIGHT);
            assert_eq!(bounds.3, center.y as i32 + ACTUATOR_MARKER_HALF_HEIGHT);
        }
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
        let point = project_tile_center(key_cell, camera);
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
        assert_eq!(
            game.menu.active_screen().map(|screen| screen.id.as_str()),
            Some("game_over")
        );
    }

    #[test]
    fn game_over_draws_panel_after_death() {
        let mut game = SmartBoyHeroIsoGame::new();
        game.game_over = true;
        game.menu.open("game_over");
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
        assert_eq!(
            game.menu.active_screen().map(|screen| screen.id.as_str()),
            Some("game_over")
        );
    }

    #[test]
    fn configured_feedback_draws_over_the_room_layer() {
        let mut game = SmartBoyHeroIsoGame::new();
        game.set_feedback("trap_trigger");
        let preset = game
            .menu
            .config
            .feedback
            .preset("trap_trigger")
            .expect("trap feedback should exist")
            .clone();
        let mut framebuffer = Framebuffer::new(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT);

        game.draw(&mut framebuffer);

        assert!(
            framebuffer_has_pixel_near(
                &framebuffer,
                ScreenPoint {
                    x: preset.x as f32,
                    y: preset.y as f32
                },
                44,
                DANGER,
            ),
            "configured feedback text should be drawn after the map"
        );
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
