#![deny(warnings)]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    mem::size_of,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};

use gotoo_pixel_engine::{
    ActionId, Rect, Size,
    ui::{
        UiTheme,
        experimental::{self, UiId, UiNavInput, UiOutput, UiStateStore},
        experimental_spatial::{
            GridSpec, SpatialCard, SpatialInput, SpatialOutput, SpatialState,
            run_card_grid_headless,
        },
    },
};

const WARMUP_ITERATIONS: usize = 250;
const SAMPLE_ITERATIONS: usize = 2_000;

const ACTION_A: ActionId = ActionId::new("mfe001c.a");
const ACTION_B: ActionId = ActionId::new("mfe001c.b");
const ACTION_C: ActionId = ActionId::new("mfe001c.c");
const ACTION_D: ActionId = ActionId::new("mfe001c.d");
const ACTION_E: ActionId = ActionId::new("mfe001c.e");
const ACTION_F: ActionId = ActionId::new("mfe001c.f");

static COUNT_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static ALLOCATION_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOCATION_BYTES: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        record_allocation(ptr, layout.size());
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        record_allocation(ptr, layout.size());
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        record_allocation(new_ptr, new_size);
        new_ptr
    }
}

fn record_allocation(ptr: *mut u8, bytes: usize) {
    if !ptr.is_null() && COUNT_ALLOCATIONS.load(Ordering::Relaxed) {
        ALLOCATION_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOCATION_BYTES.fetch_add(bytes as u64, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Observation {
    logical_items: usize,
    persistent_entries: usize,
}

#[derive(Debug)]
struct ProbeStats {
    label: &'static str,
    logical_items: usize,
    persistent_entries: usize,
    allocation_calls_median: u64,
    allocation_calls_p95: u64,
    allocation_bytes_median: u64,
    allocation_bytes_p95: u64,
    time_ns_median: u64,
    time_ns_p95: u64,
    time_ns_min: u64,
    time_ns_max: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        eprintln!("WARNING: run this probe with --release for reportable timing data.");
    }

    println!("GPE.UI MFE-001C COST PROBE");
    println!("os={} arch={}", std::env::consts::OS, std::env::consts::ARCH);
    println!(
        "warmup_iterations={} sample_iterations={}",
        WARMUP_ITERATIONS, SAMPLE_ITERATIONS
    );
    println!(
        "allocation_bytes = cumulative bytes requested by alloc/alloc_zeroed/realloc while one transaction is counted"
    );
    println!();

    let theme = UiTheme::default();
    let mut tiny_state = UiStateStore::default();
    let tiny = measure("tiny", || {
        let (output, ()) = experimental::run_headless(
            Size {
                width: 320,
                height: 180,
            },
            &mut tiny_state,
            UiNavInput::default(),
            theme,
            |ui| {
                ui.panel(|ui| {
                    ui.text("PAUSED");
                    let _ = ui.button("RESUME");
                });
            },
        );
        black_box(output.dump().len());
        let metrics = output.metrics();
        Observation {
            logical_items: metrics.node_count,
            persistent_entries: metrics.persistent_state_entries,
        }
    });

    let ids = stable_ids(theme);
    let cards = [
        card(ids[0], "ALPHA", ACTION_A),
        card(ids[1], "BRAVO", ACTION_B),
        card(ids[2], "CHARLIE", ACTION_C),
        card(ids[3], "DELTA", ACTION_D),
        card(ids[4], "ECHO", ACTION_E),
        card(ids[5], "FOXTROT", ACTION_F),
    ];
    let mut spatial_state = SpatialState::default();
    let spatial = measure("spatial_grid_6", || {
        let output = run_card_grid_headless(
            Rect {
                x: 0,
                y: 0,
                width: 464,
                height: 174,
            },
            &mut spatial_state,
            SpatialInput::default(),
            GridSpec {
                min_cell_width: 118,
                preferred_cell_height: 78,
                gap: 8,
                padding: 6,
            },
            &cards,
        );
        black_box(output.dump().len());
        Observation {
            logical_items: output.layouts().len(),
            persistent_entries: usize::from(spatial_state.focused_id().is_some())
                + usize::from(spatial_state.pointer_capture_id().is_some())
                + spatial_state.touch_capture_count(),
        }
    });

    print_stats(&tiny);
    print_stats(&spatial);

    println!("TYPE SIZES (inline only; heap capacity excluded)");
    println!("UiStateStore={} bytes", size_of::<UiStateStore>());
    println!("UiOutput={} bytes", size_of::<UiOutput>());
    println!("SpatialState={} bytes", size_of::<SpatialState>());
    println!("SpatialOutput={} bytes", size_of::<SpatialOutput>());

    let executable = std::env::current_exe()?;
    let executable_bytes = std::fs::metadata(&executable)?.len();
    println!();
    println!("CURRENT PROBE EXECUTABLE");
    println!("path={}", executable.display());
    println!("bytes={executable_bytes}");
    println!();
    println!("NOTE: executable size above is observational, not an incremental UI delta.");
    println!("Native/Web artifact deltas require bounded same-toolchain comparison builds.");

    Ok(())
}

fn card(id: UiId, title: &'static str, action: ActionId) -> SpatialCard<'static> {
    SpatialCard {
        id,
        title,
        subtitle: "PROBE",
        image: None,
        action,
    }
}

fn stable_ids(theme: UiTheme) -> Vec<UiId> {
    let mut state = UiStateStore::default();
    let (_, ids) = experimental::run_headless(
        Size {
            width: 320,
            height: 180,
        },
        &mut state,
        UiNavInput::default(),
        theme,
        |ui| {
            ["alpha", "bravo", "charlie", "delta", "echo", "foxtrot"]
                .into_iter()
                .map(|key| ui.keyed(key, |ui| ui.button(key).id()))
                .collect::<Vec<_>>()
        },
    );
    ids
}

fn measure(mut label: &'static str, mut run: impl FnMut() -> Observation) -> ProbeStats {
    for _ in 0..WARMUP_ITERATIONS {
        black_box(run());
    }

    let mut times = Vec::with_capacity(SAMPLE_ITERATIONS);
    let mut allocation_calls = Vec::with_capacity(SAMPLE_ITERATIONS);
    let mut allocation_bytes = Vec::with_capacity(SAMPLE_ITERATIONS);
    let mut last_observation = Observation::default();

    for _ in 0..SAMPLE_ITERATIONS {
        ALLOCATION_CALLS.store(0, Ordering::Relaxed);
        ALLOCATION_BYTES.store(0, Ordering::Relaxed);

        COUNT_ALLOCATIONS.store(true, Ordering::Relaxed);
        let started = Instant::now();
        let observation = black_box(run());
        let elapsed = started.elapsed();
        COUNT_ALLOCATIONS.store(false, Ordering::Relaxed);

        last_observation = observation;
        times.push(u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX));
        allocation_calls.push(ALLOCATION_CALLS.load(Ordering::Relaxed));
        allocation_bytes.push(ALLOCATION_BYTES.load(Ordering::Relaxed));
    }

    times.sort_unstable();
    allocation_calls.sort_unstable();
    allocation_bytes.sort_unstable();

    if label.is_empty() {
        label = "unnamed";
    }

    ProbeStats {
        label,
        logical_items: last_observation.logical_items,
        persistent_entries: last_observation.persistent_entries,
        allocation_calls_median: percentile(&allocation_calls, 50),
        allocation_calls_p95: percentile(&allocation_calls, 95),
        allocation_bytes_median: percentile(&allocation_bytes, 50),
        allocation_bytes_p95: percentile(&allocation_bytes, 95),
        time_ns_median: percentile(&times, 50),
        time_ns_p95: percentile(&times, 95),
        time_ns_min: times.first().copied().unwrap_or(0),
        time_ns_max: times.last().copied().unwrap_or(0),
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = sorted
        .len()
        .saturating_sub(1)
        .saturating_mul(percentile.min(100))
        / 100;
    sorted[index]
}

fn print_stats(stats: &ProbeStats) {
    println!("{}", stats.label.to_ascii_uppercase());
    println!("logical_items={}", stats.logical_items);
    println!("persistent_logical_entries={}", stats.persistent_entries);
    println!(
        "allocation_calls median={} p95={}",
        stats.allocation_calls_median, stats.allocation_calls_p95
    );
    println!(
        "allocation_bytes median={} p95={}",
        stats.allocation_bytes_median, stats.allocation_bytes_p95
    );
    println!(
        "transaction_ns min={} median={} p95={} max={}",
        stats.time_ns_min, stats.time_ns_median, stats.time_ns_p95, stats.time_ns_max
    );
    println!();
}
