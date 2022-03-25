//! half-arsed ecs implementation

use std::any::TypeId;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::render::RenderContext;

#[derive(Default)]
pub struct Transform {
	pos: [f32; 2],
	size: [f32; 2],
	rot: f32,
}

pub struct Entity {
	trans: Transform,
	parent: Weak<RefCell<Entity>>,
	children: Vec<Rc<RefCell<Entity>>>,
	components: Vec<Rc<RefCell<dyn Component>>>,
}

pub struct UpdateInfo {
	pub delta_time: f64,
}

impl Entity {
	fn with_raw_parent(parent: Weak<RefCell<Entity>>) -> Self {
		Self {
			trans: Transform::default(),
			parent,
			children: vec![],
			components: vec![],
		}
	}
	pub fn new_rc() -> Rc<RefCell<Self>> {
		Rc::new(RefCell::new(Self::with_raw_parent(Weak::new())))
	}
	pub fn new() -> Self {
		Self::with_raw_parent(Weak::new())
	}
	pub fn with_parent(parent: &Rc<RefCell<Entity>>) -> Rc<RefCell<Self>> {
		let res = Rc::new(RefCell::new(Self::with_raw_parent(Rc::downgrade(parent))));
		{
			let clone = parent.clone();
			let mut borrow = (*clone).borrow_mut();
			borrow.children.push(res.clone());
		}
		res
	}
}

impl Component for Entity {
	fn update(&mut self, info: &UpdateInfo) {
		for component in &self.components {
			let clone = component.clone();
			let mut borrow = (*clone).borrow_mut();
			borrow.update(info);
		}
		for child in &self.children {
			let clone = child.clone();
			let mut borrow = (*clone).borrow_mut();
			borrow.update(info);
		}
	}
	fn render(&self, context: &mut RenderContext) {
		for component in &self.components {
			let clone = component.clone();
			let borrow = (*clone).borrow();
			borrow.render(context);
		}
		for child in &self.children {
			let clone = child.clone();
			let borrow = (*clone).borrow();
			borrow.render(context);
		}
	}
}

pub trait Component {
	fn update(&mut self, info: &UpdateInfo);
	fn render(&self, context: &mut RenderContext);
	fn type_id(&self) -> TypeId
	where
		Self: 'static,
	{
		TypeId::of::<Self>()
	}
}
