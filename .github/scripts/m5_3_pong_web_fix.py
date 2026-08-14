from pathlib import Path

path = Path("examples/pong.rs")
text = path.read_text()
needle = "impl PongApp {\n    pub fn new() -> Self {\n"
replacement = "impl Default for PongApp {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl PongApp {\n    pub fn new() -> Self {\n"
if text.count(needle) != 1:
    raise RuntimeError(f"expected one PongApp impl, found {text.count(needle)}")
path.write_text(text.replace(needle, replacement, 1))
