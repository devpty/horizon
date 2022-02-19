use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;

macro_rules! bprintln {
	($($arg:tt)*) => ({
		println!("cargo:warning=\x1b[G\x1b[K{}", format!($($arg)*));
	})
}

#[derive(Debug)]
enum InError {
	Io(io::Error),
}

type InResult<T> = Result<T, InError>;

fn visit_dir<T: Fn(&Path) -> ()>(dir: &Path, cb: &T) -> InResult<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir).map_err(InError::Io)? {
			visit_dir(entry.map_err(InError::Io)?.path().as_path(), cb)?;
		}
	} else {
		cb(&dir);
	}
	Ok(())
}

fn main() {
	let out_path = env::var_os("OUT_DIR").unwrap();
	let out_dir = Path::new(&out_path);
	visit_dir(Path::new("src/assets"), &|file: &Path| {
		let ext = file.extension();
		let ext_str = ext.unwrap_or(OsStr::new("")).to_string_lossy();
		match ext_str.as_ref() {
			"png" => {
				let target = out_dir.join(file
					.strip_prefix("src/")
					.expect("File not in source tree?")
					.with_extension("spv"));
				bprintln!("{:?} -> {:?}", file, target);
			},
			_ => {},
		}
	}).expect("Failed to scan files");
	let dest_path = Path::new(&out_dir).join("hello.rs");
	fs::write(
	    &dest_path,
	    "pub fn message() -> &'static str {
	        \"Hello, World!\"
	    }
	    "
	).unwrap();
	println!("cargo:rerun-if-changed=src/shaders/");
}
