#!/usr/bin/env python3

from pathlib import Path
import math
import random
import struct
import wave


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
        'const GAME_OVER_SOUND: SoundId = SoundId::new("tetris.game_over");\n',
        'const GAME_OVER_SOUND: SoundId = SoundId::new("tetris.game_over");\n'
        'const LINE_FLASH_SOUND: SoundId = SoundId::new("tetris.line_flash");\n',
        "Tetris line flash sound id",
    )

    text = replace_once(
        text,
        '        (LOCK_SOUND, TetrisSound::Lock),\n'
        '        (GAME_OVER_SOUND, TetrisSound::GameOver),\n',
        '        (LOCK_SOUND, TetrisSound::Lock),\n'
        '        (LINE_FLASH_SOUND, TetrisSound::LineFlash),\n'
        '        (GAME_OVER_SOUND, TetrisSound::GameOver),\n',
        "Tetris sound bank",
    )

    text = replace_once(
        text,
        '    Lock,\n    LineClear(u32),\n',
        '    Lock,\n    LineFlash,\n    LineClear(u32),\n',
        "Tetris sound enum",
    )

    text = replace_once(
        text,
        '        TetrisSound::Lock => (0.09, 0x7100_0003),\n'
        '        TetrisSound::LineClear(lines) => (0.14 + lines as f32 * 0.025, 0x7100_1000 + lines),\n',
        '        TetrisSound::Lock => (0.09, 0x7100_0003),\n'
        '        TetrisSound::LineFlash => (0.028, 0x7100_0004),\n'
        '        TetrisSound::LineClear(lines) => (0.14 + lines as f32 * 0.025, 0x7100_1000 + lines),\n',
        "Tetris flash duration",
    )

    text = replace_once(
        text,
        '            TetrisSound::Lock => (125.0 - 35.0 * progress, 0.30),\n'
        '            TetrisSound::LineClear(lines) => (390.0 + lines as f32 * 70.0 + 460.0 * progress, 0.30),\n',
        '            TetrisSound::Lock => (125.0 - 35.0 * progress, 0.30),\n'
        '            TetrisSound::LineFlash => (560.0 + 120.0 * progress, 0.13),\n'
        '            TetrisSound::LineClear(lines) => (390.0 + lines as f32 * 70.0 + 460.0 * progress, 0.30),\n',
        "Tetris flash timbre",
    )

    text = replace_once(
        text,
        '        let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };\n'
        '        let mixed = match kind {\n'
        '            TetrisSound::Lock => (0.72 * square + 0.28 * noise) * sample,\n'
        '            TetrisSound::GameOver => (0.78 * square + 0.22 * noise) * sample,\n'
        '            _ => square * sample,\n'
        '        };\n'
        '        let shaped = match kind {\n'
        '            TetrisSound::Move | TetrisSound::Lock => mixed * envelope * envelope,\n'
        '            _ => mixed * envelope,\n'
        '        };\n',
        '        let square = if phase.fract() < 0.5 { 1.0 } else { -1.0 };\n'
        '        let sine = (phase * std::f32::consts::TAU).sin();\n'
        '        let mixed = match kind {\n'
        '            TetrisSound::Lock => (0.72 * square + 0.28 * noise) * sample,\n'
        '            TetrisSound::LineFlash => sine * sample,\n'
        '            TetrisSound::GameOver => (0.78 * square + 0.22 * noise) * sample,\n'
        '            _ => square * sample,\n'
        '        };\n'
        '        let shaped = match kind {\n'
        '            TetrisSound::Move | TetrisSound::Lock | TetrisSound::LineFlash => {\n'
        '                mixed * envelope * envelope\n'
        '            }\n'
        '            _ => mixed * envelope,\n'
        '        };\n',
        "Tetris flash waveform",
    )

    text = replace_once(
        text,
        '        if self.world.has_pending_clear() {\n'
        '            self.line_clear_elapsed = self.line_clear_elapsed.saturating_add(frame.delta_time);\n'
        '            if self.line_clear_elapsed >= LINE_CLEAR_DELAY {\n',
        '        if self.world.has_pending_clear() {\n'
        '            let previous_blink =\n'
        '                self.line_clear_elapsed.as_millis() / LINE_CLEAR_BLINK.as_millis();\n'
        '            let entering_clear = self.line_clear_elapsed.is_zero();\n'
        '            self.line_clear_elapsed = self.line_clear_elapsed.saturating_add(frame.delta_time);\n'
        '            let current_blink =\n'
        '                self.line_clear_elapsed.as_millis() / LINE_CLEAR_BLINK.as_millis();\n'
        '            let flash_pulse = entering_clear\n'
        '                || (current_blink != previous_blink && current_blink.is_multiple_of(2));\n'
        '            if flash_pulse && self.line_clear_elapsed < LINE_CLEAR_DELAY {\n'
        '                let _ = self.sounds.play(frame.audio, LINE_FLASH_SOUND);\n'
        '            }\n'
        '            if self.line_clear_elapsed >= LINE_CLEAR_DELAY {\n',
        "Tetris flash playback",
    )

    path.write_text(text)


def write_soft_crunch() -> None:
    path = Path("examples/snake/assets/eat.wav")
    sample_rate = 44_100
    duration = 0.105
    sample_count = int(sample_rate * duration)
    rng = random.Random(0xC0C0A)
    samples: list[int] = []
    low_noise = 0.0

    for index in range(sample_count):
        t = index / sample_rate
        progress = t / duration
        raw_noise = rng.uniform(-1.0, 1.0)
        low_noise = low_noise * 0.78 + raw_noise * 0.22
        grain = raw_noise - low_noise

        bite_1 = math.exp(-max(0.0, t - 0.004) * 52.0) if t >= 0.004 else t / 0.004
        bite_2_t = t - 0.038
        bite_2 = (
            math.exp(-max(0.0, bite_2_t - 0.003) * 62.0)
            if bite_2_t >= 0.003
            else max(0.0, bite_2_t / 0.003)
        )
        bite_2 = max(0.0, bite_2)
        envelope = min(1.0, bite_1 + 0.72 * bite_2) * (1.0 - progress) ** 0.45

        resonance = math.sin(math.tau * (155.0 - 45.0 * progress) * t)
        crunch = 0.095 * low_noise + 0.045 * grain + 0.038 * resonance
        sample = max(-0.18, min(0.18, crunch * envelope))
        samples.append(int(sample * 32767.0))

    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(sample_rate)
        wav.writeframes(b"".join(struct.pack("<h", sample) for sample in samples))


def main() -> None:
    patch_tetris()
    write_soft_crunch()


if __name__ == "__main__":
    main()
