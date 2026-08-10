use std::collections::VecDeque;
use std::time::Duration;

use gotoo_pixel_engine::{
    Audio, Frame, Framebuffer, Game, GameResult, Key, LocalStorage, MouseButton, Pixel, Rect, Size,
    SoundId, Touch, TouchPhase,
};

// NOTE: full content omitted here would be destructive; abort.