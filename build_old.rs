use std::env;
use std::fs;
use std::io;
use std::path::Path;

macro_rules! bprintln {
	($($arg:tt)*) => ({
		println!("cargo:warning=\x1b[G\x1b[K{}", format!($($arg)*));
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
	// let path = env::var_os("OUT_DIR").unwrap();
	// let out_dir = Path::new(&path);
	// visit_dir(Path::new("src/shaders"), &|file: &Path| {
	// 	let ext = file.extension();
	// 	if ext.unwrap_or(std::ffi::OsStr::new("")) == "wgsl" {
	// 		let target = out_dir.join(file
	// 			.strip_prefix("src/")
	// 			.expect("File not in source tree?")
	// 			.with_extension("spv"));
	// 		bprintln!("{:?} -> {:?}", file, target);
	// 	}
	// }).expect("Failed to scan files");
	// let dest_path = Path::new(&out_dir).join("hello.rs");
	// fs::write(
	//     &dest_path,
	//     "pub fn message() -> &'static str {
	//         \"Hello, World!\"
	//     }
	//     "
	// ).unwrap();
	// println!("cargo:rerun-if-changed=src/shaders/");
}
