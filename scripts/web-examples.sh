#!/usr/bin/env bash

# Compatibility shim. scripts/dev.py owns the canonical Web example list.
_gpe_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mapfile -t GPE_WEB_GAME_EXAMPLES < <(python3 "${_gpe_script_dir}/dev.py" list-web-examples)
unset _gpe_script_dir
