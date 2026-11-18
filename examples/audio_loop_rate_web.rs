//! Browser contract probe for the real Frame.audio backend; no audible-quality claim.
#[cfg(target_arch = "wasm32")]
mod probe {
    use gotoo_pixel_engine::{
        Audio, AudioBus, EngineConfig, Frame, Game, GameResult, PlaybackId, SoundId, run,
    };
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    const SOUND: SoundId = SoundId::new("rate.probe");
    const WAV: &[u8] = &[
        82, 73, 70, 70, 40, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0,
        68, 172, 0, 0, 136, 88, 1, 0, 2, 0, 16, 0, 100, 97, 116, 97, 4, 0, 0, 0, 0, 0, 1, 0,
    ];
    thread_local! {static STATUS:RefCell<String>=RefCell::new("waiting".into());}
    fn status(value: impl Into<String>) {
        STATUS.with(|s| *s.borrow_mut() = value.into());
    }
    #[wasm_bindgen]
    pub fn loop_rate_status() -> String {
        STATUS.with(|s| s.borrow().clone())
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace=globalThis,js_name=__gpeFailSource)]
        fn fail_source(stage: &str);
        #[wasm_bindgen(js_namespace=globalThis,js_name=__gpeContextClosed)]
        fn context_closed() -> bool;
    }

    struct Probe {
        survivor: Option<PlaybackId>,
    }
    fn exercise(audio: &mut dyn Audio) -> Result<PlaybackId, String> {
        let check =
            |result: Result<(), gotoo_pixel_engine::AudioError>| result.map_err(|e| e.to_string());
        check(audio.register_wav(SOUND, WAV))?;
        check(audio.set_master_volume(0.8))?;
        check(audio.set_bus_volume(AudioBus::Sfx, 0.4))?;
        let a = audio.start_loop(SOUND).map_err(|e| e.to_string())?;
        let b = audio
            .start_loop_on_bus(SOUND, AudioBus::Music)
            .map_err(|e| e.to_string())?;
        for rate in [1.0, 1.5, 0.75, 1.0, 0.5, 2.0, 1.0] {
            check(audio.set_loop_playback_rate(a, rate))?;
        }
        check(audio.set_bus_muted(AudioBus::Sfx, true))?;
        check(audio.set_loop_playback_rate(a, 1.5))?;
        check(audio.set_bus_muted(AudioBus::Sfx, false))?;
        check(audio.set_loop_playback_rate(a, 1.0))?;
        for invalid in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -1.0,
            0.0,
            0.49,
            2.01,
        ] {
            if audio.set_loop_playback_rate(a, invalid).is_ok() {
                return Err("invalid rate accepted".into());
            }
        }
        for id in [PlaybackId::new(0), PlaybackId::new(u64::MAX)] {
            if audio.set_loop_playback_rate(id, 1.5).is_ok() {
                return Err("unknown ID accepted".into());
            }
        }
        for (index, stage) in ["create", "connect", "start"].into_iter().enumerate() {
            fail_source(stage);
            if audio.start_loop(SOUND).is_ok() {
                return Err(format!("{stage} failure not propagated"));
            }
            // The backend reserves IDs monotonically before fallible node creation.
            let failed = PlaybackId::new(index as u64 + 3);
            if audio.stop_loop(failed).is_ok() || audio.set_loop_playback_rate(failed, 1.5).is_ok()
            {
                return Err(format!("{stage} failure leaked an active ID"));
            }
        }
        check(audio.stop_loop(a))?;
        if audio.set_loop_playback_rate(a, 1.5).is_ok() {
            return Err("stopped ID accepted".into());
        }
        check(audio.stop_loop(b))?;
        audio.start_loop(SOUND).map_err(|e| e.to_string())
    }
    impl Game for Probe {
        fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
            if let Some(id) = self.survivor {
                if !context_closed() {
                    return GameResult::Continue;
                }
                if frame.audio.set_loop_playback_rate(id, 1.5).is_ok() {
                    status("FAIL: closed context accepted rate");
                } else if frame.audio.stop_loop(id).is_err() || frame.audio.stop_loop(id).is_ok() {
                    status("FAIL: closed context cleanup");
                } else {
                    status("PASS");
                }
                return GameResult::Exit;
            }
            match exercise(frame.audio) {
                Ok(id) => {
                    self.survivor = Some(id);
                    status("ready-close");
                    GameResult::Continue
                }
                Err(error) => {
                    status(format!("FAIL: {error}"));
                    GameResult::Exit
                }
            }
        }
    }
    #[wasm_bindgen]
    pub fn start_loop_rate_probe() -> Result<(), JsValue> {
        run(
            EngineConfig {
                title: "Audio loop rate probe".into(),
                framebuffer_width: 32,
                framebuffer_height: 32,
                window_width: 128,
                window_height: 128,
            },
            Probe { survivor: None },
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
