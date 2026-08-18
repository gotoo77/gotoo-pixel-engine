use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    Bonus, BonusKind, Boulder, BoulderState, Cell, Direction, Door, Enemy, EnemyIntent, EnemyKind,
    EnemyRole, Food, Lever, LeverKind, Level, LevelTiming, Pit, Trap,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LevelSpec {
    pub(super) name: String,
    pub(super) width: i32,
    pub(super) height: i32,
    #[serde(default)]
    pub(super) timing: LevelTimingSpec,
    pub(super) hero: HeroSpec,
    pub(super) exit: CellSpec,
    #[serde(default)]
    pub(super) walls: Vec<CellSpec>,
    #[serde(default)]
    pub(super) doors: Vec<DoorSpec>,
    #[serde(default)]
    pub(super) levers: Vec<LeverSpec>,
    #[serde(default)]
    pub(super) traps: Vec<TrapSpec>,
    #[serde(default)]
    pub(super) pits: Vec<CellSpec>,
    #[serde(default)]
    pub(super) boulders: Vec<BoulderSpec>,
    #[serde(default)]
    pub(super) foods: Vec<CellSpec>,
    #[serde(default)]
    pub(super) bonuses: Vec<BonusSpec>,
    #[serde(default)]
    pub(super) enemies: Vec<EnemySpec>,
}

impl LevelSpec {
    pub(super) fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|error| format!("invalid SBH level JSON: {error}"))
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(super) fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|error| format!("failed to serialize SBH level JSON: {error}"))
    }

    pub(super) fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();

        if self.name.trim().is_empty() {
            report.error("name must not be empty");
        }
        if self.width <= 0 {
            report.error(format!("width must be positive, got {}", self.width));
        }
        if self.height <= 0 {
            report.error(format!("height must be positive, got {}", self.height));
        }
        if self.hero.power < 0 {
            report.error(format!("hero.power must be >= 0, got {}", self.hero.power));
        }

        self.validate_position("hero", self.hero.x, self.hero.y, &mut report);
        self.validate_position("exit", self.exit.x, self.exit.y, &mut report);
        for (index, wall) in self.walls.iter().enumerate() {
            self.validate_position(&format!("walls[{index}]"), wall.x, wall.y, &mut report);
        }
        for (index, door) in self.doors.iter().enumerate() {
            self.validate_position(&format!("doors[{index}]"), door.x, door.y, &mut report);
        }
        for (index, lever) in self.levers.iter().enumerate() {
            self.validate_position(&format!("levers[{index}]"), lever.x, lever.y, &mut report);
        }
        for (index, trap) in self.traps.iter().enumerate() {
            self.validate_position(&format!("traps[{index}]"), trap.x, trap.y, &mut report);
            if trap.group.is_none() && !trap.initially_active {
                report.warning(format!(
                    "traps[{index}] is inactive and has no group, so no current mechanism can arm it"
                ));
            }
        }
        for (index, pit) in self.pits.iter().enumerate() {
            self.validate_position(&format!("pits[{index}]"), pit.x, pit.y, &mut report);
        }
        for (index, boulder) in self.boulders.iter().enumerate() {
            self.validate_position(
                &format!("boulders[{index}]"),
                boulder.x,
                boulder.y,
                &mut report,
            );
        }
        for (index, food) in self.foods.iter().enumerate() {
            self.validate_position(&format!("foods[{index}]"), food.x, food.y, &mut report);
        }
        for (index, bonus) in self.bonuses.iter().enumerate() {
            self.validate_position(&format!("bonuses[{index}]"), bonus.x, bonus.y, &mut report);
            match bonus.kind {
                BonusKindSpec::Fixed => {
                    if bonus.amount.is_none() {
                        report.error(format!("bonuses[{index}].amount is required for kind=fixed"));
                    }
                    if bonus.min.is_some() || bonus.max.is_some() {
                        report.warning(format!(
                            "bonuses[{index}] is fixed; min/max are ignored"
                        ));
                    }
                }
                BonusKindSpec::Mystery => match (bonus.min, bonus.max) {
                    (Some(min), Some(max)) if min <= max => {}
                    (Some(min), Some(max)) => report.error(format!(
                        "bonuses[{index}] has invalid mystery range: min {min} > max {max}"
                    )),
                    _ => report.error(format!(
                        "bonuses[{index}].min and .max are required for kind=mystery"
                    )),
                },
            }
        }
        for (index, enemy) in self.enemies.iter().enumerate() {
            self.validate_position(&format!("enemies[{index}]"), enemy.x, enemy.y, &mut report);
            if enemy.power < 0 {
                report.error(format!(
                    "enemies[{index}].power must be >= 0, got {}",
                    enemy.power
                ));
            }
            match enemy.kind {
                EnemyKindSpec::Walker if enemy.direction.is_none() => report.error(format!(
                    "enemies[{index}].direction is required for kind=walker"
                )),
                EnemyKindSpec::Walker => {}
                _ if enemy.direction.is_some() => report.warning(format!(
                    "enemies[{index}].direction is ignored for kind={}",
                    enemy.kind.as_str()
                )),
                _ => {}
            }
        }

        self.validate_hard_collisions(&mut report);
        self.validate_groups(&mut report);
        report
    }

    pub(super) fn into_level(self) -> Result<Level, ValidationReport> {
        let report = self.validate();
        if !report.is_valid() {
            return Err(report);
        }

        Ok(Level {
            width: self.width,
            height: self.height,
            timing: self.timing.into(),
            name: self.name,
            hero_start: self.hero.cell(),
            hero_power: self.hero.power,
            exit: self.exit.cell(),
            walls: self.walls.into_iter().map(CellSpec::cell).collect(),
            doors: self
                .doors
                .into_iter()
                .map(|door| Door {
                    cell: Cell::new(door.x, door.y),
                    group: door.group,
                    initially_open: door.initially_open,
                })
                .collect(),
            levers: self
                .levers
                .into_iter()
                .map(|lever| Lever {
                    cell: Cell::new(lever.x, lever.y),
                    group: lever.group,
                    kind: lever.kind.into(),
                })
                .collect(),
            traps: self
                .traps
                .into_iter()
                .map(|trap| Trap {
                    cell: Cell::new(trap.x, trap.y),
                    group: trap.group,
                    initially_active: trap.initially_active,
                })
                .collect(),
            pits: self
                .pits
                .into_iter()
                .map(|pit| Pit { cell: pit.cell() })
                .collect(),
            boulders: self
                .boulders
                .into_iter()
                .map(|boulder| Boulder {
                    cell: Cell::new(boulder.x, boulder.y),
                    group: boulder.group,
                    direction: boulder.direction.into(),
                    state: BoulderState::Ready,
                })
                .collect(),
            foods: self
                .foods
                .into_iter()
                .map(|food| Food { cell: food.cell() })
                .collect(),
            bonuses: self
                .bonuses
                .into_iter()
                .map(BonusSpec::into_bonus)
                .collect(),
            enemies: self
                .enemies
                .into_iter()
                .map(EnemySpec::into_enemy)
                .collect(),
        })
    }

    fn validate_position(&self, path: &str, x: i32, y: i32, report: &mut ValidationReport) {
        if self.width > 0 && self.height > 0 && (x < 0 || y < 0 || x >= self.width || y >= self.height)
        {
            report.error(format!(
                "{path} position ({x}, {y}) is outside {}x{} bounds",
                self.width, self.height
            ));
        }
    }

    fn validate_hard_collisions(&self, report: &mut ValidationReport) {
        let walls: HashSet<(i32, i32)> = self.walls.iter().map(CellSpec::point).collect();
        if walls.len() != self.walls.len() {
            report.warning("walls contains duplicate cells");
        }

        for (path, x, y) in std::iter::once(("hero".to_string(), self.hero.x, self.hero.y))
            .chain(std::iter::once(("exit".to_string(), self.exit.x, self.exit.y)))
            .chain(
                self.doors
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("doors[{index}]"), value.x, value.y)),
            )
            .chain(
                self.levers
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("levers[{index}]"), value.x, value.y)),
            )
            .chain(
                self.traps
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("traps[{index}]"), value.x, value.y)),
            )
            .chain(
                self.pits
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("pits[{index}]"), value.x, value.y)),
            )
            .chain(
                self.boulders
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("boulders[{index}]"), value.x, value.y)),
            )
            .chain(
                self.foods
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("foods[{index}]"), value.x, value.y)),
            )
            .chain(
                self.bonuses
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("bonuses[{index}]"), value.x, value.y)),
            )
            .chain(
                self.enemies
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (format!("enemies[{index}]"), value.x, value.y)),
            )
        {
            if walls.contains(&(x, y)) {
                report.error(format!("{path} overlaps wall at ({x}, {y})"));
            }
        }

        let mut actor_cells = HashSet::new();
        actor_cells.insert((self.hero.x, self.hero.y));
        for (index, enemy) in self.enemies.iter().enumerate() {
            if !actor_cells.insert((enemy.x, enemy.y)) {
                report.error(format!(
                    "enemies[{index}] overlaps another actor at ({}, {})",
                    enemy.x, enemy.y
                ));
            }
        }
    }

    fn validate_groups(&self, report: &mut ValidationReport) {
        let actuator_groups: HashSet<u8> = self.levers.iter().map(|lever| lever.group).collect();
        let mut target_groups: HashSet<u8> = self.doors.iter().map(|door| door.group).collect();
        target_groups.extend(self.traps.iter().filter_map(|trap| trap.group));
        target_groups.extend(self.boulders.iter().map(|boulder| boulder.group));

        for (index, door) in self.doors.iter().enumerate() {
            if !door.initially_open && !actuator_groups.contains(&door.group) {
                report.warning(format!(
                    "doors[{index}] is closed but group {} has no current lever/pressure plate",
                    door.group
                ));
            }
        }
        for (index, trap) in self.traps.iter().enumerate() {
            if let Some(group) = trap.group
                && !actuator_groups.contains(&group)
            {
                report.warning(format!(
                    "traps[{index}] references group {group} but no current lever/pressure plate uses it"
                ));
            }
        }
        for (index, boulder) in self.boulders.iter().enumerate() {
            if !actuator_groups.contains(&boulder.group) {
                report.warning(format!(
                    "boulders[{index}] references group {} but no current lever/pressure plate uses it",
                    boulder.group
                ));
            }
        }
        for (index, lever) in self.levers.iter().enumerate() {
            if !target_groups.contains(&lever.group) {
                report.warning(format!(
                    "levers[{index}] uses group {} but no current door/trap/boulder references it",
                    lever.group
                ));
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct ValidationReport {
    pub(super) errors: Vec<String>,
    pub(super) warnings: Vec<String>,
}

impl ValidationReport {
    pub(super) fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    fn error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    fn warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
}

impl fmt::Display for ValidationReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(formatter, "error: {error}")?;
        }
        for warning in &self.warnings {
            writeln!(formatter, "warning: {warning}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LevelTimingSpec {
    #[default]
    TurnBased,
    SemiContinuous,
}

impl From<LevelTimingSpec> for LevelTiming {
    fn from(value: LevelTimingSpec) -> Self {
        match value {
            LevelTimingSpec::TurnBased => Self::TurnBased,
            LevelTimingSpec::SemiContinuous => Self::SemiContinuous,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CellSpec {
    pub(super) x: i32,
    pub(super) y: i32,
}

impl CellSpec {
    fn cell(self) -> Cell {
        Cell::new(self.x, self.y)
    }

    fn point(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HeroSpec {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) power: i32,
}

impl HeroSpec {
    fn cell(self) -> Cell {
        Cell::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DoorSpec {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) group: u8,
    #[serde(default)]
    pub(super) initially_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LeverSpec {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) group: u8,
    pub(super) kind: LeverKindSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum LeverKindSpec {
    Latch,
    PressurePlate,
}

impl From<LeverKindSpec> for LeverKind {
    fn from(value: LeverKindSpec) -> Self {
        match value {
            LeverKindSpec::Latch => Self::Latch,
            LeverKindSpec::PressurePlate => Self::PressurePlate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrapSpec {
    pub(super) x: i32,
    pub(super) y: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) group: Option<u8>,
    #[serde(default)]
    pub(super) initially_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BoulderSpec {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) group: u8,
    pub(super) direction: DirectionSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BonusSpec {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) kind: BonusKindSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) amount: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) min: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) max: Option<i32>,
}

impl BonusSpec {
    fn into_bonus(self) -> Bonus {
        let kind = match self.kind {
            BonusKindSpec::Fixed => BonusKind::Fixed(self.amount.expect("validated fixed bonus")),
            BonusKindSpec::Mystery => BonusKind::Mystery {
                min: self.min.expect("validated mystery bonus min"),
                max: self.max.expect("validated mystery bonus max"),
            },
        };
        Bonus {
            cell: Cell::new(self.x, self.y),
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum BonusKindSpec {
    Fixed,
    Mystery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EnemySpec {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) power: i32,
    pub(super) kind: EnemyKindSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) direction: Option<DirectionSpec>,
    #[serde(default)]
    pub(super) role: EnemyRoleSpec,
}

impl EnemySpec {
    fn into_enemy(self) -> Enemy {
        let kind = match self.kind {
            EnemyKindSpec::Guard => EnemyKind::Guard,
            EnemyKindSpec::Walker => EnemyKind::Walker {
                direction: self.direction.expect("validated walker direction").into(),
            },
            EnemyKindSpec::Rat => EnemyKind::Rat,
            EnemyKindSpec::Cat => EnemyKind::Cat,
        };
        Enemy {
            cell: Cell::new(self.x, self.y),
            power: self.power,
            kind,
            role: self.role.into(),
            intent: EnemyIntent::Patrol,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum EnemyKindSpec {
    Guard,
    Walker,
    Rat,
    Cat,
}

impl EnemyKindSpec {
    fn as_str(self) -> &'static str {
        match self {
            Self::Guard => "guard",
            Self::Walker => "walker",
            Self::Rat => "rat",
            Self::Cat => "cat",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum EnemyRoleSpec {
    #[default]
    Normal,
    KeyWarden,
}

impl From<EnemyRoleSpec> for EnemyRole {
    fn from(value: EnemyRoleSpec) -> Self {
        match value {
            EnemyRoleSpec::Normal => Self::Normal,
            EnemyRoleSpec::KeyWarden => Self::KeyWarden,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DirectionSpec {
    Up,
    Down,
    Left,
    Right,
}

impl From<DirectionSpec> for Direction {
    fn from(value: DirectionSpec) -> Self {
        match value {
            DirectionSpec::Up => Self::Up,
            DirectionSpec::Down => Self::Down,
            DirectionSpec::Left => Self::Left,
            DirectionSpec::Right => Self::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMELL_A_RAT: &str = include_str!("../../../assets/smart_boy_hero/levels/smell_a_rat.json");

    #[test]
    fn parses_and_serializes_fixture() {
        let spec = LevelSpec::parse(SMELL_A_RAT).expect("fixture should parse");
        let encoded = spec.to_json().expect("fixture should serialize");
        let reparsed = LevelSpec::parse(&encoded).expect("serialized fixture should parse");

        assert_eq!(spec, reparsed);
    }

    #[test]
    fn invalid_json_is_rejected() {
        let error = LevelSpec::parse("{ nope }").expect_err("invalid JSON should fail");

        assert!(error.contains("invalid SBH level JSON"));
    }

    #[test]
    fn out_of_bounds_position_is_an_error() {
        let mut spec = LevelSpec::parse(SMELL_A_RAT).expect("fixture should parse");
        spec.hero.x = spec.width;

        let report = spec.validate();

        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|error| error.contains("hero position")));
    }

    #[test]
    fn missing_walker_direction_is_an_error() {
        let mut spec = LevelSpec::parse(SMELL_A_RAT).expect("fixture should parse");
        spec.enemies[0].kind = EnemyKindSpec::Walker;
        spec.enemies[0].direction = None;

        let report = spec.validate();

        assert!(!report.is_valid());
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("direction is required"))
        );
    }

    #[test]
    fn closed_door_without_actuator_is_a_warning() {
        let mut spec = LevelSpec::parse(SMELL_A_RAT).expect("fixture should parse");
        spec.levers.clear();

        let report = spec.validate();

        assert!(report.is_valid());
        assert!(
            report
                .warnings
                .iter()
                .any(|warning| warning.contains("doors[0] is closed"))
        );
    }

    #[test]
    fn fixture_converts_to_runtime_level() {
        let spec = LevelSpec::parse(SMELL_A_RAT).expect("fixture should parse");
        let level = spec.into_level().expect("fixture should validate");

        assert_eq!(level.name, "SMELL A RAT");
        assert_eq!(level.width, 12);
        assert_eq!(level.height, 8);
        assert_eq!(level.hero_start, Cell::new(1, 4));
        assert_eq!(level.hero_power, 6);
        assert_eq!(level.exit, Cell::new(10, 4));
        assert_eq!(level.doors.len(), 1);
        assert_eq!(level.levers.len(), 1);
        assert_eq!(level.pits, vec![Pit { cell: Cell::new(5, 6) }]);
        assert_eq!(level.foods, vec![Food { cell: Cell::new(5, 2) }]);
        assert_eq!(level.enemies.len(), 3);
        assert_eq!(level.enemies[0].kind, EnemyKind::Rat);
        assert_eq!(level.enemies[1].kind, EnemyKind::Rat);
        assert_eq!(level.enemies[2].kind, EnemyKind::Cat);
    }
}
