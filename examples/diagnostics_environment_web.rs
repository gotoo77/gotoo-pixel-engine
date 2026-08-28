#[cfg(not(feature = "diagnostics"))]
use gotoo_pixel_engine::run;
use gotoo_pixel_engine::{EngineConfig, Frame, Game, GameResult, Pixel};
#[cfg(feature = "diagnostics")]
use gotoo_pixel_engine::{EngineDiagnostics, EngineDiagnosticsHandle, run_with_diagnostics};
use wasm_bindgen::prelude::*;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const FINAL_TICK: u64 = 120;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(message: &str);
}

struct BrowserProbe {
    tick: u64,
    state: u64,
    published: bool,
    #[cfg(feature = "diagnostics")]
    diagnostics: EngineDiagnosticsHandle,
}

impl Game for BrowserProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.published {
            return GameResult::Continue;
        }

        self.tick += 1;
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.tick ^ 0x9e37_79b9_7f4a_7c15);
        frame.framebuffer.clear(Pixel::rgb(3, 5, 11));
        frame.framebuffer.fill_rect(
            (self.tick % 280) as i32,
            24,
            40,
            32,
            Pixel::rgb(
                (self.state >> 8) as u8,
                (self.state >> 24) as u8,
                (self.state >> 40) as u8,
            ),
        );
        frame.framebuffer.draw_line(
            0,
            (self.tick % u64::from(HEIGHT)) as i32,
            WIDTH as i32 - 1,
            ((self.tick * 7) % u64::from(HEIGHT)) as i32,
            Pixel::rgb(235, 210, 80),
        );
        frame
            .framebuffer
            .draw_text(12, 12, "GPE BROWSER DIAGNOSTICS", Pixel::rgb(180, 220, 245));

        if self.tick == FINAL_TICK {
            let framebuffer_hash = fnv1a64(frame.framebuffer.as_rgba8());
            #[cfg(feature = "diagnostics")]
            let result = format!(
                "mode=on ticks={} state_hash={:016x} framebuffer_hash={:016x} observation={:#?}",
                self.tick,
                self.state,
                framebuffer_hash,
                self.diagnostics.try_read()
            );
            #[cfg(not(feature = "diagnostics"))]
            let result = format!(
                "mode=off ticks={} state_hash={:016x} framebuffer_hash={:016x}",
                self.tick, self.state, framebuffer_hash
            );
            publish_result(&result);
            self.published = true;
        }

        GameResult::Continue
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn publish_result(result: &str) {
    console_log(&format!("GPE_ENV_RESULT {result}"));
    #[cfg(target_arch = "wasm32")]
    {
        let global = js_sys::global();
        let _ = js_sys::Reflect::set(
            &global,
            &JsValue::from_str("__GPE_ENV_RESULT"),
            &JsValue::from_str(result),
        );
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let config = EngineConfig {
        title: "GPE Browser Diagnostics Environmental Validation".into(),
        framebuffer_width: WIDTH,
        framebuffer_height: HEIGHT,
        window_width: 960,
        window_height: 540,
    };

    #[cfg(feature = "diagnostics")]
    {
        let (handle, registration) = EngineDiagnostics::enabled();
        let game = BrowserProbe {
            tick: 0,
            state: 0x4750_452d_454e_5631,
            published: false,
            diagnostics: handle.clone(),
        };
        run_with_diagnostics(config, game, registration).map_err(|error| error.to_string().into())
    }
    #[cfg(not(feature = "diagnostics"))]
    {
        let game = BrowserProbe {
            tick: 0,
            state: 0x4750_452d_454e_5631,
            published: false,
        };
        run(config, game).map_err(|error| error.to_string().into())
    }
}
