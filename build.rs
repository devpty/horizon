// use std::env;
use std::fs;
use std::io;
use std::path::Path;

macro_rules! bprintln {
	($($arg:tt)*) => ({
		println!("cargo:warning={}", format!($($arg)*));
	})
}

fn visit_dir(dir: &Path, cb: &dyn Fn(&Path)) -> io::Result<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			visit_dir(entry?.path().as_path(), cb)?;
		}
	} else {
		cb(&dir);
	}
	Ok(())
}

fn main() {
	// let out_dir = Path::new(&env::var_os("OUT_DIR").unwrap());
	visit_dir(Path::new("./"), &|file| {
		let ext = file.extension();
		if ext == "wgsl" {
			bprintln!("{}", file);
		}
	}).expect("Failed to scan files");
	// let dest_path = Path::new(&out_dir).join("hello.rs");
	// fs::write(
	//     &dest_path,
	//     "pub fn message() -> &'static str {
	//         \"Hello, World!\"
	//     }
	//     "
	// ).unwrap();
	println!("cargo:rerun-if-changed=src/shaders/");
}
