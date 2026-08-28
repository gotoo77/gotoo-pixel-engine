use std::fmt;
use std::sync::Arc;

#[cfg(all(feature = "diagnostics", not(target_arch = "wasm32")))]
use crate::diagnostics::{AdapterBackend, AdapterDeviceType};
#[cfg(feature = "diagnostics")]
use crate::diagnostics::{
    DeviceLostReason, DiagnosticsWriter, RendererDiagnostics, RendererRole, SurfaceAlphaMode,
    SurfaceConfiguration, SurfaceFailure, SurfaceFormat, SurfacePresentMode, WgpuErrorCategory,
};
use crate::{Framebuffer, Size, Viewport};
use winit::dpi::PhysicalSize;
use winit::window::Window;

const SHADER: &str = r#"
override encode_surface_srgb: f32 = 0.0;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    var output: VertexOutput;
    let position = positions[vertex_index];
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.tex_coords = position * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return output;
}

@group(0) @binding(0) var framebuffer_texture: texture_2d<f32>;
@group(0) @binding(1) var framebuffer_sampler: sampler;

fn linear_to_srgb_channel(channel: f32) -> f32 {
    let clamped = clamp(channel, 0.0, 1.0);
    if clamped <= 0.0031308 {
        return clamped * 12.92;
    }

    return 1.055 * pow(clamped, 1.0 / 2.4) - 0.055;
}

fn linear_to_srgb(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        linear_to_srgb_channel(color.r),
        linear_to_srgb_channel(color.g),
        linear_to_srgb_channel(color.b),
        color.a,
    );
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(framebuffer_texture, framebuffer_sampler, input.tex_coords);

    // Some surfaces, including browser WebGPU surfaces on common platforms,
    // expose only non-sRGB formats. Encode explicitly there so Pixel's sRGB
    // bytes keep the same perceptual meaning across presentation targets.
    if encode_surface_srgb > 0.5 {
        return linear_to_srgb(color);
    }

    return color;
}
"#;

#[derive(Debug)]
pub enum RendererInitError {
    CreateSurface(wgpu::CreateSurfaceError),
    RequestAdapter(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
}

impl fmt::Display for RendererInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateSurface(err) => write!(f, "failed to create wgpu surface: {err}"),
            Self::RequestAdapter(err) => write!(f, "failed to request wgpu adapter: {err}"),
            Self::RequestDevice(err) => write!(f, "failed to request wgpu device: {err}"),
        }
    }
}

impl std::error::Error for RendererInitError {}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    framebuffer_size: Size,
    viewport: Viewport,
    framebuffer_texture: wgpu::Texture,
    framebuffer_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "diagnostics")]
    diagnostics: Option<RendererDiagnostics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderOutcome {
    Presented,
    SurfaceChanged,
    Skipped,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        framebuffer_width: u32,
        framebuffer_height: u32,
    ) -> Result<Self, RendererInitError> {
        Self::new_inner(
            window,
            framebuffer_width,
            framebuffer_height,
            #[cfg(feature = "diagnostics")]
            None,
        )
        .await
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) async fn new_with_diagnostics(
        window: Arc<Window>,
        framebuffer_width: u32,
        framebuffer_height: u32,
        writer: DiagnosticsWriter,
        role: RendererRole,
    ) -> Result<Self, RendererInitError> {
        Self::new_inner(
            window,
            framebuffer_width,
            framebuffer_height,
            Some(writer.begin_renderer(role)),
        )
        .await
    }

    async fn new_inner(
        window: Arc<Window>,
        framebuffer_width: u32,
        framebuffer_height: u32,
        #[cfg(feature = "diagnostics")] mut diagnostics: Option<RendererDiagnostics>,
    ) -> Result<Self, RendererInitError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = match instance.create_surface(window) {
            Ok(surface) => surface,
            Err(error) => {
                #[cfg(feature = "diagnostics")]
                if let Some(diagnostics) = diagnostics.as_mut() {
                    diagnostics.initialization_failed(WgpuErrorCategory::CreateSurface);
                }
                return Err(RendererInitError::CreateSurface(error));
            }
        };

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
        {
            Ok(adapter) => adapter,
            Err(error) => {
                #[cfg(feature = "diagnostics")]
                if let Some(diagnostics) = diagnostics.as_mut() {
                    diagnostics.initialization_failed(WgpuErrorCategory::RequestAdapter);
                }
                return Err(RendererInitError::RequestAdapter(error));
            }
        };

        // Web's infallible Adapter::get_info dispatches to a JS property read. A
        // null WebGPU adapter can cross the binding as an opaque Rust Adapter,
        // so truthful unknown facts are safer than a diagnostics-only exception.
        #[cfg(all(feature = "diagnostics", not(target_arch = "wasm32")))]
        if let Some(diagnostics) = diagnostics.as_ref() {
            let info = adapter.get_info();
            diagnostics.adapter_facts(
                adapter_backend(info.backend),
                adapter_device_type(info.device_type),
                (!info.name.is_empty()).then_some(info.name.as_str()),
            );
        }

        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                label: Some("m0-device"),
                trace: wgpu::Trace::Off,
            })
            .await
        {
            Ok(device_and_queue) => device_and_queue,
            Err(error) => {
                #[cfg(feature = "diagnostics")]
                if let Some(diagnostics) = diagnostics.as_mut() {
                    diagnostics.initialization_failed(WgpuErrorCategory::RequestDevice);
                }
                return Err(RendererInitError::RequestDevice(error));
            }
        };

        #[cfg(feature = "diagnostics")]
        if let Some((writer, source)) = diagnostics
            .as_ref()
            .and_then(RendererDiagnostics::callback_writer)
        {
            device.set_device_lost_callback(move |reason, message| {
                RendererDiagnostics::device_lost(
                    &writer,
                    source,
                    device_lost_reason(reason),
                    &message,
                );
            });
        }

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let encode_surface_srgb = surface_needs_shader_srgb_encode(surface_format);

        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Fifo)
        {
            wgpu::PresentMode::Fifo
        } else {
            surface_caps.present_modes[0]
        };

        let alpha_mode = surface_caps.alpha_modes[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        #[cfg(feature = "diagnostics")]
        if let Some(diagnostics) = diagnostics.as_ref() {
            diagnostics.surface_configured(surface_configuration(&config));
        }

        let framebuffer_size = Size {
            width: framebuffer_width,
            height: framebuffer_height,
        };
        let viewport = Viewport::new(
            Size {
                width: config.width,
                height: config.height,
            },
            framebuffer_size,
        );

        let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cpu-framebuffer-texture"),
            size: wgpu_extent(framebuffer_size),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let framebuffer_view =
            framebuffer_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let framebuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cpu-framebuffer-nearest-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("framebuffer-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let framebuffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("framebuffer-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&framebuffer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&framebuffer_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("framebuffer-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("framebuffer-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("framebuffer-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &[(
                        "encode_surface_srgb",
                        if encode_surface_srgb { 1.0 } else { 0.0 },
                    )],
                    ..Default::default()
                },
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        #[cfg(feature = "diagnostics")]
        if let Some(diagnostics) = diagnostics.as_ref() {
            diagnostics.ready();
        }

        Ok(Self {
            surface,
            device,
            queue,
            config,
            framebuffer_size,
            viewport,
            framebuffer_texture,
            framebuffer_bind_group,
            render_pipeline,
            #[cfg(feature = "diagnostics")]
            diagnostics,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.viewport = Viewport::new(
            Size {
                width: size.width,
                height: size.height,
            },
            self.framebuffer_size,
        );
        self.surface.configure(&self.device, &self.config);
        #[cfg(feature = "diagnostics")]
        if let Some(diagnostics) = self.diagnostics.as_ref() {
            diagnostics.surface_configured(surface_configuration(&self.config));
        }
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn render(&mut self, framebuffer: &Framebuffer) -> RenderOutcome {
        debug_assert_eq!(framebuffer.width(), self.framebuffer_size.width);
        debug_assert_eq!(framebuffer.height(), self.framebuffer_size.height);

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.framebuffer_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            framebuffer.as_rgba8(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(framebuffer.width() * 4),
                rows_per_image: Some(framebuffer.height()),
            },
            wgpu_extent(self.framebuffer_size),
        );

        let surface_texture = self.surface.get_current_texture();
        #[cfg(feature = "diagnostics")]
        if let Some(failure) = surface_failure_from_acquisition(&surface_texture)
            && let Some(diagnostics) = self.diagnostics.as_ref()
        {
            diagnostics.surface_failure(failure);
        }
        let frame = match surface_texture {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                #[cfg(feature = "diagnostics")]
                if let Some(diagnostics) = self.diagnostics.as_ref() {
                    diagnostics.suboptimal();
                }
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                return RenderOutcome::SurfaceChanged;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return RenderOutcome::SurfaceChanged;
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                return RenderOutcome::Skipped;
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return RenderOutcome::Skipped;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return RenderOutcome::Skipped;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("framebuffer-render-encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("framebuffer-render-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.framebuffer_bind_group, &[]);
            render_pass.set_viewport(
                self.viewport.rect.x as f32,
                self.viewport.rect.y as f32,
                self.viewport.rect.width as f32,
                self.viewport.rect.height as f32,
                0.0,
                1.0,
            );
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
        #[cfg(feature = "diagnostics")]
        if let Some(diagnostics) = self.diagnostics.as_mut() {
            diagnostics.presented();
        }

        RenderOutcome::Presented
    }
}

#[cfg(all(feature = "diagnostics", not(target_arch = "wasm32")))]
fn adapter_backend(backend: wgpu::Backend) -> AdapterBackend {
    match backend {
        wgpu::Backend::Noop => AdapterBackend::Noop,
        wgpu::Backend::Vulkan => AdapterBackend::Vulkan,
        wgpu::Backend::Metal => AdapterBackend::Metal,
        wgpu::Backend::Dx12 => AdapterBackend::Dx12,
        wgpu::Backend::Gl => AdapterBackend::Gl,
        wgpu::Backend::BrowserWebGpu => AdapterBackend::BrowserWebGpu,
    }
}

#[cfg(all(feature = "diagnostics", not(target_arch = "wasm32")))]
fn adapter_device_type(device_type: wgpu::DeviceType) -> AdapterDeviceType {
    match device_type {
        wgpu::DeviceType::IntegratedGpu => AdapterDeviceType::IntegratedGpu,
        wgpu::DeviceType::DiscreteGpu => AdapterDeviceType::DiscreteGpu,
        wgpu::DeviceType::VirtualGpu => AdapterDeviceType::VirtualGpu,
        wgpu::DeviceType::Cpu => AdapterDeviceType::Cpu,
        wgpu::DeviceType::Other => AdapterDeviceType::Other,
    }
}

#[cfg(feature = "diagnostics")]
fn surface_configuration(config: &wgpu::SurfaceConfiguration) -> SurfaceConfiguration {
    SurfaceConfiguration {
        format: match config.format {
            wgpu::TextureFormat::Bgra8Unorm => SurfaceFormat::Bgra8Unorm,
            wgpu::TextureFormat::Bgra8UnormSrgb => SurfaceFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm => SurfaceFormat::Rgba8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb => SurfaceFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba16Float => SurfaceFormat::Rgba16Float,
            _ => SurfaceFormat::Other,
        },
        present_mode: match config.present_mode {
            wgpu::PresentMode::Fifo => SurfacePresentMode::Fifo,
            wgpu::PresentMode::FifoRelaxed => SurfacePresentMode::FifoRelaxed,
            wgpu::PresentMode::Immediate => SurfacePresentMode::Immediate,
            wgpu::PresentMode::Mailbox => SurfacePresentMode::Mailbox,
            wgpu::PresentMode::AutoVsync => SurfacePresentMode::AutoVsync,
            wgpu::PresentMode::AutoNoVsync => SurfacePresentMode::AutoNoVsync,
        },
        alpha_mode: match config.alpha_mode {
            wgpu::CompositeAlphaMode::Auto => SurfaceAlphaMode::Auto,
            wgpu::CompositeAlphaMode::Opaque => SurfaceAlphaMode::Opaque,
            wgpu::CompositeAlphaMode::PreMultiplied => SurfaceAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::PostMultiplied => SurfaceAlphaMode::PostMultiplied,
            wgpu::CompositeAlphaMode::Inherit => SurfaceAlphaMode::Inherit,
        },
        width: config.width,
        height: config.height,
    }
}

#[cfg(feature = "diagnostics")]
fn device_lost_reason(reason: wgpu::DeviceLostReason) -> DeviceLostReason {
    match reason {
        wgpu::DeviceLostReason::Unknown => DeviceLostReason::Unknown,
        wgpu::DeviceLostReason::Destroyed => DeviceLostReason::Destroyed,
    }
}

#[cfg(feature = "diagnostics")]
fn surface_failure_from_acquisition(
    outcome: &wgpu::CurrentSurfaceTexture,
) -> Option<SurfaceFailure> {
    match outcome {
        wgpu::CurrentSurfaceTexture::Timeout => Some(SurfaceFailure::Timeout),
        wgpu::CurrentSurfaceTexture::Occluded => Some(SurfaceFailure::Occluded),
        wgpu::CurrentSurfaceTexture::Outdated => Some(SurfaceFailure::Outdated),
        wgpu::CurrentSurfaceTexture::Lost => Some(SurfaceFailure::Lost),
        wgpu::CurrentSurfaceTexture::Validation => Some(SurfaceFailure::Validation),
        wgpu::CurrentSurfaceTexture::Success(_) | wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
            None
        }
    }
}

fn wgpu_extent(size: Size) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    }
}

fn surface_needs_shader_srgb_encode(surface_format: wgpu::TextureFormat) -> bool {
    !surface_format.is_srgb()
}

#[cfg(test)]
mod tests {
    use super::surface_needs_shader_srgb_encode;
    #[cfg(feature = "diagnostics")]
    use super::{
        SurfaceAlphaMode, SurfaceFailure, SurfaceFormat, SurfacePresentMode, surface_configuration,
        surface_failure_from_acquisition,
    };

    #[test]
    fn srgb_surface_formats_use_hardware_encoding() {
        assert!(!surface_needs_shader_srgb_encode(
            wgpu::TextureFormat::Bgra8UnormSrgb
        ));
        assert!(!surface_needs_shader_srgb_encode(
            wgpu::TextureFormat::Rgba8UnormSrgb
        ));
    }

    #[test]
    fn non_srgb_surface_formats_need_shader_encoding() {
        assert!(surface_needs_shader_srgb_encode(
            wgpu::TextureFormat::Bgra8Unorm
        ));
        assert!(surface_needs_shader_srgb_encode(
            wgpu::TextureFormat::Rgba8Unorm
        ));
        assert!(surface_needs_shader_srgb_encode(
            wgpu::TextureFormat::Rgba16Float
        ));
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn surface_configuration_mapping_is_colocated_and_distinct() {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: 320,
            height: 240,
            present_mode: wgpu::PresentMode::Mailbox,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
        };
        let observed = surface_configuration(&config);
        assert_eq!(observed.format, SurfaceFormat::Rgba8UnormSrgb);
        assert_eq!(observed.present_mode, SurfacePresentMode::Mailbox);
        assert_eq!(observed.alpha_mode, SurfaceAlphaMode::PreMultiplied);
        assert_eq!((observed.width, observed.height), (320, 240));
    }

    #[cfg(feature = "diagnostics")]
    #[test]
    fn surface_failure_categories_are_not_reconstructed_from_render_outcome() {
        let cases = [
            (
                wgpu::CurrentSurfaceTexture::Timeout,
                SurfaceFailure::Timeout,
            ),
            (
                wgpu::CurrentSurfaceTexture::Occluded,
                SurfaceFailure::Occluded,
            ),
            (
                wgpu::CurrentSurfaceTexture::Outdated,
                SurfaceFailure::Outdated,
            ),
            (wgpu::CurrentSurfaceTexture::Lost, SurfaceFailure::Lost),
            (
                wgpu::CurrentSurfaceTexture::Validation,
                SurfaceFailure::Validation,
            ),
        ];
        for (outcome, expected) in cases {
            assert_eq!(surface_failure_from_acquisition(&outcome), Some(expected));
        }
    }
}
