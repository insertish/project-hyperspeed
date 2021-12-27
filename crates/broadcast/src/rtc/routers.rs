use mediasoup::router::{Router, RouterOptions};
use mediasoup::producer::{Producer, ProducerId};
use ftl_protocol::protocol::FtlHandshakeFinalised;

use super::{codecs::init_codecs, workers::WorkerPool, producers::init_producers};

#[derive(Clone)]
pub enum DataSource {
    Ftl(FtlHandshakeFinalised)
}

#[derive(Clone)]
pub struct HyperspeedRouter {
    pub router: Router,
    pub channel_id: String,
    pub producers: Vec<Producer>,
    pub source: DataSource
}

impl HyperspeedRouter {
    pub async fn new(channel_id: String, source: DataSource) -> HyperspeedRouter {
        let mut options = RouterOptions::default();
        init_codecs(&mut options, &source);

        let router = WorkerPool::get()
            .get_worker()
            .create_router(options)
            .await
            .unwrap();

        let producers = init_producers(&router, &source).await;

        HyperspeedRouter {
            router,
            channel_id,
            producers,
            source
        }
    }

    pub fn clone_router(&self) -> Router {
        self.router.clone()
    }

    pub fn get_producer_ids(&self) -> Vec<ProducerId> {
        self.producers.iter()
            .map(|v| v.id().clone())
            .collect()
    }
}
