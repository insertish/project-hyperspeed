use ftl::launch;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    launch().await.unwrap();
    Ok(())
}
