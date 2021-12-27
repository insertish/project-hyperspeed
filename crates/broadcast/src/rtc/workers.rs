use mediasoup::{worker::Worker, worker::WorkerSettings, worker_manager::WorkerManager};
use once_cell::sync::OnceCell;
use log::debug;

static WORKER_POOL: OnceCell<WorkerPool> = OnceCell::new();

// ! Worker pool taken from Vortex source code.
// ! This is single-threaded, which is enough for now.

#[derive(Debug)]
pub struct WorkerPool {
    _manager: WorkerManager,
    worker: Worker,
}

impl WorkerPool {
    pub async fn init() {
        let worker_pool = WorkerPool::new().await;
        WORKER_POOL.set(worker_pool).unwrap();
    }

    pub fn get() -> &'static WorkerPool {
        WORKER_POOL
            .get()
            .expect("Mediasoup worker pool not initialized")
    }

    pub async fn new() -> Self {
        let manager = WorkerManager::new();
        let mut settings = WorkerSettings::default();
        // ! FIXME: hardcoded value
        settings.rtc_ports_range = 10100..=10200;

        let worker = manager.create_worker(settings).await.unwrap();
        debug!("Initialized worker pool");
        WorkerPool {
            _manager: manager,
            worker,
        }
    }

    pub fn get_worker(&self) -> &Worker {
        &self.worker
    }
}
