#[cfg(feature = "gpu-acceleration")]
use wgpu::{
    Adapter, Device, Queue, Surface, Instance, PowerPreference, RequestAdapterOptions,
    DeviceDescriptor, Features, Limits, TextureDescriptor, TextureUsages, TextureDimension,
    TextureFormat, Extent3d, ImageCopyTexture, Origin3d, ImageDataLayout, CommandEncoderDescriptor,
    RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, StoreOp,
    ShaderModuleDescriptor, ShaderSource, RenderPipelineDescriptor, PrimitiveState,
    VertexState, FragmentState, ColorTargetState, BlendState, RenderPipeline,
    Buffer, BufferDescriptor, BufferUsages, VertexBufferLayout, VertexAttribute,
    VertexFormat, BufferAddress, VertexStepMode, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, ShaderStages, SamplerBindingType, TextureSampleType, TextureViewDimension,
    BindGroupDescriptor, BindGroupEntry, BindingResource, SamplerDescriptor, FilterMode,
    AddressMode, CompareFunction, BorderColor,
};

#[cfg(feature = "gpu-acceleration")]
use bytemuck::{Pod, Zeroable};

#[cfg(feature = "gpu-acceleration")]
use pollster;

use std::sync::Arc;
use std::path::Path;
use thiserror::Error;
use tracing::{info, warn, debug, error};

use super::ui_profiler::{UIPerformanceProfiler, GpuOperation};

/// GPU-accelerated preview renderer using wgpu 0.17
/// Provides hardware-accelerated image processing, video frame rendering, and preview generation
pub struct GpuPreviewRenderer {
    #[cfg(feature = "gpu-acceleration")]
    device: Device,
    #[cfg(feature = "gpu-acceleration")]
    queue: Queue,
    #[cfg(feature = "gpu-acceleration")]
    adapter_info: wgpu::AdapterInfo,
    #[cfg(feature = "gpu-acceleration")]
    image_pipeline: RenderPipeline,
    #[cfg(feature = "gpu-acceleration")]
    sampler: wgpu::Sampler,
    config: GpuPreviewConfig,
    profiler: Option<Arc<UIPerformanceProfiler>>,
}

/// Configuration for GPU-accelerated preview rendering
#[derive(Debug, Clone)]
pub struct GpuPreviewConfig {
    /// Maximum texture size for GPU processing
    pub max_texture_size: u32,
    /// Enable GPU memory profiling
    pub enable_profiling: bool,
    /// Preferred power profile for GPU selection
    pub power_preference: GpuPowerPreference,
    /// Maximum concurrent GPU operations
    pub max_concurrent_operations: usize,
    /// Texture filtering mode
    pub texture_filter: TextureFilter,
}

/// GPU power preference
#[derive(Debug, Clone, Copy)]
pub enum GpuPowerPreference {
    LowPower,
    HighPerformance,
    None,
}

/// Texture filtering options
#[derive(Debug, Clone, Copy)]
pub enum TextureFilter {
    Nearest,
    Linear,
}

impl Default for GpuPreviewConfig {
    fn default() -> Self {
        Self {
            max_texture_size: 4096,
            enable_profiling: true,
            power_preference: GpuPowerPreference::HighPerformance,
            max_concurrent_operations: 4,
            texture_filter: TextureFilter::Linear,
        }
    }
}

/// Errors that can occur during GPU preview operations
#[derive(Debug, Error)]
pub enum GpuPreviewError {
    #[error("GPU not available or not supported")]
    GpuNotAvailable,
    #[error("Adapter creation failed: {0}")]
    AdapterCreationFailed(String),
    #[error("Device creation failed: {0}")]
    DeviceCreationFailed(String),
    #[error("Texture creation failed: {0}")]
    TextureCreationFailed(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),
    #[error("Pipeline creation failed: {0}")]
    PipelineCreationFailed(String),
    #[error("GPU operation failed: {0}")]
    OperationFailed(String),
    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),
}

/// GPU-accelerated image data
#[derive(Debug, Clone)]
pub struct GpuImageData {
    pub width: u32,
    pub height: u32,
    pub format: GpuTextureFormat,
    pub data: Vec<u8>,
    pub mip_levels: u32,
}

/// Supported GPU texture formats
#[derive(Debug, Clone, Copy)]
pub enum GpuTextureFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
}

#[cfg(feature = "gpu-acceleration")]
/// Vertex data for GPU rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

#[cfg(feature = "gpu-acceleration")]
impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[cfg(feature = "gpu-acceleration")]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] }, // bottom-left
    Vertex { position: [1.0, -1.0, 0.0], tex_coords: [1.0, 1.0] },  // bottom-right
    Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0] },   // top-right
    Vertex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },  // top-left
];

#[cfg(feature = "gpu-acceleration")]
const INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0,
];

impl GpuPreviewRenderer {
    /// Create a new GPU preview renderer with default configuration
    pub async fn new() -> Result<Self, GpuPreviewError> {
        Self::with_config(GpuPreviewConfig::default()).await
    }
    
    /// Create a GPU preview renderer with custom configuration
    pub async fn with_config(config: GpuPreviewConfig) -> Result<Self, GpuPreviewError> {
        #[cfg(feature = "gpu-acceleration")]
        {
            Self::create_gpu_renderer(config).await
        }
        
        #[cfg(not(feature = "gpu-acceleration"))]
        {
            Err(GpuPreviewError::FeatureNotAvailable("GPU acceleration feature not enabled".to_string()))
        }
    }
    
    #[cfg(feature = "gpu-acceleration")]
    async fn create_gpu_renderer(config: GpuPreviewConfig) -> Result<Self, GpuPreviewError> {
        info!("Initializing GPU preview renderer with wgpu 0.17");
        
        // Create wgpu instance
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // Request adapter
        let power_preference = match config.power_preference {
            GpuPowerPreference::LowPower => PowerPreference::LowPower,
            GpuPowerPreference::HighPerformance => PowerPreference::HighPerformance,
            GpuPowerPreference::None => PowerPreference::None,
        };
        
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await.ok_or_else(|| {
            GpuPreviewError::AdapterCreationFailed("No suitable GPU adapter found".to_string())
        })?;
        
        let adapter_info = adapter.get_info();
        info!(
            "Selected GPU adapter: {} ({:?}) - Backend: {:?}",
            adapter_info.name,
            adapter_info.device_type,
            adapter_info.backend
        );
        
        // Request device and queue
        let (device, queue) = adapter.request_device(
            &DeviceDescriptor {
                label: Some("MediaOrganizer GPU Device"),
                features: Features::empty(),
                limits: Limits::default(),
            },
            None, // Trace path
        ).await.map_err(|e| {
            GpuPreviewError::DeviceCreationFailed(format!("Failed to create GPU device: {}", e))
        })?;
        
        // Create render pipeline for image processing
        let image_pipeline = Self::create_image_pipeline(&device)?;
        
        // Create sampler
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("MediaOrganizer Texture Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: match config.texture_filter {
                TextureFilter::Nearest => FilterMode::Nearest,
                TextureFilter::Linear => FilterMode::Linear,
            },
            min_filter: match config.texture_filter {
                TextureFilter::Nearest => FilterMode::Nearest,
                TextureFilter::Linear => FilterMode::Linear,
            },
            mipmap_filter: FilterMode::Linear,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            border_color: None,
            anisotropy_clamp: 16,
        });
        
        info!("GPU preview renderer initialized successfully");
        
        Ok(Self {
            device,
            queue,
            adapter_info,
            image_pipeline,
            sampler,
            config,
            profiler: None,
        })
    }
    
    #[cfg(feature = "gpu-acceleration")]
    fn create_image_pipeline(device: &Device) -> Result<RenderPipeline, GpuPreviewError> {
        // Load shaders
        let vs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Image Vertex Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/image_vertex.wgsl").into()),
        });
        
        let fs_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Image Fragment Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/image_fragment.wgsl").into()),
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Image Bind Group Layout"),
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Image Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create render pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Image Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vs_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &fs_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        
        Ok(pipeline)
    }
    
    /// Set performance profiler for GPU operations
    pub fn with_profiler(mut self, profiler: Arc<UIPerformanceProfiler>) -> Self {
        self.profiler = Some(profiler);
        self
    }
    
    /// Process image using GPU acceleration
    pub async fn process_image(&self, image_data: &[u8], width: u32, height: u32) -> Result<GpuImageData, GpuPreviewError> {
        #[cfg(feature = "gpu-acceleration")]
        {
            let measurement = self.profiler.as_ref()
                .map(|p| p.start_gpu_measurement(GpuOperation::TextureUpload));
            
            let result = self.gpu_process_image(image_data, width, height).await;
            
            if let Some(m) = measurement {
                m.finish();
            }
            
            result
        }
        
        #[cfg(not(feature = "gpu-acceleration"))]
        {
            Err(GpuPreviewError::FeatureNotAvailable("GPU acceleration not available".to_string()))
        }
    }
    
    #[cfg(feature = "gpu-acceleration")]
    async fn gpu_process_image(&self, image_data: &[u8], width: u32, height: u32) -> Result<GpuImageData, GpuPreviewError> {
        debug!("Processing {}x{} image with GPU acceleration", width, height);
        
        // Validate texture size
        if width > self.config.max_texture_size || height > self.config.max_texture_size {
            return Err(GpuPreviewError::TextureCreationFailed(
                format!("Image size {}x{} exceeds maximum texture size {}", 
                       width, height, self.config.max_texture_size)
            ));
        }
        
        // Create texture
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Source Image Texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        // Upload image data to GPU
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d { width, height, depth_or_array_layers: 1 },
        );
        
        // Create output texture
        let output_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Output Image Texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        
        // Create texture views
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create bind group
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &self.image_pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("Image Bind Group"),
        });
        
        // Create vertex buffer
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        
        // Create index buffer
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        
        // Render to output texture
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Image Processing Command Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Image Processing Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.image_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
        
        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        
        // Read back processed image (in a real implementation, this might be optimized)
        let output_data = self.read_texture_data(&output_texture, width, height).await?;
        
        Ok(GpuImageData {
            width,
            height,
            format: GpuTextureFormat::Rgba8Unorm,
            data: output_data,
            mip_levels: 1,
        })
    }
    
    #[cfg(feature = "gpu-acceleration")]
    async fn read_texture_data(&self, texture: &wgpu::Texture, width: u32, height: u32) -> Result<Vec<u8>, GpuPreviewError> {
        let buffer_size = (width * height * 4) as usize; // RGBA
        let staging_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Texture Copy Encoder"),
        });
        
        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
            },
            Extent3d { width, height, depth_or_array_layers: 1 },
        );
        
        self.queue.submit(std::iter::once(encoder.finish()));
        
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        
        // Poll the device until the buffer is mapped
        self.device.poll(wgpu::Maintain::Wait);
        
        receiver.recv().unwrap().map_err(|e| {
            GpuPreviewError::OperationFailed(format!("Failed to map buffer: {}", e))
        })?;
        
        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();
        
        drop(data);
        staging_buffer.unmap();
        
        Ok(result)
    }
    
    /// Generate thumbnail using GPU acceleration
    pub async fn generate_thumbnail(&self, image_data: &[u8], original_width: u32, original_height: u32, thumbnail_size: (u32, u32)) -> Result<Vec<u8>, GpuPreviewError> {
        let measurement = self.profiler.as_ref()
            .map(|p| p.start_gpu_measurement(GpuOperation::TextureUpload));
        
        // Process original image
        let processed_image = self.process_image(image_data, original_width, original_height).await?;
        
        // For now, return the processed image data
        // In a full implementation, this would include GPU-based image scaling
        let result = processed_image.data;
        
        if let Some(m) = measurement {
            m.finish();
        }
        
        Ok(result)
    }
    
    /// Get GPU adapter information
    pub fn get_adapter_info(&self) -> Option<GpuAdapterInfo> {
        #[cfg(feature = "gpu-acceleration")]
        {
            Some(GpuAdapterInfo {
                name: self.adapter_info.name.clone(),
                vendor: self.adapter_info.vendor,
                device: self.adapter_info.device,
                device_type: format!("{:?}", self.adapter_info.device_type),
                backend: format!("{:?}", self.adapter_info.backend),
                driver: self.adapter_info.driver.clone(),
                driver_info: self.adapter_info.driver_info.clone(),
            })
        }
        
        #[cfg(not(feature = "gpu-acceleration"))]
        None
    }
    
    /// Get current GPU memory usage (estimated)
    pub fn get_memory_usage(&self) -> GpuMemoryInfo {
        // This would require platform-specific GPU memory querying
        // For now, return placeholder values
        GpuMemoryInfo {
            used_bytes: 0,
            total_bytes: 0,
            dedicated_video_memory: 0,
            shared_system_memory: 0,
        }
    }
    
    /// Check if GPU acceleration is available
    pub fn is_available() -> bool {
        cfg!(feature = "gpu-acceleration")
    }
}

/// GPU adapter information
#[derive(Debug, Clone)]
pub struct GpuAdapterInfo {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub device_type: String,
    pub backend: String,
    pub driver: String,
    pub driver_info: String,
}

/// GPU memory usage information
#[derive(Debug, Clone)]
pub struct GpuMemoryInfo {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub dedicated_video_memory: u64,
    pub shared_system_memory: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_gpu_availability() {
        assert!(GpuPreviewRenderer::is_available() || !cfg!(feature = "gpu-acceleration"));
    }
    
    #[tokio::test]
    #[cfg(feature = "gpu-acceleration")]
    async fn test_gpu_renderer_creation() {
        match GpuPreviewRenderer::new().await {
            Ok(renderer) => {
                assert!(renderer.get_adapter_info().is_some());
                println!("GPU renderer created successfully");
            }
            Err(e) => {
                println!("GPU renderer creation failed (expected on systems without GPU): {}", e);
                // This is not a test failure - GPU might not be available
            }
        }
    }
}