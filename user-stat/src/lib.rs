use std::{ops::Deref, pin::Pin, sync::Arc};

use anyhow::Result;
use futures::Stream;
use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest, User,
};
use sqlx::PgPool;
use tonic::{async_trait, Request, Response, Status};

pub use config::AppConfig;

mod abi;
mod config;

pub mod pb;

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;

#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

#[allow(unused)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        let query = request.into_inner();
        self.query(query).await
    }

    type RawQueryStream = ResponseStream;

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        let query = request.into_inner();
        self.raw_query(query).await
    }
}

impl UserStatsService {
    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.server.db_url)
            .await
            .expect("Failed to connect to database");
        let inner = UserStatsServiceInner { config, pool };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::pb::{IdQuery, TimeQuery};
    use crate::{AppConfig, UserStatsService, UserStatsServiceInner};
    use anyhow::Result;
    use chrono::{Duration, Utc};
    use prost_types::Timestamp;
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;
    use std::env;
    use std::path::Path;
    use std::sync::Arc;

    impl UserStatsService {
        pub async fn new_for_test() -> Result<(TestPg, Self)> {
            let config = AppConfig::try_load()?;
            let (tdb, pool) = get_test_pool(Some(config.server.db_url.as_str())).await;
            let svc = Self {
                inner: Arc::new(UserStatsServiceInner { config, pool }),
            };

            Ok((tdb, svc))
        }
    }

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://alon:alon123456@localhost:5432/stats".to_string(),
        };

        let p = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("migrations");
        let tdb = TestPg::new(url, p);
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test data
        let sql = include_str!("../fixtures/data.sql").split(';');
        let mut ts = pool.begin().await.expect("Begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("Failed to execute sql");
        }
        ts.commit().await.expect("Commit transaction failed");

        (tdb, pool)
    }

    pub fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }

    pub fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    pub fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now().checked_sub_signed(Duration::days(days)).unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}
