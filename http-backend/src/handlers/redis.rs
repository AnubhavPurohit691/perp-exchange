use deadpool_redis::{Config, Pool, Runtime};
#[derive(Clone)]
pub struct Redis {
    pub pool: Pool,
}

impl Redis {
    pub fn new(redis_url: &str) -> Self {
        let cfg = Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .expect("failed to create redis pool");
        Self { pool }
    }
}
