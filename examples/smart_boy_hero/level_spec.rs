// `world.rs` is reused through `#[path]` by multiple examples, so rustc resolves
// its child module beside that file. Keep this small entry point here while the
// level-spec implementation stays grouped under `world/`.
include!("world/level_spec.rs");
