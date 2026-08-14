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
breakout_path.write_text(breakout)
