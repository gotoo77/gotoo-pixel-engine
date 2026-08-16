#!/usr/bin/env bash

# Canonical list of Web game entrypoints used by local validation/build scripts.
# Keep the historical web_demo checked separately: it is not part of the
# published Arcade catalog but remains a useful compatibility target.
GPE_WEB_GAME_EXAMPLES=(
    snake_web
    breakout_web
    tetris_web
    pong_web
    space_invaders_web
    smart_boy_hero_web
    arcade_web
)
