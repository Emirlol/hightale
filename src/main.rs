#[tokio::main]
async fn main() -> anyhow::Result<()> {
	server::main().await
}