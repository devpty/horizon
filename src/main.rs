mod lib;
use lib as horizon;

#[tokio::main]
async fn main() {
	env_logger::Builder::from_default_env()
		.filter_level(log::LevelFilter::Info)
		.filter_module("horizon", log::LevelFilter::max())
		.init();
	horizon::start(horizon::StartInfo {

	}).await;
}
