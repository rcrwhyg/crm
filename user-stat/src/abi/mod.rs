use chrono::{DateTime, TimeZone, Utc};
use futures::stream;
use itertools::Itertools as _;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{
    pb::{QueryRequest, RawQueryRequest, User},
    ResponseStream, ServiceResult, UserStatsService,
};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        // generate sql based on query
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");

        sql.push_str(" AND ");
        sql.push_str(&id_conditions);

        println!("Generated SQL: {}", sql);

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        // TODO: query must only return email and name, so we should use sql parser to parse the query
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query: {}",
                req.query
            )));
        };

        Ok(Response::new(Box::pin(stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

fn timestamp_query(name: &str, lower: Option<Timestamp>, upper: Option<Timestamp>) -> String {
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }

    if lower.is_none() {
        let upper = ts_to_utc(upper.unwrap());
        return format!("{} <= '{}'", name, upper.to_rfc3339());
    }

    if upper.is_none() {
        let lower = ts_to_utc(lower.unwrap());
        return format!("{} >= '{}'", name, lower.to_rfc3339());
    }

    format!(
        "{} BETWEEN '{}' AND '{}'",
        name,
        ts_to_utc(lower.unwrap()).to_rfc3339(),
        ts_to_utc(upper.unwrap()).to_rfc3339()
    )
}

fn ts_to_utc(ts: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::Duration;
    use stream::StreamExt as _;

    use crate::{
        pb::{IdQuery, QueryRequestBuilder, TimeQuery},
        AppConfig,
    };

    use super::*;

    #[tokio::test]
    async fn test_raw_query_should_work() -> Result<()> {
        let config = AppConfig::try_load().expect("Failed to load config");
        let svc = UserStatsService::new(config).await;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query: "select * from user_stats where created_at > '2024-05-01' limit 5"
                    .to_string(),
            })
            .await?
            .into_inner();

        while let Some(user) = stream.next().await {
            println!("{:?}", user?);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_query_should_work() -> Result<()> {
        let config = AppConfig::try_load().expect("Failed to load config");
        let svc = UserStatsService::new(config).await;
        let query = QueryRequestBuilder::default()
            // .timestamp(("created_at".to_string(), tq(Some(30), None)))
            .timestamp(("last_visited_at".to_string(), tq(Some(1), None)))
            // .id(("recent_watched".to_string(), id(&[296273, 299163, 271297])))
            .id(("viewed_but_not_started".to_string(), id(&[402193])))
            .build()
            .unwrap();
        let mut stream = svc.query(query).await?.into_inner();

        while let Some(user) = stream.next().await {
            println!("{:?}", user?);
        }

        Ok(())
    }

    fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }

    fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now().checked_sub_signed(Duration::days(days)).unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
}
