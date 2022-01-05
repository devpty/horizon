mod lib;
use lib as horizon;

#[tokio::main]
async fn main() {
	horizon::start(horizon::StartInfo {

	}).await;
}
