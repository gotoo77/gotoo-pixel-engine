use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(not(feature = "diagnostics"))]
use gotoo_pixel_engine::run;
#[cfg(feature = "diagnostics")]
use gotoo_pixel_engine::{DiagnosticObservation, EngineDiagnostics, run_with_diagnostics};
use gotoo_pixel_engine::{EngineConfig, Frame, Game, GameResult, Pixel};
#[cfg(not(target_arch = "wasm32"))]
use gotoo_pixel_engine::{ToolFrame, ToolWindowConfig, ToolWindowMode};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const DEFAULT_TICKS: u64 = 360;

#[derive(Debug, Clone, Copy)]
struct FinalState {
    ticks: u64,
    state_hash: u64,
    framebuffer_hash: u64,
}

struct EnvironmentProbe {
    tick: u64,
    max_ticks: u64,
    state: u64,
    tool_enabled: bool,
    final_state: Arc<Mutex<Option<FinalState>>>,
}

impl EnvironmentProbe {
    fn tool_requested(&self) -> bool {
        self.tool_enabled && ((40..90).contains(&self.tick) || (120..170).contains(&self.tick))
    }
}

impl Game for EnvironmentProbe {
    fn update(&mut self, frame: &mut Frame<'_>) -> GameResult {
        if self.tick >= self.max_ticks {
            return GameResult::Exit;
        }

        self.tick += 1;
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.tick ^ 0x9e37_79b9_7f4a_7c15);

        let red = (self.state >> 8) as u8;
        let green = (self.state >> 24) as u8;
        let blue = (self.state >> 40) as u8;
        frame.framebuffer.clear(Pixel::rgb(3, 5, 11));
        frame.framebuffer.fill_rect(
            (self.tick % 280) as i32,
            24,
            40,
            32,
            Pixel::rgb(red, green, blue),
        );
        frame.framebuffer.draw_line(
            0,
            (self.tick % u64::from(HEIGHT)) as i32,
            WIDTH as i32 - 1,
            ((self.tick * 7) % u64::from(HEIGHT)) as i32,
            Pixel::rgb(235, 210, 80),
        );
        frame.framebuffer.draw_text(
            12,
            12,
            "GPE ENVIRONMENTAL VALIDATION",
            Pixel::rgb(180, 220, 245),
        );

        if self.tick == self.max_ticks {
            let final_state = FinalState {
                ticks: self.tick,
                state_hash: self.state,
                framebuffer_hash: fnv1a64(frame.framebuffer.as_rgba8()),
            };
            *self.final_state.lock().expect("final-state mutex") = Some(final_state);
        }

        GameResult::Continue
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn tool_window_config(&self) -> Option<ToolWindowConfig> {
        self.tool_requested().then(|| ToolWindowConfig {
            title: "GPE Environmental Tool Renderer".into(),
            framebuffer_width: 160,
            framebuffer_height: 100,
            window_width: 480,
            window_height: 300,
            mode: ToolWindowMode::Modeless,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_tool_window(&mut self, frame: &mut ToolFrame<'_>) {
        frame.framebuffer.clear(Pixel::rgb(13, 7, 20));
        frame.framebuffer.fill_rect(
            (self.tick % 130) as i32,
            35,
            30,
            20,
            Pixel::rgb(190, 110, 240),
        );
        frame
            .framebuffer
            .draw_text(8, 8, "REAL TOOL RENDERER", Pixel::rgb(220, 200, 250));
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(feature = "diagnostics")]
fn print_diagnostics(observation: &DiagnosticObservation) {
    println!(
        "ENV_RUNTIME availability={:?} value={:?} degraded={}",
        observation.runtime.availability, observation.runtime.value, observation.degraded
    );
    if let Some(renderers) = observation.renderers.value.as_ref() {
        println!(
            "ENV_RENDERERS retained={} identities={} identities_saturated={} dropped={}",
            renderers.retained,
            renderers.identities_issued.value,
            renderers.identities_issued.saturated,
            renderers.records_dropped.value
        );
        for record in renderers.records.iter().flatten() {
            println!(
                "ENV_RENDERER role={:?} incarnation={} lifecycle={:?} backend={:?} device_type={:?} adapter_name={:?} surface={:?} present={:?} surface_failure={:?} device_lost={:?} wgpu_error={:?}",
                record.source.role,
                record.source.incarnation.get(),
                record.lifecycle.value,
                record.adapter.backend.value,
                record.adapter.device_type.value,
                record.adapter.name.value.as_ref().map(|name| (
                    name.as_str(),
                    name.stored_len(),
                    name.truncated
                )),
                record.surface_configuration.value,
                record.last_present.value,
                record.last_surface_failure.availability,
                record.device_lost.availability,
                record.last_wgpu_error.availability
            );
        }
    }
    println!(
        "ENV_AUDIO availability={:?} value={:?}",
        observation.audio.availability, observation.audio.value
    );
    if let Some(events) = observation.events.value.as_ref() {
        println!(
            "ENV_EVENTS retained={} wrapped={} overwritten={} dropped={}",
            events.retained, events.ever_wrapped, events.overwritten.value, events.dropped.value
        );
    }
}

fn arguments() -> (u64, bool) {
    let mut ticks = DEFAULT_TICKS;
    let mut tool_enabled = true;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--ticks" => {
                ticks = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(DEFAULT_TICKS);
            }
            "--no-tool" => tool_enabled = false,
            _ => {}
        }
    }
    (ticks.max(1), tool_enabled)
}

fn main() -> Result<(), gotoo_pixel_engine::EngineError> {
    let (max_ticks, tool_enabled) = arguments();
    let final_state = Arc::new(Mutex::new(None));
    let game = EnvironmentProbe {
        tick: 0,
        max_ticks,
        state: 0x4750_452d_454e_5631,
        tool_enabled,
        final_state: Arc::clone(&final_state),
    };
    let config = EngineConfig {
        title: "GPE Diagnostics Environmental Validation".into(),
        framebuffer_width: WIDTH,
        framebuffer_height: HEIGHT,
        window_width: 960,
        window_height: 540,
    };
    let started = Instant::now();

    #[cfg(feature = "diagnostics")]
    let diagnostics = {
        let (handle, registration) = EngineDiagnostics::enabled();
        run_with_diagnostics(config, game, registration)?;
        Some(handle.try_read())
    };
    #[cfg(not(feature = "diagnostics"))]
    run(config, game)?;

    let final_state = final_state
        .lock()
        .expect("final-state mutex")
        .expect("final state captured before exit");
    println!(
        "ENV_RESULT mode={} ticks={} state_hash={:016x} framebuffer_hash={:016x} duration_us={}",
        if cfg!(feature = "diagnostics") {
            "on"
        } else {
            "off"
        },
        final_state.ticks,
        final_state.state_hash,
        final_state.framebuffer_hash,
        started.elapsed().as_micros()
    );
    #[cfg(feature = "diagnostics")]
    if let Some(observation) = diagnostics.as_ref() {
        print_diagnostics(observation);
    }

    Ok(())
}
