//! rendering and cropping stuff

use log::warn;

use crate::state::Vertex;

pub type Vert2 = [f32; 2];

pub struct RenderContext<'a> {
	stack: Vec<Vec<Vert2>>,
	vertex_buffer: &'a mut ResizeBuffer<Vertex>,
	index_buffer: &'a mut ResizeBuffer<u16>,
}

impl<'a> RenderContext<'a> {
	pub fn new(vertex_buffer: &'a mut ResizeBuffer<Vertex>, index_buffer: &'a mut ResizeBuffer<u16>) -> Self {
		Self {
			stack: Vec::new(),
			vertex_buffer, index_buffer,
		}
	}
	fn top_clip(&self) -> &[Vert2] {
		&self.stack[self.stack.len() - 1]
	}
	pub fn push_clip(&mut self, clip: &[Vert2]) {
		let res = polygon2::intersection(clip, self.top_clip());
		self.stack.push(match res.len() {
			// i hope it properly clips empty polygons
			0 => vec![],
			1 => res[0],
			other => {
				warn!("polygon split into {} chunks! discarding all but first", other);
				res[0]
			},
		});
	}
	pub fn pop_clip(&mut self) {
		self.stack.pop().expect("clip stack empty!");
	}
	pub fn polygon(&mut self, poly: &[Vert2], uv: [[f32; 2]; 4], color: [u8; 4]) {
		let clipped = polygon2::intersection(&poly, self.top_clip());

	}
	pub fn rect(size: [f32; 4], rotation: f32, uv: [f32; 4], color: [u8; 4]) {

	}
}

fn next_power_of_2(value: usize) -> usize {
	if value == 0 { return 0; }
	let mut v = value - 1;
	v |= v >> 1;
	v |= v >> 2;
	v |= v >> 2;
	v |= v >> 4;
	v |= v >> 8;
	v |= v >> 16;
	#[cfg(not(target_pointer_width =  "32"))] { v |= v >> 32;
	#[cfg(not(target_pointer_width =  "64"))] { v |= v >> 64;
	}	}
	return v + 1;
}

/// a resizable wgpu buffer
pub struct ResizeBuffer<T: bytemuck::Pod> {
	/// cpu-side buffer
	pub data: Vec<T>,
	/// gpu-side buffer
	pub buffer: wgpu::Buffer,
	/// capacity of buffer
	capacity: usize,
	/// length of the buffer the last time it's written
	old_len: usize,
	/// index for inserting new entries
	insert_index: usize,
	usage: wgpu::BufferUsages,
}
impl<T: bytemuck::Pod> ResizeBuffer<T> {
	fn create_shit(device: &wgpu::Device, capacity: usize, usage: wgpu::BufferUsages) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ResizeBuffer"),
			size: capacity as u64,
			mapped_at_creation: false,
			usage,
		})
	}
	pub fn new(data: Vec<T>, usage: wgpu::BufferUsages, device: &wgpu::Device) -> Self {
		let cap = next_power_of_2(data.len());
		let usage = usage | wgpu::BufferUsages::COPY_DST;
		Self {
			buffer: Self::create_shit(device, cap, usage),
			old_len: cap,
			capacity: cap,
			insert_index: data.len(),
			data, usage,
		}
	}
	pub fn write_data(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
		if self.capacity < self.data.len() {
			self.buffer.destroy();
			self.capacity = next_power_of_2(self.data.len());
			self.old_len = self.capacity;
			self.buffer = Self::create_shit(device, self.capacity, self.usage);
		}
		queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
		// this fills the rest of the capacity with 0's
		// for now we just draw data.len() vertexes / indexes
		// let data_len = self.data.len();
		// if self.old_len > data_len {
		// 	let write_bytes = self.data.len() * mem::size_of::<T>();
		// 	let rem_bytes = self.old_len * mem::size_of::<T>() - write_bytes;
		// 	self.old_len = self.data.len();
		// 	// todo: make this not allocate?
		// 	queue.write_buffer(&self.buffer, write_bytes as u64, &vec![0; rem_bytes]);
		// }
	}
	pub fn reset(&mut self) {
		self.insert_index = 0;
	}
	pub fn add(&mut self, v: T) {
		if self.data.len() > self.insert_index {
			self.data[self.insert_index] = v;
		} else {
			self.data.push(v);
		}
	}
}
