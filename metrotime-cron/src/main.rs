use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use std::collections::HashMap;
use std::fmt;
use html_parser::{Dom, Node};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use futures::TryStreamExt;
use sqlx::{Connection, PgConnection, Row};
use chrono::Utc;
use std::str::FromStr;
use strum_macros::EnumString;
use std::env;


#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    // Extract some useful information from the request

    query_and_save().await.expect("error in main");

    Ok(())
}

pub async fn query_and_save() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.ttrack.info/api/timetrack/json/";

    let body = reqwest::get(url)
        .await?
        .text()
        .await?;
    let routes: Vec<TimeTrack> = serde_json::from_str(&body)?;

    println!("body: {:#?}", body);
    println!("routes: {:#?}", &routes);

    let behind_routes: Vec<&TimeTrack> =
        routes.iter()
            .filter(|r| r.gtfs_stop_sequence_status.is_some() && r.gtfs_stop_sequence_status.clone().unwrap() == Status::Behind)
            .collect();

    let total_mins_behind: i64 =
        -1 * behind_routes.iter()
            .map(|r| r.gtfs_stop_sequence_sched_difference_mins.unwrap()).sum::<i64>();

    println!("total routes: {}", &routes.len());
    println!("total routes behind: {}", &behind_routes.len());
    println!("total mins behind: {}", &total_mins_behind);

    save(routes).await?;

    Ok(())
}

async fn save(routes: Vec<TimeTrack>) -> Result<(), Box<dyn std::error::Error>> {
    let db = env::var("DB_URL")?;
    let mut conn = PgConnection::connect(&db).await?;

    let time: chrono::DateTime<Utc> = Utc::now(); // TODO: use route time?
    for route in routes {
        sqlx::query(
            "INSERT INTO snapshots (time, bus, route, location, lat, lon, status, deviation, diffmins)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(time)
            .bind(route.current_route)
            .bind(route.routenumber)
            .bind(route.current_location)
            .bind(route.bus_lat)
            .bind(route.bus_lon)
            .bind::<String>(route.gtfs_stop_sequence_status.unwrap_or_default().into())
            .bind(route.gtfs_stop_sequence_deviation.unwrap_or_default())
            .bind(route.gtfs_stop_sequence_sched_difference_mins.unwrap_or_default())
            .execute(&mut conn).await?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TimeTrack {
    routerun: String,
    current_route: String,
    time_stamp: String,
    current_location: String,
    routenumber: i64,
    bus_lat: String,
    bus_lon: String,
    gtfs_stop_sequence_status: Option<Status>,
    gtfs_stop_sequence_deviation: Option<String>,
    gtfs_stop_sequence_sched_difference_mins: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
enum Status {
    #[serde(rename = "BEHIND")]
    Behind,

    #[serde(rename = "AHEAD")]
    Ahead,

    #[serde(rename = "ON TIME")]
    OnTime,

    #[serde(rename = "NO DATA")]
    #[default]
    NoData
}
//
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Status> for String {
    fn from(value: Status) -> Self {
        format!("{}", value)
    }
}