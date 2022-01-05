use std::mem;
use std::borrow::Cow;
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};


/// A texture from the atlas
///
/// # properties
///
/// 0: u, v, width, height of the texture in the atlas
///
/// # sizing
///
/// 0: 8 bytes
///
/// padding: none
///
/// total: 8 bytes
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Texture([u16; 4]);

/// A rectangle instance with a position, size, and uv mapping
///
/// # properties
///
/// position: x, y, width, height, starting from the top-left of the screen
///
/// texture: texture to render as
///
/// rotation: binary rotation, 0 to 2³² → 0° to 360°, 360° is represented as an
/// integer overflow back to 0
///
/// # sizing
///
/// position: 8 bytes
///
/// texture: 8 bytes
///
/// rotation: 4 bytes
///
/// padding: none
///
/// total: 20 bytes
///
#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Rect {
	/// x, y, width, height
	position: [i16; 4],
	/// u, v, width, height
	texture: Texture,
	rotation: u32,
}

impl Rect {
	fn new(px: i16, py: i16, pw: i16, ph: i16, texture: Texture, rotation: u32) -> Rect {
		Rect {position: [px, py, pw, ph], texture, rotation}
	}
}

/// game-specific info is stored here
pub struct StartInfo {

}

/// stolen from wgpu/examples/cube
fn create_texels(size: u16) -> Vec<u8> {
	(0..size * size)
		.map(|id| {
			// get high five for recognizing this ;)
			let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
			let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
			let (mut x, mut y, mut count) = (cx, cy, 0);
			while count < 0xFF && x * x + y * y < 4.0 {
				let old_x = x;
				x = x * x - y * y + cx;
				y = 2.0 * old_x * y + cy;
				count += 1;
			}
			count
		})
		.collect()
}

struct RenderInfo {
	rect_buf: wgpu::Buffer,
}

impl RenderInfo {
	fn generate_matrix(width: u16, height: u16) -> cgmath::Matrix4<f32> {
		let projection = cgmath::ortho(0f32, width as f32, height as f32, 0f32, 0f32, 100f32);
		// supposedly there's an OPENGL_TO_WGPU_MATRIX but i don't think we need that?
		projection
	}
	fn init(
		config: &wgpu::SurfaceConfiguration,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
	) -> Self {
		let rect_size = mem::size_of::<Rect>();
		let size = 512u16;
		let texture = Texture([0, 0, size, size]);
		let rect_data = [
			Rect::new(10, 20, 30, 40, texture, 0)
		];
		let rect_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Rect Buffer"),
			contents: bytemuck::cast_slice(&rect_data),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: wgpu::BufferSize::new(64),
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						sample_type: wgpu::TextureSampleType::Uint,
						view_dimension: wgpu::TextureViewDimension::D2,
					},
					count: None,
				},
			],
		});
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let atlas_texels = create_texels(size);
		let atlas_extent = wgpu::Extent3d {
			width: size as u32,
			height: size as u32,
			depth_or_array_layers: 1,
		};
		let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size: atlas_extent,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::R8Uint,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});
		let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
		queue.write_texture(
			atlas_texture.as_image_copy(),
			&atlas_texels,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(std::num::NonZeroU32::new(size as u32).unwrap()),
				rows_per_image: None,
			},
			atlas_extent,
		);
		let mat_total = Self::generate_matrix(config.width as u16, config.height as u16);
		let mat_ref: &[f32; 16] = mat_total.as_ref();
		let uniform_buf =  device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform Buffer"),
			contents: bytemuck::cast_slice(mat_ref),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: uniform_buf.as_entire_binding(),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(&atlas_view),
				},
			],
		});
		let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: None,
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/shader.wgsl"))),
		});
		let rect_buffers = [wgpu::VertexBufferLayout {
			array_stride: rect_size as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
		}]
	}
}

/// the main function of horizon
pub async fn start(info: StartInfo) {

}
