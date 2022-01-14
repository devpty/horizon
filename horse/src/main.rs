use std::io;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TestStructure(String, u32, f64);

fn main() {
	let st = TestStructure("Test text".to_string(), 420, 6.9);
	let out = io::Cursor::new(vec![]);
	println!("raw: {:?}", st);
	println!("serialized: {:#x?}", horizon_horse::to_write(&st, out).unwrap().into_inner());
}
