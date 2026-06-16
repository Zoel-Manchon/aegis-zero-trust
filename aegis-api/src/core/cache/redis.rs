
// HARDENING NOTE:
// Redis is security infrastructure here, not just cache. For production: enable
// TLS, AUTH/ACLs, key prefix isolation per environment, maxmemory policy review,
// persistence strategy review, command renaming/disablement where appropriate,
// and monitor failures as auth-degrading events.
use redis::{AsyncCommands, aio::ConnectionManager};

#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }

    pub async fn incr_with_ttl(&self, key: &str, ttl_seconds: usize) -> redis::RedisResult<i64> {
        let mut conn = self.manager.clone();

        let count: i64 = conn.incr(key, 1).await?;

        if count == 1 {
            let _: () = conn.expire(key, ttl_seconds as i64).await?;
        }

        Ok(count)
    }

    pub async fn get_i64(&self, key: &str) -> redis::RedisResult<Option<i64>> {
        let mut conn = self.manager.clone();
        conn.get(key).await
    }

    pub async fn get_string(&self, key: &str) -> redis::RedisResult<Option<String>> {
        let mut conn = self.manager.clone();
        conn.get(key).await
    }

    pub async fn set_ex(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: usize,
    ) -> redis::RedisResult<()> {
        let mut conn = self.manager.clone();
        conn.set_ex(key, value, ttl_seconds as u64).await
    }

    pub async fn del(&self, key: &str) -> redis::RedisResult<()> {
        let mut conn = self.manager.clone();
        conn.del(key).await
    }
}
