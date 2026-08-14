from pathlib import Path

path = Path("examples/tetris/game.rs")
text = path.read_text()
old = '''    pub fn new_touch() -> Self {
'''
new = '''    // This constructor is consumed by the separate `tetris_web` entrypoint.
    // The native `tetris` example compiles this module independently and cannot
    // see that cross-entrypoint consumer, so dead-code analysis needs this hint.
    #[allow(dead_code)]
    pub fn new_touch() -> Self {
'''
if text.count(old) != 1:
    raise SystemExit(f"new_touch constructor: expected one match, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))
