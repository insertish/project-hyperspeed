use mediasoup::producer::Producer;
use mediasoup::router::{Router, RouterOptions};
use ftl_protocol::protocol::FtlHandshakeFinalised;

use super::{codecs::init_codecs, workers::get_worker_pool, producers::init_producers};

pub enum DataSource {
    Ftl(FtlHandshakeFinalised)
}

pub struct HyperspeedRouter {
    router: Router,
    channel_id: String,
    producers: Vec<Producer>,
    // source: DataSource
}

impl HyperspeedRouter {
    pub async fn new(channel_id: String, source: DataSource) -> HyperspeedRouter {
        let mut options = RouterOptions::default();
        init_codecs(&mut options, &source);

        let router = get_worker_pool()
            .get_worker()
            .create_router(options)
            .await
            .unwrap();

        let producers = init_producers(&router, &source).await;

        HyperspeedRouter {
            router,
            channel_id,
            producers,
            // source
        }
    }
}
