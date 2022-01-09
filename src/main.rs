mod lib;
use lib as horizon;

#[tokio::main]
async fn main() {
	env_logger::Builder::from_default_env().filter_level(log::LevelFilter::Info).init();
	horizon::start(horizon::StartInfo {

	}).await;
}
