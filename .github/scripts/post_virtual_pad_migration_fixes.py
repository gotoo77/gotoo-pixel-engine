from pathlib import Path

snake_path = Path("examples/snake/game.rs")
snake = snake_path.read_text()
unused_touch_helper = '''    fn directions_from_touches(
        &mut self,
        touches: &[Touch],
        layout: SnakeLayout,
    ) -> Vec<Direction> {
        self.d_pad
            .directions_from_touches_in_layout(touches, layout)
    }
'''
if snake.count(unused_touch_helper) != 1:
    raise SystemExit("expected one legacy TouchControls::directions_from_touches helper")
snake = snake.replace(unused_touch_helper, "", 1)
snake_path.write_text(snake)

breakout_path = Path("examples/breakout.rs")
breakout = breakout_path.read_text()
old_test = '''    #[test]
    fn substeps_catch_fast_brick_collision() {
        let mut app = BreakoutApp::new();
        let brick = app.bricks[0];
        app.ball_stuck = false;
        app.ball_x = brick.rect.x as f32 + 4.0;
        app.ball_y = brick.rect.y as f32 + brick.rect.height as f32 + 8.0;
        app.ball_vx = 0.0;
        app.ball_vy = -900.0;

        let feedback = app.update_ball(0.02);

        assert!(!app.bricks[0].active);
        assert!(app.score > 0);
        assert!(feedback.brick_hits > 0);
    }
'''
new_test = '''    #[test]
    fn substeps_catch_fast_brick_collision() {
        let mut app = BreakoutApp::new();
        // Approach the bottom brick row from below so no other active brick can
        // legitimately intercept the ball before the brick under test.
        let brick_index = (BRICK_ROWS - 1) * BRICK_COLUMNS;
        let brick = app.bricks[brick_index];
        app.ball_stuck = false;
        app.ball_x = brick.rect.x as f32 + 4.0;
        app.ball_y = brick.rect.y as f32 + brick.rect.height as f32 + 8.0;
        app.ball_vx = 0.0;
        app.ball_vy = -900.0;

        let feedback = app.update_ball(0.02);

        assert!(!app.bricks[brick_index].active);
        assert!(app.score > 0);
        assert!(feedback.brick_hits > 0);
    }
'''
if breakout.count(old_test) != 1:
    raise SystemExit("expected one Breakout fast collision regression test")
breakout = breakout.replace(old_test, new_test, 1)

old_parity = "self.ball_vx = if self.lives % 2 == 0 {"
new_parity = "self.ball_vx = if self.lives.is_multiple_of(2) {"
if breakout.count(old_parity) != 1:
    raise SystemExit("expected one Breakout life parity expression")
breakout = breakout.replace(old_parity, new_parity, 1)

old_bricks = '''    for row in 0..BRICK_ROWS {
        for column in 0..BRICK_COLUMNS {
            bricks.push(Brick {
                rect: Rect {
                    x: BRICK_START_X + column as i32 * (BRICK_WIDTH as i32 + BRICK_GAP_X),
                    y: BRICK_START_Y + row as i32 * (BRICK_HEIGHT as i32 + BRICK_GAP_Y),
                    width: BRICK_WIDTH,
                    height: BRICK_HEIGHT,
                },
                active: true,
                color: BRICK_COLORS[row],
                row,
            });
        }
    }
'''
new_bricks = '''    for (row, color) in BRICK_COLORS.into_iter().enumerate() {
        for column in 0..BRICK_COLUMNS {
            bricks.push(Brick {
                rect: Rect {
                    x: BRICK_START_X + column as i32 * (BRICK_WIDTH as i32 + BRICK_GAP_X),
                    y: BRICK_START_Y + row as i32 * (BRICK_HEIGHT as i32 + BRICK_GAP_Y),
                    width: BRICK_WIDTH,
                    height: BRICK_HEIGHT,
                },
                active: true,
                color,
                row,
            });
        }
    }
'''
if breakout.count(old_bricks) != 1:
    raise SystemExit("expected one Breakout brick construction loop")
breakout = breakout.replace(old_bricks, new_bricks, 1)
breakout_path.write_text(breakout)

pong_path = Path("examples/pong.rs")
pong = pong_path.read_text()
old_pong_parity = "self.ball_vy = if (self.p1_score + self.p2_score) % 2 == 0 {"
new_pong_parity = "self.ball_vy = if (self.p1_score + self.p2_score).is_multiple_of(2) {"
if pong.count(old_pong_parity) != 1:
    raise SystemExit("expected one Pong score parity expression")
pong = pong.replace(old_pong_parity, new_pong_parity, 1)
pong_path.write_text(pong)
