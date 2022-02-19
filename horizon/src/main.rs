#[tokio::main]
async fn main() {
	env_logger::Builder::from_default_env()
		.filter_level(log::LevelFilter::Warn)
		.filter_module("horizon", log::LevelFilter::max())
		.init();
	horizon::start(horizon::StartInfo {
		integer_mode: true,
	}).await;
}
