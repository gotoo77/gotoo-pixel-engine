from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def patch_tetris() -> None:
    path = Path("examples/tetris/game.rs")
    text = path.read_text()

    text = replace_once(
        text,
        "const GRAVITY: Duration = Duration::from_millis(500);\nconst SOFT_DROP: Duration = Duration::from_millis(45);\n",
        "const GRAVITY: Duration = Duration::from_millis(500);\n"
        "const SOFT_DROP: Duration = Duration::from_millis(45);\n"
        "const LINE_CLEAR_DELAY: Duration = Duration::from_millis(600);\n"
        "const LINE_CLEAR_BLINK: Duration = Duration::from_millis(100);\n"
        "const SCORE_POPUP_DURATION: Duration = Duration::from_millis(900);\n"
        "const HORIZONTAL_REPEAT_DELAY: Duration = Duration::from_millis(180);\n"
        "const HORIZONTAL_REPEAT_PERIOD: Duration = Duration::from_millis(60);\n",
        "tetris timing constants",
    )

    text = replace_once(
        text,
        "const GAME_OVER: Pixel = Pixel::rgb(245, 76, 76);\nconst TOUCH_FILL: Pixel = Pixel::rgb(18, 28, 34);\n",
        "const GAME_OVER: Pixel = Pixel::rgb(245, 76, 76);\n"
        "const LINE_FLASH: Pixel = Pixel::rgb(245, 245, 220);\n"
        "const LINE_SCORE_COLORS: [Pixel; 4] = [\n"
        "    Pixel::rgb(80, 220, 230),\n"
        "    Pixel::rgb(80, 205, 105),\n"
        "    Pixel::rgb(235, 150, 55),\n"
        "    Pixel::rgb(210, 95, 235),\n"
        "];\n"
        "const TOUCH_FILL: Pixel = Pixel::rgb(18, 28, 34);\n",
        "tetris score colors",
    )

    text = replace_once(
        text,
        "    game_over: bool,\n    events: Vec<TetrisEvent>,\n",
        "    game_over: bool,\n    pending_rows: Vec<usize>,\n    events: Vec<TetrisEvent>,\n",
        "tetris world pending rows field",
    )
    text = replace_once(
        text,
        "            game_over: false,\n            events: Vec::new(),\n",
        "            game_over: false,\n            pending_rows: Vec::new(),\n            events: Vec::new(),\n",
        "tetris world pending rows init",
    )
    text = replace_once(
        text,
        "        if self.game_over {\n            return false;\n        }\n        let candidate = Piece {\n",
        "        if self.game_over || self.has_pending_clear() {\n            return false;\n        }\n        let candidate = Piece {\n",
        "tetris translate pending guard",
    )
    text = replace_once(
        text,
        "        if self.game_over || self.active.kind == Kind::O {\n",
        "        if self.game_over || self.has_pending_clear() || self.active.kind == Kind::O {\n",
        "tetris rotate pending guard",
    )

    old_drop = '''    fn hard_drop(&mut self) {
        if self.game_over {
            return;
        }
        let mut distance = 0;
        while self.translate(0, 1) {
            distance += 1;
        }
        self.score = self.score.saturating_add(distance * 2);
        self.lock();
    }
'''
    new_drop = '''    fn hard_drop(&mut self) {
        if self.game_over || self.has_pending_clear() {
            return;
        }
        while self.translate(0, 1) {}
        self.lock();
    }
'''
    text = replace_once(text, old_drop, new_drop, "tetris hard drop score removal")

    old_lock = '''    fn lock(&mut self) {
        let cells = self.active.cells();
        if cells.iter().any(|&(_, y)| y < 0) {
            self.game_over = true;
            self.events.push(TetrisEvent::GameOver);
            return;
        }

        for (x, y) in cells {
            self.board[y as usize][x as usize] = Some(self.active.kind);
        }
        self.events.push(TetrisEvent::Locked);

        let cleared = self.clear_lines();
        self.lines += cleared;
        self.score = self.score.saturating_add(match cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        });
        if cleared > 0 {
            self.events.push(TetrisEvent::LinesCleared(cleared));
        }

        self.active = spawn(self.next);
        self.next = self.bag.next();
        if !self.valid(self.active) {
            self.game_over = true;
            self.events.push(TetrisEvent::GameOver);
        }
    }

    fn clear_lines(&mut self) -> u32 {
        let mut write = BOARD_HEIGHT - 1;
        let mut cleared = 0;
        for read in (0..BOARD_HEIGHT).rev() {
            if self.board[read as usize].iter().all(Option::is_some) {
                cleared += 1;
            } else {
                if write != read {
                    self.board[write as usize] = self.board[read as usize];
                }
                write -= 1;
            }
        }
        while write >= 0 {
            self.board[write as usize] = [None; BOARD_WIDTH as usize];
            write -= 1;
        }
        cleared
    }
'''
    new_lock = '''    fn lock(&mut self) {
        if self.has_pending_clear() {
            return;
        }

        let cells = self.active.cells();
        if cells.iter().any(|&(_, y)| y < 0) {
            self.game_over = true;
            self.events.push(TetrisEvent::GameOver);
            return;
        }

        for (x, y) in cells {
            self.board[y as usize][x as usize] = Some(self.active.kind);
        }
        self.events.push(TetrisEvent::Locked);

        self.pending_rows = self.full_rows();
        if self.pending_rows.is_empty() {
            self.spawn_next();
        }
    }

    fn has_pending_clear(&self) -> bool {
        !self.pending_rows.is_empty()
    }

    fn pending_rows(&self) -> &[usize] {
        &self.pending_rows
    }

    fn full_rows(&self) -> Vec<usize> {
        (0..BOARD_HEIGHT as usize)
            .filter(|&row| self.board[row].iter().all(Option::is_some))
            .collect()
    }

    fn finish_pending_clear(&mut self) -> u32 {
        let rows = std::mem::take(&mut self.pending_rows);
        if rows.is_empty() {
            return 0;
        }

        let cleared = rows.len() as u32;
        self.clear_rows(&rows);
        self.lines = self.lines.saturating_add(cleared);
        self.score = self.score.saturating_add(line_clear_score(cleared));
        self.events.push(TetrisEvent::LinesCleared(cleared));
        self.spawn_next();
        cleared
    }

    fn spawn_next(&mut self) {
        self.active = spawn(self.next);
        self.next = self.bag.next();
        if !self.valid(self.active) {
            self.game_over = true;
            self.events.push(TetrisEvent::GameOver);
        }
    }

    fn clear_rows(&mut self, rows: &[usize]) {
        let mut write = BOARD_HEIGHT - 1;
        for read in (0..BOARD_HEIGHT).rev() {
            if rows.contains(&(read as usize)) {
                continue;
            }
            if write != read {
                self.board[write as usize] = self.board[read as usize];
            }
            write -= 1;
        }
        while write >= 0 {
            self.board[write as usize] = [None; BOARD_WIDTH as usize];
            write -= 1;
        }
    }

    #[cfg(test)]
    fn clear_lines(&mut self) -> u32 {
        let rows = self.full_rows();
        let cleared = rows.len() as u32;
        self.clear_rows(&rows);
        cleared
    }
'''
    text = replace_once(text, old_lock, new_lock, "tetris delayed line clear world")

    text = replace_once(
        text,
        "}\n\n#[derive(Debug)]\npub struct TetrisGame {\n",
        '''}

fn line_clear_score(lines: u32) -> u32 {
    match lines {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    }
}

#[derive(Debug, Default)]
struct HorizontalRepeat {
    direction: i32,
    held_for: Duration,
    repeat_accumulator: Duration,
}

impl HorizontalRepeat {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Copy)]
struct ScorePopup {
    lines: u32,
    remaining: Duration,
}

impl ScorePopup {
    fn new(lines: u32) -> Self {
        Self {
            lines,
            remaining: SCORE_POPUP_DURATION,
        }
    }
}

#[derive(Debug)]
pub struct TetrisGame {
''',
        "tetris helper structs",
    )

    text = replace_once(
        text,
        "    accumulator: Duration,\n    controls: ControlMap,\n",
        "    accumulator: Duration,\n    line_clear_elapsed: Duration,\n    score_popup: Option<ScorePopup>,\n    horizontal_repeat: HorizontalRepeat,\n    controls: ControlMap,\n",
        "tetris game state fields",
    )
    text = replace_once(
        text,
        "            accumulator: Duration::ZERO,\n            controls: default_controls(),\n",
        "            accumulator: Duration::ZERO,\n            line_clear_elapsed: Duration::ZERO,\n            score_popup: None,\n            horizontal_repeat: HorizontalRepeat::default(),\n            controls: default_controls(),\n",
        "tetris game state init",
    )

    old_input = '''    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.controls);
        }
        self.controls.update(frame.input);

        if self.controls.action(CONTROL_EXIT).pressed() {
            return GameResult::Exit;
        }
        if self.world.game_over {
            if self.controls.action(CONTROL_HARD_DROP).pressed() {
                self.world.restart();
                self.accumulator = Duration::ZERO;
            }
            return GameResult::Continue;
        }
        if self.controls.action(CONTROL_LEFT).pressed() && self.world.translate(-1, 0) {
            self.world.events.push(TetrisEvent::Moved);
        }
        if self.controls.action(CONTROL_RIGHT).pressed() && self.world.translate(1, 0) {
            self.world.events.push(TetrisEvent::Moved);
        }
        if self.controls.action(CONTROL_ROTATE).pressed() && self.world.rotate() {
            self.world.events.push(TetrisEvent::Rotated);
        }
        if self.controls.action(CONTROL_HARD_DROP).pressed() {
            self.world.hard_drop();
            self.accumulator = Duration::ZERO;
        }
        GameResult::Continue
    }
'''
    new_input = '''    fn input(&mut self, frame: &Frame<'_>) -> GameResult {
        if let Some(virtual_pad) = &mut self.virtual_pad {
            virtual_pad.update(frame.input, &mut self.controls);
        }
        self.controls.update(frame.input);

        if self.controls.action(CONTROL_EXIT).pressed() {
            return GameResult::Exit;
        }
        if self.world.game_over {
            if self.controls.action(CONTROL_HARD_DROP).pressed() {
                self.world.restart();
                self.accumulator = Duration::ZERO;
                self.line_clear_elapsed = Duration::ZERO;
                self.score_popup = None;
                self.horizontal_repeat.reset();
            }
            return GameResult::Continue;
        }
        if self.world.has_pending_clear() {
            self.horizontal_repeat.reset();
            return GameResult::Continue;
        }

        self.update_horizontal_repeat(frame.delta_time);
        if self.controls.action(CONTROL_ROTATE).pressed() && self.world.rotate() {
            self.world.events.push(TetrisEvent::Rotated);
        }
        if self.controls.action(CONTROL_HARD_DROP).pressed() {
            self.world.hard_drop();
            self.accumulator = Duration::ZERO;
            self.horizontal_repeat.reset();
        }
        GameResult::Continue
    }

    fn update_horizontal_repeat(&mut self, delta_time: Duration) {
        let left = self.controls.action(CONTROL_LEFT).held();
        let right = self.controls.action(CONTROL_RIGHT).held();
        let direction = match (left, right) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };

        if direction == 0 {
            self.horizontal_repeat.reset();
            return;
        }

        if self.horizontal_repeat.direction != direction {
            self.horizontal_repeat.direction = direction;
            self.horizontal_repeat.held_for = Duration::ZERO;
            self.horizontal_repeat.repeat_accumulator = Duration::ZERO;
            self.move_horizontal(direction);
            return;
        }

        self.horizontal_repeat.held_for = self
            .horizontal_repeat
            .held_for
            .saturating_add(delta_time);
        if self.horizontal_repeat.held_for < HORIZONTAL_REPEAT_DELAY {
            return;
        }

        self.horizontal_repeat.repeat_accumulator = self
            .horizontal_repeat
            .repeat_accumulator
            .saturating_add(delta_time);
        while self.horizontal_repeat.repeat_accumulator >= HORIZONTAL_REPEAT_PERIOD {
            self.horizontal_repeat.repeat_accumulator -= HORIZONTAL_REPEAT_PERIOD;
            if !self.move_horizontal(direction) {
                self.horizontal_repeat.repeat_accumulator = Duration::ZERO;
                break;
            }
        }
    }

    fn move_horizontal(&mut self, direction: i32) -> bool {
        if self.world.translate(direction, 0) {
            self.world.events.push(TetrisEvent::Moved);
            true
        } else {
            false
        }
    }

    fn update_score_popup(&mut self, delta_time: Duration) {
        let Some(popup) = self.score_popup.as_mut() else {
            return;
        };
        popup.remaining = popup.remaining.saturating_sub(delta_time);
        if popup.remaining.is_zero() {
            self.score_popup = None;
        }
    }
'''
    text = replace_once(text, old_input, new_input, "tetris input repeat")

    old_board_render = '''        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let px = BOARD_X + x * CELL_SIZE;
                let py = BOARD_Y + y * CELL_SIZE;
                fb.draw_rect(px, py, CELL_SIZE as u32, CELL_SIZE as u32, GRID);
                if let Some(kind) = self.world.board[y as usize][x as usize] {
                    draw_block(fb, x, y, kind);
                }
            }
        }
        if !self.world.game_over {
'''
    new_board_render = '''        let flash_on = self.world.has_pending_clear()
            && (self.line_clear_elapsed.as_millis() / LINE_CLEAR_BLINK.as_millis()) % 2 == 0;
        for y in 0..BOARD_HEIGHT {
            let clearing = self.world.pending_rows().contains(&(y as usize));
            for x in 0..BOARD_WIDTH {
                let px = BOARD_X + x * CELL_SIZE;
                let py = BOARD_Y + y * CELL_SIZE;
                fb.draw_rect(px, py, CELL_SIZE as u32, CELL_SIZE as u32, GRID);
                if let Some(kind) = self.world.board[y as usize][x as usize] {
                    if clearing && flash_on {
                        fb.fill_rect(
                            px + 1,
                            py + 1,
                            (CELL_SIZE - 1) as u32,
                            (CELL_SIZE - 1) as u32,
                            LINE_FLASH,
                        );
                    } else {
                        draw_block(fb, x, y, kind);
                    }
                }
            }
        }
        if !self.world.game_over && !self.world.has_pending_clear() {
'''
    text = replace_once(text, old_board_render, new_board_render, "tetris line flash render")

    text = replace_once(
        text,
        '''        if self.virtual_pad.is_some() {
            draw_touch_controls(fb);
        }
''',
        '''        if let Some(popup) = self.score_popup {
            draw_score_popup(fb, popup);
        }
        if self.virtual_pad.is_some() {
            draw_touch_controls(fb);
        }
''',
        "tetris score popup render",
    )

    old_update = '''impl Game for TetrisGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.input(frame);
        if result == GameResult::Exit {
            return result;
        }
        if !self.world.game_over {
            self.accumulator = self.accumulator.saturating_add(frame.delta_time);
            let period = if self.controls.action(CONTROL_SOFT_DROP).held() {
                SOFT_DROP
            } else {
                GRAVITY
            };
            while self.accumulator >= period && !self.world.game_over {
                self.accumulator -= period;
                self.world.gravity_step();
            }
        }
        self.consume_events(frame);
        self.render(frame);
        GameResult::Continue
    }
}
'''
    new_update = '''impl Game for TetrisGame {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        let result = self.input(frame);
        if result == GameResult::Exit {
            return result;
        }

        self.update_score_popup(frame.delta_time);
        if self.world.has_pending_clear() {
            self.line_clear_elapsed = self
                .line_clear_elapsed
                .saturating_add(frame.delta_time);
            if self.line_clear_elapsed >= LINE_CLEAR_DELAY {
                let cleared = self.world.finish_pending_clear();
                self.line_clear_elapsed = Duration::ZERO;
                self.accumulator = Duration::ZERO;
                if cleared > 0 {
                    self.score_popup = Some(ScorePopup::new(cleared));
                }
            }
        } else if !self.world.game_over {
            self.accumulator = self.accumulator.saturating_add(frame.delta_time);
            let period = if self.controls.action(CONTROL_SOFT_DROP).held() {
                SOFT_DROP
            } else {
                GRAVITY
            };
            while self.accumulator >= period
                && !self.world.game_over
                && !self.world.has_pending_clear()
            {
                self.accumulator -= period;
                self.world.gravity_step();
            }
        }
        self.consume_events(frame);
        self.render(frame);
        GameResult::Continue
    }
}
'''
    text = replace_once(text, old_update, new_update, "tetris update line clear animation")

    marker = "fn draw_touch_controls(framebuffer: &mut Framebuffer) {\n"
    popup_fn = '''fn draw_score_popup(framebuffer: &mut Framebuffer, popup: ScorePopup) {
    let points = line_clear_score(popup.lines);
    let color = LINE_SCORE_COLORS[popup.lines.clamp(1, 4) as usize - 1];
    let score_text = format!("+{points}");
    let (score_width, score_height) = Framebuffer::text_size(&score_text, 2);
    let playfield_width = (BOARD_WIDTH * CELL_SIZE) as u32;
    let score_x = BOARD_X + playfield_width.saturating_sub(score_width) as i32 / 2;
    let score_y = BOARD_Y + 76;
    framebuffer.fill_rect(
        score_x - 4,
        score_y - 4,
        score_width + 8,
        score_height + 8,
        BG,
    );
    framebuffer.draw_text_scaled(score_x, score_y, &score_text, 2, color);

    let line_text = if popup.lines == 1 {
        "1 LINE".to_string()
    } else {
        format!("{} LINES", popup.lines)
    };
    let (line_width, _) = Framebuffer::text_size(&line_text, 1);
    let line_x = BOARD_X + playfield_width.saturating_sub(line_width) as i32 / 2;
    framebuffer.draw_text(line_x, score_y + score_height as i32 + 7, &line_text, color);
}

'''
    text = replace_once(text, marker, popup_fn + marker, "tetris popup helper")

    test_marker = '''    #[test]
    fn tetris_sound_bank_owns_all_feedback_assets() {
'''
    new_tests = '''    #[test]
    fn hard_drop_without_line_clear_does_not_score() {
        let mut world = TetrisWorld::new();
        world.hard_drop();
        assert_eq!(world.score, 0);
    }

    #[test]
    fn completed_line_waits_until_clear_is_finished() {
        let mut world = TetrisWorld::new();
        let row = (BOARD_HEIGHT - 1) as usize;
        world.board[row] = [Some(Kind::I); BOARD_WIDTH as usize];
        world.pending_rows = world.full_rows();

        assert!(world.has_pending_clear());
        assert!(world.board[row].iter().all(Option::is_some));
        assert_eq!(world.score, 0);

        assert_eq!(world.finish_pending_clear(), 1);
        assert_eq!(world.score, 100);
        assert_eq!(world.lines, 1);
        assert!(world.board[row].iter().all(Option::is_none));
    }

    #[test]
    fn line_clear_scoring_is_explicit() {
        assert_eq!(line_clear_score(1), 100);
        assert_eq!(line_clear_score(2), 300);
        assert_eq!(line_clear_score(3), 500);
        assert_eq!(line_clear_score(4), 800);
        assert_eq!(line_clear_score(0), 0);
    }

'''
    text = replace_once(text, test_marker, new_tests + test_marker, "tetris polish tests")

    path.write_text(text)


def patch_snake() -> None:
    path = Path("examples/snake/game.rs")
    text = path.read_text()

    text = replace_once(
        text,
        "    LocalStorage, MouseButton, Pixel, Rect, Size, SoundBank, SoundId, Touch, TouchPhase,\n",
        "    LocalStorage, MouseButton, Pixel, Rect, Size, SoundBank, SoundId, Touch, TouchPhase,\n"
        "    pcm16_mono_wav,\n",
        "snake pcm wav import",
    )
    text = replace_once(
        text,
        '''const EAT_SOUND: SoundId = SoundId::new("snake.eat");
const DEATH_SOUND: SoundId = SoundId::new("snake.death");
''',
        '''const EAT_SOUND: SoundId = SoundId::new("snake.eat");
const DEATH_SOUND: SoundId = SoundId::new("snake.death");
const TURN_SOUND: SoundId = SoundId::new("snake.turn");
const SNAKE_AUDIO_SAMPLE_RATE: u32 = 44_100;
''',
        "snake turn sound id",
    )

    text = replace_once(
        text,
        '''            if result.ate_food {
                events.food_eaten += 1;
            }
            if result.game_over {
''',
        '''            if result.turned {
                events.turns += 1;
            }
            if result.ate_food {
                events.food_eaten += 1;
            }
            if result.game_over {
''',
        "snake turn event collection",
    )

    text = replace_once(
        text,
        '''    fn play_sounds(&mut self, audio: &mut dyn Audio, events: SnakeEvents) {
        for _ in 0..events.food_eaten {
''',
        '''    fn play_sounds(&mut self, audio: &mut dyn Audio, events: SnakeEvents) {
        for _ in 0..events.turns {
            let _ = self.sounds.play(audio, TURN_SOUND);
        }
        for _ in 0..events.food_eaten {
''',
        "snake turn sound playback",
    )
    text = replace_once(
        text,
        '''struct SnakeEvents {
    food_eaten: u32,
    game_over: bool,
}
''',
        '''struct SnakeEvents {
    turns: u32,
    food_eaten: u32,
    game_over: bool,
}
''',
        "snake turn event state",
    )

    old_bank = '''fn snake_sound_bank() -> SoundBank {
    let mut sounds = SoundBank::new();
    for (id, bytes) in SNAKE_SOUNDS {
        sounds
            .insert_wav(id, bytes.to_vec())
            .expect("Snake sound ids should be unique");
    }
    sounds
}
'''
    new_bank = '''fn snake_sound_bank() -> SoundBank {
    let mut sounds = SoundBank::new();
    for (id, bytes) in SNAKE_SOUNDS {
        sounds
            .insert_wav(id, bytes.to_vec())
            .expect("Snake sound ids should be unique");
    }
    sounds
        .insert_wav(TURN_SOUND, synthesize_turn_sound())
        .expect("Snake turn sound id should be unique");
    sounds
}

fn synthesize_turn_sound() -> Vec<u8> {
    let duration = 0.035_f32;
    let sample_count = (SNAKE_AUDIO_SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut phase = 0.0_f32;

    for index in 0..sample_count {
        let progress = index as f32 / sample_count as f32;
        let envelope = (1.0 - progress).powi(2);
        let frequency = 175.0 + 45.0 * progress;
        phase += frequency / SNAKE_AUDIO_SAMPLE_RATE as f32;
        let wave = (phase * std::f32::consts::TAU).sin();
        let sample = wave * envelope * 0.11;
        samples.push((sample * i16::MAX as f32) as i16);
    }

    pcm16_mono_wav(SNAKE_AUDIO_SAMPLE_RATE, &samples)
        .expect("Snake procedural turn sound should use a supported PCM format")
}
'''
    text = replace_once(text, old_bank, new_bank, "snake turn sound synthesis")

    old_tick_head = '''        if let Some(direction) = self.turn_queue.pop_front() {
            self.direction = direction;
        }

        let next_head = self.snake[0].next(self.direction);
'''
    new_tick_head = '''        let turned = if let Some(direction) = self.turn_queue.pop_front() {
            self.direction = direction;
            true
        } else {
            false
        };

        let next_head = self.snake[0].next(self.direction);
'''
    text = replace_once(text, old_tick_head, new_tick_head, "snake applied turn detection")
    text = replace_once(
        text,
        '''            return TickResult {
                ate_food: false,
                game_over: true,
            };
''',
        '''            return TickResult {
                turned,
                ate_food: false,
                game_over: true,
            };
''',
        "snake collision tick result",
    )
    text = replace_once(
        text,
        '''        TickResult {
            ate_food: will_grow,
            game_over: false,
        }
''',
        '''        TickResult {
            turned,
            ate_food: will_grow,
            game_over: false,
        }
''',
        "snake normal tick result",
    )
    text = replace_once(
        text,
        '''struct TickResult {
    ate_food: bool,
    game_over: bool,
}
''',
        '''struct TickResult {
    turned: bool,
    ate_food: bool,
    game_over: bool,
}
''',
        "snake tick result turn field",
    )

    text = replace_once(
        text,
        "        D_PAD_CENTER_FILL, D_PAD_FILL, DEATH_SOUND, DPadTracker, Direction, EAT_SOUND, EXIT_KEY,\n",
        "        D_PAD_CENTER_FILL, D_PAD_FILL, DEATH_SOUND, DPadTracker, Direction, EAT_SOUND, EXIT_KEY,\n"
        "        TURN_SOUND,\n",
        "snake test turn sound import",
    )

    marker = '''    #[test]
    fn keyboard_direction_controls_work_in_both_modes() {
'''
    tests = '''    #[test]
    fn applied_turn_is_reported_on_tick() {
        let mut game = touch_game();
        let events = game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Up]),
        );

        assert_eq!(events.turns, 1);
        assert_eq!(game.world.direction, Direction::Up);
    }

    #[test]
    fn invalid_reverse_does_not_report_a_turn() {
        let mut game = touch_game();
        let events = game.update_logic(
            TICK_PERIOD,
            SnakeControls::with_directions([Direction::Left]),
        );

        assert_eq!(events.turns, 0);
        assert_eq!(game.world.direction, Direction::Right);
    }

    #[test]
    fn turn_sound_is_registered_and_playable() {
        let mut bank = snake_sound_bank();
        let mut audio = NoopAudio::default();
        bank.play(&mut audio, TURN_SOUND)
            .expect("generated Snake turn sound should be playable");
    }

'''
    text = replace_once(text, marker, tests + marker, "snake turn sound tests")

    path.write_text(text)


patch_tetris()
patch_snake()
