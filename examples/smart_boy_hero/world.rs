pub(super) const GRID_WIDTH: i32 = 12;
pub(super) const GRID_HEIGHT: i32 = 8;
pub(super) const LEVEL_COUNT: usize = 20;
const SHOUT_RADIUS: i32 = 5;
pub(super) const ROCK_THROW_RANGE: i32 = 6;
pub(super) const ROCK_HEARING_RADIUS: i32 = 3;
const RAT_FEAR_RADIUS: i32 = 2;
const BOULDER_STEPS_PER_TICK: usize = 3;

mod level_spec;
use level_spec::LevelSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SmartBoyWorld {
    level_index: usize,
    level: Level,
    phase: Phase,
    hero: Cell,
    hero_power: i32,
    enemies: Vec<Enemy>,
    foods: Vec<Food>,
    bonuses: Vec<Bonus>,
    latched_doors_open: Vec<bool>,
    doors_open: Vec<bool>,
    latched_traps_active: Vec<bool>,
    traps_active: Vec<bool>,
    pressure_plates_active: Vec<bool>,
    boulders: Vec<Boulder>,
    core_key_cell: Option<Cell>,
    has_core_key: bool,
    core_gate: Option<Cell>,
    core_gate_open: bool,
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
            foods: level.foods.clone(),
            bonuses: level.bonuses.clone(),
            latched_doors_open: vec![false; level.doors.len()],
            doors_open: vec![false; level.doors.len()],
            latched_traps_active: vec![false; level.traps.len()],
            traps_active: vec![false; level.traps.len()],
            pressure_plates_active: vec![false; level.levers.len()],
            boulders: level.boulders.clone(),
            core_key_cell: None,
            has_core_key: false,
            core_gate: (level_index == LEVEL_COUNT).then_some(Cell::new(21, 8)),
            core_gate_open: false,
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
            PlayerAction::ThrowRock(target) => self.throw_rock(target, &mut report),
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

    #[allow(dead_code)]
    pub(super) fn grid_width(&self) -> i32 {
        self.level.width
    }

    #[allow(dead_code)]
    pub(super) fn grid_height(&self) -> i32 {
        self.level.height
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

    pub(super) fn foods(&self) -> &[Food] {
        &self.foods
    }

    pub(super) fn bonuses(&self) -> &[Bonus] {
        &self.bonuses
    }

    pub(super) fn door_open(&self, index: usize) -> bool {
        if self.core_gate.is_some_and(|gate| {
            self.level
                .doors
                .get(index)
                .is_some_and(|door| door.cell == gate)
        }) {
            return self.core_gate_open;
        }
        self.doors_open.get(index).copied().unwrap_or(false)
    }

    pub(super) fn traps(&self) -> &[Trap] {
        &self.level.traps
    }

    pub(super) fn pits(&self) -> &[Pit] {
        &self.level.pits
    }

    pub(super) fn trap_active(&self, index: usize) -> bool {
        self.traps_active.get(index).copied().unwrap_or(false)
    }

    #[allow(dead_code)]
    pub(super) fn boulders(&self) -> &[Boulder] {
        &self.boulders
    }

    #[allow(dead_code)]
    pub(super) fn core_key_cell(&self) -> Option<Cell> {
        self.core_key_cell
    }

    #[allow(dead_code)]
    pub(super) fn has_core_key(&self) -> bool {
        self.has_core_key
    }

    #[allow(dead_code)]
    pub(super) fn enemy_is_key_warden(&self, index: usize) -> bool {
        self.enemies
            .get(index)
            .is_some_and(|enemy| matches!(enemy.role, EnemyRole::KeyWarden))
    }

    #[allow(dead_code)]
    pub(super) fn door_is_core_gate(&self, index: usize) -> bool {
        self.core_gate.is_some_and(|gate| {
            self.level
                .doors
                .get(index)
                .is_some_and(|door| door.cell == gate)
        })
    }

    #[allow(dead_code)]
    pub(super) fn lever_actuator(&self, index: usize) -> ActuatorKind {
        let Some(lever) = self.level.levers.get(index) else {
            return ActuatorKind::Door;
        };
        if self
            .boulders
            .iter()
            .any(|boulder| boulder.group == lever.group)
        {
            ActuatorKind::Boulder
        } else if self
            .level
            .traps
            .iter()
            .any(|trap| trap.group == Some(lever.group))
        {
            ActuatorKind::Trap
        } else {
            ActuatorKind::Door
        }
    }

    pub(super) fn can_throw_rock_to(&self, target: Cell) -> bool {
        self.in_bounds(target)
            && target != self.hero
            && self.hero.manhattan_distance(target) <= ROCK_THROW_RANGE
            && !self.wall_at(target)
            && self.closed_door_at(target).is_none()
    }

    fn try_move_hero(&mut self, direction: Direction, report: &mut TurnReport) -> bool {
        let target = self.hero.step(direction);
        if !self.in_bounds(target) || self.wall_at(target) {
            report.events.push(WorldEvent::Blocked);
            return false;
        }
        if self.locked_core_gate_at(target) {
            if self.has_core_key {
                self.core_gate_open = true;
                report.events.push(WorldEvent::CoreGateUnlocked);
                report.events.push(WorldEvent::DoorOpened);
            } else {
                report.events.push(WorldEvent::LockedGateBlocked);
                return false;
            }
        }
        if self.closed_door_at(target).is_some() {
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
            let enemy = self.enemies.remove(enemy_index);
            self.hero = target;
            report.events.push(WorldEvent::CombatWon { power });
            report.events.push(WorldEvent::EnemyKilled {
                cell: target,
                power,
            });
            self.resolve_hero_entered_cell(target, report);
            self.drop_core_key_if_warden(&enemy, target, report);
            if self.phase == Phase::Running {
                self.collect_core_key_at(target, report);
            }
        } else {
            self.phase = Phase::Dead;
            report.events.push(WorldEvent::HeroDied);
        }
    }

    fn resolve_hero_entered_cell(&mut self, cell: Cell, report: &mut TurnReport) {
        if let Some(danger) = self.active_danger_at(cell) {
            self.phase = Phase::Dead;
            report.events.push(danger.trigger_event());
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
            let Some(direction) = self.enemy_direction_for_turn(index, report) else {
                index += 1;
                continue;
            };

            let target = self.enemies[index].cell.step(direction);
            if target == self.hero {
                let power = self.enemies[index].power;
                if self.hero_power > power {
                    self.hero_power -= power;
                    let cell = self.enemies[index].cell;
                    let enemy = self.enemies.remove(index);
                    report.events.push(WorldEvent::WalkerDestroyed { power });
                    report.events.push(WorldEvent::EnemyKilled { cell, power });
                    self.drop_core_key_if_warden(&enemy, cell, report);
                    continue;
                }

                self.phase = Phase::Dead;
                report.events.push(WorldEvent::HeroDied);
                return;
            }

            if !self.in_bounds(target)
                || self.wall_at(target)
                || self.closed_door_at(target).is_some()
                || self.enemy_at_except(target, index).is_some()
            {
                self.resolve_enemy_blocked(index, direction, report);
            } else {
                if let Some(danger) = self.active_danger_at(target) {
                    let power = self.enemies[index].power;
                    self.enemies[index].cell = target;
                    let enemy = self.enemies.remove(index);
                    report.events.push(danger.trigger_event());
                    report.events.push(WorldEvent::EnemyKilled {
                        cell: target,
                        power,
                    });
                    self.drop_core_key_if_warden(&enemy, target, report);
                    trap_kills += 1;
                    continue;
                }
                self.enemies[index].cell = target;
                self.resolve_enemy_arrival(index, report);
                self.report_enemy_moved(index, report);
            }

            index += 1;
        }

        if trap_kills >= 2 {
            report
                .events
                .push(WorldEvent::SmartChain { count: trap_kills });
        }
    }

    fn enemy_direction_for_turn(
        &mut self,
        index: usize,
        report: &mut TurnReport,
    ) -> Option<Direction> {
        match self.enemies[index].kind {
            EnemyKind::Guard => None,
            EnemyKind::Walker { direction } => {
                self.walker_direction_for_turn(index, direction, report)
            }
            EnemyKind::Rat => self.rat_direction_for_turn(index, report),
            EnemyKind::Cat => self.cat_direction_for_turn(index, report),
        }
    }

    fn resolve_enemy_blocked(
        &mut self,
        index: usize,
        direction: Direction,
        report: &mut TurnReport,
    ) {
        if matches!(self.enemies[index].kind, EnemyKind::Walker { .. }) {
            self.enemies[index].kind = EnemyKind::Walker {
                direction: direction.opposite(),
            };
            report.events.push(WorldEvent::WalkerTurned);
        }
    }

    fn resolve_enemy_arrival(&mut self, index: usize, report: &mut TurnReport) {
        match self.enemies[index].kind {
            EnemyKind::Walker { .. } => self.resolve_investigation_arrival(index, report),
            EnemyKind::Rat => self.eat_food_at(self.enemies[index].cell, report),
            EnemyKind::Guard | EnemyKind::Cat => {}
        }
    }

    fn report_enemy_moved(&self, index: usize, report: &mut TurnReport) {
        match self.enemies[index].kind {
            EnemyKind::Walker { .. } => report.events.push(WorldEvent::WalkerMoved),
            EnemyKind::Rat => report.events.push(WorldEvent::RatMoved),
            EnemyKind::Cat => report.events.push(WorldEvent::CatMoved),
            EnemyKind::Guard => {}
        }
    }

    fn shout(&mut self, report: &mut TurnReport) {
        let target = self.hero;
        let heard = self.emit_noise(target, SHOUT_RADIUS);
        report.events.push(WorldEvent::Shouted {
            cell: target,
            heard,
        });
    }

    fn throw_rock(&mut self, target: Cell, report: &mut TurnReport) -> bool {
        if !self.can_throw_rock_to(target) {
            report.events.push(WorldEvent::Blocked);
            return false;
        }

        let heard = self.emit_noise(target, ROCK_HEARING_RADIUS);
        report.events.push(WorldEvent::RockImpacted {
            cell: target,
            heard,
        });
        true
    }

    fn emit_noise(&mut self, target: Cell, radius: i32) -> usize {
        let mut heard = 0;
        for enemy in &mut self.enemies {
            let EnemyKind::Walker { direction } = enemy.kind else {
                continue;
            };
            if enemy.cell.manhattan_distance(target) > radius {
                continue;
            }
            enemy.intent = EnemyIntent::Investigate {
                target,
                patrol_direction: direction,
            };
            heard += 1;
        }
        heard
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

    fn rat_direction_for_turn(
        &mut self,
        index: usize,
        report: &mut TurnReport,
    ) -> Option<Direction> {
        let rat = self.enemies[index].cell;
        if let Some(cat) = self.nearest_enemy_kind_cell(rat, EnemyKind::Cat)
            && rat.manhattan_distance(cat) <= RAT_FEAR_RADIUS
            && let Some(direction) = self.step_away_from(index, cat)
        {
            report.events.push(WorldEvent::RatScared);
            return Some(direction);
        }

        let food = self.nearest_food_cell(rat)?;
        let direction = match self.step_toward(index, food) {
            PathStep::Step(direction) => direction,
            PathStep::Arrived => {
                self.eat_food_at(rat, report);
                return None;
            }
            PathStep::Blocked => return None,
        };
        report.events.push(WorldEvent::RatSmelledFood);
        Some(direction)
    }

    fn cat_direction_for_turn(
        &mut self,
        index: usize,
        report: &mut TurnReport,
    ) -> Option<Direction> {
        let cat = self.enemies[index].cell;
        let rat = self.nearest_enemy_kind_cell(cat, EnemyKind::Rat)?;
        let direction = match self.step_toward(index, rat) {
            PathStep::Step(direction) => direction,
            PathStep::Arrived | PathStep::Blocked => return None,
        };
        report.events.push(WorldEvent::CatChasedRat);
        Some(direction)
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
                if !self.in_bounds(next)
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

    fn step_away_from(&self, enemy_index: usize, threat: Cell) -> Option<Direction> {
        let start = self.enemies[enemy_index].cell;
        let current_distance = start.manhattan_distance(threat);
        let mut best = None;
        for direction in directions_toward(threat, start) {
            let cell = start.step(direction);
            if !self.in_bounds(cell)
                || self.wall_at(cell)
                || self.closed_door_at(cell).is_some()
                || self.enemy_at_except(cell, enemy_index).is_some()
            {
                continue;
            }
            let distance = cell.manhattan_distance(threat);
            if distance <= current_distance {
                continue;
            }
            if best.is_none_or(|(_, best_distance)| distance > best_distance) {
                best = Some((direction, distance));
            }
        }
        best.map(|(direction, _)| direction)
    }

    fn nearest_enemy_kind_cell(&self, from: Cell, kind: EnemyKind) -> Option<Cell> {
        self.enemies
            .iter()
            .filter(|enemy| enemy.kind == kind)
            .map(|enemy| enemy.cell)
            .min_by_key(|cell| (from.manhattan_distance(*cell), cell.y, cell.x))
    }

    fn nearest_food_cell(&self, from: Cell) -> Option<Cell> {
        self.foods
            .iter()
            .map(|food| food.cell)
            .min_by_key(|cell| (from.manhattan_distance(*cell), cell.y, cell.x))
    }

    fn collect_at(&mut self, cell: Cell, report: &mut TurnReport) {
        self.collect_core_key_at(cell, report);

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

    fn eat_food_at(&mut self, cell: Cell, report: &mut TurnReport) {
        if let Some(index) = self.foods.iter().position(|food| food.cell == cell) {
            self.foods.remove(index);
            report.events.push(WorldEvent::FoodEaten);
        }
    }

    fn collect_core_key_at(&mut self, cell: Cell, report: &mut TurnReport) {
        if self.core_key_cell == Some(cell) {
            self.core_key_cell = None;
            self.has_core_key = true;
            report.events.push(WorldEvent::CoreKeyAcquired);
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
            .find(|(index, door)| {
                door.cell == cell
                    && if self.core_gate == Some(cell) {
                        !self.core_gate_open
                    } else {
                        !self.doors_open[*index]
                    }
            })
            .map(|(index, _)| index)
    }

    fn locked_core_gate_at(&self, cell: Cell) -> bool {
        self.core_gate == Some(cell) && !self.core_gate_open
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

    fn active_pit_at(&self, cell: Cell) -> Option<usize> {
        self.level.pits.iter().position(|pit| pit.cell == cell)
    }

    fn active_danger_at(&self, cell: Cell) -> Option<DangerKind> {
        if self.active_trap_at(cell).is_some() {
            return Some(DangerKind::Trap);
        }
        if self.active_pit_at(cell).is_some() {
            return Some(DangerKind::Pit);
        }
        None
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

                if !self.in_bounds(target)
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
                    for enemy in crushed {
                        let power = enemy.power;
                        report.events.push(WorldEvent::EnemyKilled {
                            cell: target,
                            power,
                        });
                        report.events.push(WorldEvent::BoulderCrushedEnemy {
                            cell: target,
                            power,
                            chain: kills,
                        });
                        self.drop_core_key_if_warden(&enemy, target, report);
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

    fn remove_enemies_at(&mut self, cell: Cell) -> Vec<Enemy> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.enemies.len() {
            if self.enemies[index].cell == cell {
                removed.push(self.enemies.remove(index));
            } else {
                index += 1;
            }
        }
        removed
    }

    fn drop_core_key_if_warden(&mut self, enemy: &Enemy, cell: Cell, report: &mut TurnReport) {
        if !matches!(enemy.role, EnemyRole::KeyWarden)
            || self.has_core_key
            || self.core_key_cell.is_some()
        {
            return;
        }
        let drop_cell = self.recoverable_core_key_cell(cell);
        self.core_key_cell = Some(drop_cell);
        report
            .events
            .push(WorldEvent::CoreKeyDropped { cell: drop_cell });
    }

    fn recoverable_core_key_cell(&self, preferred: Cell) -> Cell {
        if self.can_drop_recoverable_core_key_at(preferred) {
            return preferred;
        }

        [
            Direction::Right,
            Direction::Down,
            Direction::Left,
            Direction::Up,
        ]
        .into_iter()
        .map(|direction| preferred.step(direction))
        .find(|&cell| self.can_drop_recoverable_core_key_at(cell))
        .unwrap_or(preferred)
    }

    fn can_drop_recoverable_core_key_at(&self, cell: Cell) -> bool {
        self.in_bounds(cell)
            && !self.wall_at(cell)
            && self.closed_door_at(cell).is_none()
            && self.active_trap_at(cell).is_none()
            && self.active_pit_at(cell).is_none()
            && self.enemy_at(cell).is_none()
            && !self.boulders.iter().any(|boulder| boulder.cell == cell)
    }

    fn boulder_blocks_at(&self, cell: Cell, except: usize) -> bool {
        self.boulders
            .iter()
            .enumerate()
            .any(|(index, boulder)| index != except && boulder.cell == cell)
    }

    fn in_bounds(&self, cell: Cell) -> bool {
        level_in_bounds(&self.level, cell)
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
    CoreKeyDropped {
        cell: Cell,
    },
    CoreKeyAcquired,
    LockedGateBlocked,
    CoreGateUnlocked,
    PressurePlateOn,
    PressurePlateOff,
    DoorOpened,
    DoorClosed,
    TrapArmed,
    TrapDisarmed,
    TrapTriggered,
    PitFall,
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
    RockImpacted {
        cell: Cell,
        heard: usize,
    },
    WalkerLostTarget,
    WalkerMoved,
    WalkerResumedPatrol,
    WalkerSpottedHero,
    WalkerTurned,
    RatSmelledFood,
    RatScared,
    RatMoved,
    CatChasedRat,
    CatMoved,
    FoodEaten,
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
    #[allow(dead_code)]
    ThrowRock(Cell),
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

    pub(super) fn step(self, direction: Direction) -> Self {
        let (dx, dy) = direction.offset();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
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
    pub(super) role: EnemyRole,
    pub(super) intent: EnemyIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnemyRole {
    Normal,
    KeyWarden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnemyKind {
    Guard,
    Walker { direction: Direction },
    Rat,
    Cat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DangerKind {
    Trap,
    Pit,
}

impl DangerKind {
    fn trigger_event(self) -> WorldEvent {
        match self {
            Self::Trap => WorldEvent::TrapTriggered,
            Self::Pit => WorldEvent::PitFall,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ActuatorKind {
    Door,
    Trap,
    Boulder,
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
pub(super) struct Food {
    pub(super) cell: Cell,
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
pub(super) struct Pit {
    pub(super) cell: Cell,
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
    width: i32,
    height: i32,
    timing: LevelTiming,
    name: &'static str,
    hero_start: Cell,
    hero_power: i32,
    exit: Cell,
    walls: Vec<Cell>,
    doors: Vec<Door>,
    levers: Vec<Lever>,
    traps: Vec<Trap>,
    pits: Vec<Pit>,
    boulders: Vec<Boulder>,
    foods: Vec<Food>,
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
        19 => level_smell_a_rat(),
        _ => unreachable!("level index is wrapped by LEVEL_COUNT"),
    }
}

fn level_seriously() -> Level {
    let mut walls = vertical_wall(3, &[2, 4]);
    walls.extend(cells(&[(7, 0), (7, 1), (7, 6), (7, 7)]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "SERIOUSLY?",
        hero_start: Cell::new(1, 3),
        hero_power: 10,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![guard(3, 4, 3), guard(3, 2, 15)],
    }
}

fn level_math_is_hard() -> Level {
    let walls = vertical_wall(4, &[3]);

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "MATH IS HARD",
        hero_start: Cell::new(1, 3),
        hero_power: 5,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![fixed_bonus(2, 1, 6)],
        enemies: vec![guard(4, 3, 10)],
    }
}

fn level_pay_the_price() -> Level {
    let mut walls = vertical_wall(3, &[3]);
    walls.extend(vertical_wall(7, &[3]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "PAY THE PRICE",
        hero_start: Cell::new(1, 3),
        hero_power: 10,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![guard(3, 3, 4), guard(7, 3, 5)],
    }
}

fn level_order_matters() -> Level {
    let mut walls = vertical_wall(6, &[4]);
    walls.extend(cells(&[(2, 2), (3, 1), (4, 2), (2, 3), (4, 3)]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "ORDER MATTERS",
        hero_start: Cell::new(1, 4),
        hero_power: 6,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![fixed_bonus(3, 2, 5)],
        enemies: vec![guard(3, 3, 2), guard(6, 4, 8)],
    }
}

fn level_just_leave() -> Level {
    let mut walls = vertical_wall(5, &[1, 4]);
    walls.extend(cells(&[(5, 2), (5, 3), (5, 5), (9, 1), (9, 2)]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "JUST LEAVE",
        hero_start: Cell::new(1, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![guard(5, 4, 99)],
    }
}

fn level_hes_moving() -> Level {
    let mut walls = horizontal_wall(3, &[5]);
    walls.extend(horizontal_wall(5, &[5]));
    walls.extend(cells(&[(5, 1), (5, 6)]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "HE'S MOVING",
        hero_start: Cell::new(1, 4),
        hero_power: 8,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(5, 4, 9, Direction::Up)],
    }
}

fn level_wait_for_it() -> Level {
    let mut walls = horizontal_wall(3, &[5]);
    walls.extend(horizontal_wall(5, &[5]));
    walls.extend(cells(&[(5, 1), (5, 6)]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "WAIT FOR IT",
        hero_start: Cell::new(3, 4),
        hero_power: 7,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
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
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "LET HIM COME",
        hero_start: Cell::new(3, 4),
        hero_power: 12,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
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
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "LUCKY BOY?",
        hero_start: Cell::new(1, 4),
        hero_power: 6,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
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
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
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
        pits: vec![],
        boulders: vec![],
        foods: vec![],
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
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
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
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(3, 2, 99, Direction::Right)],
    }
}

fn level_living_plate_b() -> Level {
    let mut walls = vertical_wall(6, &[4]);
    walls.push(Cell::new(6, 2));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
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
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(3, 2, 99, Direction::Right)],
    }
}

fn level_living_plate_c() -> Level {
    let mut walls = vertical_wall(7, &[4]);
    walls.extend(cells(&[(3, 2), (6, 2)]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
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
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(4, 2, 99, Direction::Right)],
    }
}

fn level_watch_your_step() -> Level {
    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "WATCH YOUR STEP",
        hero_start: Cell::new(1, 4),
        hero_power: 9,
        exit: Cell::new(10, 4),
        walls: vec![],
        doors: vec![],
        levers: vec![],
        traps: vec![active_trap(3, 4)],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![],
    }
}

fn level_set_the_trap() -> Level {
    let mut walls = horizontal_wall(3, &[1]);
    walls.extend(horizontal_wall(5, &[]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "SET THE TRAP",
        hero_start: Cell::new(1, 4),
        hero_power: 5,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(1, 3, 1)],
        traps: vec![group_trap(6, 4, 1)],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(9, 4, 9, Direction::Left)],
    }
}

fn level_clockwork() -> Level {
    let mut walls = horizontal_wall(3, &[]);
    walls.extend(horizontal_wall(5, &[]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::TurnBased,
        name: "CLOCKWORK",
        hero_start: Cell::new(1, 4),
        hero_power: 12,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(2, 4, 1)],
        traps: vec![group_trap(6, 4, 1)],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(8, 4, 9, Direction::Left)],
    }
}

fn level_come_here() -> Level {
    let mut walls = horizontal_wall(3, &[5, 6, 7]);
    walls.extend(horizontal_wall(5, &[5, 6, 7]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::SemiContinuous,
        name: "COME HERE",
        hero_start: Cell::new(3, 4),
        hero_power: 5,
        exit: Cell::new(10, 4),
        walls,
        doors: vec![],
        levers: vec![],
        traps: vec![active_trap(6, 4)],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![walker(9, 4, 9, Direction::Up)],
    }
}

fn level_group_therapy() -> Level {
    let mut walls = horizontal_wall(2, &[5]);
    walls.extend(horizontal_wall(4, &[]));

    Level {
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::SemiContinuous,
        name: "GROUP THERAPY",
        hero_start: Cell::new(5, 3),
        hero_power: 5,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(5, 2, 1)],
        traps: vec![group_trap(7, 3, 1), group_trap(8, 3, 1)],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
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
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
        timing: LevelTiming::SemiContinuous,
        name: "SMART WAY",
        hero_start: Cell::new(4, 3),
        hero_power: 20,
        exit: Cell::new(10, 3),
        walls,
        doors: vec![],
        levers: vec![pressure_plate(5, 3, 1)],
        traps: vec![group_trap(6, 2, 1), group_trap(6, 4, 1)],
        pits: vec![],
        boulders: vec![],
        foods: vec![],
        bonuses: vec![],
        enemies: vec![
            walker(7, 2, 7, Direction::Up),
            walker(7, 4, 7, Direction::Down),
        ],
    }
}

fn level_smell_a_rat() -> Level {
    LevelSpec::parse(include_str!(
        "../../assets/smart_boy_hero/levels/smell_a_rat.json"
    ))
    .expect("SMELL A RAT JSON should parse")
    .into_level()
    .unwrap_or_else(|report| panic!("SMELL A RAT level spec is invalid:\n{report}"))
}

#[allow(dead_code)]
fn level_iso_slice() -> Level {
    let width = 26;
    let height = 18;
    let mut walls = Vec::new();
    for x in 0..width {
        walls.push(Cell::new(x, 0));
        walls.push(Cell::new(x, height - 1));
    }
    for y in 1..height - 1 {
        walls.push(Cell::new(0, y));
        walls.push(Cell::new(width - 1, y));
    }
    for y in 1..height - 1 {
        if ![3, 8, 12].contains(&y) {
            walls.push(Cell::new(7, y));
        }
        if ![8, 12].contains(&y) {
            walls.push(Cell::new(13, y));
        }
        if y != 8 {
            walls.push(Cell::new(21, y));
        }
    }
    for x in 1..7 {
        if x != 3 {
            walls.push(Cell::new(x, 5));
        }
    }
    for x in 1..13 {
        if ![5, 8].contains(&x) {
            walls.push(Cell::new(x, 13));
        }
    }
    walls.extend(cells(&[
        (4, 7),
        (4, 11),
        (9, 6),
        (10, 6),
        (11, 6),
        (15, 5),
        (16, 5),
        (19, 11),
        (23, 5),
        (23, 12),
    ]));

    Level {
        width,
        height,
        timing: LevelTiming::SemiContinuous,
        name: "THE CLOCKWORK KEEP",
        hero_start: Cell::new(2, 9),
        hero_power: 55,
        exit: Cell::new(24, 8),
        walls,
        doors: vec![
            Door {
                cell: Cell::new(7, 3),
                group: 4,
                initially_open: false,
            },
            Door {
                cell: Cell::new(21, 8),
                group: 2,
                initially_open: false,
            },
        ],
        levers: vec![
            lever(5, 3, 4),
            pressure_plate(10, 12, 1),
            pressure_plate(15, 12, 5),
        ],
        traps: vec![
            group_trap(9, 11, 1),
            group_trap(11, 12, 1),
            group_trap(10, 14, 1),
            group_trap(17, 9, 1),
        ],
        pits: vec![],
        boulders: vec![boulder(14, 8, Direction::Right, 5)],
        foods: vec![],
        bonuses: vec![fixed_bonus(3, 3, 12)],
        enemies: vec![
            walker(5, 8, 8, Direction::Right),
            walker(6, 10, 8, Direction::Left),
            guard(6, 7, 14),
            walker(9, 10, 9, Direction::Down),
            walker(12, 14, 9, Direction::Left),
            guard(12, 12, 16),
            key_warden(17, 8, 34, Direction::Right),
            walker(19, 8, 9, Direction::Left),
            walker(20, 9, 9, Direction::Up),
            walker(18, 10, 9, Direction::Right),
            guard(17, 12, 18),
            walker(23, 7, 10, Direction::Down),
            guard(23, 9, 20),
            walker(23, 10, 10, Direction::Up),
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

fn level_in_bounds(level: &Level, cell: Cell) -> bool {
    cell.x >= 0 && cell.y >= 0 && cell.x < level.width && cell.y < level.height
}

fn guard(x: i32, y: i32, power: i32) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power,
        kind: EnemyKind::Guard,
        role: EnemyRole::Normal,
        intent: EnemyIntent::Patrol,
    }
}

fn walker(x: i32, y: i32, power: i32, direction: Direction) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power,
        kind: EnemyKind::Walker { direction },
        role: EnemyRole::Normal,
        intent: EnemyIntent::Patrol,
    }
}

fn key_warden(x: i32, y: i32, power: i32, direction: Direction) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power,
        kind: EnemyKind::Walker { direction },
        role: EnemyRole::KeyWarden,
        intent: EnemyIntent::Patrol,
    }
}

#[cfg(test)]
fn rat(x: i32, y: i32) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power: 1,
        kind: EnemyKind::Rat,
        role: EnemyRole::Normal,
        intent: EnemyIntent::Patrol,
    }
}

#[cfg(test)]
fn cat(x: i32, y: i32) -> Enemy {
    Enemy {
        cell: Cell::new(x, y),
        power: 2,
        kind: EnemyKind::Cat,
        role: EnemyRole::Normal,
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

#[cfg(test)]
fn food(x: i32, y: i32) -> Food {
    Food {
        cell: Cell::new(x, y),
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

#[cfg(test)]
fn pit(x: i32, y: i32) -> Pit {
    Pit {
        cell: Cell::new(x, y),
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
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
            timing: LevelTiming::TurnBased,
            name: "TEST",
            hero_start: Cell::new(1, 1),
            hero_power,
            exit: Cell::new(10, 1),
            walls: vec![],
            doors: vec![],
            levers: vec![],
            traps: vec![],
            pits: vec![],
            boulders: vec![],
            foods: vec![],
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
            foods: level.foods.clone(),
            bonuses: level.bonuses.clone(),
            latched_doors_open: vec![false; level.doors.len()],
            doors_open: vec![false; level.doors.len()],
            latched_traps_active: vec![false; level.traps.len()],
            traps_active: vec![false; level.traps.len()],
            pressure_plates_active: vec![false; level.levers.len()],
            boulders: level.boulders.clone(),
            core_key_cell: None,
            has_core_key: false,
            core_gate: None,
            core_gate_open: false,
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
                if !level_in_bounds(&level, next)
                    || visited.contains(&next)
                    || statically_blocked(&level, next)
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
                if !level_in_bounds(level, next)
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

    fn static_exit_reachable(level: &Level) -> bool {
        static_exit_reachable_avoiding_cells(level, &[])
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
                if !level_in_bounds(level, next)
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

    fn throw_rock(cell: Cell) -> PlayerAction {
        PlayerAction::ThrowRock(cell)
    }

    fn walk_to_iso_boulder_plate(world: &mut SmartBoyWorld) {
        world.apply(up());
        for _ in 0..12 {
            world.apply(right());
        }
        for action in [down(), down(), down(), down(), right()] {
            world.apply(action);
        }
    }

    fn finish_iso_from_boulder_plate(world: &mut SmartBoyWorld) {
        world.update_tick();
        for _ in 0..4 {
            world.apply(up());
        }
        for _ in 0..9 {
            world.apply(right());
        }
    }

    fn core_gate_world() -> SmartBoyWorld {
        let mut level = test_level(20);
        level.doors.push(Door {
            cell: Cell::new(2, 1),
            group: 9,
            initially_open: false,
        });
        let mut world = world_from(level);
        world.core_gate = Some(Cell::new(2, 1));
        world
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

        walk_to_iso_boulder_plate(&mut world);
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
            walk_to_iso_boulder_plate(world);
            world.update_tick();
            world.update_tick();
            world.update_tick();
        }

        assert_eq!(first, second);
    }

    #[test]
    fn iso_slice_can_produce_boulder_multi_kill_after_rock_setup() {
        let mut world = SmartBoyWorld::iso_slice(7);
        let mut boulder_chain = 0;

        world.apply(throw_rock(Cell::new(5, 9)));
        walk_to_iso_boulder_plate(&mut world);
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
    fn iso_chapter_initial_state_blocks_trivial_core_route() {
        let level = level_iso_slice();

        assert!(!static_exit_reachable(&level));
    }

    #[test]
    fn iso_chapter_controlled_traps_start_inactive() {
        let world = SmartBoyWorld::iso_slice(7);

        assert!((0..world.traps().len()).all(|index| !world.trap_active(index)));
    }

    #[test]
    fn pressure_plate_transition_activates_controlled_trap() {
        let mut level = semi_test_level(10);
        level.levers.push(pressure_plate(2, 1, 7));
        level.traps.push(group_trap(3, 1, 7));
        let mut world = world_from(level);

        assert!(!world.trap_active(0));
        let report = world.apply(right());

        assert!(world.trap_active(0));
        assert!(report.events.contains(&WorldEvent::PressurePlateOn));
        assert!(report.events.contains(&WorldEvent::TrapArmed));
    }

    #[test]
    fn iso_chapter_reference_route_reaches_clockwork_core() {
        let mut world = SmartBoyWorld::iso_slice(7);

        walk_to_iso_boulder_plate(&mut world);
        finish_iso_from_boulder_plate(&mut world);

        assert_eq!(world.phase(), Phase::Won);
        assert!(world.hero_power() > 0);
    }

    #[test]
    fn iso_chapter_boulder_kill_then_walk_onto_dropped_key_acquires_it() {
        let mut world = SmartBoyWorld::iso_slice(7);

        walk_to_iso_boulder_plate(&mut world);
        let drop = world.update_tick();
        let key_cell = world
            .core_key_cell()
            .expect("Clockwork Keep Warden should drop a core key");

        assert!(
            drop.events
                .contains(&WorldEvent::CoreKeyDropped { cell: key_cell })
        );
        assert!(!world.has_core_key());

        for _ in 0..4 {
            world.apply(up());
        }
        while world.hero() != key_cell {
            let direction = if world.hero().x < key_cell.x {
                Direction::Right
            } else if world.hero().x > key_cell.x {
                Direction::Left
            } else if world.hero().y < key_cell.y {
                Direction::Down
            } else {
                Direction::Up
            };
            let report = world.apply(PlayerAction::Move(direction));
            if world.hero() == key_cell {
                assert!(report.events.contains(&WorldEvent::CoreKeyAcquired));
            }
        }

        assert!(world.has_core_key());
        assert_eq!(world.core_key_cell(), None);
    }

    #[test]
    fn iso_chapter_warden_initially_holds_progression_key() {
        let world = SmartBoyWorld::iso_slice(7);
        let warden = world
            .enemies()
            .iter()
            .find(|enemy| matches!(enemy.role, EnemyRole::KeyWarden))
            .expect("Clockwork Keep should contain a Key Warden");

        assert_eq!(warden.cell, Cell::new(17, 8));
        assert_eq!(warden.power, 34);
        assert_eq!(world.core_key_cell(), None);
        assert!(!world.has_core_key());
        assert!(!world.door_open(1));
    }

    #[test]
    fn direct_killing_key_warden_drops_and_acquires_key_on_entered_cell() {
        let mut level = test_level(50);
        level.enemies.push(key_warden(2, 1, 10, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(right());

        assert!(report.events.contains(&WorldEvent::CoreKeyDropped {
            cell: Cell::new(2, 1),
        }));
        assert!(report.events.contains(&WorldEvent::CoreKeyAcquired));
        assert_eq!(world.hero(), Cell::new(2, 1));
        assert!(world.enemy_at(Cell::new(2, 1)).is_none());
        assert!(world.has_core_key());
        assert_eq!(world.core_key_cell(), None);
    }

    #[test]
    fn trap_killing_key_warden_drops_recoverable_key() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(1, 3);
        level.traps.push(active_trap(3, 1));
        level.enemies.push(key_warden(2, 1, 34, Direction::Right));
        let mut world = world_from(level);

        let report = world.update_tick();
        let key_cell = world
            .core_key_cell()
            .expect("trap-killed Warden should drop the core key");

        assert!(
            report
                .events
                .contains(&WorldEvent::CoreKeyDropped { cell: key_cell })
        );
        assert_ne!(key_cell, Cell::new(3, 1));
        assert!(world.active_trap_at(key_cell).is_none());
        assert!(world.enemy_at(Cell::new(3, 1)).is_none());
        assert!(!world.has_core_key());

        for action in [right(), right(), right(), up()] {
            world.apply(action);
        }
        let pickup = world.apply(up());

        assert_eq!(world.hero(), key_cell);
        assert!(pickup.events.contains(&WorldEvent::CoreKeyAcquired));
        assert!(world.has_core_key());
        assert_eq!(world.core_key_cell(), None);
    }

    #[test]
    fn boulder_killing_key_warden_drops_recoverable_key() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(1, 3);
        level.enemies.push(key_warden(3, 1, 34, Direction::Right));
        level.boulders.push(Boulder {
            cell: Cell::new(2, 1),
            group: 1,
            direction: Direction::Right,
            state: BoulderState::Rolling { kills: 0 },
        });
        let mut world = world_from(level);

        let report = world.update_tick();
        let key_cell = world
            .core_key_cell()
            .expect("boulder-killed Warden should drop the core key");

        assert!(
            report
                .events
                .contains(&WorldEvent::CoreKeyDropped { cell: key_cell })
        );
        assert_ne!(key_cell, Cell::new(3, 1));
        assert!(world.enemy_at(Cell::new(3, 1)).is_none());
        assert!(
            !world
                .boulders()
                .iter()
                .any(|boulder| boulder.cell == key_cell)
        );
        assert!(!world.has_core_key());

        for action in [right(), right(), right(), up()] {
            world.apply(action);
        }
        let pickup = world.apply(up());

        assert_eq!(world.hero(), key_cell);
        assert!(pickup.events.contains(&WorldEvent::CoreKeyAcquired));
        assert!(world.has_core_key());
        assert_eq!(world.core_key_cell(), None);
    }

    #[test]
    fn key_warden_corpse_never_blocks_drop_cell() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(1, 3);
        level.traps.push(active_trap(3, 1));
        level.enemies.push(key_warden(2, 1, 34, Direction::Right));
        let mut world = world_from(level);

        world.update_tick();
        let key_cell = world
            .core_key_cell()
            .expect("Warden death should leave a key cell");

        assert!(world.enemy_at(Cell::new(3, 1)).is_none());
        assert!(world.enemy_at(key_cell).is_none());

        for action in [right(), right(), right(), up()] {
            world.apply(action);
        }
        let pickup = world.apply(up());

        assert_eq!(world.hero(), key_cell);
        assert!(pickup.events.contains(&WorldEvent::CoreKeyAcquired));
        assert!(world.has_core_key());
    }

    #[test]
    fn locked_core_gate_refuses_without_key_and_unlocks_with_key() {
        let mut world = core_gate_world();

        let locked = world.apply(right());
        assert!(locked.events.contains(&WorldEvent::LockedGateBlocked));
        assert_eq!(world.hero(), Cell::new(1, 1));
        assert!(!world.door_open(0));

        world.has_core_key = true;
        let opened = world.apply(right());

        assert!(opened.events.contains(&WorldEvent::CoreGateUnlocked));
        assert!(opened.events.contains(&WorldEvent::DoorOpened));
        assert_eq!(world.hero(), Cell::new(2, 1));
        assert!(world.door_open(0));
    }

    #[test]
    fn key_warden_drop_is_not_duplicated() {
        let mut level = test_level(50);
        level.enemies.push(key_warden(2, 1, 10, Direction::Right));
        let mut world = world_from(level);

        let first = world.apply(right());
        world.apply(left());
        let second = world.apply(right());

        assert_eq!(
            first
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::CoreKeyDropped { .. }))
                .count(),
            1
        );
        assert_eq!(
            second
                .events
                .iter()
                .filter(|event| matches!(event, WorldEvent::CoreKeyDropped { .. }))
                .count(),
            0
        );
    }

    #[test]
    fn restart_restores_warden_key_and_core_gate() {
        let mut world = SmartBoyWorld::iso_slice(7);

        walk_to_iso_boulder_plate(&mut world);
        finish_iso_from_boulder_plate(&mut world);
        assert_eq!(world.phase(), Phase::Won);
        assert!(world.has_core_key());
        assert!(world.door_open(1));

        world.restart();

        assert_eq!(world.core_key_cell(), None);
        assert!(!world.has_core_key());
        assert!(!world.door_open(1));
        assert!(
            world
                .enemies()
                .iter()
                .any(|enemy| matches!(enemy.role, EnemyRole::KeyWarden))
        );
    }

    #[test]
    fn iso_chapter_side_room_shortcut_persists_after_leaving() {
        let mut world = SmartBoyWorld::iso_slice(7);

        world.apply(right());
        for _ in 0..6 {
            world.apply(up());
        }
        world.apply(right());
        world.apply(right());
        assert!(world.door_open(0));

        world.apply(left());
        world.apply(down());

        assert!(world.door_open(0));
    }

    #[test]
    fn iso_chapter_enemy_death_persists_after_returning_to_entrance() {
        let mut world = SmartBoyWorld::iso_slice(7);
        let killed_cell = Cell::new(5, 8);

        world.apply(up());
        world.apply(right());
        world.apply(right());
        world.apply(right());
        assert!(
            !world
                .enemies()
                .iter()
                .any(|enemy| enemy.cell == killed_cell)
        );

        world.apply(left());
        world.apply(left());
        world.update_tick();

        assert!(
            !world
                .enemies()
                .iter()
                .any(|enemy| enemy.cell == killed_cell)
        );
    }

    #[test]
    fn iso_chapter_boulder_state_persists_after_leaving_yard() {
        let mut world = SmartBoyWorld::iso_slice(7);

        walk_to_iso_boulder_plate(&mut world);
        assert_ne!(world.boulders()[0].state, BoulderState::Ready);

        world.apply(left());
        world.apply(down());
        world.update_tick();

        assert_ne!(world.boulders()[0].state, BoulderState::Ready);
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
            role: EnemyRole::Normal,
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
    fn rock_target_inside_range_is_accepted() {
        let world = world_from(test_level(10));

        assert!(world.can_throw_rock_to(Cell::new(4, 4)));
    }

    #[test]
    fn rock_target_outside_range_is_rejected() {
        let world = world_from(test_level(10));

        assert!(!world.can_throw_rock_to(Cell::new(8, 5)));
    }

    #[test]
    fn throw_rock_outside_range_does_not_consume_turn() {
        let mut world = world_from(test_level(10));

        let report = world.apply(throw_rock(Cell::new(8, 5)));

        assert!(!report.turn_consumed);
        assert!(report.events.contains(&WorldEvent::Blocked));
        assert_eq!(world.turn_count(), 0);
    }

    #[test]
    fn throw_rock_noise_comes_from_target_not_hero() {
        let mut level = test_level(10);
        level.enemies.push(walker(7, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(throw_rock(Cell::new(4, 1)));

        assert!(report.events.contains(&WorldEvent::RockImpacted {
            cell: Cell::new(4, 1),
            heard: 1,
        }));
        assert_eq!(
            world.enemies()[0].intent,
            EnemyIntent::Investigate {
                target: Cell::new(4, 1),
                patrol_direction: Direction::Right,
            }
        );
    }

    #[test]
    fn throw_rock_walker_outside_hearing_radius_ignores_impact() {
        let mut level = test_level(10);
        level.enemies.push(walker(8, 1, 9, Direction::Right));
        let mut world = world_from(level);

        let report = world.apply(throw_rock(Cell::new(4, 1)));

        assert_eq!(
            report
                .events
                .iter()
                .find(|event| matches!(event, WorldEvent::RockImpacted { .. })),
            Some(&WorldEvent::RockImpacted {
                cell: Cell::new(4, 1),
                heard: 0,
            })
        );
        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
    }

    #[test]
    fn throw_rock_can_redirect_multiple_walkers() {
        let mut level = test_level(10);
        level.enemies.push(walker(6, 1, 9, Direction::Right));
        level.enemies.push(walker(4, 3, 9, Direction::Left));
        let mut world = world_from(level);

        let report = world.apply(throw_rock(Cell::new(4, 1)));

        assert!(report.events.contains(&WorldEvent::RockImpacted {
            cell: Cell::new(4, 1),
            heard: 2,
        }));
        assert!(world.enemies().iter().all(|enemy| matches!(
            enemy.intent,
            EnemyIntent::Investigate {
                target: Cell { x: 4, y: 1 },
                ..
            }
        )));
    }

    #[test]
    fn throw_rock_bfs_targets_rock_cell_deterministically() {
        let mut level = semi_test_level(10);
        level.walls.push(Cell::new(5, 1));
        level.enemies.push(walker(6, 1, 9, Direction::Right));
        let mut world = world_from(level);

        world.apply(throw_rock(Cell::new(4, 1)));
        for _ in 0..4 {
            world.update_tick();
        }

        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
        assert_eq!(world.enemies()[0].intent, EnemyIntent::Patrol);
    }

    #[test]
    fn adjacent_hero_still_takes_priority_over_rock_target() {
        let mut level = semi_test_level(10);
        level.hero_start = Cell::new(4, 1);
        level.enemies.push(walker(5, 1, 2, Direction::Right));
        let mut world = world_from(level);

        world.apply(throw_rock(Cell::new(7, 1)));
        let report = world.update_tick();

        assert!(report.events.contains(&WorldEvent::WalkerSpottedHero));
        assert!(
            report
                .events
                .contains(&WorldEvent::WalkerDestroyed { power: 2 })
        );
        assert!(world.enemies().is_empty());
    }

    #[test]
    fn restart_restores_throw_rock_investigation_state() {
        let mut world = SmartBoyWorld::iso_slice(42);
        let initial = world.clone();

        world.apply(throw_rock(Cell::new(5, 9)));
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
    fn rat_moves_toward_food_and_eats_it() {
        let mut level = semi_test_level(10);
        level.enemies.push(rat(3, 1));
        level.foods.push(food(5, 1));
        let mut world = world_from(level);

        let smelled = world.update_tick();
        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
        assert!(smelled.events.contains(&WorldEvent::RatSmelledFood));

        let eaten = world.update_tick();
        assert_eq!(world.enemies()[0].cell, Cell::new(5, 1));
        assert!(world.foods().is_empty());
        assert!(eaten.events.contains(&WorldEvent::FoodEaten));
    }

    #[test]
    fn nearby_cat_makes_rat_flee_before_food() {
        let mut level = semi_test_level(10);
        level.enemies.push(rat(3, 1));
        level.enemies.push(cat(2, 1));
        level.foods.push(food(5, 1));
        let mut world = world_from(level);

        let report = world.update_tick();

        assert_eq!(world.enemies()[0].cell, Cell::new(4, 1));
        assert!(report.events.contains(&WorldEvent::RatScared));
        assert!(!report.events.contains(&WorldEvent::RatSmelledFood));
    }

    #[test]
    fn pit_kills_rat_that_flees_into_it() {
        let mut level = semi_test_level(10);
        level.enemies.push(rat(3, 1));
        level.enemies.push(cat(2, 1));
        level.pits.push(pit(4, 1));
        let mut world = world_from(level);

        let report = world.update_tick();

        assert!(
            world
                .enemies()
                .iter()
                .all(|enemy| enemy.kind != EnemyKind::Rat)
        );
        assert!(report.events.contains(&WorldEvent::PitFall));
        assert!(report.events.contains(&WorldEvent::EnemyKilled {
            cell: Cell::new(4, 1),
            power: 1,
        }));
    }

    #[test]
    fn rat_on_food_plate_temporarily_opens_matching_door() {
        let mut level = semi_test_level(10);
        level.levers.push(pressure_plate(5, 1, 7));
        level.doors.push(Door {
            cell: Cell::new(7, 1),
            group: 7,
            initially_open: false,
        });
        level.enemies.push(rat(3, 1));
        level.foods.push(food(5, 1));
        let mut world = world_from(level);

        world.update_tick();
        let opened = world.update_tick();

        assert!(world.door_open(0));
        assert!(opened.events.contains(&WorldEvent::PressurePlateOn));
        assert!(opened.events.contains(&WorldEvent::DoorOpened));
        assert!(opened.events.contains(&WorldEvent::FoodEaten));
    }

    #[test]
    fn level_twenty_combines_cat_fear_pit_food_and_plate() {
        let mut world = SmartBoyWorld::for_level(19, 0xB0A);

        let pit = world.update_tick();
        assert!(pit.events.contains(&WorldEvent::PitFall));
        assert!(pit.events.contains(&WorldEvent::RatScared));

        world.update_tick();
        let opened = world.update_tick();

        assert!(world.door_open(0));
        assert!(opened.events.contains(&WorldEvent::PressurePlateOn));
        assert!(opened.events.contains(&WorldEvent::DoorOpened));
        assert!(opened.events.contains(&WorldEvent::FoodEaten));
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
            assert!(level_in_bounds(&level, level.hero_start));
            assert!(level_in_bounds(&level, level.exit));
            assert!(
                level
                    .walls
                    .iter()
                    .copied()
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .doors
                    .iter()
                    .map(|door| door.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .levers
                    .iter()
                    .map(|lever| lever.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .traps
                    .iter()
                    .map(|trap| trap.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .pits
                    .iter()
                    .map(|pit| pit.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .foods
                    .iter()
                    .map(|food| food.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .bonuses
                    .iter()
                    .map(|bonus| bonus.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
            assert!(
                level
                    .enemies
                    .iter()
                    .map(|enemy| enemy.cell)
                    .all(|cell| level_in_bounds(&level, cell))
            );
        }
    }
}
