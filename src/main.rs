use async_trait::async_trait;
use ftl_protocol::protocol::FtlHandshakeFinalised;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    use ftl_protocol::server::IngestServer;

    struct MyServer {}

    #[async_trait]
    impl IngestServer for MyServer {
        async fn get_stream_key(&self, channel_id: &str) -> Result<String, ()> {
            match channel_id {
                "77" => Ok("ieDQxSZ7q58EEeLTvja4QKKGzndwUkVQ".to_string()),
                _ => unimplemented!()
            }
        }

        async fn allocate_ingest(&self, channel_id: &str, handshake: &FtlHandshakeFinalised) -> Result<u16, ()> {
            dbg!(handshake);

            match channel_id {
                "77" => Ok(65534),
                _ => unimplemented!()
            }
        }
    }

    MyServer {}.launch("127.0.0.1:8084".to_string()).await?;

    Ok(())
}
