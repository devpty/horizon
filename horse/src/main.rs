use std::io::{self, Seek};
// use std::{fs};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TestStructV1 {
	v_keep: String,
	v_old: String,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TestStruct {
	v_keep: String,
	v_new: String,
}
impl From<TestStructV1> for TestStruct {
	fn from(v: TestStructV1) -> Self {
		Self { v_keep: v.v_keep, v_new: "New value!".to_string()}
	}
}
horizon_horse::impl_versioned!{
	(1 0 0, TestStructV1)
	(1 1 0, TestStructV1 => TestStruct)
}

fn main() {
	let st = TestStructV1 {
		v_keep: "Keep This".to_string(),
		v_old: "This shouldn't".to_string(),
	};
	println!("raw: {:?}\n", st);
	let data = horizon_horse::to_bytes(
		&st, horizon_horse::FormatStyle::Compact).unwrap();
	println!("ser: {:x?}\n", data);
	println!("len: {}\n", data.len());
	let mut st_tree = horizon_horse::from_bytes(&data).unwrap();
	println!("tree: {:?}\n", st_tree);
	let st_de: TestStruct = st_tree.deserialize_ver(horizon_horse::Version(1, 0, 0)).unwrap();
	println!("parse: {:?}\n", st_de);
}
