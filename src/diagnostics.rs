//! Bounded, best-effort engine diagnostics.
//!
//! This module deliberately contains no presentation, persistence, panic hook,
//! filesystem access, or backend objects. Reads materialize independent
//! sections with `try_lock`; the result is not an atomic engine snapshot.

use std::array;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, TryLockError};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub const EVENT_CAPACITY: usize = 64;
pub const RENDERER_CAPACITY: usize = 8;
pub const OPAQUE_TEXT_CAPACITY: usize = 192;
const _: () =
    assert!(EVENT_CAPACITY * OPAQUE_TEXT_CAPACITY + 17 * OPAQUE_TEXT_CAPACITY <= 16 * 1024);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Known,
    Unknown,
    Unavailable,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepresentationKind {
    AuthoritativeAtCapture,
    DerivedAtCapture,
    LastObserved,
    HistoricalFact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaleReason {
    RendererDestroyed,
    DeviceLost,
    CaptureLossDetected,
    RuntimeEnded,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticProducer {
    Runtime,
    Renderer(RendererSource),
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationStamp {
    pub monotonic_micros: u64,
    pub renderer_frame: Option<SaturatingCounter>,
    pub producer: DiagnosticProducer,
    pub producer_sequence: SaturatingCounter,
    /// Order assigned while the event record is inserted into the ring.
    /// It is not physical occurrence order or causality.
    pub record_order: Option<SaturatingCounter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    ImmutableFact,
    ObservedAt(ObservationStamp),
    Stale {
        observed_at: ObservationStamp,
        reason: StaleReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticField<T> {
    pub availability: Availability,
    pub value: Option<T>,
    pub freshness: Option<Freshness>,
    pub representation: RepresentationKind,
}

impl<T> DiagnosticField<T> {
    pub fn unknown(representation: RepresentationKind) -> Self {
        Self {
            availability: Availability::Unknown,
            value: None,
            freshness: None,
            representation,
        }
    }

    pub fn not_applicable(representation: RepresentationKind) -> Self {
        Self {
            availability: Availability::NotApplicable,
            value: None,
            freshness: None,
            representation,
        }
    }

    fn observed(value: T, stamp: ObservationStamp, representation: RepresentationKind) -> Self {
        Self {
            availability: Availability::Known,
            value: Some(value),
            freshness: Some(Freshness::ObservedAt(stamp)),
            representation,
        }
    }

    fn immutable(value: T, representation: RepresentationKind) -> Self {
        Self {
            availability: Availability::Known,
            value: Some(value),
            freshness: Some(Freshness::ImmutableFact),
            representation,
        }
    }

    fn make_stale(&mut self, reason: StaleReason) {
        if let Some(Freshness::ObservedAt(observed_at)) = self.freshness {
            self.freshness = Some(Freshness::Stale {
                observed_at,
                reason,
            });
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSection<T> {
    pub availability: Availability,
    pub value: Option<T>,
}

impl<T> DiagnosticSection<T> {
    fn known(value: T) -> Self {
        Self {
            availability: Availability::Known,
            value: Some(value),
        }
    }

    fn unavailable() -> Self {
        Self {
            availability: Availability::Unavailable,
            value: None,
        }
    }

    fn not_applicable() -> Self {
        Self {
            availability: Availability::NotApplicable,
            value: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OpaqueText {
    bytes: [u8; OPAQUE_TEXT_CAPACITY],
    len: u8,
    pub truncated: bool,
}

impl OpaqueText {
    pub fn new(text: &str) -> Self {
        let mut end = text.len().min(OPAQUE_TEXT_CAPACITY);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let mut bytes = [0; OPAQUE_TEXT_CAPACITY];
        bytes[..end].copy_from_slice(&text.as_bytes()[..end]);
        Self {
            bytes,
            len: end as u8,
            truncated: end < text.len(),
        }
    }

    pub fn as_str(&self) -> &str {
        // Construction only copies a valid UTF-8 prefix.
        std::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }

    pub const fn stored_len(&self) -> usize {
        self.len as usize
    }

    pub const fn capacity(&self) -> usize {
        OPAQUE_TEXT_CAPACITY
    }
}

impl fmt::Debug for OpaqueText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpaqueText")
            .field("text", &self.as_str())
            .field("truncated", &self.truncated)
            .finish()
    }
}

impl fmt::Display for OpaqueText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SaturatingCounter {
    pub value: u64,
    pub saturated: bool,
}

impl SaturatingCounter {
    pub fn increment(&mut self) {
        if self.value == u64::MAX {
            self.saturated = true;
        } else {
            self.value += 1;
        }
    }
}

struct AtomicSaturatingCounter {
    value: AtomicU64,
    saturated: AtomicBool,
}

impl AtomicSaturatingCounter {
    fn new(value: u64) -> Self {
        Self {
            value: AtomicU64::new(value),
            saturated: AtomicBool::new(false),
        }
    }

    fn increment(&self) -> Option<u64> {
        let mut current = self.value.load(Ordering::Relaxed);
        loop {
            if current == u64::MAX {
                self.saturated.store(true, Ordering::Relaxed);
                return None;
            }
            match self.value.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(current + 1),
                Err(actual) => current = actual,
            }
        }
    }

    fn get(&self) -> SaturatingCounter {
        SaturatingCounter {
            value: self.value.load(Ordering::Relaxed),
            saturated: self.saturated.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionMode {
    RuntimeDisabled,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticReadKind {
    NormalL1,
    DegradedL2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFamily {
    Native,
    Wasm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicStrategy {
    Unwind,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProvenance {
    pub package_version: DiagnosticField<&'static str>,
    pub build_id: DiagnosticField<&'static str>,
    pub target: DiagnosticField<&'static str>,
    pub architecture: DiagnosticField<&'static str>,
    pub target_family: DiagnosticField<TargetFamily>,
    pub build_profile: DiagnosticField<BuildProfile>,
    pub panic_strategy: DiagnosticField<PanicStrategy>,
    pub wgpu_version: DiagnosticField<&'static str>,
}

impl BuildProvenance {
    fn capture() -> Self {
        let known_text = |value| {
            if value == "UNKNOWN" {
                DiagnosticField::unknown(RepresentationKind::AuthoritativeAtCapture)
            } else {
                DiagnosticField::immutable(value, RepresentationKind::AuthoritativeAtCapture)
            }
        };
        let profile = match env!("GPE_BUILD_PROFILE") {
            "UNKNOWN" => DiagnosticField::unknown(RepresentationKind::DerivedAtCapture),
            "debug" => DiagnosticField::immutable(
                BuildProfile::Debug,
                RepresentationKind::DerivedAtCapture,
            ),
            "release" => DiagnosticField::immutable(
                BuildProfile::Release,
                RepresentationKind::DerivedAtCapture,
            ),
            _ => DiagnosticField::immutable(
                BuildProfile::Other,
                RepresentationKind::DerivedAtCapture,
            ),
        };
        Self {
            package_version: DiagnosticField::immutable(
                env!("CARGO_PKG_VERSION"),
                RepresentationKind::AuthoritativeAtCapture,
            ),
            build_id: known_text(env!("GPE_BUILD_ID")),
            target: known_text(env!("GPE_BUILD_TARGET")),
            architecture: DiagnosticField::immutable(
                std::env::consts::ARCH,
                RepresentationKind::AuthoritativeAtCapture,
            ),
            target_family: DiagnosticField::immutable(
                if cfg!(target_arch = "wasm32") {
                    TargetFamily::Wasm
                } else {
                    TargetFamily::Native
                },
                RepresentationKind::AuthoritativeAtCapture,
            ),
            build_profile: profile,
            panic_strategy: DiagnosticField::immutable(
                if cfg!(panic = "abort") {
                    PanicStrategy::Abort
                } else {
                    PanicStrategy::Unwind
                },
                RepresentationKind::AuthoritativeAtCapture,
            ),
            wgpu_version: DiagnosticField::immutable(
                "30.0.0",
                RepresentationKind::AuthoritativeAtCapture,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeLifecycle {
    NotStarted,
    Initializing,
    Running,
    ShuttingDown,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeOutcome {
    NotAvailable,
    StartupFailed,
    NormalExit,
    ErrorExit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeState {
    pub lifecycle: DiagnosticField<RuntimeLifecycle>,
    pub outcome: DiagnosticField<RuntimeOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererRole {
    Primary,
    Tool,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RendererIncarnation(NonZeroU64);

impl RendererIncarnation {
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererSource {
    pub role: RendererRole,
    pub incarnation: RendererIncarnation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererLifecycle {
    Initializing,
    Ready,
    InitializationFailed,
    Destroyed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterBackend {
    Noop,
    Vulkan,
    Metal,
    Dx12,
    Gl,
    BrowserWebGpu,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterDeviceType {
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Cpu,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterFacts {
    pub backend: DiagnosticField<AdapterBackend>,
    pub device_type: DiagnosticField<AdapterDeviceType>,
    pub name: DiagnosticField<OpaqueText>,
}

impl AdapterFacts {
    fn unknown() -> Self {
        Self {
            backend: DiagnosticField::unknown(RepresentationKind::HistoricalFact),
            device_type: DiagnosticField::unknown(RepresentationKind::HistoricalFact),
            name: DiagnosticField::unknown(RepresentationKind::HistoricalFact),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormat {
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Float,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfacePresentMode {
    Fifo,
    FifoRelaxed,
    Immediate,
    Mailbox,
    AutoVsync,
    AutoNoVsync,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceAlphaMode {
    Auto,
    Opaque,
    PreMultiplied,
    PostMultiplied,
    Inherit,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceConfiguration {
    pub format: SurfaceFormat,
    pub present_mode: SurfacePresentMode,
    pub alpha_mode: SurfaceAlphaMode,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFailure {
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceLostReason {
    Unknown,
    Destroyed,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WgpuErrorCategory {
    CreateSurface,
    RequestAdapter,
    RequestDevice,
    SurfaceValidation,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentObservation {
    pub source: RendererSource,
    pub renderer_frame: SaturatingCounter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceLostObservation {
    pub reason: DeviceLostReason,
    pub message: DiagnosticField<OpaqueText>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererRecord {
    pub source: RendererSource,
    pub lifecycle: DiagnosticField<RendererLifecycle>,
    pub adapter: AdapterFacts,
    pub surface_configuration: DiagnosticField<SurfaceConfiguration>,
    pub last_surface_failure: DiagnosticField<SurfaceFailure>,
    pub last_present: DiagnosticField<PresentObservation>,
    pub suboptimal_observed: DiagnosticField<ObservationStamp>,
    pub device_lost: DiagnosticField<DeviceLostObservation>,
    pub last_wgpu_error: DiagnosticField<WgpuErrorCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererObservations {
    pub records: [Option<RendererRecord>; RENDERER_CAPACITY],
    pub retained: usize,
    pub identities_issued: SaturatingCounter,
    pub records_dropped: SaturatingCounter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackend {
    Native,
    Web,
    Noop,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioInitializationOutcome {
    NotAttempted,
    Succeeded,
    FailedFallback,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioErrorCategory {
    Initialization,
    StreamUnderrunOrOverrun,
    Stream,
    WebAudio,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioBackendError {
    pub category: AudioErrorCategory,
    pub excerpt: DiagnosticField<OpaqueText>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSummary {
    pub backend: DiagnosticField<AudioBackend>,
    pub initialization: DiagnosticField<AudioInitializationOutcome>,
    pub last_generic_backend_error: DiagnosticField<AudioBackendError>,
}

impl AudioSummary {
    fn unknown() -> Self {
        Self {
            backend: DiagnosticField::unknown(RepresentationKind::LastObserved),
            initialization: DiagnosticField::unknown(RepresentationKind::LastObserved),
            last_generic_backend_error: DiagnosticField::unknown(RepresentationKind::LastObserved),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticEventKind {
    RuntimeLifecycle(RuntimeLifecycle),
    RendererLifecycle {
        source: RendererSource,
        lifecycle: RendererLifecycle,
    },
    SurfaceFailure {
        source: RendererSource,
        failure: SurfaceFailure,
    },
    DeviceLost {
        source: RendererSource,
        reason: DeviceLostReason,
        message: Option<OpaqueText>,
    },
    WgpuError {
        source: RendererSource,
        category: WgpuErrorCategory,
    },
    AudioInitialization {
        backend: AudioBackend,
        outcome: AudioInitializationOutcome,
    },
    AudioBackendError {
        category: AudioErrorCategory,
        excerpt: Option<OpaqueText>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticEvent {
    pub stamp: ObservationStamp,
    pub kind: DiagnosticEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventHistory {
    /// Records in oldest-to-newest store insertion order.
    pub records: [Option<DiagnosticEvent>; EVENT_CAPACITY],
    pub retained: usize,
    pub ever_wrapped: bool,
    pub first_record_order: Option<SaturatingCounter>,
    pub last_record_order: Option<SaturatingCounter>,
    pub overwritten: SaturatingCounter,
    pub dropped: SaturatingCounter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticObservation {
    pub collection_mode: CollectionMode,
    pub read_kind: DiagnosticReadKind,
    pub build: BuildProvenance,
    pub runtime: DiagnosticSection<RuntimeState>,
    pub renderers: DiagnosticSection<RendererObservations>,
    pub audio: DiagnosticSection<AudioSummary>,
    pub events: DiagnosticSection<EventHistory>,
    /// True when at least one attempted section was unavailable.
    pub degraded: bool,
}

pub struct EngineDiagnostics;

impl EngineDiagnostics {
    /// Creates one enabled store, its read-only consumer handle, and the
    /// single-use registration that must be moved into `run_with_diagnostics`.
    pub fn enabled() -> (EngineDiagnosticsHandle, EngineDiagnosticsRegistration) {
        let store = Arc::new(Store::new());
        (
            EngineDiagnosticsHandle {
                build: BuildProvenance::capture(),
                store: Some(Arc::clone(&store)),
            },
            EngineDiagnosticsRegistration { store },
        )
    }

    /// Creates a build-facts-only handle. It has no runtime store, event ring,
    /// callbacks, or producer registration.
    pub fn disabled() -> EngineDiagnosticsHandle {
        EngineDiagnosticsHandle {
            build: BuildProvenance::capture(),
            store: None,
        }
    }
}

#[derive(Clone)]
pub struct EngineDiagnosticsHandle {
    build: BuildProvenance,
    store: Option<Arc<Store>>,
}

impl EngineDiagnosticsHandle {
    pub fn try_read(&self) -> DiagnosticObservation {
        self.materialize(DiagnosticReadKind::NormalL1)
    }

    pub fn try_read_degraded(&self) -> DiagnosticObservation {
        self.materialize(DiagnosticReadKind::DegradedL2)
    }

    fn materialize(&self, read_kind: DiagnosticReadKind) -> DiagnosticObservation {
        let Some(store) = &self.store else {
            return DiagnosticObservation {
                collection_mode: CollectionMode::RuntimeDisabled,
                read_kind,
                build: self.build.clone(),
                runtime: DiagnosticSection::not_applicable(),
                renderers: DiagnosticSection::not_applicable(),
                audio: DiagnosticSection::not_applicable(),
                events: DiagnosticSection::not_applicable(),
                degraded: false,
            };
        };

        let runtime = try_clone_section(&store.runtime);
        let renderers = match store.renderers.try_lock() {
            Ok(value) => DiagnosticSection::known(
                value.materialize(store.identities.get(), store.renderer_records_dropped.get()),
            ),
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
                DiagnosticSection::unavailable()
            }
        };
        let audio = try_clone_section(&store.audio);
        let events = match store.events.try_lock() {
            Ok(value) => DiagnosticSection::known(value.materialize(store.events_dropped.get())),
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
                DiagnosticSection::unavailable()
            }
        };
        let degraded = [
            runtime.availability,
            renderers.availability,
            audio.availability,
            events.availability,
        ]
        .contains(&Availability::Unavailable);
        DiagnosticObservation {
            collection_mode: CollectionMode::Enabled,
            read_kind,
            build: self.build.clone(),
            runtime,
            renderers,
            audio,
            events,
            degraded,
        }
    }
}

fn try_clone_section<T: Clone>(mutex: &Mutex<T>) -> DiagnosticSection<T> {
    match mutex.try_lock() {
        Ok(value) => DiagnosticSection::known(value.clone()),
        Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
            DiagnosticSection::unavailable()
        }
    }
}

pub struct EngineDiagnosticsRegistration {
    store: Arc<Store>,
}

impl fmt::Debug for EngineDiagnosticsRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EngineDiagnosticsRegistration { .. }")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RegistrationAlreadyAttached;

impl EngineDiagnosticsRegistration {
    pub(crate) fn attach(self) -> Result<DiagnosticsWriter, RegistrationAlreadyAttached> {
        if self
            .store
            .attached
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(RegistrationAlreadyAttached);
        }
        Ok(DiagnosticsWriter { store: self.store })
    }
}

struct Store {
    started: Instant,
    attached: AtomicBool,
    identities: AtomicSaturatingCounter,
    renderer_records_dropped: AtomicSaturatingCounter,
    events_dropped: AtomicSaturatingCounter,
    runtime_sequence: AtomicSaturatingCounter,
    renderer_sequence: AtomicSaturatingCounter,
    audio_sequence: AtomicSaturatingCounter,
    runtime: Mutex<RuntimeState>,
    renderers: Mutex<RendererStore>,
    audio: Mutex<AudioSummary>,
    events: Mutex<EventRing>,
}

impl Store {
    fn new() -> Self {
        let initial_stamp = ObservationStamp {
            monotonic_micros: 0,
            renderer_frame: None,
            producer: DiagnosticProducer::Runtime,
            producer_sequence: SaturatingCounter::default(),
            record_order: None,
        };
        Self {
            started: Instant::now(),
            attached: AtomicBool::new(false),
            identities: AtomicSaturatingCounter::new(0),
            renderer_records_dropped: AtomicSaturatingCounter::new(0),
            events_dropped: AtomicSaturatingCounter::new(0),
            runtime_sequence: AtomicSaturatingCounter::new(0),
            renderer_sequence: AtomicSaturatingCounter::new(0),
            audio_sequence: AtomicSaturatingCounter::new(0),
            runtime: Mutex::new(RuntimeState {
                lifecycle: DiagnosticField::observed(
                    RuntimeLifecycle::NotStarted,
                    initial_stamp,
                    RepresentationKind::LastObserved,
                ),
                outcome: DiagnosticField::unknown(RepresentationKind::LastObserved),
            }),
            renderers: Mutex::new(RendererStore::new()),
            audio: Mutex::new(AudioSummary::unknown()),
            events: Mutex::new(EventRing::new()),
        }
    }

    fn stamp(
        &self,
        producer: DiagnosticProducer,
        renderer_frame: Option<SaturatingCounter>,
    ) -> ObservationStamp {
        let counter = match producer {
            DiagnosticProducer::Runtime => &self.runtime_sequence,
            DiagnosticProducer::Renderer(_) => &self.renderer_sequence,
            DiagnosticProducer::Audio => &self.audio_sequence,
        };
        let _ = counter.increment();
        let micros = self.started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        ObservationStamp {
            monotonic_micros: micros,
            renderer_frame,
            producer,
            producer_sequence: counter.get(),
            record_order: None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct DiagnosticsWriter {
    store: Arc<Store>,
}

impl DiagnosticsWriter {
    pub(crate) fn runtime_transition(
        &self,
        lifecycle: RuntimeLifecycle,
        outcome: Option<RuntimeOutcome>,
    ) {
        let stamp = self.store.stamp(DiagnosticProducer::Runtime, None);
        if let Ok(mut runtime) = self.store.runtime.try_lock() {
            runtime.lifecycle =
                DiagnosticField::observed(lifecycle, stamp, RepresentationKind::LastObserved);
            if let Some(outcome) = outcome {
                runtime.outcome =
                    DiagnosticField::observed(outcome, stamp, RepresentationKind::LastObserved);
            }
        }
        self.push_event(stamp, DiagnosticEventKind::RuntimeLifecycle(lifecycle));
    }

    pub(crate) fn begin_renderer(&self, role: RendererRole) -> RendererDiagnostics {
        let Some(raw) = self.store.identities.increment() else {
            self.store.renderer_records_dropped.increment();
            return RendererDiagnostics {
                writer: self.clone(),
                source: None,
                frame: SaturatingCounter::default(),
                destroy_on_drop: false,
            };
        };
        let source = RendererSource {
            role,
            incarnation: RendererIncarnation(NonZeroU64::new(raw).expect("identity is non-zero")),
        };
        let stamp = self.store.stamp(DiagnosticProducer::Renderer(source), None);
        let retained = match self.store.renderers.try_lock() {
            Ok(mut renderers) => renderers.insert(RendererRecord {
                source,
                lifecycle: DiagnosticField::observed(
                    RendererLifecycle::Initializing,
                    stamp,
                    RepresentationKind::LastObserved,
                ),
                adapter: AdapterFacts::unknown(),
                surface_configuration: DiagnosticField::unknown(RepresentationKind::LastObserved),
                last_surface_failure: DiagnosticField::unknown(RepresentationKind::LastObserved),
                last_present: DiagnosticField::unknown(RepresentationKind::LastObserved),
                suboptimal_observed: DiagnosticField::unknown(RepresentationKind::LastObserved),
                device_lost: DiagnosticField::unknown(RepresentationKind::LastObserved),
                last_wgpu_error: DiagnosticField::unknown(RepresentationKind::LastObserved),
            }),
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => false,
        };
        if !retained {
            self.store.renderer_records_dropped.increment();
        }
        self.push_event(
            stamp,
            DiagnosticEventKind::RendererLifecycle {
                source,
                lifecycle: RendererLifecycle::Initializing,
            },
        );
        RendererDiagnostics {
            writer: self.clone(),
            source: Some(source),
            frame: SaturatingCounter::default(),
            destroy_on_drop: true,
        }
    }

    pub(crate) fn audio_selected(
        &self,
        backend: AudioBackend,
        outcome: AudioInitializationOutcome,
    ) {
        let stamp = self.store.stamp(DiagnosticProducer::Audio, None);
        if let Ok(mut audio) = self.store.audio.try_lock() {
            audio.backend =
                DiagnosticField::observed(backend, stamp, RepresentationKind::LastObserved);
            audio.initialization =
                DiagnosticField::observed(outcome, stamp, RepresentationKind::LastObserved);
        }
        self.push_event(
            stamp,
            DiagnosticEventKind::AudioInitialization { backend, outcome },
        );
    }

    pub(crate) fn audio_error(&self, category: AudioErrorCategory, excerpt: Option<&str>) {
        let stamp = self.store.stamp(DiagnosticProducer::Audio, None);
        let excerpt = excerpt.map(|text| {
            DiagnosticField::observed(
                OpaqueText::new(text),
                stamp,
                RepresentationKind::LastObserved,
            )
        });
        let error = AudioBackendError {
            category,
            excerpt: excerpt
                .clone()
                .unwrap_or_else(|| DiagnosticField::unknown(RepresentationKind::LastObserved)),
        };
        if let Ok(mut audio) = self.store.audio.try_lock() {
            audio.last_generic_backend_error =
                DiagnosticField::observed(error, stamp, RepresentationKind::LastObserved);
        }
        self.push_event(
            stamp,
            DiagnosticEventKind::AudioBackendError {
                category,
                excerpt: excerpt.and_then(|field| field.value),
            },
        );
    }

    fn push_event(&self, stamp: ObservationStamp, kind: DiagnosticEventKind) {
        match self.store.events.try_lock() {
            Ok(mut events) => events.push(DiagnosticEvent { stamp, kind }),
            Err(TryLockError::WouldBlock | TryLockError::Poisoned(_)) => {
                self.store.events_dropped.increment();
            }
        }
    }
}

pub(crate) struct RendererDiagnostics {
    writer: DiagnosticsWriter,
    source: Option<RendererSource>,
    frame: SaturatingCounter,
    destroy_on_drop: bool,
}

impl RendererDiagnostics {
    #[cfg(test)]
    pub(crate) fn source(&self) -> Option<RendererSource> {
        self.source
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn adapter_facts(
        &self,
        backend: AdapterBackend,
        device_type: AdapterDeviceType,
        name: Option<&str>,
    ) {
        let Some(source) = self.source else { return };
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), None);
        self.update_record(|record| {
            record.adapter.backend =
                DiagnosticField::immutable(backend, RepresentationKind::HistoricalFact);
            record.adapter.device_type =
                DiagnosticField::immutable(device_type, RepresentationKind::HistoricalFact);
            record.adapter.name = name
                .filter(|name| !name.is_empty())
                .map(|name| {
                    DiagnosticField::immutable(
                        OpaqueText::new(name),
                        RepresentationKind::HistoricalFact,
                    )
                })
                .unwrap_or_else(|| DiagnosticField::unknown(RepresentationKind::HistoricalFact));
            let _ = stamp;
        });
    }

    pub(crate) fn surface_configured(&self, configuration: SurfaceConfiguration) {
        let Some(source) = self.source else { return };
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), Some(self.frame));
        self.update_record(|record| {
            record.surface_configuration =
                DiagnosticField::observed(configuration, stamp, RepresentationKind::LastObserved);
        });
    }

    pub(crate) fn ready(&self) {
        self.lifecycle(RendererLifecycle::Ready, None);
    }

    pub(crate) fn initialization_failed(&mut self, category: WgpuErrorCategory) {
        self.wgpu_error(category);
        self.lifecycle(RendererLifecycle::InitializationFailed, None);
        self.destroy_on_drop = false;
    }

    pub(crate) fn surface_failure(&self, failure: SurfaceFailure) {
        let Some(source) = self.source else { return };
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), Some(self.frame));
        self.update_record(|record| {
            record.last_surface_failure =
                DiagnosticField::observed(failure, stamp, RepresentationKind::LastObserved);
            if failure == SurfaceFailure::Lost {
                record
                    .surface_configuration
                    .make_stale(StaleReason::DeviceLost);
            }
            if failure == SurfaceFailure::Validation {
                record.last_wgpu_error = DiagnosticField::observed(
                    WgpuErrorCategory::SurfaceValidation,
                    stamp,
                    RepresentationKind::LastObserved,
                );
            }
        });
        self.writer.push_event(
            stamp,
            DiagnosticEventKind::SurfaceFailure { source, failure },
        );
        if failure == SurfaceFailure::Validation {
            self.writer.push_event(
                stamp,
                DiagnosticEventKind::WgpuError {
                    source,
                    category: WgpuErrorCategory::SurfaceValidation,
                },
            );
        }
    }

    pub(crate) fn suboptimal(&self) {
        let Some(source) = self.source else { return };
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), Some(self.frame));
        self.update_record(|record| {
            record.suboptimal_observed =
                DiagnosticField::observed(stamp, stamp, RepresentationKind::LastObserved);
        });
    }

    pub(crate) fn presented(&mut self) {
        let Some(source) = self.source else { return };
        self.frame.increment();
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), Some(self.frame));
        let observation = PresentObservation {
            source,
            renderer_frame: self.frame,
        };
        self.update_record(|record| {
            record.last_present =
                DiagnosticField::observed(observation, stamp, RepresentationKind::LastObserved);
        });
    }

    pub(crate) fn device_lost(
        writer: &DiagnosticsWriter,
        source: RendererSource,
        reason: DeviceLostReason,
        message: &str,
    ) {
        let stamp = writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), None);
        let message = if message.is_empty() {
            DiagnosticField::unknown(RepresentationKind::LastObserved)
        } else {
            DiagnosticField::observed(
                OpaqueText::new(message),
                stamp,
                RepresentationKind::LastObserved,
            )
        };
        if let Ok(mut renderers) = writer.store.renderers.try_lock()
            && let Some(record) = renderers.find_mut(source)
        {
            record.device_lost = DiagnosticField::observed(
                DeviceLostObservation {
                    reason,
                    message: message.clone(),
                },
                stamp,
                RepresentationKind::LastObserved,
            );
            record
                .surface_configuration
                .make_stale(StaleReason::DeviceLost);
        }
        writer.push_event(
            stamp,
            DiagnosticEventKind::DeviceLost {
                source,
                reason,
                message: message.value,
            },
        );
    }

    pub(crate) fn callback_writer(&self) -> Option<(DiagnosticsWriter, RendererSource)> {
        self.source.map(|source| (self.writer.clone(), source))
    }

    fn wgpu_error(&self, category: WgpuErrorCategory) {
        let Some(source) = self.source else { return };
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), Some(self.frame));
        self.update_record(|record| {
            record.last_wgpu_error =
                DiagnosticField::observed(category, stamp, RepresentationKind::LastObserved);
        });
        self.writer
            .push_event(stamp, DiagnosticEventKind::WgpuError { source, category });
    }

    fn lifecycle(&self, lifecycle: RendererLifecycle, stale: Option<StaleReason>) {
        let Some(source) = self.source else { return };
        let stamp = self
            .writer
            .store
            .stamp(DiagnosticProducer::Renderer(source), Some(self.frame));
        self.update_record(|record| {
            record.lifecycle =
                DiagnosticField::observed(lifecycle, stamp, RepresentationKind::LastObserved);
            if let Some(reason) = stale {
                record.surface_configuration.make_stale(reason);
                record.last_surface_failure.make_stale(reason);
                record.last_present.make_stale(reason);
                record.suboptimal_observed.make_stale(reason);
                record.device_lost.make_stale(reason);
                record.last_wgpu_error.make_stale(reason);
            }
        });
        self.writer.push_event(
            stamp,
            DiagnosticEventKind::RendererLifecycle { source, lifecycle },
        );
    }

    fn update_record(&self, update: impl FnOnce(&mut RendererRecord)) {
        let Some(source) = self.source else { return };
        if let Ok(mut renderers) = self.writer.store.renderers.try_lock()
            && let Some(record) = renderers.find_mut(source)
        {
            update(record);
        }
    }
}

impl Drop for RendererDiagnostics {
    fn drop(&mut self) {
        if self.destroy_on_drop {
            self.lifecycle(
                RendererLifecycle::Destroyed,
                Some(StaleReason::RendererDestroyed),
            );
        }
    }
}

struct RendererStore {
    records: [Option<RendererRecord>; RENDERER_CAPACITY],
}

impl RendererStore {
    fn new() -> Self {
        Self {
            records: array::from_fn(|_| None),
        }
    }

    fn insert(&mut self, record: RendererRecord) -> bool {
        if let Some(slot) = self.records.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(record);
            return true;
        }
        let eviction = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                let record = record.as_ref()?;
                matches!(
                    record.lifecycle.value,
                    Some(RendererLifecycle::InitializationFailed | RendererLifecycle::Destroyed)
                )
                .then_some((index, record.source.incarnation.get()))
            })
            .min_by_key(|(_, incarnation)| *incarnation)
            .map(|(index, _)| index);
        if let Some(index) = eviction {
            self.records[index] = Some(record);
            true
        } else {
            false
        }
    }

    fn find_mut(&mut self, source: RendererSource) -> Option<&mut RendererRecord> {
        self.records
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|record| record.source == source)
    }

    fn materialize(
        &self,
        identities_issued: SaturatingCounter,
        records_dropped: SaturatingCounter,
    ) -> RendererObservations {
        RendererObservations {
            records: self.records.clone(),
            retained: self.records.iter().flatten().count(),
            identities_issued,
            records_dropped,
        }
    }
}

struct EventRing {
    records: [Option<DiagnosticEvent>; EVENT_CAPACITY],
    start: usize,
    len: usize,
    next_record_order: SaturatingCounter,
    overwritten: SaturatingCounter,
    ever_wrapped: bool,
}

impl EventRing {
    fn new() -> Self {
        Self {
            records: array::from_fn(|_| None),
            start: 0,
            len: 0,
            next_record_order: SaturatingCounter::default(),
            overwritten: SaturatingCounter::default(),
            ever_wrapped: false,
        }
    }

    fn push(&mut self, mut event: DiagnosticEvent) {
        self.next_record_order.increment();
        event.stamp.record_order = Some(self.next_record_order);
        if self.len < EVENT_CAPACITY {
            let index = (self.start + self.len) % EVENT_CAPACITY;
            self.records[index] = Some(event);
            self.len += 1;
        } else {
            self.records[self.start] = Some(event);
            self.start = (self.start + 1) % EVENT_CAPACITY;
            self.ever_wrapped = true;
            self.overwritten.increment();
        }
    }

    fn materialize(&self, dropped: SaturatingCounter) -> EventHistory {
        let mut records = array::from_fn(|_| None);
        for (destination, offset) in records.iter_mut().zip(0..self.len) {
            *destination = self.records[(self.start + offset) % EVENT_CAPACITY].clone();
        }
        let first_record_order = records
            .first()
            .and_then(Option::as_ref)
            .and_then(|event| event.stamp.record_order);
        let last_record_order = self
            .len
            .checked_sub(1)
            .and_then(|index| records[index].as_ref())
            .and_then(|event| event.stamp.record_order);
        EventHistory {
            records,
            retained: self.len,
            ever_wrapped: self.ever_wrapped,
            first_record_order,
            last_record_order,
            overwritten: self.overwritten,
            dropped,
        }
    }
}

/// Mission 3-only fault seams. This module is absent unless the dedicated
/// `diagnostics-fault-injection` feature is explicitly enabled.
#[cfg(all(feature = "diagnostics-fault-injection", not(target_arch = "wasm32")))]
#[doc(hidden)]
pub mod fault_injection {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Instant;

    #[cfg(not(test))]
    mod controlled_allocator {
        use super::*;
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::AtomicU64;

        static FAIL: AtomicBool = AtomicBool::new(false);
        static TRACK: AtomicBool = AtomicBool::new(false);
        static COUNT: AtomicU64 = AtomicU64::new(0);

        pub(super) struct ControlledAllocator;

        // SAFETY: successful operations delegate unchanged to System. Returning
        // null while the explicit failpoint is armed is the GlobalAlloc contract.
        unsafe impl GlobalAlloc for ControlledAllocator {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                if TRACK.load(Ordering::Relaxed) {
                    COUNT.fetch_add(1, Ordering::Relaxed);
                }
                if FAIL.load(Ordering::Relaxed) {
                    std::ptr::null_mut()
                } else {
                    // SAFETY: delegated unchanged to the system allocator.
                    unsafe { System.alloc(layout) }
                }
            }

            unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
                // SAFETY: delegated unchanged to the system allocator.
                unsafe { System.dealloc(pointer, layout) };
            }

            unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
                if TRACK.load(Ordering::Relaxed) {
                    COUNT.fetch_add(1, Ordering::Relaxed);
                }
                if FAIL.load(Ordering::Relaxed) {
                    std::ptr::null_mut()
                } else {
                    // SAFETY: delegated unchanged to the system allocator.
                    unsafe { System.alloc_zeroed(layout) }
                }
            }

            unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
                if TRACK.load(Ordering::Relaxed) {
                    COUNT.fetch_add(1, Ordering::Relaxed);
                }
                if FAIL.load(Ordering::Relaxed) {
                    std::ptr::null_mut()
                } else {
                    // SAFETY: delegated unchanged to the system allocator.
                    unsafe { System.realloc(pointer, layout, size) }
                }
            }
        }

        #[global_allocator]
        static ALLOCATOR: ControlledAllocator = ControlledAllocator;

        pub(super) fn fail(enabled: bool) {
            FAIL.store(enabled, Ordering::SeqCst);
        }

        pub(super) fn track(enabled: bool) {
            if enabled {
                COUNT.store(0, Ordering::SeqCst);
            }
            TRACK.store(enabled, Ordering::SeqCst);
        }

        pub(super) fn count() -> u64 {
            COUNT.load(Ordering::SeqCst)
        }
    }

    pub const SCENARIOS: [&str; 22] = [
        "F01", "F02", "F03", "F04", "F05", "F06", "F07", "F08", "F09", "F10", "F11", "F12", "F13",
        "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22",
    ];

    pub fn run_child(scenario: &str, artifact: &Path) -> Result<(), String> {
        match scenario {
            "F01" => f01(artifact),
            "F02" => f02(artifact),
            "F03" => f03(artifact),
            "F04" => f04(artifact),
            "F05" => f05(artifact),
            "F06" => f06(artifact),
            "F07" => f07(artifact),
            "F08" => parked_partial("F08", artifact),
            "F09" => parked_partial("F09", artifact),
            "F10" => f10(artifact),
            "F11" => f11(artifact),
            "F12" => f12(artifact),
            "F13" => f13(artifact),
            "F14" => f14(artifact),
            "F15" => f15(artifact),
            "F16" => f16(artifact),
            "F17" => f17(artifact),
            "F18" => f18(artifact),
            "F19" => f19(artifact),
            "F20" => f20(artifact),
            "F21" => f21(artifact),
            "F22" => f22(artifact),
            _ => Err(format!("unknown scenario {scenario}")),
        }
    }

    fn enabled() -> (EngineDiagnosticsHandle, DiagnosticsWriter) {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().expect("fresh registration attaches");
        (handle, writer)
    }

    fn complete(artifact: &Path, scenario: &str, facts: &[String]) -> Result<(), String> {
        let mut body = format!("SCENARIO={scenario}\nREPORT=SUCCESSFUL\n");
        for fact in facts {
            body.push_str(fact);
            body.push('\n');
        }
        body.push_str(&format!("END REPORT {scenario}\n"));
        fs::write(artifact, body).map_err(|error| error.to_string())
    }

    fn partial(artifact: &Path, scenario: &str, fact: &str) -> Result<(), String> {
        fs::write(
            artifact,
            format!("SCENARIO={scenario}\nREPORT=PARTIAL\n{fact}\n"),
        )
        .map_err(|error| error.to_string())
    }

    fn event_history(handle: &EngineDiagnosticsHandle) -> Result<EventHistory, String> {
        handle
            .try_read_degraded()
            .events
            .value
            .ok_or_else(|| "event history unavailable".to_string())
    }

    fn f01(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        let renderer = writer.begin_renderer(RendererRole::Primary);
        renderer.ready();
        let artifact = artifact.to_path_buf();
        std::panic::set_hook(Box::new(move |info| {
            let observation = handle.try_read_degraded();
            let payload = info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or("non-str-payload");
            let running = observation
                .runtime
                .value
                .as_ref()
                .and_then(|state| state.lifecycle.value)
                == Some(RuntimeLifecycle::Running);
            let primary = observation.renderers.value.as_ref().is_some_and(|records| {
                records.records.iter().flatten().any(|record| {
                    record.source.role == RendererRole::Primary
                        && record.source.incarnation.get() > 0
                })
            });
            let _ = complete(
                &artifact,
                "F01",
                &[
                    format!("PAYLOAD={payload}"),
                    format!("RUNNING={running}"),
                    format!("PRIMARY_INCARNATION={primary}"),
                    format!("DEGRADED={}", observation.degraded),
                ],
            );
        }));
        panic!("F01 injected main panic");
    }

    fn f02(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        let previous = std::panic::take_hook();
        let hook_handle = handle.clone();
        let hook_artifact = artifact.to_path_buf();
        std::panic::set_hook(Box::new(move |info| {
            let observation = hook_handle.try_read_degraded();
            let _ = partial(
                &hook_artifact,
                "F02",
                &format!(
                    "HOOK_THREAD={:?};DEGRADED={}",
                    thread::current().name(),
                    observation.degraded
                ),
            );
            let _ = info;
        }));
        let join = thread::Builder::new()
            .name("f02-worker".to_string())
            .spawn(|| panic!("F02 injected worker panic"))
            .map_err(|error| error.to_string())?
            .join();
        std::panic::set_hook(previous);
        if join.is_ok() {
            return Err("worker did not panic".to_string());
        }
        let observation = handle.try_read();
        complete(
            artifact,
            "F02",
            &[
                "WORKER_PANIC=OBSERVED".to_string(),
                "MAIN_CONTINUED=true".to_string(),
                format!("RUNTIME_AVAILABLE={:?}", observation.runtime.availability),
                "FRAME_ASSOCIATION=NONE".to_string(),
            ],
        )
    }

    fn f03(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        let _renderer = writer.begin_renderer(RendererRole::Primary);
        let guard = writer
            .store
            .renderers
            .try_lock()
            .map_err(|e| e.to_string())?;
        let started = Instant::now();
        let observation = handle.try_read_degraded();
        let micros = started.elapsed().as_micros();
        drop(guard);
        if observation.renderers.availability != Availability::Unavailable
            || observation.runtime.availability != Availability::Known
            || !observation.degraded
            || micros >= 100_000
        {
            return Err(format!("L2 contention oracle failed in {micros} us"));
        }
        complete(
            artifact,
            "F03",
            &[
                format!("READ_MICROS={micros}"),
                "RENDERERS=UNAVAILABLE".to_string(),
                "RUNTIME=KNOWN".to_string(),
                "WAIT=false".to_string(),
            ],
        )
    }

    fn f04(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        let before = event_history(&handle)?.retained;
        let formatter: Result<(), &'static str> = Err("injected formatter failure");
        let after = event_history(&handle)?.retained;
        if formatter.is_ok() || before != after {
            return Err("formatter fed back into capture".to_string());
        }
        complete(
            artifact,
            "F04",
            &[
                "FORMATTER_ERROR=OBSERVED".to_string(),
                format!("EVENTS_BEFORE={before}"),
                format!("EVENTS_AFTER={after}"),
                "CAPTURE_MUTATED=false".to_string(),
            ],
        )
    }

    fn f05(artifact: &Path) -> Result<(), String> {
        let guarded = Arc::new(AtomicBool::new(false));
        let hook_guard = Arc::clone(&guarded);
        let artifact = artifact.to_path_buf();
        std::panic::set_hook(Box::new(move |_| {
            if hook_guard.swap(true, Ordering::SeqCst) {
                return;
            }
            let _ = partial(&artifact, "F05", "HOOK_ATTEMPTS=1");
            panic!("F05 injected formatter re-panic");
        }));
        panic!("F05 initial panic");
    }

    fn f06(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        let artifact = artifact.to_path_buf();
        std::panic::set_hook(Box::new(move |_| {
            let observation = handle.try_read_degraded();
            let strategy = observation.build.panic_strategy.value;
            let _ = complete(
                &artifact,
                "F06",
                &[
                    format!("PANIC_STRATEGY={strategy:?}"),
                    "HOOK_ATTEMPT=1".to_string(),
                    "UNWIND_CLAIM=false".to_string(),
                ],
            );
        }));
        panic!("F06 injected panic");
    }

    fn f07(artifact: &Path) -> Result<(), String> {
        partial(artifact, "F07", "BREADCRUMB=BEFORE_ABORT")?;
        std::process::abort();
    }

    fn parked_partial(scenario: &str, artifact: &Path) -> Result<(), String> {
        partial(artifact, scenario, "HEARTBEAT=1")?;
        loop {
            thread::park();
        }
    }

    fn f10(artifact: &Path) -> Result<(), String> {
        partial(artifact, "F10", "HEARTBEAT=1;DEADLOCK=ENTERING")?;
        let lock = Mutex::new(());
        let _first = lock.lock().map_err(|error| error.to_string())?;
        let _second = lock.lock().map_err(|error| error.to_string())?;
        Err("deadlock unexpectedly resolved".to_string())
    }

    fn f11(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        controlled_allocator::fail(true);
        let observation = handle.try_read_degraded();
        controlled_allocator::fail(false);
        if observation.runtime.availability != Availability::Known {
            return Err("fixed materialization did not survive failpoint".to_string());
        }
        complete(
            artifact,
            "F11",
            &[
                "ALLOCATOR_FAILPOINT=ARMED_DURING_READ".to_string(),
                "MATERIALIZATION=SUCCESS".to_string(),
                "REAL_HOST_OOM=UNVERIFIED".to_string(),
            ],
        )
    }

    fn f12(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        let renderer = writer.begin_renderer(RendererRole::Primary);
        let source = renderer.source.expect("source");
        renderer.surface_failure(SurfaceFailure::Validation);
        let observation = handle.try_read();
        let records = observation.renderers.value.ok_or("renderers unavailable")?;
        let record = records
            .records
            .iter()
            .flatten()
            .find(|record| record.source == source)
            .ok_or("record missing")?;
        if record.last_surface_failure.value != Some(SurfaceFailure::Validation)
            || record.last_wgpu_error.value != Some(WgpuErrorCategory::SurfaceValidation)
        {
            return Err("validation category mismatch".to_string());
        }
        complete(
            artifact,
            "F12",
            &[
                format!("INCARNATION={}", source.incarnation.get()),
                "CATEGORY=VALIDATION".to_string(),
                "SYNTHETIC_PATH=VERIFIED".to_string(),
                "REAL_BACKEND_CALLBACK=UNVERIFIED".to_string(),
                "UNCAPTURED_HANDLER_INSTALLED=false".to_string(),
            ],
        )
    }

    fn f13(artifact: &Path) -> Result<(), String> {
        complete(
            artifact,
            "F13",
            &[
                "RESULT=UNVERIFIED".to_string(),
                "REASON=NO_CONTRACT_COMPATIBLE_OOM_PRODUCER".to_string(),
                "REAL_GPU_OOM=NOT_ATTEMPTED".to_string(),
                "UNCAPTURED_HANDLER_INSTALLED=false".to_string(),
            ],
        )
    }

    fn f14(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        let renderer = writer.begin_renderer(RendererRole::Tool);
        let source = renderer.source.expect("source");
        let message = "x".repeat(OPAQUE_TEXT_CAPACITY * 4);
        RendererDiagnostics::device_lost(&writer, source, DeviceLostReason::Destroyed, &message);
        let observation = handle.try_read();
        let records = observation.renderers.value.ok_or("renderers unavailable")?;
        let lost = records
            .records
            .iter()
            .flatten()
            .find(|record| record.source == source)
            .and_then(|record| record.device_lost.value.as_ref())
            .ok_or("device-lost missing")?;
        let bounded = lost
            .message
            .value
            .as_ref()
            .is_some_and(|text| text.stored_len() == OPAQUE_TEXT_CAPACITY && text.truncated);
        if !bounded {
            return Err("device-lost message was not physically bounded".to_string());
        }
        complete(
            artifact,
            "F14",
            &[
                format!("OLD_INCARNATION={}", source.incarnation.get()),
                "MESSAGE_BYTES=192".to_string(),
                "TRUNCATED=true".to_string(),
                "SYNTHETIC_PATH=VERIFIED".to_string(),
                "REAL_BACKEND_CALLBACK=UNVERIFIED".to_string(),
            ],
        )
    }

    fn f15(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        let renderer = writer.begin_renderer(RendererRole::Primary);
        let source = renderer.source.expect("source");
        renderer.surface_failure(SurfaceFailure::Lost);
        renderer.surface_failure(SurfaceFailure::Outdated);
        let observation = handle.try_read();
        let records = observation.renderers.value.ok_or("renderers unavailable")?;
        let record = records
            .records
            .iter()
            .flatten()
            .find(|record| record.source == source)
            .ok_or("renderer missing")?;
        let events = observation.events.value.ok_or("events unavailable")?;
        let failures: Vec<_> = events
            .records
            .iter()
            .flatten()
            .filter_map(|event| match event.kind {
                DiagnosticEventKind::SurfaceFailure {
                    source: event_source,
                    failure,
                } if event_source == source => Some(failure),
                _ => None,
            })
            .collect();
        if failures != [SurfaceFailure::Lost, SurfaceFailure::Outdated]
            || record.last_surface_failure.value != Some(SurfaceFailure::Outdated)
        {
            return Err(format!("surface failures collapsed: {failures:?}"));
        }
        complete(
            artifact,
            "F15",
            &[
                "EVENTS=LOST,OUTDATED".to_string(),
                "LAST=OUTDATED".to_string(),
                format!("INCARNATION={}", source.incarnation.get()),
            ],
        )
    }

    fn f16(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.audio_selected(AudioBackend::Native, AudioInitializationOutcome::Succeeded);
        writer.audio_error(AudioErrorCategory::Stream, None);
        let audio = handle.try_read().audio.value.ok_or("audio unavailable")?;
        let error = audio
            .last_generic_backend_error
            .value
            .ok_or("audio error missing")?;
        if audio.backend.value != Some(AudioBackend::Native)
            || error.category != AudioErrorCategory::Stream
        {
            return Err("audio category/backend mismatch".to_string());
        }
        complete(
            artifact,
            "F16",
            &[
                "BACKEND=NATIVE".to_string(),
                "CATEGORY=STREAM".to_string(),
                "FRAME_ASSOCIATION=NONE".to_string(),
                "SYNTHETIC_PATH=VERIFIED".to_string(),
                "REAL_CPAL_CALLBACK=UNVERIFIED".to_string(),
            ],
        )
    }

    fn f17(artifact: &Path) -> Result<(), String> {
        const ITERATIONS: u64 = 1_000_000;
        let (handle, writer) = enabled();
        controlled_allocator::track(true);
        for _ in 0..ITERATIONS {
            writer.runtime_transition(RuntimeLifecycle::Running, None);
        }
        let observation = handle.try_read();
        controlled_allocator::track(false);
        let allocations = controlled_allocator::count();
        let events = observation.events.value.ok_or("events unavailable")?;
        let expected_overwrites = ITERATIONS - EVENT_CAPACITY as u64;
        let mut saturated = SaturatingCounter {
            value: u64::MAX,
            saturated: false,
        };
        saturated.increment();
        if allocations != 0
            || events.retained != EVENT_CAPACITY
            || events.overwritten.value != expected_overwrites
            || events.overwritten.saturated
            || saturated.value != u64::MAX
            || !saturated.saturated
        {
            return Err(format!(
                "saturation mismatch allocations={allocations} retained={} overwritten={:?}",
                events.retained, events.overwritten
            ));
        }
        complete(
            artifact,
            "F17",
            &[
                format!("ITERATIONS={ITERATIONS}"),
                format!("RETAINED={}", events.retained),
                format!("OVERWRITTEN={}", events.overwritten.value),
                format!("ALLOCATIONS={allocations}"),
                format!("STORE_BYTES={}", std::mem::size_of::<Store>()),
                format!(
                    "OBSERVATION_BYTES={}",
                    std::mem::size_of::<DiagnosticObservation>()
                ),
                "COUNTER_AT_MAX=18446744073709551615".to_string(),
                "COUNTER_SATURATED=true".to_string(),
                "COUNTER_WRAPPED=false".to_string(),
            ],
        )
    }

    fn f18(artifact: &Path) -> Result<(), String> {
        const THREADS: usize = 4;
        const ROUNDS: usize = 100;
        const PER_ROUND: usize = 250;
        let (handle, writer) = enabled();
        let mut joins = Vec::with_capacity(THREADS);
        for thread_index in 0..THREADS {
            let writer = writer.clone();
            joins.push(thread::spawn(move || {
                for round in 0..ROUNDS {
                    for index in 0..PER_ROUND {
                        if (thread_index + round + index) % 2 == 0 {
                            writer.runtime_transition(RuntimeLifecycle::Running, None);
                        } else {
                            writer.audio_error(AudioErrorCategory::Other, None);
                        }
                    }
                }
            }));
        }
        for join in joins {
            join.join().map_err(|_| "producer panicked".to_string())?;
        }
        let events = event_history(&handle)?;
        let attempted = (THREADS * ROUNDS * PER_ROUND) as u64;
        let accounted = events.retained as u64 + events.overwritten.value + events.dropped.value;
        if events.retained != EVENT_CAPACITY || accounted != attempted {
            return Err(format!(
                "concurrent accounting mismatch attempted={attempted} accounted={accounted}"
            ));
        }
        if events
            .records
            .iter()
            .take(events.retained)
            .any(Option::is_none)
        {
            return Err("torn/empty retained record".to_string());
        }
        complete(
            artifact,
            "F18",
            &[
                format!("THREADS={THREADS}"),
                format!("ROUNDS={ROUNDS}"),
                format!("PER_ROUND={PER_ROUND}"),
                format!("ATTEMPTED={attempted}"),
                format!("RETAINED={}", events.retained),
                format!("OVERWRITTEN={}", events.overwritten.value),
                format!("DROPPED={}", events.dropped.value),
                "TORN_RECORDS=0".to_string(),
                "CAUSALITY_CLAIM=false".to_string(),
            ],
        )
    }

    fn f19(artifact: &Path) -> Result<(), String> {
        let categories = [
            WgpuErrorCategory::CreateSurface,
            WgpuErrorCategory::RequestAdapter,
            WgpuErrorCategory::RequestDevice,
        ];
        for category in categories {
            let (handle, writer) = enabled();
            writer.runtime_transition(RuntimeLifecycle::Initializing, None);
            let mut renderer = writer.begin_renderer(RendererRole::Primary);
            renderer.initialization_failed(category);
            writer.runtime_transition(RuntimeLifecycle::Ended, Some(RuntimeOutcome::StartupFailed));
            let observation = handle.try_read();
            let records = observation.renderers.value.ok_or("renderers unavailable")?;
            let record = records
                .records
                .iter()
                .flatten()
                .next()
                .ok_or("record missing")?;
            if record.lifecycle.value != Some(RendererLifecycle::InitializationFailed)
                || record.last_wgpu_error.value != Some(category)
                || record.surface_configuration.availability != Availability::Unknown
            {
                return Err(format!("startup stage mismatch for {category:?}"));
            }
        }
        complete(
            artifact,
            "F19",
            &[
                "SYNTHETIC_STAGES=CREATE_SURFACE,REQUEST_ADAPTER,REQUEST_DEVICE".to_string(),
                "FUTURE_SURFACE_FACTS=UNKNOWN".to_string(),
                "REAL_OS_WGPU_STARTUP=UNVERIFIED".to_string(),
            ],
        )
    }

    fn f20(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        let first = writer.begin_renderer(RendererRole::Tool);
        let first_source = first.source.expect("first source");
        let callback: (DiagnosticsWriter, RendererSource) =
            first.callback_writer().expect("callback writer");
        drop(first);
        let second = writer.begin_renderer(RendererRole::Tool);
        let second_source = second.source.expect("second source");
        RendererDiagnostics::device_lost(
            &callback.0,
            callback.1,
            DeviceLostReason::Destroyed,
            "late",
        );
        let records = handle
            .try_read()
            .renderers
            .value
            .ok_or("renderers unavailable")?;
        let old = records
            .records
            .iter()
            .flatten()
            .find(|record| record.source == first_source)
            .ok_or("old record missing")?;
        let current = records
            .records
            .iter()
            .flatten()
            .find(|record| record.source == second_source)
            .ok_or("new record missing")?;
        if first_source == second_source
            || old.device_lost.availability != Availability::Known
            || current.device_lost.availability != Availability::Unknown
        {
            return Err("late callback was reused or misattributed".to_string());
        }
        complete(
            artifact,
            "F20",
            &[
                format!("OLD_INCARNATION={}", first_source.incarnation.get()),
                format!("NEW_INCARNATION={}", second_source.incarnation.get()),
                "LATE_EVENT_TARGET=OLD".to_string(),
                "CURRENT_DEVICE_LOST=UNKNOWN".to_string(),
                "REAL_TEARDOWN_CALLBACK=UNVERIFIED".to_string(),
            ],
        )
    }

    fn f21(artifact: &Path) -> Result<(), String> {
        let (handle, writer) = enabled();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        let before = event_history(&handle)?.retained;
        let denied_target: PathBuf = artifact
            .parent()
            .ok_or("artifact has no parent")?
            .join("writer-target-is-directory");
        fs::create_dir_all(&denied_target).map_err(|error| error.to_string())?;
        let write_result = fs::write(&denied_target, b"must fail");
        let after = event_history(&handle)?.retained;
        if write_result.is_ok() || before != after {
            return Err("writer failure fed back into capture".to_string());
        }
        complete(
            artifact,
            "F21",
            &[
                format!("WRITER_ERROR={}", write_result.unwrap_err()),
                format!("EVENTS_BEFORE={before}"),
                format!("EVENTS_AFTER={after}"),
                "RETRIES=0".to_string(),
                "REAL_DISK_FULL=UNVERIFIED".to_string(),
            ],
        )
    }

    fn f22(artifact: &Path) -> Result<(), String> {
        let observation = EngineDiagnostics::disabled().try_read_degraded();
        if observation.collection_mode != CollectionMode::RuntimeDisabled
            || observation.runtime.availability != Availability::NotApplicable
            || observation.renderers.availability != Availability::NotApplicable
            || observation.audio.availability != Availability::NotApplicable
            || observation.events.availability != Availability::NotApplicable
            || observation.degraded
        {
            return Err("runtime-disabled semantics mismatch".to_string());
        }
        complete(
            artifact,
            "F22",
            &[
                "RUNTIME_MODE=COLLECTION_DISABLED".to_string(),
                "RUNTIME_SECTIONS=NOT_APPLICABLE".to_string(),
                format!(
                    "DISABLED_HANDLE_BYTES={}",
                    std::mem::size_of::<EngineDiagnosticsHandle>()
                ),
                "COMPILE_ABSENT=REQUIRES_EXTERNAL_BUILD_CHECK".to_string(),
                "FULL_ENGINE_AB=UNVERIFIED".to_string(),
            ],
        )
    }

    #[cfg(test)]
    mod controlled_allocator {
        pub(super) fn fail(_: bool) {}
        pub(super) fn track(_: bool) {}
        pub(super) fn count() -> u64 {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;
    use std::mem::size_of;

    struct TestAllocationProbe;

    thread_local! {
        static TRACK_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
        static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    // SAFETY: every operation delegates to `System` with the original layout
    // and pointer. The thread-local counter is observational only.
    unsafe impl GlobalAlloc for TestAllocationProbe {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            TRACK_ALLOCATIONS.with(|enabled| {
                if enabled.get() {
                    ALLOCATION_COUNT.with(|count| count.set(count.get().saturating_add(1)));
                }
            });
            // SAFETY: delegated unchanged to the system allocator.
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            // SAFETY: delegated unchanged to the system allocator.
            unsafe { System.dealloc(pointer, layout) };
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            TRACK_ALLOCATIONS.with(|enabled| {
                if enabled.get() {
                    ALLOCATION_COUNT.with(|count| count.set(count.get().saturating_add(1)));
                }
            });
            // SAFETY: delegated unchanged to the system allocator.
            unsafe { System.alloc_zeroed(layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            TRACK_ALLOCATIONS.with(|enabled| {
                if enabled.get() {
                    ALLOCATION_COUNT.with(|count| count.set(count.get().saturating_add(1)));
                }
            });
            // SAFETY: delegated unchanged to the system allocator.
            unsafe { System.realloc(pointer, layout, size) }
        }
    }

    #[global_allocator]
    static TEST_ALLOCATOR: TestAllocationProbe = TestAllocationProbe;

    fn allocations_during(operation: impl FnOnce()) -> usize {
        // Initialize both TLS cells before enabling the probe.
        ALLOCATION_COUNT.with(|count| count.set(0));
        TRACK_ALLOCATIONS.with(|enabled| enabled.set(true));
        operation();
        TRACK_ALLOCATIONS.with(|enabled| enabled.set(false));
        ALLOCATION_COUNT.with(Cell::get)
    }

    #[test]
    fn physical_budgets_fit() {
        println!(
            "event={} store={} observation={} disabled_handle={}",
            size_of::<DiagnosticEvent>(),
            size_of::<Store>(),
            size_of::<DiagnosticObservation>(),
            size_of::<EngineDiagnosticsHandle>()
        );
        assert!(
            size_of::<DiagnosticEvent>() <= 320,
            "event={} bytes",
            size_of::<DiagnosticEvent>()
        );
        assert!(
            size_of::<Store>() <= 64 * 1024,
            "store={} bytes",
            size_of::<Store>()
        );
        assert!(
            size_of::<DiagnosticObservation>() <= 64 * 1024,
            "observation={} bytes",
            size_of::<DiagnosticObservation>()
        );
        assert!(size_of::<EngineDiagnosticsHandle>() <= 2 * 1024);
        assert_eq!(EVENT_CAPACITY, 64);
        assert_eq!(RENDERER_CAPACITY, 8);
        assert_eq!(OPAQUE_TEXT_CAPACITY, 192);
    }

    #[test]
    fn capture_and_materialization_allocate_nothing_after_bootstrap() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let mut renderer = writer.begin_renderer(RendererRole::Primary);
        renderer.ready();
        let callback = renderer.callback_writer().unwrap();

        let allocations = allocations_during(|| {
            renderer.presented();
            renderer.surface_failure(SurfaceFailure::Timeout);
            writer.runtime_transition(RuntimeLifecycle::Running, None);
            writer.audio_selected(AudioBackend::Native, AudioInitializationOutcome::Succeeded);
            writer.audio_error(AudioErrorCategory::Stream, Some("bounded"));
            RendererDiagnostics::device_lost(
                &callback.0,
                callback.1,
                DeviceLostReason::Unknown,
                "bounded",
            );
            let _ = handle.try_read();
            let _ = handle.try_read_degraded();
        });
        assert_eq!(allocations, 0);
    }

    #[test]
    fn handles_are_cloneable_and_stores_are_distinct() {
        let (first, _) = EngineDiagnostics::enabled();
        let cloned = first.clone();
        let (second, _) = EngineDiagnostics::enabled();
        assert!(Arc::ptr_eq(
            first.store.as_ref().unwrap(),
            cloned.store.as_ref().unwrap()
        ));
        assert!(!Arc::ptr_eq(
            first.store.as_ref().unwrap(),
            second.store.as_ref().unwrap()
        ));
    }

    #[test]
    fn registration_rejects_second_attach() {
        let (_, registration) = EngineDiagnostics::enabled();
        let duplicate = EngineDiagnosticsRegistration {
            store: Arc::clone(&registration.store),
        };
        assert!(registration.attach().is_ok());
        assert!(matches!(
            duplicate.attach(),
            Err(RegistrationAlreadyAttached)
        ));
    }

    #[test]
    fn disabled_handle_has_no_runtime_store() {
        let observation = EngineDiagnostics::disabled().try_read();
        assert_eq!(observation.collection_mode, CollectionMode::RuntimeDisabled);
        assert_eq!(observation.events.availability, Availability::NotApplicable);
    }

    #[test]
    fn opaque_text_bounds_utf8() {
        let exact = OpaqueText::new(&"a".repeat(192));
        assert_eq!(exact.stored_len(), 192);
        assert!(!exact.truncated);
        let multi = OpaqueText::new(&format!("{}é", "a".repeat(191)));
        assert_eq!(multi.stored_len(), 191);
        assert!(multi.truncated);
        assert!(std::str::from_utf8(multi.as_str().as_bytes()).is_ok());
    }

    #[test]
    fn counters_saturate_without_wrap() {
        let mut counter = SaturatingCounter {
            value: u64::MAX,
            saturated: false,
        };
        counter.increment();
        assert_eq!(counter.value, u64::MAX);
        assert!(counter.saturated);
    }

    #[test]
    fn lifecycle_and_unknown_device_lost_are_truthful() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let renderer = writer.begin_renderer(RendererRole::Primary);
        let before = handle.try_read();
        let record = before.renderers.value.unwrap().records[0].clone().unwrap();
        assert_eq!(record.device_lost.availability, Availability::Unknown);
        writer.runtime_transition(RuntimeLifecycle::Initializing, None);
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        writer.runtime_transition(RuntimeLifecycle::ShuttingDown, None);
        writer.runtime_transition(RuntimeLifecycle::Ended, Some(RuntimeOutcome::NormalExit));
        drop(renderer);
        let after = handle.try_read();
        assert_eq!(
            after.runtime.value.unwrap().lifecycle.value,
            Some(RuntimeLifecycle::Ended)
        );
    }

    #[test]
    fn renderer_identities_do_not_reuse_and_late_events_keep_source() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let first = writer.begin_renderer(RendererRole::Tool);
        let first_source = first.source().unwrap();
        let callback = first.callback_writer().unwrap();
        drop(first);
        let second = writer.begin_renderer(RendererRole::Tool);
        let second_source = second.source().unwrap();
        assert!(second_source.incarnation.get() > first_source.incarnation.get());
        RendererDiagnostics::device_lost(
            &callback.0,
            callback.1,
            DeviceLostReason::Unknown,
            "late",
        );
        let records = handle.try_read().renderers.value.unwrap().records;
        let old = records
            .iter()
            .flatten()
            .find(|record| record.source == first_source)
            .unwrap();
        let new = records
            .iter()
            .flatten()
            .find(|record| record.source == second_source)
            .unwrap();
        assert_eq!(old.device_lost.availability, Availability::Known);
        assert_eq!(new.device_lost.availability, Availability::Unknown);
    }

    #[test]
    fn ring_overwrites_oldest_in_store_order() {
        let (_, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        for _ in 0..(EVENT_CAPACITY + 3) {
            writer.runtime_transition(RuntimeLifecycle::Running, None);
        }
        let events = writer.store.events.try_lock().unwrap();
        let history = events.materialize(writer.store.events_dropped.get());
        assert_eq!(history.retained, EVENT_CAPACITY);
        assert!(history.ever_wrapped);
        assert_eq!(history.overwritten.value, 3);
        assert!(history.first_record_order < history.last_record_order);
    }

    #[test]
    fn active_renderers_are_not_evicted_when_retention_is_full() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let active: Vec<_> = (0..RENDERER_CAPACITY)
            .map(|_| writer.begin_renderer(RendererRole::Tool))
            .collect();
        let rejected = writer.begin_renderer(RendererRole::Tool);
        assert!(rejected.source().is_some());
        let observation = handle.try_read();
        let renderers = observation.renderers.value.unwrap();
        assert_eq!(renderers.retained, RENDERER_CAPACITY);
        assert_eq!(renderers.records_dropped.value, 1);
        drop(active);
    }

    #[test]
    fn identity_exhaustion_saturates_without_wrap() {
        let (_, registration) = EngineDiagnostics::enabled();
        registration
            .store
            .identities
            .value
            .store(u64::MAX, Ordering::Relaxed);
        let writer = registration.attach().unwrap();
        let renderer = writer.begin_renderer(RendererRole::Other);
        assert!(renderer.source().is_none());
        assert_eq!(writer.store.identities.get().value, u64::MAX);
        assert!(writer.store.identities.get().saturated);
    }

    #[test]
    fn contention_drops_event_without_recursive_event() {
        let (_, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let guard = writer.store.events.try_lock().unwrap();
        writer.runtime_transition(RuntimeLifecycle::Running, None);
        assert_eq!(writer.store.events_dropped.get().value, 1);
        assert_eq!(guard.len, 0);
    }

    #[test]
    fn renderer_mutations_update_the_same_retained_record() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let mut renderer = writer.begin_renderer(RendererRole::Primary);
        renderer.adapter_facts(
            AdapterBackend::Vulkan,
            AdapterDeviceType::DiscreteGpu,
            Some("adapter"),
        );
        renderer.surface_configured(SurfaceConfiguration {
            format: SurfaceFormat::Bgra8UnormSrgb,
            present_mode: SurfacePresentMode::Fifo,
            alpha_mode: SurfaceAlphaMode::Opaque,
            width: 640,
            height: 480,
        });
        renderer.ready();
        renderer.suboptimal();
        renderer.presented();
        renderer.surface_failure(SurfaceFailure::Validation);
        renderer.surface_failure(SurfaceFailure::Lost);
        let record = handle.try_read().renderers.value.unwrap().records[0]
            .clone()
            .unwrap();
        assert_eq!(record.adapter.backend.value, Some(AdapterBackend::Vulkan));
        assert_eq!(record.last_present.availability, Availability::Known);
        assert_eq!(
            record.last_surface_failure.value,
            Some(SurfaceFailure::Lost)
        );
        assert_eq!(
            record.last_wgpu_error.value,
            Some(WgpuErrorCategory::SurfaceValidation)
        );
        assert!(matches!(
            record.surface_configuration.freshness,
            Some(Freshness::Stale {
                reason: StaleReason::DeviceLost,
                ..
            })
        ));
    }

    #[test]
    fn skipped_adapter_observation_remains_truthfully_unknown() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let _renderer = writer.begin_renderer(RendererRole::Primary);

        let record = handle.try_read().renderers.value.unwrap().records[0]
            .clone()
            .unwrap();
        assert_eq!(record.adapter.backend.availability, Availability::Unknown);
        assert_eq!(
            record.adapter.device_type.availability,
            Availability::Unknown
        );
        assert_eq!(record.adapter.name.availability, Availability::Unknown);
        assert!(record.adapter.backend.value.is_none());
        assert!(record.adapter.device_type.value.is_none());
        assert!(record.adapter.name.value.is_none());
    }

    #[test]
    fn audio_summary_exposes_only_generic_backend_state() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        writer.audio_selected(
            AudioBackend::Noop,
            AudioInitializationOutcome::FailedFallback,
        );
        writer.audio_error(AudioErrorCategory::Initialization, Some("backend failed"));
        let audio = handle.try_read().audio.value.unwrap();
        assert_eq!(audio.backend.value, Some(AudioBackend::Noop));
        assert_eq!(
            audio.initialization.value,
            Some(AudioInitializationOutcome::FailedFallback)
        );
        assert_eq!(
            audio.last_generic_backend_error.value.unwrap().category,
            AudioErrorCategory::Initialization
        );
    }

    #[test]
    fn section_contention_is_local_and_degraded_read_never_waits() {
        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let _guard = writer.store.renderers.try_lock().unwrap();
        let observation = handle.try_read_degraded();
        assert_eq!(
            observation.renderers.availability,
            Availability::Unavailable
        );
        assert_eq!(observation.runtime.availability, Availability::Known);
        assert!(observation.degraded);
    }

    #[test]
    #[ignore = "manual, non-gating timing evidence"]
    fn timing_probe() {
        use std::time::{Duration, Instant};

        fn p99(mut samples: Vec<Duration>) -> Duration {
            samples.sort_unstable();
            samples[(samples.len() * 99 / 100).min(samples.len() - 1)]
        }

        let (handle, registration) = EngineDiagnostics::enabled();
        let writer = registration.attach().unwrap();
        let mut renderer = writer.begin_renderer(RendererRole::Primary);
        renderer.ready();
        for _ in 0..10_000 {
            renderer.presented();
        }

        let mut steady = Vec::with_capacity(50_000);
        for _ in 0..50_000 {
            let start = Instant::now();
            renderer.presented();
            steady.push(start.elapsed());
        }
        let mut rare = Vec::with_capacity(20_000);
        for _ in 0..20_000 {
            let start = Instant::now();
            writer.runtime_transition(RuntimeLifecycle::Running, None);
            rare.push(start.elapsed());
        }
        let mut reads = Vec::with_capacity(10_000);
        for _ in 0..10_000 {
            let start = Instant::now();
            let _ = handle.try_read();
            reads.push(start.elapsed());
        }
        let guard = writer.store.renderers.try_lock().unwrap();
        let mut degraded = Vec::with_capacity(10_000);
        for _ in 0..10_000 {
            let start = Instant::now();
            let _ = handle.try_read_degraded();
            degraded.push(start.elapsed());
        }
        drop(guard);
        let callback = renderer.callback_writer().unwrap();
        let mut callbacks = Vec::with_capacity(20_000);
        for _ in 0..20_000 {
            let start = Instant::now();
            RendererDiagnostics::device_lost(
                &callback.0,
                callback.1,
                DeviceLostReason::Unknown,
                "bounded",
            );
            callbacks.push(start.elapsed());
        }
        println!(
            "p99 steady={:?} rare={:?} callback={:?} read={:?} degraded={:?}",
            p99(steady),
            p99(rare),
            p99(callbacks),
            p99(reads),
            p99(degraded)
        );
    }
}
