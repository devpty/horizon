//! rendering and cropping stuff

use std::mem;

use crate::state::Vertex;

pub type Vert2 = [f32; 2];

#[derive(Debug)]
pub struct RenderContext<'a> {
	stack: Vec<Vec<Vert2>>,
	vertex_buffer: &'a mut ResizeBuffer<Vertex>,
	index_buffer: &'a mut ResizeBuffer<[u16; 3]>,
}

impl<'a> RenderContext<'a> {
	pub fn new(vertex_buffer: &'a mut ResizeBuffer<Vertex>, index_buffer: &'a mut ResizeBuffer<[u16; 3]>) -> Self {
		vertex_buffer.reset();
		index_buffer.reset();
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
			1 => res[0].clone(),
			other => {
				log::warn!("polygon split into {} chunks! discarding all but first", other);
				res[0].clone()
			},
		});
	}
	fn inverse_uv(p: [Vert2; 4], z: Vert2) -> Vert2 {
		todo!("clipping isn't supported");
		// doesn't work in some cases
		// TODO: port MapInv from https://desmos.com/calculator/e1qvwtqsti instead

		// MapInv(P, L):
		//
		// δ1.x = (L4.y - L3.y) * (L4.x - L3.x - L1.x + L2.x) - (L4.x - L3.x) * (L4.y - L3.y - L1.y + L2.y)
		// δ1.y = (L4.y - L1.y) * (L4.x - L3.x - L1.x + L2.x) - (L4.x - L1.x) * (L4.y - L3.y - L1.y + L2.y)
		// TODO: do these please :(
		// δ2
		// δ3
		//
		// _s1 = (-δ2 + (δ2 * δ2 - 4 * δ1 * δ3).sqrt()) / 2 * δ1
		// _s2 = (-δ2 - (δ2 * δ2 - 4 * δ1 * δ3).sqrt()) / 2 * δ1
		// _s = if 0 <= _s1 <= 1 {_s1} else if 0 <= _s2 <= 1 {_s2} else if (_s2 - 0.5).abs() < (_s1 - 0.5).abs() {_s2} else {_s1}
		// _pM.x = L3.x * (1 - _s.y) + L2.x * _s.y
		// _pM.y = L1.y * (1 - _s.x) + L2.y * _s.x
		// _pm.x = L4.x * (1 - _s.y) + L1.x * _s.y
		// _pm.y = L3.y * (1 - _s.x) + L4.y * _s.x
		// _p = (P - _pm) / (_pM - _pm)
		// Pms = P - L4
		//
		// Mix = L3 - L4
		// Miy = L1 - L4
		// MiD = 1 / (Mix.x * Miy.y - Miy.x * Mix.y)
		//
		// Mia =  Miy.y * MiD
		// Mib = -Miy.x * MiD
		// Mic = -Mix.y * MiD
		// Mid =  Mix.x * MiD
		// _mi.x = Pms.x * Mia + Pms.y * Mib
		// _mi.y = Pms.x * Mic + Pms.y * Mid
		//
		// if δ1.abs() > 0 {_s} else if δ1.flip().abs() > 0 {_p} else {_mi}

	}
	pub fn pop_clip(&mut self) {
		self.stack.pop().expect("clip stack empty!");
	}
	pub fn polygon(&mut self, poly: &[Vert2], uv: [Vert2; 4], color: [u8; 4]) {
		todo!("clipping isn't supported");
		// let clipped = polygon2::intersection(&poly, self.top_clip());
	}
	pub fn rect(&mut self, position: Vert2, origin: Vert2, size: Vert2, rotation: f32, uv: [Vert2; 4], col: [u8; 4]) {
		let start = [
			position[0] - size[0] * origin[0],
			position[1] - size[1] * origin[1],
		];
		let end = [
			start[0] + size[0],
			start[1] + size[1],
		];
		if self.stack.len() > 0 {
			todo!("clipping isn't supported");
		} else {
			let start_index = self.vertex_buffer.insert_index as u16;
			self.vertex_buffer.add(Vertex {
				uv: uv[0], col, pos: start
			});
			self.vertex_buffer.add(Vertex {
				uv: uv[1], col, pos: [end[0], start[1]]
			});
			self.vertex_buffer.add(Vertex {
				uv: uv[2], col, pos: end,
			});
			self.vertex_buffer.add(Vertex {
				uv: uv[3], col, pos: [start[0], end[1]],
			});
			// 0------1
			// |`. #1 |
			// |  `.  |
			// | #2 `.|
			// 3------2
			self.index_buffer.add([start_index, start_index + 1, start_index + 2]);
			self.index_buffer.add([start_index, start_index + 2, start_index + 3]);
		}
	}
}

/// a resizable wgpu buffer
#[derive(Debug)]
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
			size: (capacity * mem::size_of::<T>()) as u64,
			mapped_at_creation: false,
			usage,
		})
	}
	pub fn new(data: Vec<T>, usage: wgpu::BufferUsages, device: &wgpu::Device) -> Self {
		let cap = data.len().next_power_of_two();
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
			self.capacity = self.data.len().next_power_of_two();
			self.old_len = self.capacity;
			self.buffer = Self::create_shit(device, self.capacity, self.usage);
		}
		queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data[..self.insert_index]));
		// this fills the rest of the capacity with 0's
		// for now we just draw data.len() vertexes / indexes
		// let data_len = self.data.len();
		// if self.old_len > data_len {
		// 	let write_bytes = self.data.len() * mem::size_of::<T>();
		// 	let rem_bytes = self.old_len * mem::size_of::<T>() - write_bytes;
		// 	self.old_len = self.data.len();
		// 	// make this not allocate?
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
		self.insert_index += 1;
	}
	pub fn len(&self) -> usize {
		self.insert_index
	}
}
