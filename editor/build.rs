// use std::path::Path;

fn main() {
	// rkpk::build::auto_make(Path::new("src/assets/graph/"));
	let mut builder = asset::build::Builder::new();
	// builder.bundle_path("assets", "src/assets/").unwrap();
	builder
		.bundle_data("generated.txt", br#"data generated at build time"#)
		.unwrap();
	let mut packer = rkpk::build::Packer::new();
	packer.add_dir("src/assets/graph/").unwrap();
	packer
		.save_build_info("assets/graph", "graph", &mut builder)
		.unwrap();
	builder.build("bundle.w64").unwrap();
}
