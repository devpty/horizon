//! useful utilities and things
use std::{iter, slice, mem};

/// ring buffer thing
///
/// todo: use a vecdeque instead
#[derive(Clone, Debug)]
pub struct RingBuffer<T: Clone>(Vec<T>, usize);
impl<T: Clone> RingBuffer<T> {
	pub fn new_fill(size: usize, value: T) -> Self {
		Self(vec![value; size], 0)
	}
	pub fn push(&mut self, item: T) {
		self.0[self.1] = item;
		self.1 += 1;
		if self.1 == self.0.len() { self.1 = 0; }
	}
	pub fn iter(&self) -> iter::Chain<slice::Iter<'_, T>, slice::Iter<'_, T>> {
		let (l, r) = self.0.split_at(self.1);
		r.iter().chain(l.iter())
	}
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

fn next_power_of_2(value: usize) -> usize {
	let mut v = value - 1;
	v |= v >> 1;
	v |= v >> 2;
	v |= v >> 2;
	#[cfg(target_pointer_width =   "8")] { v |= v >>  4; }
	#[cfg(target_pointer_width =  "16")] { v |= v >>  8; }
	#[cfg(target_pointer_width =  "32")] { v |= v >> 16; }
	#[cfg(target_pointer_width =  "64")] { v |= v >> 32; }
	#[cfg(target_pointer_width = "128")] { v |= v >> 64; }
	return v + 1;
}

/// a resizable wgpu buffer
pub struct ResizeBuffer<T: bytemuck::Pod> {
	/// cpu-side buffer
	pub data: Vec<T>,
	/// gpu-side buffer
	buffer: wgpu::Buffer,
	/// capacity of buffer
	capacity: usize,
}
impl<T: bytemuck::Pod> ResizeBuffer<T> {
	fn new(data: Vec<T>, buffer: wgpu::Buffer) -> Self {
		Self { buffer, capacity: next_power_of_2(data.len()), data }
	}
	fn write_data(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
		// 1. resize the queue if it too small
		if self.capacity < self.data.len() {
			self.buffer.destroy();
			self.capacity = next_power_of_2(self.data.len());
			self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
				label: Some("ResizeBuffer"),
				size: self.capacity as u64,
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				mapped_at_creation: false,
			});
		}
		queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
		let write_bytes = self.data.len() * mem::size_of::<T>();
		let rem_bytes = self.capacity * mem::size_of::<T>() - write_bytes;
		queue.write_buffer(&self.buffer, write_bytes as u64, &vec![0; rem_bytes]);
	}
}
