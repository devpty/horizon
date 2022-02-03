use std::{path::Path, time::Instant};

fn main() {
	let mut t = Instant::now();
	let mut cache = rkpk::ImageCache::new();
	let mut packer = rkpk::Packer::new(&mut cache);
	timer(&mut t, "create");
	// packer.add_new("hell", None,
	// 	"rkpk/src/assets/hell.jpg", rkpk::Image::File,
	// 	rkpk::ImageLoad::Tiled {
	// 		init: (0, 0, 83, 83).into(),
	// 		gap: (0, 0).into(),
	// 		count: (5312 / 83, 2988 / 83).into(),
	// 	}
	// ).unwrap();
	// packer.add_new("metro1024rb", None,
	// 	"rkpk/src/assets/metro1024rb.png", rkpk::Image::File,
	// 	rkpk::ImageLoad::Tiled {
	// 		init: (0, 0, 16, 16).into(),
	// 		gap: (0, 0).into(),
	// 		count: (1024 / 16, 1024 / 16).into(),
	// 	}
	// ).unwrap();
	timer(&mut t, "append");
	packer.deduplicate().unwrap();
	timer(&mut t, "deduplicate");
	packer.pack().unwrap();
	timer(&mut t, "pack");
	let composite = packer.composite().unwrap();
	timer(&mut t, "composite");
	println!("{:?}", composite);
	for (key, image) in composite {
		image.save_to_disk(&Path::new(&format!("data-{:?}.png", key)), image::ImageFormat::Png).unwrap();
		timer(&mut t, "save");
	}
	timer(&mut t, "done");
}

fn timer(t: &mut Instant, l: &str) {
	let now = Instant::now();
	println!("[{:#?}] {}", now.duration_since(*t), l);
	*t = now;
}
