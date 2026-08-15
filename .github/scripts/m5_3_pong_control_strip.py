from pathlib import Path


def replace_once(text: str, old: str, new: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match, found {count}: {old[:100]!r}")
    return text.replace(old, new, 1)


path = Path("examples/pong.rs")
text = path.read_text()

text = replace_once(
    text,
    "pub const FRAMEBUFFER_WIDTH: u32 = 320;\npub const FRAMEBUFFER_HEIGHT: u32 = 180;\n",
    "pub const FRAMEBUFFER_WIDTH: u32 = 320;\npub const FRAMEBUFFER_HEIGHT: u32 = 180;\npub const TOUCH_FRAMEBUFFER_HEIGHT: u32 = 260;\n",
)

old_rects = '''const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);
const TOUCH_P1_UP: Rect = Rect {
    x: 4,
    y: 32,
    width: 56,
    height: 52,
};
const TOUCH_P1_DOWN: Rect = Rect {
    x: 4,
    y: 112,
    width: 56,
    height: 52,
};
const TOUCH_P2_UP: Rect = Rect {
    x: 260,
    y: 32,
    width: 56,
    height: 52,
};
const TOUCH_P2_DOWN: Rect = Rect {
    x: 260,
    y: 112,
    width: 56,
    height: 52,
};
const TOUCH_ACTION_RECT: Rect = Rect {
    x: 120,
    y: 154,
    width: 80,
    height: 22,
};
'''

new_rects = '''const TOUCH_ACCENT: Pixel = Pixel::rgb(245, 190, 90);
const TOUCH_P1_UP: Rect = Rect {
    x: 8,
    y: 188,
    width: 56,
    height: 30,
};
const TOUCH_P1_DOWN: Rect = Rect {
    x: 8,
    y: 224,
    width: 56,
    height: 30,
};
const TOUCH_P2_UP: Rect = Rect {
    x: 256,
    y: 188,
    width: 56,
    height: 30,
};
const TOUCH_P2_DOWN: Rect = Rect {
    x: 256,
    y: 224,
    width: 56,
    height: 30,
};
const TOUCH_ACTION_RECT: Rect = Rect {
    x: 120,
    y: 206,
    width: 80,
    height: 32,
};
'''
text = replace_once(text, old_rects, new_rects)

text = replace_once(
    text,
    "fn draw_touch_controls(framebuffer: &mut Framebuffer, state: MatchState) {\n    for (rect, label) in [",
    "fn draw_touch_controls(framebuffer: &mut Framebuffer, state: MatchState) {\n    framebuffer.draw_line(0, FRAMEBUFFER_HEIGHT as i32, FRAMEBUFFER_WIDTH as i32 - 1, FRAMEBUFFER_HEIGHT as i32, BORDER);\n\n    for (rect, label) in [",
)

old_test = '''        assert_eq!(actions.len(), 5);
        for action in [P1_UP, P1_DOWN, P2_UP, P2_DOWN, TOUCH_ACTION] {
            assert!(actions.contains(&action));
        }
    }
'''
new_test = '''        assert_eq!(actions.len(), 5);
        for action in [P1_UP, P1_DOWN, P2_UP, P2_DOWN, TOUCH_ACTION] {
            assert!(actions.contains(&action));
        }
        assert!(TOUCH_FRAMEBUFFER_HEIGHT > FRAMEBUFFER_HEIGHT);
        assert!(pad
            .buttons()
            .iter()
            .all(|button| button.rect.y >= FRAMEBUFFER_HEIGHT as i32));
    }
'''
text = replace_once(text, old_test, new_test)

path.write_text(text)
