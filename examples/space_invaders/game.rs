use std::time::Duration;

use gotoo_pixel_engine::{Frame, Framebuffer, Game, GameResult, Key, Pixel};

pub const FRAMEBUFFER_WIDTH: u32 = 256;
pub const FRAMEBUFFER_HEIGHT: u32 = 224;

const PLAYER_Y: i32 = 198;
const PLAYER_W: i32 = 13;
const PLAYER_SPEED: f32 = 120.0;
const PLAYER_BULLET_SPEED: f32 = 190.0;
const ENEMY_BULLET_SPEED: f32 = 72.0;

const ALIEN_COLS: usize = 11;
const ALIEN_ROWS: usize = 5;
const ALIEN_W: i32 = 10;
const ALIEN_H: i32 = 7;
const ALIEN_X_SPACING: i32 = 16;
const ALIEN_Y_SPACING: i32 = 12;
const ALIEN_STEP_X: i32 = 3;
const ALIEN_STEP_Y: i32 = 8;

const BUNKER_COUNT: usize = 4;
const BUNKER_COLS: usize = 14;
const BUNKER_ROWS: usize = 8;
const BUNKER_CELL: i32 = 2;
const BUNKER_Y: i32 = 166;
const BUNKER_MASK: [u16; BUNKER_ROWS] = [
    0b00111111111100,
    0b01111111111110,
    0b11111111111111,
    0b11111111111111,
    0b11111111111111,
    0b11111000011111,
    0b11110000001111,
    0b11110000001111,
];

const BG: Pixel = Pixel::rgb(4, 8, 8);
const FOREGROUND: Pixel = Pixel::rgb(120, 255, 120);
const TEXT: Pixel = Pixel::rgb(220, 240, 220);
const SHOT: Pixel = Pixel::rgb(245, 245, 230);
const DANGER: Pixel = Pixel::rgb(255, 95, 80);

const PLAYER_MASK: [u16; 7] = [
    0b0000001000000,
    0b0000011100000,
    0b0000011100000,
    0b0011111111100,
    0b0111111111110,
    0b1111111111111,
    0b1111111111111,
];

const ALIEN_MASKS: [[[u16; 7]; 2]; 3] = [
    [
        [
            0b0001100000,
            0b0011110000,
            0b0111111000,
            0b1101101100,
            0b1111111100,
            0b0010010000,
            0b0100001000,
        ],
        [
            0b0001100000,
            0b0011110000,
            0b0111111000,
            0b1101101100,
            0b1111111100,
            0b0101101000,
            0b1000000100,
        ],
    ],
    [
        [
            0b0010010000,
            0b0001100000,
            0b0011110000,
            0b0111111000,
            0b1101101100,
            0b1111111100,
            0b0100001000,
        ],
        [
            0b0010010000,
            0b1001100100,
            0b1011110100,
            0b1111111100,
            0b0111111000,
            0b0010010000,
            0b0100001000,
        ],
    ],
    [
        [
            0b0001100000,
            0b0111111000,
            0b1111111100,
            0b1101101100,
            0b1111111100,
            0b0010010000,
            0b0101101000,
        ],
        [
            0b0001100000,
            0b0111111000,
            0b1111111100,
            0b1101101100,
            0b1111111100,
            0b0101101000,
            0b1010010100,
        ],
    ],
];

#[derive(Debug, Clone, Copy)]
struct Alien {
    row: usize,
    col: usize,
    alive: bool,
}

#[derive(Debug, Clone, Copy)]
struct Bullet {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct Bunker {
    x: i32,
    cells: [[bool; BUNKER_COLS]; BUNKER_ROWS],
}

impl Bunker {
    fn new(x: i32) -> Self {
        let mut cells = [[false; BUNKER_COLS]; BUNKER_ROWS];
        for y in 0..BUNKER_ROWS {
            for x in 0..BUNKER_COLS {
                cells[y][x] = BUNKER_MASK[y] & (1u16 << x) != 0;
            }
        }
        Self { x, cells }
    }

    fn damage_at(&mut self, px: i32, py: i32) -> bool {
        let local_x = px - self.x;
        let local_y = py - BUNKER_Y;
        if local_x < 0 || local_y < 0 {
            return false;
        }

        let cell_x = local_x / BUNKER_CELL;
        let cell_y = local_y / BUNKER_CELL;
        if cell_x >= BUNKER_COLS as i32 || cell_y >= BUNKER_ROWS as i32 {
            return false;
        }
        if !self.cells[cell_y as usize][cell_x as usize] {
            return false;
        }

        for dy in -1..=1 {
            for dx in -1..=1 {
                let x = cell_x + dx;
                let y = cell_y + dy;
                if (0..BUNKER_COLS as i32).contains(&x)
                    && (0..BUNKER_ROWS as i32).contains(&y)
                {
                    self.cells[y as usize][x as usize] = false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundState {
    Playing,
    Victory,
    GameOver,
}

#[derive(Debug, Clone)]
struct SpaceInvadersWorld {
    aliens: Vec<Alien>,
    formation_x: i32,
    formation_y: i32,
    formation_direction: i32,
    animation_frame: usize,
    movement_accumulator: Duration,
    enemy_fire_accumulator: Duration,
    player_x: f32,
    player_bullet: Option<Bullet>,
    enemy_bullets: Vec<Bullet>,
    bunkers: Vec<Bunker>,
    score: u32,
    lives: u32,
    state: RoundState,
    rng: u32,
}

impl SpaceInvadersWorld {
    fn new() -> Self {
        let mut aliens = Vec::with_capacity(ALIEN_COLS * ALIEN_ROWS);
        for row in 0..ALIEN_ROWS {
            for col in 0..ALIEN_COLS {
                aliens.push(Alien {
                    row,
                    col,
                    alive: true,
                });
            }
        }

        let bunker_x = [24, 82, 140, 198];
        let bunkers = bunker_x.into_iter().map(Bunker::new).collect();

        Self {
            aliens,
            formation_x: 32,
            formation_y: 32,
            formation_direction: 1,
            animation_frame: 0,
            movement_accumulator: Duration::ZERO,
            enemy_fire_accumulator: Duration::ZERO,
            player_x: (FRAMEBUFFER_WIDTH as i32 / 2 - PLAYER_W / 2) as f32,
            player_bullet: None,
            enemy_bullets: Vec::new(),
            bunkers,
            score: 0,
            lives: 3,
            state: RoundState::Playing,
            rng: 0x51A7_1A0D,
        }
    }

    fn restart(&mut self) {
        *self = Self::new();
    }

    fn alive_count(&self) -> usize {
        self.aliens.iter().filter(|alien| alien.alive).count()
    }

    fn movement_period(&self) -> Duration {
        let alive = self.alive_count() as u64;
        Duration::from_millis(70 + alive * 10)
    }

    fn enemy_fire_period(&self) -> Duration {
        let alive = self.alive_count() as u64;
        Duration::from_millis(340 + alive * 7)
    }

    fn alien_position(&self, alien: Alien) -> (i32, i32) {
        (
            self.formation_x + alien.col as i32 * ALIEN_X_SPACING,
            self.formation_y + alien.row as i32 * ALIEN_Y_SPACING,
        )
    }

    fn move_formation(&mut self) {
        if self.state != RoundState::Playing {
            return;
        }

        let next_x = self.formation_x + self.formation_direction * ALIEN_STEP_X;
        let touches_edge = self.aliens.iter().copied().filter(|alien| alien.alive).any(|alien| {
            let x = next_x + alien.col as i32 * ALIEN_X_SPACING;
            x <= 7 || x + ALIEN_W >= FRAMEBUFFER_WIDTH as i32 - 7
        });

        if touches_edge {
            self.formation_direction *= -1;
            self.formation_y += ALIEN_STEP_Y;
        } else {
            self.formation_x = next_x;
        }
        self.animation_frame ^= 1;

        if self.aliens.iter().copied().filter(|alien| alien.alive).any(|alien| {
            let (_, y) = self.alien_position(alien);
            y + ALIEN_H >= BUNKER_Y + BUNKER_ROWS as i32 * BUNKER_CELL
        }) {
            self.state = RoundState::GameOver;
        }
    }

    fn shoot_player(&mut self) {
        if self.state == RoundState::Playing && self.player_bullet.is_none() {
            self.player_bullet = Some(Bullet {
                x: self.player_x + (PLAYER_W / 2) as f32,
                y: (PLAYER_Y - 2) as f32,
            });
        }
    }

    fn shoot_enemy(&mut self) {
        if self.state != RoundState::Playing || self.enemy_bullets.len() >= 4 {
            return;
        }

        let mut shooters = Vec::with_capacity(ALIEN_COLS);
        for col in 0..ALIEN_COLS {
            if let Some(alien) = self
                .aliens
                .iter()
                .copied()
                .filter(|alien| alien.alive && alien.col == col)
                .max_by_key(|alien| alien.row)
            {
                shooters.push(alien);
            }
        }
        if shooters.is_empty() {
            return;
        }

        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let shooter = shooters[self.rng as usize % shooters.len()];
        let (x, y) = self.alien_position(shooter);
        self.enemy_bullets.push(Bullet {
            x: (x + ALIEN_W / 2) as f32,
            y: (y + ALIEN_H) as f32,
        });
    }

    fn hit_bunker(&mut self, x: i32, y: i32) -> bool {
        self.bunkers.iter_mut().any(|bunker| bunker.damage_at(x, y))
    }

    fn update_projectiles(&mut self, dt: f32) {
        if let Some(bullet) = &mut self.player_bullet {
            bullet.y -= PLAYER_BULLET_SPEED * dt;
        }

        if let Some(bullet) = self.player_bullet {
            let px = bullet.x.round() as i32;
            let py = bullet.y.round() as i32;
            if py < 14 || self.hit_bunker(px, py) {
                self.player_bullet = None;
            } else {
                let mut hit = None;
                for index in 0..self.aliens.len() {
                    let alien = self.aliens[index];
                    if !alien.alive {
                        continue;
                    }
                    let (x, y) = self.alien_position(alien);
                    if (x..x + ALIEN_W).contains(&px) && (y..y + ALIEN_H).contains(&py) {
                        hit = Some(index);
                        break;
                    }
                }

                if let Some(index) = hit {
                    let row = self.aliens[index].row;
                    self.aliens[index].alive = false;
                    self.score = self.score.saturating_add(match row {
                        0 => 30,
                        1 | 2 => 20,
                        _ => 10,
                    });
                    self.player_bullet = None;
                    if self.alive_count() == 0 {
                        self.state = RoundState::Victory;
                    }
                }
            }
        }

        for bullet in &mut self.enemy_bullets {
            bullet.y += ENEMY_BULLET_SPEED * dt;
        }

        let mut player_hit = false;
        let mut index = self.enemy_bullets.len();
        while index > 0 {
            index -= 1;
            let bullet = self.enemy_bullets[index];
            let px = bullet.x.round() as i32;
            let py = bullet.y.round() as i32;

            let remove = py >= FRAMEBUFFER_HEIGHT as i32 - 8
                || self.hit_bunker(px, py)
                || ((self.player_x as i32..self.player_x as i32 + PLAYER_W).contains(&px)
                    && (PLAYER_Y..PLAYER_Y + 7).contains(&py));

            if remove {
                if (self.player_x as i32..self.player_x as i32 + PLAYER_W).contains(&px)
                    && (PLAYER_Y..PLAYER_Y + 7).contains(&py)
                {
                    player_hit = true;
                }
                self.enemy_bullets.swap_remove(index);
            }
        }

        if player_hit {
            self.lives = self.lives.saturating_sub(1);
            self.player_x = (FRAMEBUFFER_WIDTH as i32 / 2 - PLAYER_W / 2) as f32;
            self.player_bullet = None;
            self.enemy_bullets.clear();
            if self.lives == 0 {
                self.state = RoundState::GameOver;
            }
        }
    }

    fn tick(&mut self, dt: Duration) {
        if self.state != RoundState::Playing {
            return;
        }

        self.movement_accumulator = self.movement_accumulator.saturating_add(dt);
        while self.movement_accumulator >= self.movement_period() {
            self.movement_accumulator -= self.movement_period();
            self.move_formation();
            if self.state != RoundState::Playing {
                break;
            }
        }

        self.enemy_fire_accumulator = self.enemy_fire_accumulator.saturating_add(dt);
        if self.enemy_fire_accumulator >= self.enemy_fire_period() {
            self.enemy_fire_accumulator = Duration::ZERO;
            self.shoot_enemy();
        }

        self.update_projectiles(dt.as_secs_f32());
    }
}

#[derive(Debug)]
pub struct SpaceInvadersGame {
    world: SpaceInvadersWorld,
}

impl SpaceInvadersGame {
    pub fn new() -> Self {
        Self {
            world: SpaceInvadersWorld::new(),
        }
    }

    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        if frame.input.key(Key::Escape).pressed() {
            return GameResult::Exit;
        }

        if self.world.state != RoundState::Playing {
            if frame.input.key(Key::Space).pressed() {
                self.world.restart();
            }
            return GameResult::Continue;
        }

        let dt = frame.delta_time.as_secs_f32();
        let left = frame.input.key(Key::Left).held() || frame.input.key(Key::A).held();
        let right = frame.input.key(Key::Right).held() || frame.input.key(Key::D).held();
        if left && !right {
            self.world.player_x -= PLAYER_SPEED * dt;
        } else if right && !left {
            self.world.player_x += PLAYER_SPEED * dt;
        }
        self.world.player_x = self
            .world
            .player_x
            .clamp(8.0, (FRAMEBUFFER_WIDTH as i32 - PLAYER_W - 8) as f32);

        if frame.input.key(Key::Space).pressed() {
            self.world.shoot_player();
        }

        GameResult::Continue
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let fb = &mut frame.framebuffer;
        fb.clear(BG);

        fb.draw_text(8, 5, "SCORE", TEXT);
        fb.draw_text(43, 5, &self.world.score.to_string(), TEXT);
        fb.draw_text(174, 5, "LIVES", TEXT);
        fb.draw_text(212, 5, &self.world.lives.to_string(), TEXT);
        fb.fill_rect(8, 18, FRAMEBUFFER_WIDTH - 16, 1, FOREGROUND);

        for alien in self.world.aliens.iter().copied().filter(|alien| alien.alive) {
            let (x, y) = self.world.alien_position(alien);
            let kind = match alien.row {
                0 => 0,
                1 | 2 => 1,
                _ => 2,
            };
            draw_mask(
                fb,
                x,
                y,
                &ALIEN_MASKS[kind][self.world.animation_frame],
                10,
                FOREGROUND,
            );
        }

        for bunker in &self.world.bunkers {
            for y in 0..BUNKER_ROWS {
                for x in 0..BUNKER_COLS {
                    if bunker.cells[y][x] {
                        fb.fill_rect(
                            bunker.x + x as i32 * BUNKER_CELL,
                            BUNKER_Y + y as i32 * BUNKER_CELL,
                            BUNKER_CELL as u32,
                            BUNKER_CELL as u32,
                            FOREGROUND,
                        );
                    }
                }
            }
        }

        if self.world.lives > 0 {
            draw_mask(
                fb,
                self.world.player_x.round() as i32,
                PLAYER_Y,
                &PLAYER_MASK,
                PLAYER_W as usize,
                FOREGROUND,
            );
        }

        if let Some(bullet) = self.world.player_bullet {
            fb.fill_rect(bullet.x.round() as i32, bullet.y.round() as i32, 1, 4, SHOT);
        }
        for bullet in &self.world.enemy_bullets {
            fb.fill_rect(bullet.x.round() as i32, bullet.y.round() as i32, 2, 4, DANGER);
        }

        fb.fill_rect(8, 211, FRAMEBUFFER_WIDTH - 16, 1, FOREGROUND);

        match self.world.state {
            RoundState::Playing => {
                fb.draw_text(76, 216, "ARROWS MOVE  SPACE FIRE", TEXT);
            }
            RoundState::Victory => {
                fb.fill_rect(65, 91, 126, 39, BG);
                fb.draw_rect(65, 91, 126, 39, FOREGROUND);
                fb.draw_text(94, 100, "YOU WIN", FOREGROUND);
                fb.draw_text(78, 116, "SPACE TO REPLAY", TEXT);
            }
            RoundState::GameOver => {
                fb.fill_rect(65, 91, 126, 39, BG);
                fb.draw_rect(65, 91, 126, 39, DANGER);
                fb.draw_text(88, 100, "GAME OVER", DANGER);
                fb.draw_text(78, 116, "SPACE TO REPLAY", TEXT);
            }
        }
    }
}

impl Game for SpaceInvadersGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.input(frame);
        if result == GameResult::Exit {
            return result;
        }
        self.world.tick(frame.delta_time);
        self.render(frame);
        GameResult::Continue
    }
}

fn draw_mask(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    rows: &[u16],
    width: usize,
    color: Pixel,
) {
    for (row, bits) in rows.iter().copied().enumerate() {
        for col in 0..width {
            let bit = width - 1 - col;
            if bits & (1u16 << bit) != 0 {
                fb.fill_rect(x + col as i32, y + row as i32, 1, 1, color);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_classic_fifty_five_invaders() {
        let world = SpaceInvadersWorld::new();
        assert_eq!(world.alive_count(), 55);
    }

    #[test]
    fn formation_accelerates_as_invaders_are_destroyed() {
        let mut world = SpaceInvadersWorld::new();
        let initial = world.movement_period();
        for alien in world.aliens.iter_mut().take(30) {
            alien.alive = false;
        }
        assert!(world.movement_period() < initial);
    }

    #[test]
    fn player_can_have_only_one_shot_on_screen() {
        let mut world = SpaceInvadersWorld::new();
        world.shoot_player();
        let first = world.player_bullet;
        world.player_x += 20.0;
        world.shoot_player();
        assert_eq!(world.player_bullet.map(|shot| shot.x), first.map(|shot| shot.x));
    }

    #[test]
    fn bunker_damage_erodes_cells() {
        let mut bunker = Bunker::new(24);
        let before = bunker.cells.iter().flatten().filter(|cell| **cell).count();
        assert!(bunker.damage_at(24 + 12, BUNKER_Y + 6));
        let after = bunker.cells.iter().flatten().filter(|cell| **cell).count();
        assert!(after < before);
    }

    #[test]
    fn restart_resets_score_lives_and_invaders() {
        let mut world = SpaceInvadersWorld::new();
        world.score = 999;
        world.lives = 0;
        world.state = RoundState::GameOver;
        world.aliens[0].alive = false;
        world.restart();
        assert_eq!(world.score, 0);
        assert_eq!(world.lives, 3);
        assert_eq!(world.alive_count(), 55);
        assert_eq!(world.state, RoundState::Playing);
    }
}
