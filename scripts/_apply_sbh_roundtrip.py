from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


world_path = ROOT / "examples/smart_boy_hero/world.rs"
world = world_path.read_text()

world = replace_once(
    world,
    "    level: Level,\n    phase: Phase,",
    "    level: Level,\n    restart_level: Option<Level>,\n    phase: Phase,",
    "SmartBoyWorld restart source field",
)

world = replace_once(
    world,
    """    #[allow(dead_code)]
    pub(super) fn iso_slice(seed: u32) -> Self {
        Self::from_level(LEVEL_COUNT, level_iso_slice(), seed)
    }

    fn from_level(level_index: usize, level: Level, seed: u32) -> Self {
""",
    """    #[allow(dead_code)]
    pub(super) fn iso_slice(seed: u32) -> Self {
        Self::from_level(LEVEL_COUNT, level_iso_slice(), seed)
    }

    pub(super) fn from_level_json(json: &str, seed: u32) -> Result<Self, String> {
        let level = LevelSpec::parse(json)?
            .into_level()
            .map_err(|report| format!("invalid SBH level spec:\\n{report}"))?;
        Ok(Self::from_external_level(level, seed))
    }

    fn from_external_level(level: Level, seed: u32) -> Self {
        let restart_level = level.clone();
        let mut world = Self::from_level(0, level, seed);
        world.restart_level = Some(restart_level);
        world
    }

    fn from_level(level_index: usize, level: Level, seed: u32) -> Self {
""",
    "external level constructor",
)

world = replace_once(
    world,
    """        let mut world = Self {
            level_index,
            phase: Phase::Running,
""",
    """        let mut world = Self {
            level_index,
            restart_level: None,
            phase: Phase::Running,
""",
    "from_level restart source initialization",
)

world = replace_once(
    world,
    """    pub(super) fn restart(&mut self) {
        if self.level_index == LEVEL_COUNT {
            *self = Self::iso_slice(self.seed);
        } else {
            *self = Self::for_level(self.level_index, self.seed);
        }
    }
""",
    """    pub(super) fn restart(&mut self) {
        if let Some(level) = self.restart_level.clone() {
            *self = Self::from_external_level(level, self.seed);
        } else if self.level_index == LEVEL_COUNT {
            *self = Self::iso_slice(self.seed);
        } else {
            *self = Self::for_level(self.level_index, self.seed);
        }
    }
""",
    "restart external source preservation",
)

world = replace_once(
    world,
    """    pub(super) fn level_name(&self) -> &'static str {
        self.level.name
    }

    pub(super) fn level_name_at(level_index: usize) -> &'static str {
        build_level(level_index % LEVEL_COUNT).name
    }
""",
    """    pub(super) fn level_name(&self) -> &str {
        &self.level.name
    }

    pub(super) fn level_name_at(level_index: usize) -> String {
        build_level(level_index % LEVEL_COUNT).name
    }
""",
    "owned level name API",
)

world = replace_once(
    world,
    "    name: &'static str,",
    "    name: String,",
    "Level owns its name",
)

for name in [
    "SERIOUSLY?",
    "MATH IS HARD",
    "PAY THE PRICE",
    "ORDER MATTERS",
    "JUST LEAVE",
    "HE'S MOVING",
    "WAIT FOR IT",
    "LET HIM COME",
    "LUCKY BOY?",
    "SMART BOY",
    "THING DID IT",
    "HOLD THE DOOR",
    "TWO SMART WAYS",
    "WATCH YOUR STEP",
    "SET THE TRAP",
    "CLOCKWORK",
    "COME HERE",
    "GROUP THERAPY",
    "SMART WAY",
    "THE CLOCKWORK KEEP",
    "TEST",
]:
    old = f'name: "{name}",'
    new = f'name: "{name}".into(),'
    count = world.count(old)
    if count != 1:
        raise SystemExit(f"level name {name!r}: expected exactly one match, found {count}")
    world = world.replace(old, new, 1)

world = replace_once(
    world,
    """        let mut world = SmartBoyWorld {
            level_index: 0,
            phase: Phase::Running,
""",
    """        let mut world = SmartBoyWorld {
            level_index: 0,
            restart_level: None,
            phase: Phase::Running,
""",
    "test world restart source initialization",
)

anchor = """    #[test]
    fn restart_recreates_initial_state() {
        let mut world = SmartBoyWorld::for_level(1, 42);
        let initial = world.clone();

        world.apply(PlayerAction::Move(Direction::Right));
        world.restart();

        assert_eq!(world, initial);
    }
"""
external_test = anchor + """

    #[test]
    fn restart_reuses_external_level_definition() {
        let mut spec = LevelSpec::parse(include_str!(
            "../../assets/smart_boy_hero/levels/smell_a_rat.json"
        ))
        .expect("fixture should parse");
        spec.name = "EXTERNAL RAT TEST".into();
        let json = spec.to_json().expect("fixture should serialize");
        let mut world = SmartBoyWorld::from_level_json(&json, 42).expect("external level should load");
        let initial = world.clone();

        assert_eq!(world.level_name(), "EXTERNAL RAT TEST");
        world.update_tick();
        assert_ne!(world, initial);

        world.restart();

        assert_eq!(world, initial);
        assert_eq!(world.level_name(), "EXTERNAL RAT TEST");
    }
"""
world = replace_once(world, anchor, external_test, "external restart regression test")

world_path.write_text(world)


game_path = ROOT / "examples/smart_boy_hero/game.rs"
game = game_path.read_text()
constructor_old = """impl SmartBoyHeroGame {
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
"""
constructor_new = """impl SmartBoyHeroGame {
    pub fn new() -> Self {
        Self::new_with_mode(SmartBoyHeroMode::Native)
    }

    pub fn from_level_json(json: &str) -> Result<Self, String> {
        let world = SmartBoyWorld::from_level_json(json, INITIAL_SEED)?;
        Ok(Self::new_with_mode_and_world(SmartBoyHeroMode::Native, world))
    }

    #[allow(dead_code)]
    pub fn new_touch() -> Self {
        Self::new_with_mode(SmartBoyHeroMode::Touch)
    }

    fn new_with_mode(mode: SmartBoyHeroMode) -> Self {
        Self::new_with_mode_and_world(mode, SmartBoyWorld::new(INITIAL_SEED))
    }

    fn new_with_mode_and_world(mode: SmartBoyHeroMode, world: SmartBoyWorld) -> Self {
        Self {
            mode,
            world,
            controls: controls(),
"""
game = replace_once(game, constructor_old, constructor_new, "game external constructor")

game = replace_once(
    game,
    """        SmartBoyWorld::level_name_at(game.selected_level),
        1,
        WIN,
""",
    """        &SmartBoyWorld::level_name_at(game.selected_level),
        1,
        WIN,
""",
    "level select owned name borrow",
)

game_path.write_text(game)


main_path = ROOT / "examples/smart_boy_hero.rs"
main = main_path.read_text()
main = replace_once(
    main,
    """#[path = "smart_boy_hero/game.rs"]
mod game;

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SmartBoyHeroGame};
use gotoo_pixel_engine::{EngineConfig, EngineError, run};
""",
    """#[path = "smart_boy_hero/game.rs"]
mod game;

use std::{fs, io};

use game::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH, SmartBoyHeroGame};
use gotoo_pixel_engine::{EngineConfig, run};
""",
    "native entrypoint imports",
)
main = replace_once(
    main,
    """fn main() -> Result<(), EngineError> {
    let (window_width, window_height) = window_size();
    run(
        EngineConfig {
            title: "Smart Boy Hero - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        SmartBoyHeroGame::new(),
    )
}
""",
    """fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (window_width, window_height) = window_size();
    let game = match std::env::args_os().nth(1) {
        Some(path) => {
            let json = fs::read_to_string(&path)?;
            SmartBoyHeroGame::from_level_json(&json)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        }
        None => SmartBoyHeroGame::new(),
    };

    run(
        EngineConfig {
            title: "Smart Boy Hero - Gotoo Pixel Engine".into(),
            framebuffer_width: FRAMEBUFFER_WIDTH,
            framebuffer_height: FRAMEBUFFER_HEIGHT,
            window_width,
            window_height,
        },
        game,
    )?;
    Ok(())
}
""",
    "native external level argument",
)
main_path.write_text(main)


readme_path = ROOT / "README.md"
readme = readme_path.read_text()
readme = replace_once(
    readme,
    """cargo run --example breakout
cargo run --example smart_boy_hero
cargo run --example arcade
```
""",
    """cargo run --example breakout
cargo run --example smart_boy_hero
cargo run --example arcade
```

Playtester Smart Boy Hero avec un niveau JSON externe (natif uniquement) :

```bash
cargo run --example smart_boy_hero -- assets/smart_boy_hero/levels/smell_a_rat.json
```

Le chemin est optionnel : sans argument, SBH conserve ses niveaux embarqués habituels.
""",
    "README external SBH playtest command",
)
readme_path.write_text(readme)
