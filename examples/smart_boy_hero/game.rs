use std::time::Duration;

use gotoo_pixel_engine::{
    ActionId, Audio, AudioError, ControlMap, Frame, Framebuffer, Game, GameResult, GamepadButton,
    Key, Pixel, Rect, Size, SoundBank, SoundId,
    ui::{MenuState, VirtualButton, VirtualPad, draw_menu_item, draw_panel, draw_text_centered},
};
use include_dir::{Dir, include_dir};

#[path = "world.rs"]
mod world;
use world::{
    Bonus, BonusKind, Cell, Direction, Enemy, EnemyIntent, EnemyKind, GRID_HEIGHT, GRID_WIDTH,
    LEVEL_COUNT, LeverKind, Phase, PlayerAction, SmartBoyWorld, WorldEvent,
};

pub const FRAMEBUFFER_WIDTH: u32 = 320;
pub const TOUCH_FRAMEBUFFER_WIDTH: u32 = 480;
pub const FRAMEBUFFER_HEIGHT: u32 = 224;
pub const TOUCH_FRAMEBUFFER_HEIGHT: u32 = 224;

const INITIAL_SEED: u32 = 0x5B00_0001;
const CELL_SIZE: i32 = 20;
const BOARD_X: i32 = 8;
const BOARD_Y: i32 = 44;
const BOARD_WIDTH: u32 = GRID_WIDTH as u32 * CELL_SIZE as u32;
const BOARD_HEIGHT: u32 = GRID_HEIGHT as u32 * CELL_SIZE as u32;
const FEEDBACK_DURATION: Duration = Duration::from_millis(480);
const FIXED_STEP: Duration = Duration::from_millis(420);

const MOVE_UP: ActionId = ActionId::new("smart_boy_hero.up");
const MOVE_RIGHT: ActionId = ActionId::new("smart_boy_hero.right");
const MOVE_DOWN: ActionId = ActionId::new("smart_boy_hero.down");
const MOVE_LEFT: ActionId = ActionId::new("smart_boy_hero.left");
const WAIT: ActionId = ActionId::new("smart_boy_hero.wait");
const SHOUT: ActionId = ActionId::new("smart_boy_hero.shout");
const RETRY: ActionId = ActionId::new("smart_boy_hero.retry");
const PAUSE: ActionId = ActionId::new("smart_boy_hero.pause");

const COMBAT_SOUND: SoundId = SoundId::new("smart_boy_hero.combat");
const BONUS_SOUND: SoundId = SoundId::new("smart_boy_hero.bonus");
const MYSTERY_BONUS_SOUND: SoundId = SoundId::new("smart_boy_hero.mystery_bonus");
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

const PAUSE_MENU_ITEMS: usize = 5;
const LEVEL_SELECT_ITEMS: usize = 2;
const SFX_ASSET_PREFIX: &str = "assets/smart_boy_hero/";
const SHOUT_SFX_KEY: &str = "shout";
const DEATH_SFX_KEY: &str = "death";
const SFX_CONFIG_JSON: &str = include_str!("../../assets/smart_boy_hero/sfx.json");
static SMART_BOY_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/smart_boy_hero");

const BG: Pixel = Pixel::rgb(8, 10, 13);
const HUD_BG: Pixel = Pixel::rgb(17, 18, 24);
const BOARD_BG: Pixel = Pixel::rgb(18, 20, 22);
const GRID: Pixel = Pixel::rgb(39, 43, 47);
const TEXT: Pixel = Pixel::rgb(238, 238, 224);
const MUTED: Pixel = Pixel::rgb(145, 154, 160);
const HERO: Pixel = Pixel::rgb(70, 215, 120);
const HERO_DARK: Pixel = Pixel::rgb(22, 94, 58);
const GUARD: Pixel = Pixel::rgb(238, 76, 82);
const WALKER: Pixel = Pixel::rgb(250, 156, 58);
const BONUS: Pixel = Pixel::rgb(86, 196, 246);
const MYSTERY: Pixel = Pixel::rgb(235, 92, 218);
const WALL: Pixel = Pixel::rgb(92, 98, 104);
const DOOR_CLOSED: Pixel = Pixel::rgb(112, 74, 214);
const DOOR_OPEN: Pixel = Pixel::rgb(58, 88, 120);
const LEVER: Pixel = Pixel::rgb(245, 210, 72);
const EXIT_COLOR: Pixel = Pixel::rgb(122, 238, 166);
const DANGER: Pixel = Pixel::rgb(255, 58, 58);
const WIN: Pixel = Pixel::rgb(248, 236, 88);
const PANEL: Pixel = Pixel::rgb(14, 18, 24);
const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 80);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SmartBoyHeroMode {
    Native,
    Touch,
}

impl SmartBoyHeroMode {
    pub const fn framebuffer_size(self) -> Size {
        match self {
            Self::Native => Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT,
            },
            Self::Touch => Size {
                width: TOUCH_FRAMEBUFFER_WIDTH,
                height: TOUCH_FRAMEBUFFER_HEIGHT,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layout {
    framebuffer_size: Size,
    hud: Rect,
    board: Rect,
    side: Rect,
    touch_panel: Option<Rect>,
    up: Option<Rect>,
    right: Option<Rect>,
    down: Option<Rect>,
    left: Option<Rect>,
    wait: Option<Rect>,
    shout: Option<Rect>,
    retry: Option<Rect>,
    pause: Option<Rect>,
}

impl Layout {
    const fn for_mode(mode: SmartBoyHeroMode) -> Self {
        let framebuffer_size = mode.framebuffer_size();
        let touch = matches!(mode, SmartBoyHeroMode::Touch);
        Self {
            framebuffer_size,
            hud: Rect {
                x: 0,
                y: 0,
                width: framebuffer_size.width,
                height: 36,
            },
            board: Rect {
                x: BOARD_X,
                y: BOARD_Y,
                width: BOARD_WIDTH,
                height: BOARD_HEIGHT,
            },
            side: Rect {
                x: BOARD_X + BOARD_WIDTH as i32 + 8,
                y: BOARD_Y,
                width: 64,
                height: BOARD_HEIGHT,
            },
            touch_panel: if touch {
                Some(Rect {
                    x: FRAMEBUFFER_WIDTH as i32,
                    y: 0,
                    width: TOUCH_FRAMEBUFFER_WIDTH - FRAMEBUFFER_WIDTH,
                    height: TOUCH_FRAMEBUFFER_HEIGHT,
                })
            } else {
                None
            },
            up: if touch {
                Some(rect(374, 38, 56, 38))
            } else {
                None
            },
            left: if touch {
                Some(rect(334, 80, 56, 38))
            } else {
                None
            },
            right: if touch {
                Some(rect(414, 80, 56, 38))
            } else {
                None
            },
            down: if touch {
                Some(rect(374, 122, 56, 38))
            } else {
                None
            },
            wait: if touch {
                Some(rect(334, 164, 62, 28))
            } else {
                None
            },
            shout: if touch {
                Some(rect(408, 164, 62, 28))
            } else {
                None
            },
            retry: if touch {
                Some(rect(371, 198, 62, 20))
            } else {
                None
            },
            pause: if touch {
                Some(rect(364, 8, 76, 24))
            } else {
                None
            },
        }
    }

    fn cell_rect(self, cell: Cell) -> Rect {
        Rect {
            x: self.board.x + cell.x * CELL_SIZE,
            y: self.board.y + cell.y * CELL_SIZE,
            width: CELL_SIZE as u32,
            height: CELL_SIZE as u32,
        }
    }
}

const fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiState {
    Running,
    PauseMenu,
    LevelSelect,
    Controls,
    ResumeGate,
}

pub struct SmartBoyHeroGame {
    mode: SmartBoyHeroMode,
    world: SmartBoyWorld,
    controls: ControlMap,
    virtual_pad: Option<VirtualPad>,
    sounds: SoundBank,
    ui_state: UiState,
    pause_menu: MenuState,
    level_select_menu: MenuState,
    selected_level: usize,
    feedback: Vec<WorldEvent>,
    feedback_timer: Duration,
    pending_audio_events: Vec<WorldEvent>,
    simulation_accumulator: Duration,
    sfx_rng: u32,
}

impl SmartBoyHeroGame {
    pub fn new() -> Self {
        Self::new_with_mode(SmartBoyHeroMode::Native)
    }

    #[allow(dead_code)]
    pub fn new_touch() -> Self {
        Self::new_with_mode(SmartBoyHeroMode::Touch)
    }

    fn new_with_mode(mode: SmartBoyHeroMode) -> Self {
        Self {
            mode,
            world: SmartBoyWorld::new(INITIAL_SEED),
            controls: controls(),
            virtual_pad: virtual_pad_for_mode(mode),
            sounds: smart_boy_sound_bank().expect("Smart Boy Hero SFX config should be valid"),
            ui_state: UiState::Running,
            pause_menu: MenuState::new(PAUSE_MENU_ITEMS),
            level_select_menu: MenuState::new(LEVEL_SELECT_ITEMS),
            selected_level: 0,
            feedback: Vec::new(),
            feedback_timer: Duration::ZERO,
            pending_audio_events: Vec::new(),
            simulation_accumulator: Duration::ZERO,
            sfx_rng: INITIAL_SEED ^ 0xA11D_10ED,
        }
    }

    fn layout(&self) -> Layout {
        Layout::for_mode(self.mode)
    }

    fn open_pause_menu(&mut self) {
        self.ui_state = UiState::PauseMenu;
        self.pause_menu = MenuState::new(PAUSE_MENU_ITEMS);
        self.selected_level = self.world.level_index();
    }

    fn enter_resume_gate(&mut self) {
        self.ui_state = UiState::ResumeGate;
    }

    fn update_running(&mut self, delta: Duration) {
        if self.controls.action(PAUSE).pressed() {
            self.open_pause_menu();
            return;
        }

        if self.controls.action(RETRY).pressed() {
            self.restart();
            return;
        }

        let mut events = Vec::new();

        if let Some(action) = requested_action(&self.controls) {
            let report = self.world.apply(action);
            events.extend(report.events);
        }

        if self.world.semi_continuous() {
            self.simulation_accumulator += delta;
            while self.simulation_accumulator >= FIXED_STEP && self.world.phase() == Phase::Running
            {
                self.simulation_accumulator -= FIXED_STEP;
                let report = self.world.update_tick();
                events.extend(report.events);
            }
        }

        if !events.is_empty() {
            self.feedback = events;
            self.pending_audio_events = self.feedback.clone();
            self.feedback_timer = FEEDBACK_DURATION;
        }
    }

    fn update_finished(&mut self) {
        if self.controls.action(PAUSE).pressed() {
            self.open_pause_menu();
            return;
        }

        if self.controls.action(RETRY).pressed() {
            self.restart();
            return;
        }

        if self.world.phase() == Phase::Won && self.controls.action(WAIT).pressed() {
            self.world.next_level();
            self.clear_transient_state();
        }
    }

    fn update_pause_menu(&mut self) -> GameResult {
        if self.controls.action(PAUSE).pressed() {
            self.enter_resume_gate();
            return GameResult::Continue;
        }

        if self.controls.action(MOVE_UP).pressed() {
            self.pause_menu.select_previous();
        }
        if self.controls.action(MOVE_DOWN).pressed() {
            self.pause_menu.select_next();
        }

        if self.controls.action(WAIT).pressed() {
            match self.pause_menu.selected() {
                Some(0) => self.enter_resume_gate(),
                Some(1) => {
                    self.ui_state = UiState::LevelSelect;
                    self.level_select_menu = MenuState::new(LEVEL_SELECT_ITEMS);
                    self.selected_level = self.world.level_index();
                }
                Some(2) => self.ui_state = UiState::Controls,
                Some(3) => {
                    self.restart();
                    self.enter_resume_gate();
                }
                Some(4) => return GameResult::Exit,
                _ => {}
            }
        }

        GameResult::Continue
    }

    fn update_level_select(&mut self) {
        if self.controls.action(PAUSE).pressed() || self.controls.action(RETRY).pressed() {
            self.ui_state = UiState::PauseMenu;
            return;
        }

        if self.controls.action(MOVE_LEFT).pressed() {
            self.selected_level = if self.selected_level == 0 {
                LEVEL_COUNT - 1
            } else {
                self.selected_level - 1
            };
        }
        if self.controls.action(MOVE_RIGHT).pressed() {
            self.selected_level = (self.selected_level + 1) % LEVEL_COUNT;
        }
        if self.controls.action(MOVE_UP).pressed() {
            self.level_select_menu.select_previous();
        }
        if self.controls.action(MOVE_DOWN).pressed() {
            self.level_select_menu.select_next();
        }

        if self.controls.action(WAIT).pressed() {
            match self.level_select_menu.selected() {
                Some(0) => {
                    self.load_level(self.selected_level);
                    self.enter_resume_gate();
                }
                Some(1) => self.ui_state = UiState::PauseMenu,
                _ => {}
            }
        }
    }

    fn update_controls_screen(&mut self) {
        if self.controls.action(PAUSE).pressed()
            || self.controls.action(RETRY).pressed()
            || self.controls.action(WAIT).pressed()
        {
            self.ui_state = UiState::PauseMenu;
        }
    }

    fn update_resume_gate(&mut self) {
        if self.pause_input_held() {
            return;
        }

        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.reset(&mut self.controls);
        }
        self.ui_state = UiState::Running;
    }

    fn restart(&mut self) {
        self.world.restart();
        self.clear_transient_state();
    }

    fn load_level(&mut self, level_index: usize) {
        self.world.load_level(level_index);
        self.clear_transient_state();
    }

    fn clear_transient_state(&mut self) {
        self.feedback.clear();
        self.feedback_timer = Duration::ZERO;
        self.pending_audio_events.clear();
        self.simulation_accumulator = Duration::ZERO;
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.reset(&mut self.controls);
        }
    }

    fn pause_input_held(&self) -> bool {
        [
            MOVE_UP, MOVE_RIGHT, MOVE_DOWN, MOVE_LEFT, WAIT, SHOUT, RETRY, PAUSE,
        ]
        .into_iter()
        .any(|action| self.controls.action(action).held())
    }

    fn tick_feedback(&mut self, delta: Duration) {
        self.feedback_timer = self.feedback_timer.saturating_sub(delta);
        if self.feedback_timer.is_zero() {
            self.feedback.clear();
        }
    }

    fn play_sounds(&mut self, audio: &mut dyn Audio, events: &[WorldEvent]) {
        for sound in self.sounds_for_events(events) {
            let _ = self.sounds.play(audio, sound);
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

    fn draw(&self, framebuffer: &mut Framebuffer) {
        let layout = self.layout();
        framebuffer.clear(BG);
        draw_hud(framebuffer, layout, &self.world);
        draw_board(framebuffer, layout, &self.world, &self.feedback);
        draw_side_panel(framebuffer, layout, &self.world, &self.feedback);

        if self.virtual_pad.is_some() {
            draw_touch_controls(framebuffer, layout, self.world.phase(), self.ui_state);
        }

        if self.ui_state != UiState::Running {
            draw_pause_overlay(framebuffer, layout, self);
            return;
        }

        match self.world.phase() {
            Phase::Running => {}
            Phase::Dead => draw_overlay(framebuffer, layout, "NOPE.", "R / RETRY"),
            Phase::Won => draw_overlay(framebuffer, layout, "SMART BOY!", "SPACE / WAIT NEXT"),
        }
    }
}

impl Game for SmartBoyHeroGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.controls);
        }
        self.controls.update(frame.input);

        let result = match self.ui_state {
            UiState::Running => {
                self.tick_feedback(frame.delta_time);
                match self.world.phase() {
                    Phase::Running => self.update_running(frame.delta_time),
                    Phase::Dead | Phase::Won => self.update_finished(),
                }
                let events = std::mem::take(&mut self.pending_audio_events);
                self.play_sounds(frame.audio, &events);
                GameResult::Continue
            }
            UiState::PauseMenu => self.update_pause_menu(),
            UiState::LevelSelect => {
                self.update_level_select();
                GameResult::Continue
            }
            UiState::Controls => {
                self.update_controls_screen();
                GameResult::Continue
            }
            UiState::ResumeGate => {
                self.update_resume_gate();
                GameResult::Continue
            }
        };

        self.draw(frame.framebuffer);
        result
    }
}

fn controls() -> ControlMap {
    let mut controls = ControlMap::new();
    controls
        .bind_key(MOVE_UP, Key::Up)
        .bind_key(MOVE_UP, Key::W)
        .bind_gamepad(MOVE_UP, GamepadButton::DPadUp)
        .bind_gamepad(MOVE_UP, GamepadButton::LeftStickUp)
        .bind_key(MOVE_RIGHT, Key::Right)
        .bind_key(MOVE_RIGHT, Key::D)
        .bind_gamepad(MOVE_RIGHT, GamepadButton::DPadRight)
        .bind_gamepad(MOVE_RIGHT, GamepadButton::LeftStickRight)
        .bind_key(MOVE_DOWN, Key::Down)
        .bind_key(MOVE_DOWN, Key::S)
        .bind_gamepad(MOVE_DOWN, GamepadButton::DPadDown)
        .bind_gamepad(MOVE_DOWN, GamepadButton::LeftStickDown)
        .bind_key(MOVE_LEFT, Key::Left)
        .bind_key(MOVE_LEFT, Key::A)
        .bind_gamepad(MOVE_LEFT, GamepadButton::DPadLeft)
        .bind_gamepad(MOVE_LEFT, GamepadButton::LeftStickLeft)
        .bind_key(WAIT, Key::Space)
        .bind_gamepad(WAIT, GamepadButton::South)
        .bind_key(SHOUT, Key::E)
        .bind_gamepad(SHOUT, GamepadButton::North)
        .bind_key(RETRY, Key::R)
        .bind_gamepad(RETRY, GamepadButton::West)
        .bind_key(PAUSE, Key::Escape)
        .bind_gamepad(PAUSE, GamepadButton::Start);
    controls
}

fn virtual_pad_for_mode(mode: SmartBoyHeroMode) -> Option<VirtualPad> {
    let layout = Layout::for_mode(mode);
    Some(VirtualPad::new([
        VirtualButton::new(MOVE_UP, layout.up?),
        VirtualButton::new(MOVE_RIGHT, layout.right?),
        VirtualButton::new(MOVE_DOWN, layout.down?),
        VirtualButton::new(MOVE_LEFT, layout.left?),
        VirtualButton::new(WAIT, layout.wait?),
        VirtualButton::new(SHOUT, layout.shout?),
        VirtualButton::new(RETRY, layout.retry?),
        VirtualButton::new(PAUSE, layout.pause?),
    ]))
}

fn requested_action(controls: &ControlMap) -> Option<PlayerAction> {
    [
        (MOVE_UP, Direction::Up),
        (MOVE_RIGHT, Direction::Right),
        (MOVE_DOWN, Direction::Down),
        (MOVE_LEFT, Direction::Left),
    ]
    .into_iter()
    .find(|(action, _)| controls.action(*action).pressed())
    .map(|(_, direction)| PlayerAction::Move(direction))
    .or_else(|| {
        controls
            .action(SHOUT)
            .pressed()
            .then_some(PlayerAction::Shout)
    })
    .or_else(|| {
        controls
            .action(WAIT)
            .pressed()
            .then_some(PlayerAction::Wait)
    })
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
        WorldEvent::BonusCollected { mystery: false, .. } => Some(BONUS_SOUND),
        WorldEvent::BonusCollected { mystery: true, .. } => Some(MYSTERY_BONUS_SOUND),
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
        WorldEvent::Blocked
        | WorldEvent::Waited
        | WorldEvent::CoreKeyDropped { .. }
        | WorldEvent::LockedGateBlocked
        | WorldEvent::BoulderMoved { .. }
        | WorldEvent::BoulderSmartChain { .. }
        | WorldEvent::WalkerLostTarget
        | WorldEvent::WalkerMoved
        | WorldEvent::WalkerResumedPatrol
        | WorldEvent::WalkerTurned => None,
        WorldEvent::SmartChain { .. } => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SfxBinding {
    key: &'static str,
    sound: SoundId,
}

const REQUIRED_SFX: [SfxBinding; 20] = [
    SfxBinding {
        key: "combat",
        sound: COMBAT_SOUND,
    },
    SfxBinding {
        key: "bonus",
        sound: BONUS_SOUND,
    },
    SfxBinding {
        key: "mystery_bonus",
        sound: MYSTERY_BONUS_SOUND,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SfxConfig {
    paths: Vec<(&'static str, String)>,
}

impl SfxConfig {
    fn parse(json: &str) -> Result<Self, AudioError> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|err| AudioError::new(format!("invalid SBH SFX JSON: {err}")))?;
        let object = value
            .as_object()
            .ok_or_else(|| AudioError::new("invalid SBH SFX JSON: root must be an object"))?;

        let mut paths =
            Vec::with_capacity(REQUIRED_SFX.len() + SHOUT_SOUNDS.len() + DEATH_SOUNDS.len());
        for binding in REQUIRED_SFX {
            let value = object.get(binding.key).ok_or_else(|| {
                AudioError::new(format!("missing required SBH SFX entry '{}'", binding.key))
            })?;
            let path = parse_sfx_path(binding.key, value)?;
            paths.push((binding.key, path.to_string()));
        }
        parse_required_variant_sfx(object, SHOUT_SFX_KEY, SHOUT_SOUNDS.len(), &mut paths)?;
        parse_required_variant_sfx(object, DEATH_SFX_KEY, DEATH_SOUNDS.len(), &mut paths)?;

        Ok(Self { paths })
    }

    fn path_for(&self, key: &str) -> Option<&str> {
        self.paths
            .iter()
            .find(|(candidate, _)| *candidate == key)
            .map(|(_, path)| path.as_str())
    }

    fn paths_for<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> {
        self.paths
            .iter()
            .filter(move |(candidate, _)| *candidate == key)
            .map(|(_, path)| path.as_str())
    }
}

fn parse_required_variant_sfx(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &'static str,
    expected_len: usize,
    paths: &mut Vec<(&'static str, String)>,
) -> Result<(), AudioError> {
    let value = object
        .get(key)
        .ok_or_else(|| AudioError::new(format!("missing required SBH SFX entry '{}'", key)))?;
    let parsed = parse_sfx_path_list(key, value)?;
    if parsed.len() != expected_len {
        return Err(AudioError::new(format!(
            "SBH SFX entry '{}' must contain exactly {} file paths",
            key, expected_len
        )));
    }
    for path in parsed {
        paths.push((key, path.to_string()));
    }
    Ok(())
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

fn smart_boy_sound_bank() -> Result<SoundBank, AudioError> {
    sound_bank_from_config(SFX_CONFIG_JSON, &SMART_BOY_ASSETS)
}

fn sound_bank_from_config(json: &str, assets: &Dir<'_>) -> Result<SoundBank, AudioError> {
    let config = SfxConfig::parse(json)?;
    let mut bank = SoundBank::new();

    for binding in REQUIRED_SFX {
        let path = config
            .path_for(binding.key)
            .expect("required SFX entries were validated");
        insert_configured_wav(&mut bank, assets, binding.key, path, binding.sound)?;
    }
    for (path, sound) in config.paths_for(SHOUT_SFX_KEY).zip(SHOUT_SOUNDS) {
        insert_configured_wav(&mut bank, assets, SHOUT_SFX_KEY, path, sound)?;
    }
    for (path, sound) in config.paths_for(DEATH_SFX_KEY).zip(DEATH_SOUNDS) {
        insert_configured_wav(&mut bank, assets, DEATH_SFX_KEY, path, sound)?;
    }

    Ok(bank)
}

fn insert_configured_wav(
    bank: &mut SoundBank,
    assets: &Dir<'_>,
    key: &str,
    path: &str,
    sound: SoundId,
) -> Result<(), AudioError> {
    let asset_path = sbh_asset_relative_path(path)?;
    let file = assets.get_file(asset_path).ok_or_else(|| {
        AudioError::new(format!("SBH SFX file not found for '{}': '{}'", key, path))
    })?;
    bank.insert_wav(sound, file.contents().to_vec())
}

fn sbh_asset_relative_path(path: &str) -> Result<&str, AudioError> {
    path.strip_prefix(SFX_ASSET_PREFIX).ok_or_else(|| {
        AudioError::new(format!(
            "SBH SFX path must start with '{}': '{}'",
            SFX_ASSET_PREFIX, path
        ))
    })
}

fn draw_hud(framebuffer: &mut Framebuffer, layout: Layout, world: &SmartBoyWorld) {
    framebuffer.fill_rect(
        layout.hud.x,
        layout.hud.y,
        layout.hud.width,
        layout.hud.height,
        HUD_BG,
    );
    framebuffer.draw_rect(
        layout.hud.x,
        layout.hud.y,
        layout.hud.width,
        layout.hud.height,
        GRID,
    );
    framebuffer.draw_text_scaled(8, 6, "SMART BOY HERO", 2, WIN);
    framebuffer.draw_text(
        8,
        26,
        &format!(
            "{}/{}  {}",
            world.level_index() + 1,
            LEVEL_COUNT,
            world.level_name()
        ),
        TEXT,
    );
    framebuffer.draw_text_scaled(202, 8, &format!("POWER {}", world.hero_power()), 2, HERO);
}

fn draw_board(
    framebuffer: &mut Framebuffer,
    layout: Layout,
    world: &SmartBoyWorld,
    feedback: &[WorldEvent],
) {
    framebuffer.fill_rect(
        layout.board.x,
        layout.board.y,
        layout.board.width,
        layout.board.height,
        BOARD_BG,
    );

    for x in 0..=GRID_WIDTH {
        let pixel_x = layout.board.x + x * CELL_SIZE;
        framebuffer.draw_line(
            pixel_x,
            layout.board.y,
            pixel_x,
            layout.board.y + layout.board.height as i32,
            GRID,
        );
    }
    for y in 0..=GRID_HEIGHT {
        let pixel_y = layout.board.y + y * CELL_SIZE;
        framebuffer.draw_line(
            layout.board.x,
            pixel_y,
            layout.board.x + layout.board.width as i32,
            pixel_y,
            GRID,
        );
    }

    for &wall in world.walls() {
        fill_cell(framebuffer, layout, wall, WALL);
    }

    draw_exit(framebuffer, layout.cell_rect(world.exit()));

    for (index, door) in world.doors().iter().enumerate() {
        draw_door(
            framebuffer,
            layout.cell_rect(door.cell),
            world.door_open(index),
        );
    }

    for lever in world.levers() {
        draw_lever(framebuffer, layout.cell_rect(lever.cell), lever.kind);
    }

    for (index, trap) in world.traps().iter().enumerate() {
        draw_trap(
            framebuffer,
            layout.cell_rect(trap.cell),
            world.trap_active(index),
        );
    }

    draw_shout_feedback(framebuffer, layout, feedback);
    draw_enemy_kill_feedback(framebuffer, layout, feedback);

    for bonus in world.bonuses() {
        draw_bonus(framebuffer, layout.cell_rect(bonus.cell), *bonus);
    }

    for enemy in world.enemies() {
        draw_enemy(framebuffer, layout.cell_rect(enemy.cell), enemy);
    }

    draw_hero(
        framebuffer,
        layout.cell_rect(world.hero()),
        world.hero_power(),
    );
    framebuffer.draw_rect(
        layout.board.x,
        layout.board.y,
        layout.board.width,
        layout.board.height,
        TEXT,
    );
}

fn draw_side_panel(
    framebuffer: &mut Framebuffer,
    layout: Layout,
    world: &SmartBoyWorld,
    feedback: &[WorldEvent],
) {
    draw_panel(framebuffer, layout.side, PANEL, GRID);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 8, "TURN", MUTED);
    framebuffer.draw_text_scaled(
        layout.side.x + 6,
        layout.side.y + 18,
        &world.turn_count().to_string(),
        2,
        TEXT,
    );
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 50, "RULE", MUTED);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 64, "WIN: >", TEXT);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 78, "EQ DIES", DANGER);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 104, "LAST", MUTED);

    let line = feedback_line(feedback);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 118, line.0, line.1);
    if let Some(extra) = line.2 {
        framebuffer.draw_text(layout.side.x + 6, layout.side.y + 132, extra, line.1);
    }

    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 140, "HELP", MUTED);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 151, "MOVE", TEXT);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 162, "AR/WASD", TEXT);
    framebuffer.draw_text(
        layout.side.x + 6,
        layout.side.y + 173,
        if world.phase() == Phase::Won {
            "SPC NEXT"
        } else {
            "SHOUT E"
        },
        TEXT,
    );
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 184, "WAIT SPC", TEXT);
    framebuffer.draw_text(layout.side.x + 6, layout.side.y + 195, "R RETRY", TEXT);
}

fn feedback_line(events: &[WorldEvent]) -> (&'static str, Pixel, Option<&'static str>) {
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::HeroDied))
    {
        return ("BAD MATH", DANGER, None);
    }
    if events.iter().any(|event| matches!(event, WorldEvent::Won)) {
        return ("GOT OUT", WIN, None);
    }
    if let Some(WorldEvent::BonusCollected { mystery: true, .. }) = events
        .iter()
        .find(|event| matches!(event, WorldEvent::BonusCollected { .. }))
    {
        return ("LUCK!", MYSTERY, Some("+? POP"));
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::BonusCollected { .. }))
    {
        return ("BONUS", BONUS, None);
    }
    if events.iter().any(|event| {
        matches!(
            event,
            WorldEvent::CombatWon { .. } | WorldEvent::WalkerDestroyed { .. }
        )
    }) {
        return ("BONK", GUARD, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::SmartChain { count: 2 }))
    {
        return ("SMART x2", WIN, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::SmartChain { .. }))
    {
        return ("SMART!", WIN, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::TrapTriggered))
    {
        return ("SNAP", DANGER, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::Shouted { heard: 0, .. }))
    {
        return ("SHOUT", MUTED, Some("NO HEAR"));
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::Shouted { .. }))
    {
        return ("SHOUT", WIN, Some("HEARD"));
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::TrapArmed))
    {
        return ("ARMED", DANGER, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::TrapDisarmed))
    {
        return ("SAFE", MUTED, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::DoorOpened))
    {
        return ("OPEN!", LEVER, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::DoorClosed))
    {
        return ("CLOSE", LEVER, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::Blocked))
    {
        return ("NO TURN", MUTED, None);
    }
    if events
        .iter()
        .any(|event| matches!(event, WorldEvent::Waited))
    {
        return ("WAIT", TEXT, None);
    }
    ("READY", MUTED, None)
}

fn draw_touch_controls(
    framebuffer: &mut Framebuffer,
    layout: Layout,
    phase: Phase,
    ui_state: UiState,
) {
    if let Some(panel) = layout.touch_panel {
        draw_panel(framebuffer, panel, BG, GRID);
    }

    if let Some(rect) = layout.pause {
        framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, PANEL);
        framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_ACCENT);
        draw_text_centered(
            framebuffer,
            rect,
            if ui_state == UiState::Running {
                "PAUSE"
            } else {
                "BACK"
            },
            1,
            TOUCH_ACCENT,
        );
    }

    let (left, right, wait, shout, retry) = match ui_state {
        UiState::Running => (
            "<",
            ">",
            if phase == Phase::Won { "NEXT" } else { "WAIT" },
            "SHOUT",
            "RETRY",
        ),
        UiState::LevelSelect => ("PREV", "NEXT", "PLAY", "", "BACK"),
        UiState::PauseMenu | UiState::Controls | UiState::ResumeGate => {
            ("<", ">", "OK", "", "BACK")
        }
    };

    for (rect, label) in [
        (layout.up, "^"),
        (layout.left, left),
        (layout.right, right),
        (layout.down, "V"),
        (layout.wait, wait),
        (layout.shout, shout),
        (layout.retry, retry),
    ] {
        if label.is_empty() {
            continue;
        }
        if let Some(rect) = rect {
            framebuffer.fill_rect(rect.x, rect.y, rect.width, rect.height, PANEL);
            framebuffer.draw_rect(rect.x, rect.y, rect.width, rect.height, TOUCH_ACCENT);
            draw_text_centered(framebuffer, rect, label, 1, TOUCH_ACCENT);
        }
    }
}

fn draw_pause_overlay(framebuffer: &mut Framebuffer, layout: Layout, game: &SmartBoyHeroGame) {
    match game.ui_state {
        UiState::PauseMenu | UiState::ResumeGate => draw_pause_menu(framebuffer, layout, game),
        UiState::LevelSelect => draw_level_select(framebuffer, layout, game),
        UiState::Controls => draw_controls_screen(framebuffer, layout),
        UiState::Running => {}
    }
}

fn centered_panel(layout: Layout, width: u32, height: u32) -> Rect {
    Rect {
        x: ((layout.framebuffer_size.width - width) / 2) as i32,
        y: ((layout.framebuffer_size.height - height) / 2) as i32,
        width,
        height,
    }
}

fn draw_pause_menu(framebuffer: &mut Framebuffer, layout: Layout, game: &SmartBoyHeroGame) {
    let panel = centered_panel(layout, 224, 176);
    draw_panel(framebuffer, panel, Pixel::rgb(10, 12, 16), WIN);
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x,
            y: panel.y + 10,
            width: panel.width,
            height: 22,
        },
        "PAUSED",
        2,
        WIN,
    );

    for (index, label) in [
        "RESUME",
        "LEVEL SELECT",
        "CONTROLS",
        "RESTART LEVEL",
        "QUIT",
    ]
    .into_iter()
    .enumerate()
    {
        draw_menu_item(
            framebuffer,
            Rect {
                x: panel.x + 18,
                y: panel.y + 44 + index as i32 * 24,
                width: panel.width - 36,
                height: 18,
            },
            label,
            game.pause_menu.selected() == Some(index),
            1,
            TEXT,
            WIN,
        );
    }
}

fn draw_level_select(framebuffer: &mut Framebuffer, layout: Layout, game: &SmartBoyHeroGame) {
    let panel = centered_panel(layout, 224, 150);
    draw_panel(framebuffer, panel, Pixel::rgb(10, 12, 16), WIN);
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x,
            y: panel.y + 10,
            width: panel.width,
            height: 16,
        },
        "SELECT LEVEL",
        1,
        WIN,
    );
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x,
            y: panel.y + 36,
            width: panel.width,
            height: 20,
        },
        &format!("<  {} / {}  >", game.selected_level + 1, LEVEL_COUNT),
        2,
        TEXT,
    );
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x + 10,
            y: panel.y + 68,
            width: panel.width - 20,
            height: 14,
        },
        SmartBoyWorld::level_name_at(game.selected_level),
        1,
        WIN,
    );

    for (index, label) in ["PLAY", "BACK"].into_iter().enumerate() {
        draw_menu_item(
            framebuffer,
            Rect {
                x: panel.x + 42,
                y: panel.y + 98 + index as i32 * 24,
                width: panel.width - 84,
                height: 18,
            },
            label,
            game.level_select_menu.selected() == Some(index),
            1,
            TEXT,
            WIN,
        );
    }
}

fn draw_controls_screen(framebuffer: &mut Framebuffer, layout: Layout) {
    let panel = centered_panel(layout, 248, 176);
    draw_panel(framebuffer, panel, Pixel::rgb(10, 12, 16), WIN);
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x,
            y: panel.y + 10,
            width: panel.width,
            height: 16,
        },
        "CONTROLS",
        1,
        WIN,
    );

    for (line, text) in [
        "MOVE   ARROWS/WASD",
        "WAIT   SPACE / SOUTH",
        "SHOUT  E / NORTH",
        "RETRY  R / WEST",
        "PAUSE  ESC / START",
        "MENU   UP DOWN SPACE",
        "LEVEL  LEFT RIGHT",
    ]
    .into_iter()
    .enumerate()
    {
        framebuffer.draw_text(panel.x + 18, panel.y + 36 + line as i32 * 14, text, TEXT);
    }

    draw_menu_item(
        framebuffer,
        Rect {
            x: panel.x + 54,
            y: panel.y + 146,
            width: panel.width - 108,
            height: 18,
        },
        "BACK",
        true,
        1,
        TEXT,
        WIN,
    );
}

fn draw_overlay(framebuffer: &mut Framebuffer, layout: Layout, title: &str, subtitle: &str) {
    let panel = Rect {
        x: layout.board.x + 26,
        y: layout.board.y + 44,
        width: layout.board.width - 52,
        height: 72,
    };
    draw_panel(framebuffer, panel, Pixel::rgb(10, 12, 16), WIN);
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x,
            y: panel.y + 12,
            width: panel.width,
            height: 24,
        },
        title,
        2,
        WIN,
    );
    draw_text_centered(
        framebuffer,
        Rect {
            x: panel.x,
            y: panel.y + 46,
            width: panel.width,
            height: 12,
        },
        subtitle,
        1,
        TEXT,
    );
}

fn fill_cell(framebuffer: &mut Framebuffer, layout: Layout, cell: Cell, color: Pixel) {
    let rect = layout.cell_rect(cell);
    framebuffer.fill_rect(
        rect.x + 1,
        rect.y + 1,
        rect.width - 1,
        rect.height - 1,
        color,
    );
}

fn draw_exit(framebuffer: &mut Framebuffer, rect: Rect) {
    framebuffer.fill_rect(
        rect.x + 3,
        rect.y + 3,
        rect.width - 6,
        rect.height - 6,
        Pixel::BLACK,
    );
    framebuffer.draw_rect(
        rect.x + 3,
        rect.y + 3,
        rect.width - 6,
        rect.height - 6,
        EXIT_COLOR,
    );
    draw_text_centered(framebuffer, rect, "OUT", 1, EXIT_COLOR);
}

fn draw_door(framebuffer: &mut Framebuffer, rect: Rect, open: bool) {
    let color = if open { DOOR_OPEN } else { DOOR_CLOSED };
    framebuffer.fill_rect(
        rect.x + 2,
        rect.y + 1,
        rect.width - 4,
        rect.height - 2,
        color,
    );
    draw_text_centered(
        framebuffer,
        rect,
        if open { "OPEN" } else { "DOOR" },
        1,
        TEXT,
    );
}

fn draw_lever(framebuffer: &mut Framebuffer, rect: Rect, kind: LeverKind) {
    match kind {
        LeverKind::Latch => {
            framebuffer.fill_rect(rect.x + 5, rect.y + 10, rect.width - 10, 5, LEVER);
            framebuffer.draw_line(rect.x + 10, rect.y + 10, rect.x + 15, rect.y + 4, LEVER);
            framebuffer.fill_circle(rect.x + 15, rect.y + 4, 2, LEVER);
        }
        LeverKind::PressurePlate => {
            framebuffer.draw_rect(
                rect.x + 4,
                rect.y + 4,
                rect.width - 8,
                rect.height - 8,
                LEVER,
            );
            framebuffer.draw_line(
                rect.x + 7,
                rect.y + rect.height as i32 / 2,
                rect.x + rect.width as i32 - 8,
                rect.y + rect.height as i32 / 2,
                LEVER,
            );
            framebuffer.draw_line(
                rect.x + rect.width as i32 / 2,
                rect.y + 7,
                rect.x + rect.width as i32 / 2,
                rect.y + rect.height as i32 - 8,
                LEVER,
            );
        }
    }
}

fn draw_trap(framebuffer: &mut Framebuffer, rect: Rect, active: bool) {
    if active {
        framebuffer.fill_rect(
            rect.x + 3,
            rect.y + 3,
            rect.width - 6,
            rect.height - 6,
            Pixel::BLACK,
        );
        framebuffer.draw_rect(
            rect.x + 3,
            rect.y + 3,
            rect.width - 6,
            rect.height - 6,
            DANGER,
        );
        for offset in [4, 9, 14] {
            framebuffer.draw_line(
                rect.x + offset,
                rect.y + rect.height as i32 - 5,
                rect.x + offset + 3,
                rect.y + 5,
                DANGER,
            );
            framebuffer.draw_line(
                rect.x + offset + 6,
                rect.y + rect.height as i32 - 5,
                rect.x + offset + 3,
                rect.y + 5,
                DANGER,
            );
        }
    } else {
        framebuffer.draw_rect(
            rect.x + 4,
            rect.y + 4,
            rect.width - 8,
            rect.height - 8,
            MUTED,
        );
        framebuffer.draw_line(
            rect.x + 6,
            rect.y + rect.height as i32 / 2,
            rect.x + rect.width as i32 - 7,
            rect.y + rect.height as i32 / 2,
            MUTED,
        );
    }
}

fn draw_shout_feedback(framebuffer: &mut Framebuffer, layout: Layout, events: &[WorldEvent]) {
    let Some(cell) = events.iter().find_map(|event| match *event {
        WorldEvent::Shouted { cell, .. } => Some(cell),
        _ => None,
    }) else {
        return;
    };
    let rect = layout.cell_rect(cell);
    let cx = rect.x + rect.width as i32 / 2;
    let cy = rect.y + rect.height as i32 / 2;
    framebuffer.draw_circle(cx, cy, 9, WIN);
    framebuffer.draw_circle(cx, cy, 24, WIN);
    framebuffer.draw_circle(cx, cy, 42, MUTED);
}

fn draw_enemy_kill_feedback(framebuffer: &mut Framebuffer, layout: Layout, events: &[WorldEvent]) {
    for cell in events.iter().filter_map(|event| match *event {
        WorldEvent::EnemyKilled { cell, .. } => Some(cell),
        _ => None,
    }) {
        let rect = layout.cell_rect(cell);
        let cx = rect.x + rect.width as i32 / 2;
        let cy = rect.y + rect.height as i32 / 2;
        framebuffer.draw_circle(cx, cy, 8, DANGER);
        for (dx, dy) in [(-9, 0), (9, 0), (0, -9), (0, 9), (-6, -6), (6, 6)] {
            framebuffer.draw_line(cx, cy, cx + dx, cy + dy, WIN);
        }
    }
}

fn draw_bonus(framebuffer: &mut Framebuffer, rect: Rect, bonus: Bonus) {
    let (label, color) = match bonus.kind {
        BonusKind::Fixed(amount) => (format!("+{amount}"), BONUS),
        BonusKind::Mystery { min, max } => (format!("?{min}-{max}"), MYSTERY),
    };
    framebuffer.fill_rect(
        rect.x + 2,
        rect.y + 2,
        rect.width - 4,
        rect.height - 4,
        Pixel::BLACK,
    );
    framebuffer.draw_rect(
        rect.x + 2,
        rect.y + 2,
        rect.width - 4,
        rect.height - 4,
        color,
    );
    draw_text_centered(framebuffer, rect, &label, 1, color);
}

fn draw_enemy(framebuffer: &mut Framebuffer, rect: Rect, enemy: &Enemy) {
    let color = match enemy.kind {
        EnemyKind::Guard => GUARD,
        EnemyKind::Walker { .. } => WALKER,
    };
    framebuffer.fill_rect(
        rect.x + 2,
        rect.y + 2,
        rect.width - 4,
        rect.height - 4,
        color,
    );
    framebuffer.draw_rect(
        rect.x + 2,
        rect.y + 2,
        rect.width - 4,
        rect.height - 4,
        Pixel::BLACK,
    );

    match enemy.kind {
        EnemyKind::Guard => {
            draw_text_centered(framebuffer, rect, &enemy.power.to_string(), 1, Pixel::BLACK);
        }
        EnemyKind::Walker { direction } => {
            draw_text_centered(
                framebuffer,
                Rect {
                    x: rect.x,
                    y: rect.y + 2,
                    width: rect.width,
                    height: 8,
                },
                &enemy.power.to_string(),
                1,
                Pixel::BLACK,
            );
            draw_walker_direction(framebuffer, rect, direction);
            if matches!(enemy.intent, EnemyIntent::Investigate { .. }) {
                framebuffer.fill_circle(rect.x + rect.width as i32 - 4, rect.y + 4, 3, WIN);
                framebuffer.draw_line(
                    rect.x + rect.width as i32 - 4,
                    rect.y + 9,
                    rect.x + rect.width as i32 - 4,
                    rect.y + 14,
                    Pixel::BLACK,
                );
            }
        }
    }
}

fn draw_walker_direction(framebuffer: &mut Framebuffer, rect: Rect, direction: Direction) {
    let cx = rect.x + rect.width as i32 / 2;
    let cy = rect.y + 14;
    let color = Pixel::BLACK;

    match direction {
        Direction::Up => {
            framebuffer.draw_line(cx, cy - 6, cx, cy + 5, color);
            framebuffer.draw_line(cx, cy - 6, cx - 5, cy - 1, color);
            framebuffer.draw_line(cx, cy - 6, cx + 5, cy - 1, color);
        }
        Direction::Down => {
            framebuffer.draw_line(cx, cy - 5, cx, cy + 6, color);
            framebuffer.draw_line(cx, cy + 6, cx - 5, cy + 1, color);
            framebuffer.draw_line(cx, cy + 6, cx + 5, cy + 1, color);
        }
        Direction::Left => {
            framebuffer.draw_line(cx - 6, cy, cx + 5, cy, color);
            framebuffer.draw_line(cx - 6, cy, cx - 1, cy - 5, color);
            framebuffer.draw_line(cx - 6, cy, cx - 1, cy + 5, color);
        }
        Direction::Right => {
            framebuffer.draw_line(cx - 5, cy, cx + 6, cy, color);
            framebuffer.draw_line(cx + 6, cy, cx + 1, cy - 5, color);
            framebuffer.draw_line(cx + 6, cy, cx + 1, cy + 5, color);
        }
    }
}

fn draw_hero(framebuffer: &mut Framebuffer, rect: Rect, power: i32) {
    framebuffer.fill_rect(
        rect.x + 2,
        rect.y + 5,
        rect.width - 4,
        rect.height - 7,
        HERO,
    );
    framebuffer.draw_rect(
        rect.x + 2,
        rect.y + 5,
        rect.width - 4,
        rect.height - 7,
        HERO_DARK,
    );
    framebuffer.fill_rect(rect.x + 7, rect.y + 1, rect.width - 14, 6, HERO);
    draw_text_centered(
        framebuffer,
        Rect {
            x: rect.x,
            y: rect.y + 8,
            width: rect.width,
            height: 10,
        },
        &power.to_string(),
        1,
        Pixel::BLACK,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotoo_pixel_engine::{Input, NoopAudio, NoopStorage, Viewport};

    #[derive(Default)]
    struct TestAudio {
        backend: NoopAudio,
        plays: Vec<SoundId>,
    }

    impl TestAudio {
        fn count(&self, id: SoundId) -> usize {
            self.plays.iter().filter(|sound| **sound == id).count()
        }
    }

    impl Audio for TestAudio {
        fn register_wav(&mut self, id: SoundId, bytes: &[u8]) -> Result<(), AudioError> {
            self.backend.register_wav(id, bytes)
        }

        fn play(&mut self, id: SoundId) -> Result<(), AudioError> {
            self.backend.play(id)?;
            self.plays.push(id);
            Ok(())
        }
    }

    fn run_frame_with_audio(
        game: &mut SmartBoyHeroGame,
        held_actions: &[ActionId],
        audio: &mut dyn Audio,
    ) -> GameResult {
        run_frame_with_audio_delta(game, held_actions, audio, Duration::from_millis(16))
    }

    fn run_frame_with_audio_delta(
        game: &mut SmartBoyHeroGame,
        held_actions: &[ActionId],
        audio: &mut dyn Audio,
        delta_time: Duration,
    ) -> GameResult {
        game.controls.clear_virtual();
        for &action in held_actions {
            game.controls.set_virtual(action, true);
        }

        let size = game.mode.framebuffer_size();
        let mut framebuffer = Framebuffer::new(size.width, size.height);
        let input = Input::default();
        let mut storage = NoopStorage;
        let mut frame = Frame {
            framebuffer: &mut framebuffer,
            input: &input,
            delta_time,
            storage: &mut storage,
            audio,
            surface_size: size,
            viewport: Viewport::new(size, size),
        };
        game.update(&mut frame)
    }

    fn run_frame_delta(
        game: &mut SmartBoyHeroGame,
        held_actions: &[ActionId],
        delta_time: Duration,
    ) -> GameResult {
        let mut audio = NoopAudio::default();
        run_frame_with_audio_delta(game, held_actions, &mut audio, delta_time)
    }

    fn run_frame(game: &mut SmartBoyHeroGame, held_actions: &[ActionId]) -> GameResult {
        let mut audio = NoopAudio::default();
        run_frame_with_audio(game, held_actions, &mut audio)
    }

    fn release_frame(game: &mut SmartBoyHeroGame) -> GameResult {
        run_frame(game, &[])
    }

    fn tap(game: &mut SmartBoyHeroGame, action: ActionId) {
        run_frame(game, &[action]);
        release_frame(game);
    }

    fn select_pause_item(game: &mut SmartBoyHeroGame, item: usize) {
        game.pause_menu = MenuState::new(PAUSE_MENU_ITEMS);
        for _ in 0..item {
            game.pause_menu.select_next();
        }
    }

    #[test]
    fn mode_sizes_match_public_constants() {
        assert_eq!(
            SmartBoyHeroMode::Native.framebuffer_size(),
            Size {
                width: FRAMEBUFFER_WIDTH,
                height: FRAMEBUFFER_HEIGHT
            }
        );
        assert_eq!(
            SmartBoyHeroMode::Touch.framebuffer_size(),
            Size {
                width: TOUCH_FRAMEBUFFER_WIDTH,
                height: TOUCH_FRAMEBUFFER_HEIGHT
            }
        );
    }

    #[test]
    fn touch_mode_has_virtual_controls_for_all_actions() {
        let game = SmartBoyHeroGame::new_touch();
        let pad = game.virtual_pad.expect("touch mode should have a pad");
        assert_eq!(pad.buttons().len(), 8);
    }

    #[test]
    fn native_mode_has_no_virtual_pad() {
        let game = SmartBoyHeroGame::new();
        assert!(game.virtual_pad.is_none());
    }

    #[test]
    fn sfx_config_parses_valid_mapping() {
        let config = SfxConfig::parse(SFX_CONFIG_JSON).expect("checked-in SFX JSON should parse");
        let combat_path = format!("{}sfx/{}.wav", SFX_ASSET_PREFIX, "combat");
        let plate_on_path = format!("{}sfx/{}.wav", SFX_ASSET_PREFIX, "plate_on");

        assert_eq!(config.path_for("combat"), Some(combat_path.as_str()));
        assert_eq!(
            config.path_for("pressure_plate_on"),
            Some(plate_on_path.as_str())
        );
        assert_eq!(
            config
                .paths_for(SHOUT_SFX_KEY)
                .map(str::to_string)
                .collect::<Vec<_>>(),
            vec![
                format!("{}sfx/smart1.wav", SFX_ASSET_PREFIX),
                format!("{}sfx/smart2.wav", SFX_ASSET_PREFIX),
                format!("{}sfx/smart3.wav", SFX_ASSET_PREFIX)
            ]
        );
        assert_eq!(
            config
                .paths_for(DEATH_SFX_KEY)
                .map(str::to_string)
                .collect::<Vec<_>>(),
            vec![
                format!("{}sfx/death1.wav", SFX_ASSET_PREFIX),
                format!("{}sfx/death2.wav", SFX_ASSET_PREFIX),
                format!("{}sfx/death3.wav", SFX_ASSET_PREFIX)
            ]
        );
    }

    #[test]
    fn sfx_config_reports_missing_required_entry() {
        let json = format!(r#"{{"combat":"{}sfx/{}.wav"}}"#, SFX_ASSET_PREFIX, "combat");
        let error = SfxConfig::parse(&json).expect_err("incomplete config should fail");

        assert!(error.to_string().contains("missing required SBH SFX entry"));
        assert!(error.to_string().contains("bonus"));
    }

    #[test]
    fn sfx_config_rejects_unsupported_audio_format() {
        let path = format!("{}sfx/{}.mp3", SFX_ASSET_PREFIX, "combat");
        let json = complete_sfx_json_with("combat", &path);
        let error = SfxConfig::parse(&json).expect_err("mp3 should not be accepted yet");

        assert!(error.to_string().contains("unsupported audio format"));
        assert!(error.to_string().contains("combat.mp3"));
    }

    #[test]
    fn sfx_bank_reports_missing_configured_file() {
        let path = format!("{}sfx/{}.wav", SFX_ASSET_PREFIX, "missing");
        let json = complete_sfx_json_with("combat", &path);
        let error = sound_bank_from_config(&json, &SMART_BOY_ASSETS)
            .expect_err("missing embedded file should fail");

        assert!(error.to_string().contains("SBH SFX file not found"));
        assert!(error.to_string().contains("missing.wav"));
    }

    #[test]
    fn sfx_config_paths_are_not_hardcoded_in_gameplay_source() {
        let config = SfxConfig::parse(SFX_CONFIG_JSON).expect("checked-in SFX JSON should parse");
        let source = include_str!("game.rs");

        for (_, path) in config.paths {
            assert!(
                !source.contains(&path),
                "SFX path should live in sfx.json, not gameplay source: {path}"
            );
        }
    }

    #[test]
    fn sound_bank_owns_all_configured_sfx_assets() {
        let bank = smart_boy_sound_bank().expect("checked-in SFX config should load");

        for binding in REQUIRED_SFX {
            assert!(bank.contains(binding.sound));
        }
        for sound in SHOUT_SOUNDS {
            assert!(bank.contains(sound));
        }
        for sound in DEATH_SOUNDS {
            assert!(bank.contains(sound));
        }
    }

    #[test]
    fn configured_sfx_assets_are_playable_wav_files() {
        let mut bank = smart_boy_sound_bank().expect("checked-in SFX config should load");
        let mut audio = NoopAudio::default();

        for binding in REQUIRED_SFX {
            bank.play(&mut audio, binding.sound)
                .expect("configured SBH SFX should be valid playable WAV");
        }
        for sound in SHOUT_SOUNDS {
            bank.play(&mut audio, sound)
                .expect("configured SBH shout variant should be valid playable WAV");
        }
        for sound in DEATH_SOUNDS {
            bank.play(&mut audio, sound)
                .expect("configured SBH death variant should be valid playable WAV");
        }
    }

    #[test]
    fn world_events_map_to_expected_sfx() {
        let sounds = sounds_for_events(&[
            WorldEvent::CombatWon { power: 2 },
            WorldEvent::BonusCollected {
                amount: 3,
                mystery: false,
            },
            WorldEvent::BonusCollected {
                amount: 7,
                mystery: true,
            },
            WorldEvent::PressurePlateOn,
            WorldEvent::PressurePlateOff,
            WorldEvent::DoorOpened,
            WorldEvent::DoorClosed,
            WorldEvent::TrapArmed,
            WorldEvent::TrapDisarmed,
            WorldEvent::TrapTriggered,
            WorldEvent::BoulderReleased {
                cell: Cell::new(2, 4),
                direction: Direction::Right,
            },
            WorldEvent::BoulderCrushedEnemy {
                cell: Cell::new(4, 4),
                power: 9,
                chain: 1,
            },
            WorldEvent::BoulderStopped {
                cell: Cell::new(10, 4),
            },
            WorldEvent::Shouted {
                cell: Cell::new(1, 1),
                heard: 2,
            },
            WorldEvent::EnemyKilled {
                cell: Cell::new(2, 1),
                power: 9,
            },
            WorldEvent::WalkerSpottedHero,
            WorldEvent::CoreKeyAcquired,
            WorldEvent::CoreGateUnlocked,
            WorldEvent::HeroDied,
            WorldEvent::Won,
        ]);

        assert_eq!(
            sounds,
            vec![
                COMBAT_SOUND,
                BONUS_SOUND,
                MYSTERY_BONUS_SOUND,
                PRESSURE_PLATE_ON_SOUND,
                PRESSURE_PLATE_OFF_SOUND,
                DOOR_OPEN_SOUND,
                DOOR_CLOSE_SOUND,
                TRAP_ARM_SOUND,
                TRAP_DISARM_SOUND,
                TRAP_TRIGGER_SOUND,
                BOULDER_RELEASE_SOUND,
                BOULDER_CRUSH_SOUND,
                BOULDER_STOP_SOUND,
                SHOUT_SOUNDS[0],
                ENEMY_KILL_SOUND,
                ENEMY_ALERT_SOUND,
                KEY_PICKUP_SOUND,
                KEY_UNLOCK_SOUND,
                DEATH_SOUNDS[0],
                VICTORY_SOUND,
            ]
        );
    }

    #[test]
    fn pressure_plate_audio_plays_transitions_once() {
        let mut game = SmartBoyHeroGame::new();
        game.load_level(10);
        let mut audio = TestAudio::default();

        run_frame_with_audio(&mut game, &[MOVE_RIGHT], &mut audio);
        release_frame_with_audio(&mut game, &mut audio);

        assert_eq!(audio.count(PRESSURE_PLATE_ON_SOUND), 1);
        assert_eq!(audio.count(DOOR_OPEN_SOUND), 1);
        assert_eq!(audio.count(PRESSURE_PLATE_OFF_SOUND), 0);
        assert_eq!(audio.count(DOOR_CLOSE_SOUND), 0);

        run_frame_with_audio(&mut game, &[MOVE_RIGHT], &mut audio);
        release_frame_with_audio(&mut game, &mut audio);
        assert_eq!(audio.count(PRESSURE_PLATE_OFF_SOUND), 0);
        assert_eq!(audio.count(DOOR_CLOSE_SOUND), 0);

        run_frame_with_audio(&mut game, &[MOVE_RIGHT], &mut audio);
        release_frame_with_audio(&mut game, &mut audio);

        assert_eq!(audio.count(PRESSURE_PLATE_ON_SOUND), 1);
        assert_eq!(audio.count(DOOR_OPEN_SOUND), 1);
        assert_eq!(audio.count(PRESSURE_PLATE_OFF_SOUND), 1);
        assert_eq!(audio.count(DOOR_CLOSE_SOUND), 1);
    }

    fn release_frame_with_audio(game: &mut SmartBoyHeroGame, audio: &mut dyn Audio) -> GameResult {
        run_frame_with_audio(game, &[], audio)
    }

    fn complete_sfx_json_with(key: &str, replacement: &str) -> String {
        let mut object = serde_json::from_str::<serde_json::Value>(SFX_CONFIG_JSON)
            .expect("checked-in SFX JSON should parse")
            .as_object()
            .expect("checked-in SFX JSON should be an object")
            .clone();
        object.insert(
            key.to_string(),
            serde_json::Value::String(replacement.to_string()),
        );
        serde_json::Value::Object(object).to_string()
    }

    #[test]
    fn pause_action_opens_pause_and_freezes_gameplay() {
        let mut game = SmartBoyHeroGame::new();
        game.load_level(6);

        tap(&mut game, PAUSE);
        assert_eq!(game.ui_state, UiState::PauseMenu);
        let turn_count = game.world.turn_count();
        let walker_cell = game.world.enemies()[0].cell;

        tap(&mut game, MOVE_DOWN);
        tap(&mut game, MOVE_DOWN);

        assert_eq!(game.ui_state, UiState::PauseMenu);
        assert_eq!(game.world.turn_count(), turn_count);
        assert_eq!(game.world.enemies()[0].cell, walker_cell);
    }

    #[test]
    fn pause_menu_freezes_semi_continuous_simulation() {
        let mut game = SmartBoyHeroGame::new();
        game.load_level(16);
        tap(&mut game, PAUSE);

        let walker_cell = game.world.enemies()[0].cell;
        run_frame_delta(&mut game, &[], FIXED_STEP * 3);

        assert_eq!(game.ui_state, UiState::PauseMenu);
        assert_eq!(game.world.enemies()[0].cell, walker_cell);
        assert_eq!(game.world.turn_count(), 0);
    }

    #[test]
    fn restart_clears_semi_continuous_accumulator() {
        let mut game = SmartBoyHeroGame::new();
        game.load_level(16);

        run_frame_delta(&mut game, &[], FIXED_STEP - Duration::from_millis(10));
        assert!(game.simulation_accumulator > Duration::ZERO);

        run_frame_delta(&mut game, &[RETRY], Duration::from_millis(16));
        assert_eq!(game.simulation_accumulator, Duration::ZERO);

        let walker_cell = game.world.enemies()[0].cell;
        run_frame_delta(&mut game, &[], Duration::from_millis(20));
        assert_eq!(game.world.enemies()[0].cell, walker_cell);
        assert_eq!(game.world.turn_count(), 0);
    }

    #[test]
    fn resume_gate_prevents_confirm_from_leaking_into_gameplay() {
        let mut game = SmartBoyHeroGame::new();

        tap(&mut game, PAUSE);
        run_frame(&mut game, &[WAIT]);
        assert_eq!(game.ui_state, UiState::ResumeGate);
        assert_eq!(game.world.turn_count(), 0);

        release_frame(&mut game);
        assert_eq!(game.ui_state, UiState::Running);
        assert_eq!(game.world.turn_count(), 0);

        release_frame(&mut game);
        assert_eq!(game.world.turn_count(), 0);
    }

    #[test]
    fn level_select_wraps_between_first_and_last_level() {
        let mut game = SmartBoyHeroGame::new();
        game.ui_state = UiState::LevelSelect;
        game.selected_level = 0;

        tap(&mut game, MOVE_LEFT);
        assert_eq!(game.selected_level, LEVEL_COUNT - 1);

        tap(&mut game, MOVE_RIGHT);
        assert_eq!(game.selected_level, 0);
    }

    #[test]
    fn level_select_loads_experimental_levels_by_one_based_number() {
        for (index, name) in [
            (10, "THING DID IT"),
            (11, "HOLD THE DOOR"),
            (12, "TWO SMART WAYS"),
            (13, "WATCH YOUR STEP"),
            (14, "SET THE TRAP"),
            (15, "CLOCKWORK"),
            (16, "COME HERE"),
            (17, "GROUP THERAPY"),
            (18, "SMART WAY"),
        ] {
            let mut game = SmartBoyHeroGame::new();
            game.ui_state = UiState::LevelSelect;
            game.selected_level = index;

            run_frame(&mut game, &[WAIT]);

            assert_eq!(game.world.level_index(), index);
            assert_eq!(game.world.level_name(), name);
            assert_eq!(game.ui_state, UiState::ResumeGate);
        }
    }

    #[test]
    fn level_select_load_resets_level_state() {
        let mut game = SmartBoyHeroGame::new();
        game.load_level(10);
        tap(&mut game, MOVE_RIGHT);
        assert!(game.world.turn_count() > 0);

        game.ui_state = UiState::LevelSelect;
        game.selected_level = 10;
        run_frame(&mut game, &[WAIT]);

        let clean = SmartBoyWorld::for_level(10, INITIAL_SEED);
        assert_eq!(game.world, clean);
        assert!(game.feedback.is_empty());
        assert_eq!(game.feedback_timer, Duration::ZERO);
    }

    #[test]
    fn pause_restart_matches_keyboard_restart() {
        let mut keyboard = SmartBoyHeroGame::new();
        let initial = keyboard.world.clone();
        tap(&mut keyboard, MOVE_RIGHT);
        tap(&mut keyboard, RETRY);
        assert_eq!(keyboard.world, initial);

        let mut paused = SmartBoyHeroGame::new();
        tap(&mut paused, MOVE_RIGHT);
        paused.ui_state = UiState::PauseMenu;
        select_pause_item(&mut paused, 3);
        run_frame(&mut paused, &[WAIT]);

        assert_eq!(paused.world, initial);
        assert_eq!(paused.ui_state, UiState::ResumeGate);
    }

    #[test]
    fn controls_screen_is_accessible_and_returns_to_pause_menu() {
        let mut game = SmartBoyHeroGame::new();
        game.ui_state = UiState::PauseMenu;
        select_pause_item(&mut game, 2);

        tap(&mut game, WAIT);
        assert_eq!(game.ui_state, UiState::Controls);

        tap(&mut game, WAIT);
        assert_eq!(game.ui_state, UiState::PauseMenu);
    }

    #[test]
    fn quit_from_pause_returns_exit() {
        let mut game = SmartBoyHeroGame::new();
        game.ui_state = UiState::PauseMenu;
        select_pause_item(&mut game, 4);

        let result = run_frame(&mut game, &[WAIT]);

        assert_eq!(result, GameResult::Exit);
    }
}
