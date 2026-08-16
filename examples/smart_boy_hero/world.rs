pub(super) const GRID_WIDTH: i32 = 12;
pub(super) const GRID_HEIGHT: i32 = 8;
pub(super) const LEVEL_COUNT: usize = 19;
const SHOUT_RADIUS: i32 = 5;
const BOULDER_STEPS_PER_TICK: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SmartBoyWorld {
    level_index: usize,
    level: Level,
    phase: Phase,
    hero: Cell,
    hero_power: i32,
    enemies: Vec<Enemy>,
    bonuses: Vec<Bonus>,
    latched_doors_open: Vec<bool>,
    doors_open: Vec<bool>,
    latched_traps_active: Vec<bool>,
    traps_active: Vec<bool>,
    pressure_plates_active: Vec<bool>,
    boulders: Vec<Boulder>,
    turn_count: u32,
    seed: u32,
    rng: MysteryRng,
}

impl SmartBoyWorld {
    pub(super) fn new(seed: u32) -> Self {
        Self::for_level(0, seed)
    }

    pub(super) fn for_level(level_index: usize, seed: u32) -> Self {
        let level_index = level_index % LEVEL_COUNT;
        let level = build_level(level_index);
        Self::from_level(level_index, level, seed)
    }

    #[allow(dead_code)]
    pub(super) fn iso_slice(seed: u32) -> Self {
        Self::from_level(LEVEL_COUNT, level_iso_slice(), seed)
    }

    fn from_level(level_index: usize, level: Level, seed: u32) -> Self {
        let mut world = Self {
            level_index,
            phase: Phase::Running,
            hero: level.hero_start,
            hero_power: level.hero_power,
            enemies: level.enemies.clone(),
            bonuses: level.bonuses.clone(),
            latched_doors_open: vec![false; level.doors.len()],
            doors_open: vec![false; level.doors.len()],
            latched_traps_active: vec![false; level.traps.len()],
            traps_active: vec![false; level.traps.len()],
            pressure_plates_active: vec![false; level.levers.len()],
            boulders: level.boulders.clone(),
            turn_count: 0,
            level,
            seed,
            rng: MysteryRng::new(seed ^ (level_index as u32).wrapping_mul(0x9E37_79B9)),
        };
        world.open_initial_doors();
        world.open_initial_traps();
        world.refresh_triggered_systems(&mut TurnReport::default());
        world
    }

    pub(super) fn restart(&mut self) {
        if self.level_index == LEVEL_COUNT {
            *self = Self::iso_slice(self.seed);
        } else {
            *self = Self::for_level(self.level_index, self.seed);
        }
    }

    pub(super) fn next_level(&mut self) {
        *self = Self::for_level((self.level_index + 1) % LEVEL_COUNT, self.seed);
    }

    pub(super) fn load_level(&mut self, level_index: usize) {
        *self = Self::for_level(level_index, self.seed);
    }

    pub(super) fn apply(&mut self, action: PlayerAction) -> TurnReport {
        if self.phase != Phase::Running {
            return TurnReport::default();
        }

        let mut report = TurnReport::default();
        let consumed = match action {
            PlayerAction::Wait => {
                report.events.push(WorldEvent::Waited);
                true
            }
            PlayerAction::Shout => {
                self.shout(&mut report);
                true
            }
            PlayerAction::Move(direction) => self.try_move_hero(direction, &mut report),
        };

        self.refresh_triggered_systems(&mut report);
        report.turn_consumed = consumed;
        if !consumed {
            return report;
        }

        if self.semi_continuous() {
            return report;
        }

        self.turn_count += 1;

        if self.phase == Phase::Running {
            self.run_world_turn(&mut report);
            self.refresh_triggered_systems(&mut report);
        }

        report
    }

    pub(super) fn update_tick(&mut self) -> TurnReport {
        if self.phase != Phase::Running {
            return TurnReport::default();
        }

        let mut report = TurnReport {
            turn_consumed: true,
            events: Vec::new(),
        };
        self.refresh_triggered_systems(&mut report);
        self.turn_count += 1;

        if self.phase == Phase::Running {
            self.run_world_turn(&mut report);
            self.refresh_triggered_systems(&mut report);
        }

        report
    }

    pub(super) fn level_index(&self) -> usize {
        self.level_index
    }

    pub(super) fn level_name(&self) -> &'static str {
        self.level.name
    }

    pub(super) fn level_name_at(level_index: usize) -> &'static str {
        build_level(level_index % LEVEL_COUNT).name
    }

    pub(super) fn phase(&self) -> Phase {
        self.phase
    }

    pub(super) fn hero(&self) -> Cell {
        self.hero
    }

    pub(super) fn hero_power(&self) -> i32 {
        self.hero_power
    }

    pub(super) fn turn_count(&self) -> u32 {
        self.turn_count
    }

    pub(super) fn semi_continuous(&self) -> bool {
        matches!(self.level.timing, LevelTiming::SemiContinuous)
    }

    pub(super) fn walls(&self) -> &[Cell] {
        &self.level.walls
    }

    pub(super) fn doors(&self) -> &[Door] {
        &self.level.doors
    }

    pub(super) fn levers(&self) -> &[Lever] {
        &self.level.levers
    }

    pub(super) fn exit(&self) -> Cell {
        self.level.exit
    }

    pub(super) fn enemies(&self) -> &[Enemy] {
        &self.enemies
    }

    pub(super) fn bonuses(&self) -> &[Bonus] {
        &self.bonuses
    }

    pub(super) fn door_open(&self, index: usize) -> bool {
        self.doors_open.get(index).copied().unwrap_or(false)
    }

    pub(super) fn traps(&self) -> &[Trap] {
        &self.level.traps
    }

    pub(super) fn trap_active(&self, index: usize) -> bool {
        self.traps_active.get(index).copied().unwrap_or(false)
    }

    #[allow(dead_code)]
    pub(super) fn boulders(&self) -> &[Boulder] {
        &self.boulders
    }

    fn try_move_hero(&mut self, direction: Direction, report: &mut TurnReport) -> bool {
        let target = self.hero.step(direction);
        if !target.is_inside() || self.wall_at(target) || self.closed_door_at(target).is_some() {
            report.events.push(WorldEvent::Blocked);
            return false;
        }

        if let Some(enemy_index) = self.enemy_at(target) {
            self.resolve_hero_attack(enemy_index, target, report);
            return true;
        }

        self.hero = target;
        self.resolve_hero_entered_cell(target, report);
        true
    }

    fn resolve_hero_attack(&mut self, enemy_index: usize, target: Cell, report: &mut TurnReport) {
        let power = self.enemies[enemy_index].power;
        if self.hero_power > power {
            self.hero_power -= power;
            self.enemies.remove(enemy_index);
            self.hero = target;
            report.events.push(WorldEvent::CombatWon { power });
            report.events.push(WorldEvent::EnemyKilled {
                cell: target,
                power,
            });
            self.resolve_hero_entered_cell(target, report);
        } else {
            self.phase = Phase::Dead;
            report.events.push(WorldEvent::HeroDied);
        }
    }

    fn resolve_hero_entered_cell(&mut self, cell: Cell, report: &mut TurnReport) {
        if self.active_trap_at(cell).is_some() {
            self.phase = Phase::Dead;
            report.events.push(WorldEvent::TrapTriggered);
            report.events.push(WorldEvent::HeroDied);
            return;
        }

        self.collect_at(cell, report);
        self.check_exit(report);
    }

    fn run_world_turn(&mut self, report: &mut TurnReport) {
        self.run_boulder_turn(report);
        if self.phase != Phase::Running {
            return;
        }

        let mut index = 0;
        let mut trap_kills = 0;
        while index < self.enemies.len() {
            let EnemyKind::Walker { direction } = self.enemies[index].kind else {
                index += 1;
                continue;
            };

            let Some(direction) = self.walker_direction_for_turn(index, direction, report) else {
                index += 1;
                continue;
            };

            let target = self.enemies[index].cell.step(direction);
            if target == self.hero {
                let power = self.enemies[index].power;
                if self.hero_power > power {
                    self.hero_power -= power;
                    let cell = self.enemies[index].cell;
                    self.enemies.remove(index);
                    report.events.push(WorldEvent::WalkerDestroyed { power });
                    report.events.push(WorldEvent::EnemyKilled { cell, power });
                    continue;
                }

                self.phase = Phase::Dead;
                report.events.push(WorldEvent::HeroDied);
                return;
            }

            if !target.is_inside()
                || self.wall_at(target)
                || self.closed_door_at(target).is_some()
                || self.enemy_at_except(target, index).is_some()
            {
                self.enemies[index].kind = EnemyKind::Walker {
                    direction: direction.opposite(),
                };
                report.events.push(WorldEvent::WalkerTurned);
            } else {
                if self.active_trap_at(target).is_some() {
                    let power = self.enemies[index].power;
                    self.enemies[index].cell = target;
                    self.enemies.remove(index);
                    report.events.push(WorldEvent::TrapTriggered);
                    report.events.push(WorldEvent::EnemyKilled {
                        cell: target,
                        power,
                    });
                    trap_kills += 1;
                    continue;
                }
                self.enemies[index].cell = target;
                self.resolve_investigation_arrival(index, report);
                report.events.push(WorldEvent::WalkerMoved);
            }

            index += 1;
        }

        if trap_kills >= 2 {
            report
                .events
                .push(WorldEvent::SmartChain { count: trap_kills });
        }
    }

    fn shout(&mut self, report: &mut TurnReport) {
        let target = self.hero;
        let mut heard = 0;
        for enemy in &mut self.enemies {
            let EnemyKind::Walker { direction } = enemy.kind else {
                continue;
            };
            if enemy.cell.manhattan_distance(target) > SHOUT_RADIUS {
                continue;
            }
            enemy.intent = EnemyIntent::Investigate {
                target,
                patrol_direction: direction,
            };
            heard += 1;
        }
        report.events.push(WorldEvent::Shouted {
            cell: target,
            heard,
        });
    }

    fn walker_direction_for_turn(
        &mut self,
        index: usize,
        patrol_direction: Direction,
        report: &mut TurnReport,
    ) -> Option<Direction> {
        if self.walker_detects_hero(index) {
            let patrol_direction = self.enemies[index]
                .intent
                .patrol_direction()
                .unwrap_or(patrol_direction);
            let spotted = !matches!(self.enemies[index].intent, EnemyIntent::ChaseHero { .. });
            self.enemies[index].intent = EnemyIntent::ChaseHero { patrol_direction };
            if spotted {
                report.events.push(WorldEvent::WalkerSpottedHero);
            }
            return self.step_to_target_or_resume(index, self.hero, patrol_direction, report);
        }

        match self.enemies[index].intent {
            EnemyIntent::Patrol => Some(patrol_direction),
            EnemyIntent::Investigate {
                target,
                patrol_direction,
            } => self.step_to_target_or_resume(index, target, patrol_direction, report),
            EnemyIntent::ChaseHero { patrol_direction } => {
                self.resume_patrol(index, patrol_direction, report);
                None
            }
        }
    }

    fn resolve_investigation_arrival(&mut self, index: usize, report: &mut TurnReport) {
        match self.enemies[index].intent {
            EnemyIntent::Investigate {
                target,
                patrol_direction,
            } if self.enemies[index].cell == target => {
                self.resume_patrol(index, patrol_direction, report);
            }
            EnemyIntent::ChaseHero { patrol_direction } if !self.walker_detects_hero(index) => {
                self.resume_patrol(index, patrol_direction, report);
            }
            _ => {}
        }
    }

    fn step_to_target_or_resume(
        &mut self,
        index: usize,
        target: Cell,
        patrol_direction: Direction,
        report: &mut TurnReport,
    ) -> Option<Direction> {
        match self.step_toward(index, target) {
            PathStep::Step(direction) => {
                self.enemies[index].kind = EnemyKind::Walker { direction };
                Some(direction)
            }
            PathStep::Arrived => {
                self.resume_patrol(index, patrol_direction, report);
                None
            }
            PathStep::Blocked => {
                self.resume_patrol(index, patrol_direction, report);
                report.events.push(WorldEvent::WalkerLostTarget);
                None
            }
        }
    }

    fn walker_detects_hero(&self, index: usize) -> bool {
        self.semi_continuous() && self.enemies[index].cell.manhattan_distance(self.hero) == 1
    }

    fn resume_patrol(
        &mut self,
        index: usize,
        patrol_direction: Direction,
        report: &mut TurnReport,
    ) {
        self.enemies[index].intent = EnemyIntent::Patrol;
        self.enemies[index].kind = EnemyKind::Walker {
            direction: patrol_direction,
        };
        report.events.push(WorldEvent::WalkerResumedPatrol);
    }

    fn step_toward(&self, enemy_index: usize, target: Cell) -> PathStep {
        let start = self.enemies[enemy_index].cell;
        if start == target {
            return PathStep::Arrived;
        }

        let mut queue = std::collections::VecDeque::from([(start, None)]);
        let mut visited = vec![start];
        while let Some((cell, first_step)) = queue.pop_front() {
            for direction in directions_toward(cell, target) {
                let next = cell.step(direction);
                if !next.is_inside()
                    || visited.contains(&next)
                    || self.wall_at(next)
                    || self.closed_door_at(next).is_some()
                    || (next != target && self.enemy_at_except(next, enemy_index).is_some())
                {
                    continue;
                }

                let first_step = first_step.unwrap_or(direction);
                if next == target {
                    return PathStep::Step(first_step);
                }
                visited.push(next);
                queue.push_back((next, Some(first_step)));
            }
        }

        PathStep::Blocked
    }

    fn collect_at(&mut self, cell: Cell, report: &mut TurnReport) {
        if let Some(index) = self.bonuses.iter().position(|bonus| bonus.cell == cell) {
            let bonus = self.bonuses.remove(index);
            let amount = match bonus.kind {
                BonusKind::Fixed(amount) => amount,
                BonusKind::Mystery { min, max } => self.rng.next_range(min, max),
            };
            self.hero_power += amount;
            report.events.push(WorldEvent::BonusCollected {
                amount,
                mystery: matches!(bonus.kind, BonusKind::Mystery { .. }),
            });
        }

        let mut opened = false;
        for lever in &self.level.levers {
            if lever.cell != cell {
                continue;
            }
            if !matches!(lever.kind, LeverKind::Latch) {
                continue;
            }
            for (index, door) in self.level.doors.iter().enumerate() {
                if door.group == lever.group && !self.latched_doors_open[index] {
                    self.latched_doors_open[index] = true;
                    opened = true;
                }
            }
            for (index, trap) in self.level.traps.iter().enumerate() {
                if trap.group == Some(lever.group) && !self.latched_traps_active[index] {
                    self.latched_traps_active[index] = true;
                    opened = true;
                }
            }
        }
        if opened {
            self.refresh_triggered_systems(report);
        }
    }

    fn check_exit(&mut self, report: &mut TurnReport) {
        if self.hero == self.level.exit {
            self.phase = Phase::Won;
            report.events.push(WorldEvent::Won);
        }
    }

    fn open_initial_doors(&mut self) {
        for (index, door) in self.level.doors.iter().enumerate() {
            self.latched_doors_open[index] = door.initially_open;
        }
    }

    fn open_initial_traps(&mut self) {
        for (index, trap) in self.level.traps.iter().enumerate() {
            self.latched_traps_active[index] = trap.initially_active;
        }
    }

    fn refresh_triggered_systems(&mut self, report: &mut TurnReport) {
        let mut pressure_groups = Vec::new();
        for index in 0..self.level.levers.len() {
            let lever = self.level.levers[index];
            let active =
                matches!(lever.kind, LeverKind::PressurePlate) && self.actor_at(lever.cell);
            let was_active = self.pressure_plates_active[index];
            self.pressure_plates_active[index] = active;

            if active {
                pressure_groups.push(lever.group);
            }
            match (was_active, active) {
                (false, true) => report.events.push(WorldEvent::PressurePlateOn),
                (true, false) => report.events.push(WorldEvent::PressurePlateOff),
                _ => {}
            }
        }

        for (index, trap) in self.level.traps.iter().enumerate() {
            let was_active = self.traps_active[index];
            self.traps_active[index] = self.latched_traps_active[index]
                || trap
                    .group
                    .is_some_and(|group| pressure_groups.contains(&group));
            match (was_active, self.traps_active[index]) {
                (false, true) => report.events.push(WorldEvent::TrapArmed),
                (true, false) => report.events.push(WorldEvent::TrapDisarmed),
                _ => {}
            }
        }

        for (index, door) in self.level.doors.iter().enumerate() {
            let was_open = self.doors_open[index];
            self.doors_open[index] =
                self.latched_doors_open[index] || pressure_groups.contains(&door.group);
            match (was_open, self.doors_open[index]) {
                (false, true) => report.events.push(WorldEvent::DoorOpened),
                (true, false) => report.events.push(WorldEvent::DoorClosed),
                _ => {}
            }
        }

        for boulder in &mut self.boulders {
            if boulder.state != BoulderState::Ready {
                continue;
            }
            if pressure_groups.contains(&boulder.group) {
                boulder.state = BoulderState::Rolling { kills: 0 };
                report.events.push(WorldEvent::BoulderReleased {
                    cell: boulder.cell,
                    direction: boulder.direction,
                });
            }
        }
    }

    fn wall_at(&self, cell: Cell) -> bool {
        self.level.walls.contains(&cell)
    }

    fn closed_door_at(&self, cell: Cell) -> Option<usize> {
        self.level
            .doors
            .iter()
            .enumerate()
            .find(|(index, door)| door.cell == cell && !self.doors_open[*index])
            .map(|(index, _)| index)
    }

    fn enemy_at(&self, cell: Cell) -> Option<usize> {
        self.enemies.iter().position(|enemy| enemy.cell == cell)
    }

    fn active_trap_at(&self, cell: Cell) -> Option<usize> {
        self.level
            .traps
            .iter()
            .enumerate()
            .find(|(index, trap)| trap.cell == cell && self.traps_active[*index])
            .map(|(index, _)| index)
    }

    fn actor_at(&self, cell: Cell) -> bool {
        self.hero == cell || self.enemy_at(cell).is_some()
    }

    fn enemy_at_except(&self, cell: Cell, except: usize) -> Option<usize> {
        self.enemies
            .iter()
            .enumerate()
            .position(|(index, enemy)| index != except && enemy.cell == cell)
    }

    fn run_boulder_turn(&mut self, report: &mut TurnReport) {
        let mut index = 0;
        while index < self.boulders.len() {
            let BoulderState::Rolling { mut kills } = self.boulders[index].state else {
                index += 1;
                continue;
            };

            for _ in 0..BOULDER_STEPS_PER_TICK {
                let target = self.boulders[index]
                    .cell
                    .step(self.boulders[index].direction);

                if !target.is_inside()
                    || self.wall_at(target)
                    || self.closed_door_at(target).is_some()
                    || self.boulder_blocks_at(target, index)
                {
                    self.boulders[index].state = BoulderState::Stopped;
                    report.events.push(WorldEvent::BoulderStopped {
                        cell: self.boulders[index].cell,
                    });
                    break;
                }

                self.boulders[index].cell = target;
                report
                    .events
                    .push(WorldEvent::BoulderMoved { cell: target });

                if target == self.hero {
                    self.phase = Phase::Dead;
                    self.boulders[index].state = BoulderState::Stopped;
                    report.events.push(WorldEvent::HeroDied);
                    report
                        .events
                        .push(WorldEvent::BoulderStopped { cell: target });
                    return;
                }

                let crushed = self.remove_enemies_at(target);
                if !crushed.is_empty() {
                    kills += crushed.len();
                    self.boulders[index].state = BoulderState::Rolling { kills };
                    for power in crushed {
                        report.events.push(WorldEvent::EnemyKilled {
                            cell: target,
                            power,
                        });
                        report.events.push(WorldEvent::BoulderCrushedEnemy {
                            cell: target,
                            power,
                            chain: kills,
                        });
                    }
                    if kills >= 2 {
                        report
                            .events
                            .push(WorldEvent::BoulderSmartChain { count: kills });
                    }
                }
            }

            if matches!(self.boulders[index].state, BoulderState::Rolling { .. }) {
                self.boulders[index].state = BoulderState::Rolling { kills };
            }

            index += 1;
        }
    }

    fn remove_enemies_at(&mut self, cell: Cell) -> Vec<i32> {
        let mut powers = Vec::new();
        let mut index = 0;
        while index < self.enemies.len() {
            if self.enemies[index].cell == cell {
                powers.push(self.enemies[index].power);
                self.enemies.remove(index);
            } else {
                index += 1;
            }
        }
        powers
    }

    fn boulder_blocks_at(&self, cell: Cell, except: usize) -> bool {
        self.boulders
            .iter()
            .enumerate()
            .any(|(index, boulder)| index != except && boulder.cell == cell)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct TurnReport {
    pub(super) turn_consumed: bool,
    pub(super) events: Vec<WorldEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorldEvent {
    Blocked,
    Waited,
    CombatWon {
        power: i32,
    },
    WalkerDestroyed {
        power: i32,
    },
    EnemyKilled {
        cell: Cell,
        power: i32,
    },
    HeroDied,
    BonusCollected {
        amount: i32,
        mystery: bool,
    },
    PressurePlateOn,
    PressurePlateOff,
    DoorOpened,
    DoorClosed,
    TrapArmed,
    TrapDisarmed,
    TrapTriggered,
    BoulderReleased {
        cell: Cell,
        direction: Direction,
    },
    BoulderMoved {
        cell: Cell,
    },
    BoulderCrushedEnemy {
        cell: Cell,
        power: i32,
        chain: usize,
    },
    BoulderStopped {
        cell: Cell,
    },
    BoulderSmartChain {
        count: usize,
    },
    Shouted {
        cell: Cell,
        heard: usize,
    },
    WalkerLostTarget,
    WalkerMoved,
    WalkerResumedPatrol,
    WalkerSpottedHero,
    WalkerTurned,
    SmartChain {
        count: usize,
    },
    Won,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlayerAction {
    Move(Direction),
    Wait,
    Shout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Phase {
    Running,
    Dead,
    Won,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub(super) fn offset(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    pub(super) fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Cell {
    pub(super) x: i32,
    pub(super) y: i32,
}

impl Cell {
    pub(super) const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn step(self, direction: Direction) -> Self {
        let (dx, dy) = direction.offset();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn is_inside(self) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < GRID_WIDTH && self.y < GRID_HEIGHT
    }

    fn manhattan_distance(self, other: Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Enemy {
    pub(super) cell: Cell,
    pub(super) power: i32,
    pub(super) kind: EnemyKind,
    pub(super) intent: EnemyIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnemyKind {
    Guard,
    Walker { direction: Direction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnemyIntent {
    Patrol,
    Investigate {
        target: Cell,
        patrol_direction: Direction,
    },
    ChaseHero {
        patrol_direction: Direction,
    },
}

impl EnemyIntent {
    fn patrol_direction(self) -> Option<Direction> {
        match self {
            Self::Patrol => None,
            Self::Investigate {
                patrol_direction, ..
            }
            | Self::ChaseHero { patrol_direction } => Some(patrol_direction),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathStep {
    Arrived,
    Step(Direction),
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Bonus {
    pub(super) cell: Cell,
    pub(super) kind: BonusKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BonusKind {
    Fixed(i32),
    Mystery { min: i32, max: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Door {
    pub(super) cell: Cell,
    group: u8,
    initially_open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Lever {
    pub(super) cell: Cell,
    group: u8,
    pub(super) kind: LeverKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LeverKind {
    Latch,
    PressurePlate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Trap {
    pub(super) cell: Cell,
    group: Option<u8>,
    initially_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Boulder {
    pub(super) cell: Cell,
    group: u8,
    pub(super) direction: Direction,
    pub(super) state: BoulderState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoulderState {
    Ready,
    Rolling { kills: usize },
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Level {
    timing: LevelTiming,
    name: &'static str,
    hero_start: Cell,
    hero_power: i32,
    exit: Cell,
    walls: Vec<Cell>,
    doors: Vec<Door>,
    levers: Vec<Lever>,
    traps: Vec<Trap>,
    boulders: Vec<Boulder>,
    bonuses: Vec<Bonus>,
    enemies: Vec<Enemy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelTiming {
    TurnBased,
    SemiContinuous,
}

fn build_level(index: usize) -> Level {
    match index {
        0 => level_seriously(),
        1 => level_math_is_hard(),
        2 => level_pay_the_price(),
        3 => level_order_matters(),
        4 => level_just_leave(),
        5 => level_hes_moving(),
        6 => level_wait_for_it(),
        7 => level_let_him_come(),
        8 => level_lucky_boy(),
        9 => level_smart_boy(),
        10 => level_living_plate_a(),
        11 => level_living_plate_b(),
        12 => level_living_plate_c(),
        13 => level_watch_your_step(),
        14 => level_set_the_trap(),
        15 => level_clockwork(),
        16 => level_come_here(),
        17 => level_group_therapy(),
        18 => level_smart_way(),
        _ => unreachable!("level index is wrapped by LEVEL_COUNT"),
    }
}

fn level_seriously() -> Level {
    let mut walls = vertical_wall(3, &[2, 4]);
    walls.extend(cells(&[(7, 0), (7, 1), (7, 6), (7, 7)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "SERIOUSLY?",
        hero_start: Cell::new(1, 3),
        hero_power: 10,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![guard(3, 4, 3), guard(3, 2, 15)],
    }
}

fn level_math_is_hard() -> Level {
    let walls = vertical_wall(4, &[3]);

    Level {
        timing: LevelTiming::TurnBased,
        name: "MATH IS HARD",
        hero_start: Cell::new(1, 3),
        hero_power: 5,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![fixed_bonus(2, 1, 6)],
        enemies: vec![guard(4, 3, 10)],
    }
}

fn level_pay_the_price() -> Level {
    let mut walls = vertical_wall(3, &[3]);
    walls.extend(vertical_wall(7, &[3]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "PAY THE PRICE",
        hero_start: Cell::new(1, 3),
        hero_power: 10,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![guard(3, 3, 4), guard(7, 3, 5)],
    }
}

fn level_order_matters() -> Level {
    let mut walls = vertical_wall(6, &[4]);
    walls.extend(cells(&[(2, 2), (3, 1), (4, 2), (2, 3), (4, 3)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "ORDER MATTERS",
        hero_start: Cell::new(1, 4),
        hero_power: 6,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![fixed_bonus(3, 2, 5)],
        enemies: vec![guard(3, 3, 2), guard(6, 4, 8)],
    }
}

fn level_just_leave() -> Level {
    let mut walls = vertical_wall(5, &[1, 4]);
    walls.extend(cells(&[(5, 2), (5, 3), (5, 5), (9, 1), (9, 2)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "JUST LEAVE",
        hero_start: Cell::new(1, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![guard(5, 4, 99)],
    }
}

fn level_hes_moving() -> Level {
    let mut walls = horizontal_wall(3, &[5]);
    walls.extend(horizontal_wall(5, &[5]));
    walls.extend(cells(&[(5, 1), (5, 6)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "HE'S MOVING",
        hero_start: Cell::new(1, 4),
        hero_power: 8,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(5, 4, 9, Direction::Up)],
    }
}

fn level_wait_for_it() -> Level {
    let mut walls = horizontal_wall(3, &[5]);
    walls.extend(horizontal_wall(5, &[5]));
    walls.extend(cells(&[(5, 1), (5, 6)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "WAIT FOR IT",
        hero_start: Cell::new(3, 4),
        hero_power: 7,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(5, 3, 9, Direction::Down)],
    }
}

fn level_let_him_come() -> Level {
    let mut walls = horizontal_wall(3, &[4, 5, 6]);
    walls.extend(horizontal_wall(5, &[4, 5, 6]));
    walls.extend(vertical_wall(7, &[4]));
    walls.extend(cells(&[(5, 1), (5, 6)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "LET HIM COME",
        hero_start: Cell::new(3, 4),
        hero_power: 12,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![fixed_bonus(6, 4, 4)],
        enemies: vec![walker(5, 4, 4, Direction::Left), guard(7, 4, 9)],
    }
}

fn level_lucky_boy() -> Level {
    let mut walls = vertical_wall(4, &[2, 5]);
    walls.extend(vertical_wall(8, &[2, 5]));
    walls.extend(horizontal_wall(3, &[0, 1, 2, 9, 10, 11]));
    walls.extend(horizontal_wall(4, &[0, 1, 2, 9, 10, 11]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "LUCKY BOY?",
        hero_start: Cell::new(1, 4),
        hero_power: 6,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![fixed_bonus(5, 2, 3), mystery_bonus(5, 5, 2, 8)],
        enemies: vec![guard(8, 2, 8), guard(8, 5, 8)],
    }
}

fn level_smart_boy() -> Level {
    let mut walls = vertical_wall(4, &[5]);
    walls.extend(vertical_wall(6, &[3]));
    walls.extend(horizontal_wall(1, &[1, 2, 3, 5, 7, 8, 9, 10]));
    walls.extend(horizontal_wall(6, &[1, 2, 3, 4, 5]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "SMART BOY",
        hero_start: Cell::new(1, 5),
        hero_power: 8,
        exit: Cell::new(10, 2),
        walls,
        doors: vec![Door {
            cell: Cell::new(6, 3),
            group: 1,
            initially_open: false,
        }],
        levers: vec![lever(2, 2, 1)],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![
            fixed_bonus(4, 5, 4),
            fixed_bonus(5, 2, 5),
            mystery_bonus(9, 5, 2, 6),
        ],
        enemies: vec![
            guard(3, 5, 6),
            walker(5, 3, 4, Direction::Up),
            guard(8, 2, 6),
            guard(10, 5, 99),
        ],
    }
}

fn level_living_plate_a() -> Level {
    let mut walls = vertical_wall(5, &[4]);
    walls.push(Cell::new(5, 2));

    Level {
        timing: LevelTiming::TurnBased,
        name: "THING DID IT",
        hero_start: Cell::new(3, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![Door {
            cell: Cell::new(5, 4),
            group: 1,
            initially_open: false,
        }],
        levers: vec![pressure_plate(4, 2, 1)],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(3, 2, 99, Direction::Right)],
    }
}

fn level_living_plate_b() -> Level {
    let mut walls = vertical_wall(6, &[4]);
    walls.push(Cell::new(6, 2));

    Level {
        timing: LevelTiming::TurnBased,
        name: "HOLD THE DOOR",
        hero_start: Cell::new(4, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![Door {
            cell: Cell::new(6, 4),
            group: 1,
            initially_open: false,
        }],
        levers: vec![pressure_plate(5, 2, 1)],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(3, 2, 99, Direction::Right)],
    }
}

fn level_living_plate_c() -> Level {
    let mut walls = vertical_wall(7, &[4]);
    walls.extend(cells(&[(3, 2), (6, 2)]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "TWO SMART WAYS",
        hero_start: Cell::new(5, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![Door {
            cell: Cell::new(7, 4),
            group: 1,
            initially_open: false,
        }],
        levers: vec![lever(2, 2, 1), pressure_plate(5, 2, 1)],
        traps: vec![],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(4, 2, 99, Direction::Right)],
    }
}

fn level_watch_your_step() -> Level {
    Level {
        timing: LevelTiming::TurnBased,
        name: "WATCH YOUR STEP",
        hero_start: Cell::new(1, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls: vec![],
        doors: vec![],
        levers: vec![],
        traps: vec![active_trap(3, 4)],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![],
    }
}

fn level_set_the_trap() -> Level {
    let mut walls = horizontal_wall(3, &[1]);
    walls.extend(horizontal_wall(5, &[]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "SET THE TRAP",
        hero_start: Cell::new(1, 4),
        hero_power: 5,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(1, 3, 1)],
        traps: vec![group_trap(6, 4, 1)],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(9, 4, 9, Direction::Left)],
    }
}

fn level_clockwork() -> Level {
    let mut walls = horizontal_wall(3, &[]);
    walls.extend(horizontal_wall(5, &[]));

    Level {
        timing: LevelTiming::TurnBased,
        name: "CLOCKWORK",
        hero_start: Cell::new(1, 4),
        hero_power: 12,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(2, 4, 1)],
        traps: vec![group_trap(6, 4, 1)],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(8, 4, 9, Direction::Left)],
    }
}

fn level_come_here() -> Level {
    let mut walls = horizontal_wall(3, &[5, 6, 7]);
    walls.extend(horizontal_wall(5, &[5, 6, 7]));

    Level {
        timing: LevelTiming::SemiContinuous,
        name: "COME HERE",
        hero_start: Cell::new(3, 4),
        hero_power: 5,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![active_trap(6, 4)],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![walker(9, 4, 9, Direction::Up)],
    }
}

fn level_group_therapy() -> Level {
    let mut walls = horizontal_wall(2, &[5]);
    walls.extend(horizontal_wall(4, &[]));

    Level {
        timing: LevelTiming::SemiContinuous,
        name: "GROUP THERAPY",
        hero_start: Cell::new(5, 3),
        hero_power: 5,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(5, 2, 1)],
        traps: vec![group_trap(7, 3, 1), group_trap(8, 3, 1)],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![
            walker(8, 3, 9, Direction::Up),
            walker(9, 3, 9, Direction::Down),
        ],
    }
}

fn level_smart_way() -> Level {
    let walls = vertical_wall(7, &[2, 4]);

    Level {
        timing: LevelTiming::SemiContinuous,
        name: "SMART WAY",
        hero_start: Cell::new(4, 3),
        hero_power: 20,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(5, 3, 1)],
        traps: vec![group_trap(6, 2, 1), group_trap(6, 4, 1)],
        boulders: vec![],
        bonuses: vec![],
        enemies: vec![
            walker(7, 2, 7, Direction::Up),
            walker(7, 4, 7, Direction::Down),
        ],
    }
}

#[allow(dead_code)]
fn level_iso_slice() -> Level {
    let mut walls = horizontal_wall(1, &[2, 3, 4, 5, 6, 7, 8, 9]);
    walls.extend(horizontal_wall(6, &[2, 3, 4, 5, 6, 7, 8, 9]));
    walls.extend(cells(&[
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (10, 2),
        (10, 3),
        (10, 5),
        (4, 2),
        (4, 5),
        (8, 2),
    ]));

    Level {
        timing: LevelTiming::SemiContinuous,
        name: "ISO SLICE",
        hero_start: Cell::new(3, 3),
        hero_power: 24,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![Door {
            cell: Cell::new(10, 4),
            group: 1,
            initially_open: true,
        }],
        levers: vec![pressure_plate(3, 5, 1)],
        traps: vec![
            group_trap(6, 3, 1),
            group_trap(7, 4, 1),
            group_trap(6, 5, 1),
        ],
        boulders: vec![boulder(2, 4, Direction::Right, 1)],
        bonuses: vec![],
        enemies: vec![
            walker(7, 3, 9, Direction::Left),
            walker(8, 4, 9, Direction::Left),
            walker(7, 5, 9, Direction::Left),
            walker(10, 4, 9, Direction::Up),
            walker(6, 5, 9, Direction::Right),
        ],
    }
}

fn cells(points: &[(i32, i32)]) -> Vec<Cell> {
    points.iter().map(|&(x, y)| Cell::new(x, y)).collect()
}

fn vertical_wall(x: i32, openings: &[i32]) -> Vec<Cell> {
    (0..GRID_HEIGHT)
        .filter(|y| !openings.contains(y))
        .map(|y| Cell::new(x, y))
        .collect()
}

fn horizontal_wall(y: i32, openings: &[i32]) -> Vec<Cell> {
    (0..GRID_WIDTH)
        .filter(|x| !openings.contains(x))
        .map(|x| Cell::new(x, y))
        .collect()
}

fn guard(x: i32, y: i32, power: i32) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power,
        kind: EnemyKind::Guard,
        intent: EnemyIntent::Patrol,
    }
}

fn walker(x: i32, y: i32, power: i32, direction: Direction) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power,
        kind: EnemyKind::Walker { direction },
        intent: EnemyIntent::Patrol,
    }
}

fn directions_toward(cell: Cell, target: Cell) -> [Direction; 4] {
    let horizontal = if target.x < cell.x {
        Direction::Left
    } else {
        Direction::Right
    };
    let vertical = if target.y < cell.y {
        Direction::Up
    } else {
        Direction::Down
    };
    let opposite_horizontal = horizontal.opposite();
    let opposite_vertical = vertical.opposite();

    if (target.x - cell.x).abs() >= (target.y - cell.y).abs() {
        [horizontal, vertical, opposite_vertical, opposite_horizontal]
    } else {
        [vertical, horizontal, opposite_horizontal, opposite_vertical]
    }
}

fn fixed_bonus(x: i32, y: i32, amount: i32) -> Bonus {
    Bonus {
        cell: Cell::new(x, y),
        kind: BonusKind::Fixed(amount),
    }
}

fn mystery_bonus(x: i32, y: i32, min: i32, max: i32) -> Bonus {
    Bonus {
        cell: Cell::new(x, y),
        kind: BonusKind::Mystery { min, max },
    }
}

fn lever(x: i32, y: i32, group: u8) -> Lever {
    Lever {
        cell: Cell::new(x, y),
        group,
        kind: LeverKind::Latch,
    }
}

fn pressure_plate(x: i32, y: i32, group: u8) -> Lever {
    Lever {
        cell: Cell::new(x, y),
        group,
        kind: LeverKind::PressurePlate,
    }
}

fn active_trap(x: i32, y: i32) -> Trap {
    Trap {
        cell: Cell::new(x, y),
        group: None,
        initially_active: true,
    }
}

fn group_trap(x: i32, y: i32, group: u8) -> Trap {
    Trap {
        cell: Cell::new(x, y),
        group: Some(group),
        initially_active: false,
    }
}

fn boulder(x: i32, y: i32, direction: Direction, group: u8) -> Boulder {
    Boulder {
        cell: Cell::new(x, y),
        group,
        direction,
        state: BoulderState::Ready,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MysteryRng {
    state: u32,
}

impl MysteryRng {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)
            .max(1);
        self.state
    }

    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        debug_assert!(min <= max);
        let span = (max - min + 1) as u32;
        min + (self.next_u32() % span) as i32
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    fn test_level(hero_power: i32) -> Level {
        Level {
            timing: LevelTiming::TurnBased,
            name: "TEST",
            hero_start: Cell::new(1, 1),
            hero_power,
            exit: Cell::new(10, 1),
            walls: vec![],
            doors: vec![],
            levers: vec![],
            traps: vec![],
            boulders: vec![],
            bonuses: vec![],
            enemies: vec![],
        }
    }

    fn semi_test_level(hero_power: i32) -> Level {
        let mut level = test_level(hero_power);
        level.timing = LevelTiming::SemiContinuous;
        level
    }

    fn world_from(level: Level) -> SmartBoyWorld {
        let mut world = SmartBoyWorld {
            level_index: 0,
            phase: Phase::Running,
            hero: level.hero_start,
            hero_power: level.hero_power,
            enemies: level.enemies.clone(),
            bonuses: level.bonuses.clone(),
            latched_doors_open: vec![false; level.doors.len()],
            doors_open: vec![false; level.doors.len()],
            latched_traps_active: vec![false; level.traps.len()],
            traps_active: vec![false; level.traps.len()],
            pressure_plates_active: vec![false; level.levers.len()],
            boulders: level.boulders.clone(),
            turn_count: 0,
            seed: 123,
            rng: MysteryRng::new(123),
            level,
        };
        world.open_initial_doors();
        world.open_initial_traps();
        world.refresh_triggered_systems(&mut TurnReport::default());
        world
    }

    fn run_actions(level_index: usize, actions: &[PlayerAction]) -> SmartBoyWorld {
        let mut world = SmartBoyWorld::for_level(level_index, 0xB0A);
        for &action in actions {
            world.apply(action);
            if world.phase() != Phase::Running {
                break;
            }
        }
        world
    }

    fn trivial_exit_reachable(level_index: usize) -> bool {
        let level = build_level(level_index);
        let mut queue = VecDeque::from([level.hero_start]);
        let mut visited = vec![level.hero_start];

        while let Some(cell) = queue.pop_front() {
            if cell == level.exit {
                return true;
            }

            for direction in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ] {
                let next = cell.step(direction);
                if !next.is_inside() || visited.contains(&next) || statically_blocked(&level, next)
                {
                    continue;
                }
                visited.push(next);
                queue.push_back(next);
            }
        }

        false
    }

    fn statically_blocked(level: &Level, cell: Cell) -> bool {
        level.walls.contains(&cell)
            || level
                .doors
                .iter()
                .any(|door| door.cell == cell && !door.initially_open)
            || level.enemies.iter().any(|enemy| enemy.cell == cell)
    }

    fn static_exit_reachable_avoiding_cells(level: &Level, forbidden: &[Cell]) -> bool {
        let mut queue = VecDeque::from([level.hero_start]);
        let mut visited = vec![level.hero_start];

        while let Some(cell) = queue.pop_front() {
            if cell == level.exit {
                return true;
            }

            for direction in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ] {
                let next = cell.step(direction);
                if !next.is_inside()
                    || forbidden.contains(&next)
                    || visited.contains(&next)
                    || statically_blocked(level, next)
                {
                    continue;
                }
                visited.push(next);
                queue.push_back(next);
            }
        }

        false
    }

    fn exit_reachable_avoiding_cell(level: &Level, forbidden: Cell) -> bool {
        let mut queue = VecDeque::from([level.hero_start]);
        let mut visited = vec![level.hero_start];

        while let Some(cell) = queue.pop_front() {
            if cell == level.exit {
                return true;
            }

            for direction in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ] {
                let next = cell.step(direction);
                if !next.is_inside()
                    || next == forbidden
                    || visited.contains(&next)
                    || level.walls.contains(&next)
                {
                    continue;
                }
                visited.push(next);
                queue.push_back(next);
            }
        }

        false
    }

    fn right() -> PlayerAction {
        PlayerAction::Move(Direction::Right)
    }

    fn left() -> PlayerAction {
        PlayerAction::Move(Direction::Left)
    }

    fn up() -> PlayerAction {
        PlayerAction::Move(Direction::Up)
    }

    fn down() -> PlayerAction {
        PlayerAction::Move(Direction::Down)
    }

    fn wait() -> PlayerAction {
        PlayerAction::Wait
    }

    fn shout() -> PlayerAction {
        PlayerAction::Shout
    }

    #[test]
    fn hero_wins_when_power_is_greater_than_enemy_power() {
        let mut level = test_level(5);
        level.enemies.push(guard(2, 1, 4));
        let mut world = world_from(level);

        let report = world.apply(PlayerAction::Move(Direction::Right));

        assert_eq!(world.phase(), Phase::Running);
        assert_eq!(world.hero_power(), 1);
        assert!(world.enemies().is_empty());
        assert!(report.events.contains(&WorldEvent::CombatWon { power: 4 }));
    }

    #[test]
    fn hero_dies_when_power_equals_enemy_power() {
        let mut level = test_level(4);
        level.enemies.push(guard(2, 1, 4));
        let mut world = world_from(level);

        world.apply(PlayerAction::Move(Direction::Right));

        assert_eq!(world.phase(), Phase::Dead);
    }

    #[test]
    fn hero_dies_when_power_is_lower_than_enemy_power() {
        let mut level = test_level(3);
        level.enemies.push(guard(2, 1, 4));
        let mut world = world_from(level);

        world.apply(PlayerAction::Move(Direction::Right));

        assert_eq!(world.phase(), Phase::Dead);
    }

    #[test]
    fn fixed_bonus_increases_power_once() {
        let mut level = test_level(5);
        level.bonuses.push(fixed_bonus(2, 1, 6));
        let mut world = world_from(level);

        world.apply(PlayerAction::Move(Direction::Right));
        world.apply(PlayerAction::Move(Direction::Left));
        world.apply(PlayerAction::Move(Direction::Right));

        assert_eq!(world.hero_power(), 11);
        assert!(world.bonuses().is_empty());
    }

    #[test]
    fn wall_collision_does_not_consume_turn() {
        let mut level = test_level(5);
        level.walls.push(Cell::new(2, 1));
        let mut world = world_from(level);

        let report = world.apply(PlayerAction::Move(Direction::Right));

        assert!(!report.turn_consumed);
        assert_eq!(world.turn_count(), 0);
        assert_eq!(world.hero(), Cell::new(1, 1));
    }

    #[test]
    fn wait_consumes_a_turn() {
        let mut world = world_from(test_level(5));

        let report = world.apply(PlayerAction::Wait);

        assert!(report.turn_consumed);
        assert_eq!(world.turn_count(), 1);
    }

    #[test]
    fn walker_advances_once_per_world_turn() {
        let mut level = test_level(10);
        level.enemies.push(walker(3, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.apply(PlayerAction::Wait);

        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
    }

    #[test]
    fn semi_continuous_tick_moves_walker_without_player_action() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(3, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
        assert_eq!(world.turn_count(), 1);
    }

    #[test]
    fn semi_continuous_patrol_evolves_over_multiple_ticks() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(3, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.update_tick();
        world.update_tick();
        world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(6, 1));
        assert_eq!(world.turn_count(), 3);
    }

    #[test]
    fn boulder_ready_does_not_move() {
        let mut level = semi_test_level(10);
        level.boulders.push(boulder(2, 1, Direction::Right, 1));
        let mut world = world_from(level);

        world.update_tick();

        assert_eq!(world.boulders()[0].cell, Cell::new(2, 1));
        assert_eq!(world.boulders()[0].state, BoulderState::Ready);
    }

    #[test]
    fn pressure_plate_releases_ready_boulder() {
        let mut level = semi_test_level(10);
        level.levers.push(pressure_plate(2, 1, 1));
        level.boulders.push(boulder(4, 1, Direction::Right, 1));
        let mut world = world_from(level);

        let report = world.apply(right());

        assert!(report.events.contains(&WorldEvent::BoulderReleased {
            cell: Cell::new(4, 1),
            direction: Direction::Right,
        }));
        assert_eq!(
            world.boulders()[0].state,
            BoulderState::Rolling { kills: 0 }
        );
    }

    #[test]
    fn rolling_boulder_moves_deterministically() {
        let mut level = semi_test_level(10);
        level.boulders.push(Boulder {
            cell: Cell::new(2, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let report = world.update_tick();

        assert_eq!(world.boulders()[0].cell, Cell::new(5, 1));
        assert!(report.events.contains(&WorldEvent::BoulderMoved {
            cell: Cell::new(3, 1),
        }));
        assert!(report.events.contains(&WorldEvent::BoulderMoved {
            cell: Cell::new(4, 1),
        }));
        assert!(report.events.contains(&WorldEvent::BoulderMoved {
            cell: Cell::new(5, 1),
        }));
    }

    #[test]
    fn rolling_boulder_stops_at_wall() {
        let mut level = semi_test_level(10);
        level.walls.push(Cell::new(4, 1));
        level.boulders.push(Boulder {
            cell: Cell::new(3, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let report = world.update_tick();

        assert_eq!(world.boulders()[0].cell, Cell::new(3, 1));
        assert_eq!(world.boulders()[0].state, BoulderState::Stopped);
        assert!(report.events.contains(&WorldEvent::BoulderStopped {
            cell: Cell::new(3, 1),
        }));
    }

    #[test]
    fn rolling_boulder_kills_walker_and_continues() {
        let mut level = semi_test_level(10);
        level.walls.push(Cell::new(3, 0));
        level.enemies.push(walker(3, 1, 9, Direction::Up));
        level.boulders.push(Boulder {
            cell: Cell::new(2, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let report = world.update_tick();

        assert!(world.enemies().is_empty());
        assert_eq!(world.boulders()[0].cell, Cell::new(5, 1));
        assert_eq!(
            world.boulders()[0].state,
            BoulderState::Rolling { kills: 1 }
        );
        assert!(report.events.contains(&WorldEvent::EnemyKilled {
            cell: Cell::new(3, 1),
            power: 9,
        }));
        assert!(report.events.contains(&WorldEvent::BoulderCrushedEnemy {
            cell: Cell::new(3, 1),
            power: 9,
            chain: 1,
        }));
    }

    #[test]
    fn rolling_boulder_can_crush_multiple_walkers_across_one_run() {
        let mut level = semi_test_level(10);
        level.walls.extend(cells(&[(3, 0), (4, 0), (4, 2)]));
        level.enemies.push(walker(3, 1, 9, Direction::Up));
        level.enemies.push(walker(4, 1, 9, Direction::Up));
        level.boulders.push(Boulder {
            cell: Cell::new(2, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let first = world.update_tick();

        assert_eq!(world.enemies().len(), 0);
        assert_eq!(world.boulders()[0].cell, Cell::new(5, 1));
        assert!(first.events.contains(&WorldEvent::BoulderCrushedEnemy {
            cell: Cell::new(3, 1),
            power: 9,
            chain: 1,
        }));
        assert!(first.events.contains(&WorldEvent::BoulderCrushedEnemy {
            cell: Cell::new(4, 1),
            power: 9,
            chain: 2,
        }));
        assert!(
            first
                .events
                .contains(&WorldEvent::BoulderSmartChain { count: 2 })
        );
    }

    #[test]
    fn rolling_boulder_kills_hero_on_trajectory() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(3, 1);
        level.boulders.push(Boulder {
            cell: Cell::new(2, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let report = world.update_tick();

        assert_eq!(world.phase(), Phase::Dead);
        assert!(report.events.contains(&WorldEvent::HeroDied));
        assert_eq!(world.boulders()[0].cell, Cell::new(3, 1));
        assert_eq!(world.boulders()[0].state, BoulderState::Stopped);
    }

    #[test]
    fn rolling_boulder_emits_one_kill_per_crushed_enemy() {
        let mut level = semi_test_level(10);
        level.walls.push(Cell::new(3, 0));
        level.enemies.push(walker(3, 1, 9, Direction::Up));
        level.boulders.push(Boulder {
            cell: Cell::new(2, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let first = world.update_tick();
        let second = world.update_tick();

        assert_eq!(
            first
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::EnemyKilled { .. }))
                .count(),
            1
        );
        assert_eq!(
            second
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::EnemyKilled { .. }))
                .count(),
            0
        );
    }

    #[test]
    fn restart_restores_iso_boulder_state() {
        let mut world = SmartBoyWorld::iso_slice(42);
        let initial = world.clone();

        world.apply(down());
        world.apply(down());
        world.update_tick();
        assert_ne!(world.boulders()[0].state, initial.boulders()[0].state);

        world.restart();

        assert_eq!(world, initial);
    }

    #[test]
    fn same_ticks_produce_same_boulder_result() {
        let mut first = SmartBoyWorld::iso_slice(7);
        let mut second = SmartBoyWorld::iso_slice(7);

        for world in [&mut first, &mut second] {
            world.apply(shout());
            world.update_tick();
            world.apply(down());
            world.apply(down());
            world.update_tick();
            world.update_tick();
            world.update_tick();
        }

        assert_eq!(first, second);
    }

    #[test]
    fn iso_slice_can_produce_boulder_multi_kill_after_shout_setup() {
        let mut world = SmartBoyWorld::iso_slice(7);
        let mut boulder_chain = 0;

        world.apply(down());
        world.apply(right());
        world.apply(shout());
        world.update_tick();
        world.update_tick();
        world.apply(left());
        world.apply(down());
        for _ in 0..8 {
            let report = world.update_tick();
            for event in report.events {
                if let WorldEvent::BoulderSmartChain { count } = event {
                    boulder_chain = boulder_chain.max(count);
                }
            }
        }

        assert!(boulder_chain >= 2, "expected boulder SMART x2 or better");
    }

    #[test]
    fn adjacent_hero_is_detected_by_walker() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(2, 1);
        level.enemies.push(walker(3, 1, 2, Direction::Up));
        let mut world = world_from(level);

        let report = world.update_tick();

        assert!(report.events.contains(&WorldEvent::WalkerSpottedHero));
        assert!(
            report
                .events
                .contains(&WorldEvent::WalkerDestroyed { power: 2 })
        );
        assert!(world.enemies().is_empty());
        assert_eq!(world.hero_power(), 8);
    }

    #[test]
    fn distant_walker_without_shout_keeps_patrolling() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(5, 1, 2, Direction::Right));
        let mut world = world_from(level);

        let report = world.update_tick();

        assert!(!report.events.contains(&WorldEvent::WalkerSpottedHero));
        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
        assert_eq!(world.enemies()[0].cell, Cell::new(6, 1));
    }

    #[test]
    fn adjacent_hero_takes_priority_over_old_shout_target() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(3, 1, 2, Direction::Up));
        let mut world = world_from(level);

        world.apply(shout());
        world.apply(right());
        let report = world.update_tick();

        assert!(report.events.contains(&WorldEvent::WalkerSpottedHero));
        assert!(
            report
                .events
                .contains(&WorldEvent::WalkerDestroyed { power: 2 })
        );
        assert!(world.enemies().is_empty());
        assert_eq!(world.hero(), Cell::new(2, 1));
    }

    #[test]
    fn stale_chase_returns_to_patrol_without_ping_ponging() {
        let mut level = semi_test_level(10);
        level.enemies.push(Enemy {
            cell: Cell::new(5, 1),
            power: 2,
            kind: EnemyKind::Walker {
                direction: Direction::Left,
            },
            intent: EnemyIntent::ChaseHero {
                patrol_direction: Direction::Right,
            },
        });
        let mut world = world_from(level);

        let report = world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(5, 1));
        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
        assert_eq!(
            world.enemies()[0].kind,
            EnemyKind::Walker {
                direction: Direction::Right,
            }
        );
        assert!(report.events.contains(&WorldEvent::WalkerResumedPatrol));
        assert!(!report.events.contains(&WorldEvent::WalkerSpottedHero));
    }

    #[test]
    fn blocked_walker_stays_put_and_reverses() {
        let mut level = test_level(10);
        level.walls.push(Cell::new(4, 1));
        level.enemies.push(walker(3, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.apply(PlayerAction::Wait);

        assert_eq!(world.enemies()[0].cell, Cell::new(3, 1));
        assert_eq!(
            world.enemies()[0].kind,
            EnemyKind::Walker {
                direction: Direction::Left
            }
        );
    }

    #[test]
    fn walker_entering_hero_triggers_combat() {
        let mut level = test_level(5);
        level.hero_start = Cell::new(3, 1);
        level.enemies.push(walker(2, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.apply(PlayerAction::Wait);

        assert_eq!(world.phase(), Phase::Running);
        assert_eq!(world.hero_power(), 3);
        assert!(world.enemies().is_empty());
    }

    #[test]
    fn lever_opens_its_door() {
        let mut level = test_level(5);
        level.levers.push(lever(2, 1, 7));
        level.doors.push(Door {
            cell: Cell::new(3, 1),
            group: 7,
            initially_open: false,
        });
        let mut world = world_from(level);

        world.apply(PlayerAction::Move(Direction::Right));

        assert!(world.door_open(0));
    }

    #[test]
    fn walker_does_not_activate_levers() {
        let mut level = test_level(10);
        level.levers.push(lever(4, 1, 7));
        level.doors.push(Door {
            cell: Cell::new(5, 1),
            group: 7,
            initially_open: false,
        });
        level.enemies.push(walker(3, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.apply(PlayerAction::Wait);

        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
        assert!(!world.door_open(0));
    }

    #[test]
    fn walker_on_pressure_plate_temporarily_opens_matching_door() {
        let mut level = test_level(10);
        level.levers.push(pressure_plate(3, 2, 7));
        level.doors.push(Door {
            cell: Cell::new(5, 1),
            group: 7,
            initially_open: false,
        });
        level.walls.push(Cell::new(4, 2));
        level.enemies.push(walker(2, 2, 2, Direction::Right));
        let mut world = world_from(level);

        assert!(!world.door_open(0));

        let on = world.apply(wait());
        assert!(world.door_open(0));
        assert!(on.events.contains(&WorldEvent::PressurePlateOn));
        assert!(on.events.contains(&WorldEvent::DoorOpened));

        let held = world.apply(wait());
        assert!(world.door_open(0));
        assert!(!held.events.contains(&WorldEvent::PressurePlateOn));
        assert!(!held.events.contains(&WorldEvent::DoorOpened));

        let off = world.apply(wait());
        assert!(!world.door_open(0));
        assert!(off.events.contains(&WorldEvent::PressurePlateOff));
        assert!(off.events.contains(&WorldEvent::DoorClosed));
    }

    #[test]
    fn shout_outside_radius_is_ignored_by_walkers() {
        let mut level = test_level(10);
        level.enemies.push(walker(8, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(shout());

        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
        assert!(report.events.contains(&WorldEvent::Shouted {
            cell: Cell::new(1, 1),
            heard: 0,
        }));
    }

    #[test]
    fn shout_inside_radius_switches_walker_to_investigate() {
        let mut level = test_level(10);
        level.enemies.push(walker(5, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(shout());

        assert!(report.events.contains(&WorldEvent::Shouted {
            cell: Cell::new(1, 1),
            heard: 1,
        }));
        assert_eq!(
            world.enemies()[0].intent,
            EnemyIntent::Investigate {
                target: Cell::new(1, 1),
                patrol_direction: Direction::Right,
            }
        );
    }

    #[test]
    fn investigating_walker_progresses_deterministically_toward_target() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(5, 1, 9, Direction::Right));
        let mut world = world_from(level);

        world.apply(shout());
        world.apply(down());
        world.update_tick();
        world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(3, 1));
        assert_eq!(
            world.enemies()[0].intent,
            EnemyIntent::Investigate {
                target: Cell::new(1, 1),
                patrol_direction: Direction::Right,
            }
        );
    }

    #[test]
    fn investigating_walker_resumes_patrol_after_reaching_target_cell() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(3, 1, 9, Direction::Right));
        let mut world = world_from(level);

        world.apply(shout());
        world.apply(down());
        world.update_tick();
        world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(1, 1));
        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
        assert_eq!(
            world.enemies()[0].kind,
            EnemyKind::Walker {
                direction: Direction::Right,
            }
        );
    }

    #[test]
    fn investigating_walker_abandons_unreachable_target() {
        let mut level = semi_test_level(10);
        level.walls.extend(vertical_wall(2, &[]));
        level.enemies.push(walker(3, 1, 9, Direction::Right));
        let mut world = world_from(level);

        world.apply(shout());
        let report = world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(3, 1));
        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
        assert!(report.events.contains(&WorldEvent::WalkerLostTarget));
    }

    #[test]
    fn shout_changes_trajectory_while_world_lives() {
        let mut level = semi_test_level(10);
        level.enemies.push(walker(5, 1, 9, Direction::Up));
        let mut world = world_from(level);

        world.apply(shout());
        world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
        assert_eq!(
            world.enemies()[0].kind,
            EnemyKind::Walker {
                direction: Direction::Left,
            }
        );
    }

    #[test]
    fn semi_continuous_plate_and_trap_react_to_walker_ticks() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(1, 3);
        level.levers.push(pressure_plate(3, 1, 7));
        level.traps.push(group_trap(4, 1, 7));
        level.enemies.push(walker(2, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let armed = world.update_tick();
        assert!(world.trap_active(0));
        assert!(armed.events.contains(&WorldEvent::PressurePlateOn));
        assert!(armed.events.contains(&WorldEvent::TrapArmed));

        let killed = world.update_tick();
        assert!(world.enemies().is_empty());
        assert!(killed.events.contains(&WorldEvent::TrapTriggered));
        assert_eq!(
            killed
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::EnemyKilled { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn same_ticks_and_inputs_produce_identical_semi_continuous_result() {
        let mut first = SmartBoyWorld::for_level(16, 0xB0A);
        let mut second = SmartBoyWorld::for_level(16, 0xB0A);

        for world in [&mut first, &mut second] {
            world.apply(right());
            world.apply(right());
            world.apply(shout());
            world.update_tick();
            world.update_tick();
            world.apply(up());
            world.update_tick();
        }

        assert_eq!(first, second);
    }

    #[test]
    fn shout_does_not_change_static_guards() {
        let mut level = test_level(10);
        level.enemies.push(guard(3, 1, 9));
        let mut world = world_from(level);
        let initial = world.enemies()[0].clone();

        let report = world.apply(shout());

        assert_eq!(world.enemies()[0], initial);
        assert!(report.events.contains(&WorldEvent::Shouted {
            cell: Cell::new(1, 1),
            heard: 0,
        }));
    }

    #[test]
    fn restart_restores_investigation_state() {
        let mut world = SmartBoyWorld::for_level(16, 42);
        let initial = world.clone();

        world.apply(right());
        world.apply(right());
        world.apply(shout());
        assert!(
            world
                .enemies()
                .iter()
                .any(|enemy| matches!(enemy.intent, EnemyIntent::Investigate { .. }))
        );

        world.restart();

        assert_eq!(world, initial);
    }

    #[test]
    fn active_trap_kills_hero_on_entry() {
        let mut level = test_level(5);
        level.traps.push(active_trap(2, 1));
        let mut world = world_from(level);

        let report = world.apply(right());

        assert_eq!(world.phase(), Phase::Dead);
        assert!(report.events.contains(&WorldEvent::TrapTriggered));
        assert!(report.events.contains(&WorldEvent::HeroDied));
    }

    #[test]
    fn inactive_trap_is_traversable() {
        let mut level = test_level(5);
        level.traps.push(group_trap(2, 1, 3));
        let mut world = world_from(level);

        let report = world.apply(right());

        assert_eq!(world.phase(), Phase::Running);
        assert_eq!(world.hero(), Cell::new(2, 1));
        assert!(!report.events.contains(&WorldEvent::TrapTriggered));
    }

    #[test]
    fn walker_entering_active_trap_is_destroyed() {
        let mut level = test_level(10);
        level.traps.push(active_trap(4, 1));
        level.enemies.push(walker(3, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(wait());

        assert!(world.enemies().is_empty());
        assert_eq!(world.phase(), Phase::Running);
        assert!(report.events.contains(&WorldEvent::TrapTriggered));
    }

    #[test]
    fn pressure_plate_temporarily_arms_matching_trap() {
        let mut level = test_level(10);
        level.levers.push(pressure_plate(2, 1, 7));
        level.traps.push(group_trap(4, 1, 7));
        let mut world = world_from(level);

        let on = world.apply(right());
        assert!(world.trap_active(0));
        assert!(on.events.contains(&WorldEvent::PressurePlateOn));
        assert!(on.events.contains(&WorldEvent::TrapArmed));

        let off = world.apply(left());
        assert!(!world.trap_active(0));
        assert!(off.events.contains(&WorldEvent::PressurePlateOff));
        assert!(off.events.contains(&WorldEvent::TrapDisarmed));
    }

    #[test]
    fn arming_trap_under_actor_does_not_trigger_without_entry() {
        let mut level = test_level(10);
        level.levers.push(pressure_plate(2, 1, 7));
        level.traps.push(group_trap(4, 1, 7));
        level.enemies.push(walker(4, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(right());

        assert!(world.trap_active(0));
        assert_eq!(world.enemies()[0].cell, Cell::new(5, 1));
        assert!(!report.events.contains(&WorldEvent::TrapTriggered));
    }

    #[test]
    fn trap_trigger_does_not_repeat_after_death() {
        let mut level = test_level(5);
        level.traps.push(active_trap(2, 1));
        let mut world = world_from(level);

        let death = world.apply(right());
        let ignored = world.apply(wait());

        assert_eq!(world.phase(), Phase::Dead);
        assert_eq!(
            death
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::TrapTriggered))
                .count(),
            1
        );
        assert!(ignored.events.is_empty());
    }

    #[test]
    fn restart_restores_trap_activation_state() {
        let mut world = SmartBoyWorld::for_level(14, 42);
        assert!(!world.trap_active(0));

        world.apply(up());
        assert!(world.trap_active(0));

        world.restart();
        assert!(!world.trap_active(0));
        assert_eq!(world.hero(), Cell::new(1, 4));
    }

    #[test]
    fn exit_wins_before_enemy_turn() {
        let mut level = test_level(5);
        level.exit = Cell::new(2, 1);
        level.enemies.push(walker(1, 2, 9, Direction::Up));
        let mut world = world_from(level);

        world.apply(PlayerAction::Move(Direction::Right));

        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.enemies()[0].cell, Cell::new(1, 2));
    }

    #[test]
    fn restart_recreates_initial_state() {
        let mut world = SmartBoyWorld::for_level(1, 42);
        let initial = world.clone();

        world.apply(PlayerAction::Move(Direction::Right));
        world.restart();

        assert_eq!(world, initial);
    }

    #[test]
    fn mystery_bonus_respects_bounds() {
        let mut level = test_level(5);
        level.bonuses.push(mystery_bonus(2, 1, 2, 8));
        let mut world = world_from(level);

        world.apply(PlayerAction::Move(Direction::Right));

        assert!((7..=13).contains(&world.hero_power()));
    }

    #[test]
    fn identical_seed_reproduces_mystery_bonus() {
        let mut first = SmartBoyWorld::for_level(8, 0xB0A);
        let mut second = SmartBoyWorld::for_level(8, 0xB0A);

        first.apply(PlayerAction::Move(Direction::Right));
        first.apply(PlayerAction::Move(Direction::Right));
        first.apply(PlayerAction::Move(Direction::Up));

        second.apply(PlayerAction::Move(Direction::Right));
        second.apply(PlayerAction::Move(Direction::Right));
        second.apply(PlayerAction::Move(Direction::Up));

        assert_eq!(first.hero_power(), second.hero_power());
    }

    #[test]
    fn interaction_levels_do_not_have_static_bypass_to_exit() {
        for index in [0, 1, 2, 3, 5, 7, 8, 9, 10, 11, 12, 14, 15, 16, 17, 18] {
            assert!(
                !trivial_exit_reachable(index),
                "level {} has a trivial static path to exit",
                index + 1
            );
        }
    }

    #[test]
    fn just_leave_keeps_the_intended_static_bypass() {
        assert!(trivial_exit_reachable(4));
    }

    #[test]
    fn level_one_requires_crossing_a_guard_chokepoint() {
        let world = run_actions(
            0,
            &[
                right(),
                down(),
                right(),
                right(),
                right(),
                right(),
                up(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );

        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.hero_power(), 7);
    }

    #[test]
    fn level_two_bonus_is_required_for_the_guard() {
        let greedy = run_actions(1, &[right(), right(), right()]);
        assert_eq!(greedy.phase(), Phase::Dead);

        let planned = run_actions(
            1,
            &[
                up(),
                up(),
                right(),
                down(),
                down(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        eprintln!(
            "L4 planned final: {:?} {:?} {}",
            planned.phase(),
            planned.hero(),
            planned.hero_power()
        );
        assert_eq!(planned.phase(), Phase::Won);
        assert_eq!(planned.hero_power(), 1);
    }

    #[test]
    fn level_three_requires_paying_two_combat_costs() {
        let world = run_actions(
            2,
            &[
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );

        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.hero_power(), 1);
    }

    #[test]
    fn level_four_wrong_branch_fails_but_better_order_wins() {
        let direct = run_actions(3, &[right(), right(), right(), right(), right()]);
        assert_eq!(direct.phase(), Phase::Dead);

        let planned = run_actions(
            3,
            &[
                right(),
                right(),
                up(),
                up(),
                down(),
                down(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(planned.phase(), Phase::Won);
        assert_eq!(planned.hero_power(), 1);
    }

    #[test]
    fn level_four_every_route_to_exit_crosses_guard_eight_position() {
        let level = build_level(3);
        let guard_eight = level
            .enemies
            .iter()
            .find(|enemy| enemy.power == 8)
            .expect("level 4 should keep a Guard 8")
            .cell;

        assert!(!exit_reachable_avoiding_cell(&level, guard_eight));
    }

    #[test]
    fn level_seven_wait_shifts_the_walker_timing() {
        let impatient = run_actions(6, &[right(), right()]);
        assert_eq!(impatient.phase(), Phase::Dead);

        let patient = run_actions(
            6,
            &[
                wait(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(patient.phase(), Phase::Won);
    }

    #[test]
    fn level_eight_can_exploit_the_walker_by_letting_it_come() {
        let world = run_actions(
            7,
            &[
                wait(),
                wait(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );

        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.hero_power(), 3);
    }

    #[test]
    fn level_nine_has_a_safe_route_without_mystery_bonus() {
        let world = run_actions(
            8,
            &[
                up(),
                up(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                down(),
                right(),
                down(),
                right(),
            ],
        );
        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.hero_power(), 1);
    }

    #[test]
    fn level_ten_intended_route_combines_lever_bonus_walker_and_guard() {
        let world = run_actions(
            9,
            &[
                up(),
                up(),
                up(),
                right(),
                left(),
                down(),
                down(),
                down(),
                right(),
                right(),
                right(),
                right(),
                up(),
                up(),
                up(),
                down(),
                right(),
                right(),
                up(),
                right(),
                right(),
                right(),
            ],
        );

        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.hero_power(), 1);
    }

    #[test]
    fn living_plate_a_teaches_that_a_walker_can_open_a_door() {
        let world = run_actions(
            10,
            &[
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );

        assert_eq!(world.phase(), Phase::Won);
    }

    #[test]
    fn living_plate_b_requires_waiting_for_the_walker_window() {
        let rushed = run_actions(11, &[right(), right()]);
        assert_eq!(rushed.phase(), Phase::Running);
        assert_eq!(rushed.hero(), Cell::new(5, 4));

        let synced = run_actions(
            11,
            &[wait(), right(), right(), right(), right(), right(), right()],
        );
        assert_eq!(synced.phase(), Phase::Won);
    }

    #[test]
    fn living_plate_c_allows_walker_route_or_hero_latch_route() {
        let walker_route = run_actions(12, &[right(), right(), right(), right(), right()]);
        assert_eq!(walker_route.phase(), Phase::Won);
        assert_eq!(walker_route.turn_count(), 5);

        let hero_route = run_actions(
            12,
            &[
                left(),
                left(),
                left(),
                up(),
                up(),
                down(),
                down(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(hero_route.phase(), Phase::Won);
        assert!(hero_route.turn_count() > walker_route.turn_count());
    }

    #[test]
    fn trap_a_teaches_visible_active_trap_danger() {
        let direct = run_actions(13, &[right(), right()]);
        assert_eq!(direct.phase(), Phase::Dead);

        let bypass = run_actions(
            13,
            &[
                up(),
                right(),
                right(),
                right(),
                down(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(bypass.phase(), Phase::Won);
    }

    #[test]
    fn trap_b_requires_holding_plate_until_walker_hits_trap() {
        let rushed = run_actions(14, &[right(), right(), right(), right()]);
        assert_eq!(rushed.phase(), Phase::Dead);

        let synced = run_actions(
            14,
            &[
                up(),
                wait(),
                wait(),
                down(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(synced.phase(), Phase::Won);
        assert_eq!(synced.hero_power(), 5);
    }

    #[test]
    fn trap_c_allows_system_solution_or_paid_combat_solution() {
        let system = run_actions(
            15,
            &[
                right(),
                wait(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(system.phase(), Phase::Won);
        assert_eq!(system.hero_power(), 12);

        let paid = run_actions(
            15,
            &[
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
                right(),
            ],
        );
        assert_eq!(paid.phase(), Phase::Won);
        assert_eq!(paid.hero_power(), 3);
    }

    #[test]
    fn shout_a_teaches_luring_a_walker_into_a_trap() {
        let mut world = SmartBoyWorld::for_level(16, 0xB0A);
        world.apply(right());
        world.apply(right());
        world.apply(shout());

        world.update_tick();
        world.apply(up());
        world.update_tick();
        world.apply(right());
        world.update_tick();

        for action in [right(), down(), right(), right(), right()] {
            world.apply(action);
        }

        assert_eq!(world.phase(), Phase::Won);
        assert_eq!(world.hero_power(), 5);
    }

    #[test]
    fn shout_b_allows_one_shout_to_create_a_double_trap_kill() {
        let mut world = SmartBoyWorld::for_level(17, 0xB0A);

        world.apply(up());
        world.apply(shout());
        let report = world.update_tick();

        assert!(world.enemies().is_empty());
        assert!(report.events.contains(&WorldEvent::SmartChain { count: 2 }));
        assert_eq!(
            report
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::EnemyKilled { .. }))
                .count(),
            2
        );
        assert_eq!(
            report
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::SmartChain { .. }))
                .count(),
            1
        );

        for action in [down(), right(), right(), right(), right(), right()] {
            world.apply(action);
        }
        assert_eq!(world.phase(), Phase::Won);
    }

    #[test]
    fn shout_c_allows_costly_direct_route_or_power_preserving_smart_route() {
        let mut direct = SmartBoyWorld::for_level(18, 0xB0A);
        for action in [
            up(),
            right(),
            right(),
            right(),
            right(),
            down(),
            right(),
            right(),
        ] {
            direct.apply(action);
        }
        assert_eq!(direct.phase(), Phase::Won);
        assert_eq!(direct.hero_power(), 13);

        let mut smart = SmartBoyWorld::for_level(18, 0xB0A);
        smart.apply(right());
        smart.apply(shout());
        smart.update_tick();
        smart.apply(left());
        smart.update_tick();
        for action in [
            up(),
            right(),
            right(),
            right(),
            right(),
            down(),
            right(),
            right(),
        ] {
            smart.apply(action);
        }

        assert_eq!(smart.phase(), Phase::Won);
        assert_eq!(smart.hero_power(), 20);
    }

    #[test]
    fn trap_experiment_layouts_do_not_allow_static_bypass_of_main_interaction() {
        for index in [14, 15] {
            let level = build_level(index);
            let forbidden = [
                level.levers[0].cell,
                level.traps[0].cell,
                level.enemies[0].cell,
            ];
            assert!(
                !static_exit_reachable_avoiding_cells(&level, &forbidden),
                "level {} can bypass plate, trap, and walker",
                index + 1
            );
        }
    }

    #[test]
    fn all_levels_fit_grid_and_have_one_start_and_exit() {
        for index in 0..LEVEL_COUNT {
            let level = build_level(index);
            assert!(level.hero_start.is_inside());
            assert!(level.exit.is_inside());
            assert!(level.walls.iter().copied().all(Cell::is_inside));
            assert!(
                level
                    .doors
                    .iter()
                    .map(|door| door.cell)
                    .all(Cell::is_inside)
            );
            assert!(
                level
                    .levers
                    .iter()
                    .map(|lever| lever.cell)
                    .all(Cell::is_inside)
            );
            assert!(
                level
                    .traps
                    .iter()
                    .map(|trap| trap.cell)
                    .all(Cell::is_inside)
            );
            assert!(
                level
                    .bonuses
                    .iter()
                    .map(|bonus| bonus.cell)
                    .all(Cell::is_inside)
            );
            assert!(
                level
                    .enemies
                    .iter()
                    .map(|enemy| enemy.cell)
                    .all(Cell::is_inside)
            );
        }
    }
}
