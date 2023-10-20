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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(db).await?;

    let mut conn = PgConnection::connect(&db).await?;

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
    // let mut rows = sqlx::query("SELECT * FROM users")
    //     .fetch(&mut conn);
    // while let Some(row) = rows.try_next().await? {
    //     let email: &str = row.try_get("email")?;
    //
    //     println!("row: {:#?}", email);
    // }

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


// model snapshots {
// id         Int      @id @default(autoincrement())
// time       DateTime @default(now())
// bus        String
// route   String
// location   String
// lat        String
// lon        String
// status     String
// deviation  String
// diffMins Int
// }

// #[tokio::main]
// async fn main_old() -> Result<(), Box<dyn std::error::Error>> {
//     let url = "https://www.metrobusmobile.com/timetrack.asp";
//
//     let options = LaunchOptionsBuilder::default()
//         .sandbox(false)
//         .build()
//         .unwrap();
//     let browser = Browser::new(options)?;
//     let tab = browser.new_tab()?;
//
//     println!("requesting {:#?}", url);
//
//     tab.navigate_to(url)?;
//     tab.wait_for_element("body")?;
//     // println!("fetched");
//
//     // println!("{:#?}", tab.get_document()?);
//     // println!("{:#?}", tab.get_content()?);
//
//     tab.wait_for_element(".list-group-item")?;
//
//     // println!("{:#?}", tab.get_document()?);
//     // println!("{:#?}", tab.get_content()?);
//
//     let content = tab.get_content()?;
//     let doc = tab.get_document()?;
//
//     let dom = Dom::parse(&content)?;
//
//     // println!("{:#?}", dom);
//
//     // let elements = tab.find_elements(".badge")?;
//     let routes = tab.find_elements(".list-group-item")?;
//     // println!("routes {:#?}", routes);
//
//     let content: Vec<String> = routes.iter().map(|e| e.get_content().unwrap()).collect();
//     println!("content {:#?}", content);
//
//     let text: Vec<String> = routes.iter().map(|e| e.get_inner_text().unwrap()).collect();
//     println!("text {:#?}", text);
//
//     // let h5s = tab.find_elements_by_xpath(r#"//*[@class="list-group-item"]/*/div/h5"#)?;
//     // println!("h5s {:#?}", h5s);
//
//     for e in routes {
//         println!("e {:#?}", e);
//         println!("content {:#?}", e.get_content()?);
//         println!("text {:#?}", e.get_inner_text()?);
//
//         let dom = Dom::parse(&e.get_content()?)?;
//         let c = &dom.children.get(0).unwrap().element().unwrap().children;
//         let c0 = &c.get(0).unwrap().element().unwrap().children;
//         let c1 = &c.get(1).unwrap().element().unwrap().children;
//         println!("c {:#?}", c);
//         println!("c0 {:#?}", c0);
//         println!("c1 {:#?}", c1);
//
//         for c in c0 {
//             let elem = c.element().unwrap();
//             println!("elem {:#?}", elem.name);
//             if elem.name == "h5" {
//                 let route = elem.children.get(0).unwrap().text().unwrap();
//                 println!("route {:#?}", route);
//             }
//             if elem.name == "small" {
//                 let location = elem.children.get(0).unwrap().text().unwrap();
//                 println!("location {:#?}", location);
//             }
//         }
//
//         for c in c1 {
//             let elem = c.element().unwrap();
//             println!("elem {:#?}", elem.name);
//             if elem.name == "h5" {
//                 let route = elem.children.get(0).unwrap().text().unwrap();
//                 println!("route {:#?}", route);
//             }
//             if elem.name == "small" {
//                 let location = elem.children.get(0).unwrap().text().unwrap();
//                 println!("location {:#?}", location);
//             }
//         }
//
//         // let route = e.find_elements("h5")?;
//         // let a = e.find_element_by_xpath("/a")?;
//         // let b = e.find_element_by_xpath("/div")?;
//
//         // println!("a: {:#?}", a.get_content()?);
//         // println!("b: {:#?}", b.get_content()?);
//
//         // let route = b.find_element_by_xpath("/h5")?.get_inner_text()?;
//         // let bus = b.find_element_by_xpath("/small")?.get_inner_text()?;
//         // let status = b.find_element_by_xpath("/span")?.get_inner_text()?;
//         //
//         // println!("route: {:#?}", route);
//         // println!("bus: {:#?}", bus);
//         // println!("status: {:#?}", status);
//
//         // let status = e.find_elements(".badge")?.get_inner_/**/text()?;
//         // let status = e.find_element(".mb-1")?.get_inner_text()?;
//         // println!("status: {:#?}", status);
//
//         // let bus = e.find_element("h5")?.get_inner_text()?;
//         // println!("bus: {:#?}", bus);
//     }
//     //
//     // for e in text {
//     //     let res: Vec<&str> = e.split("\n").collect();
//     //
//     //     println!("e: {:#?}", e);
//     //     println!("res: {:#?}", res);
//     //
//     //     if res.len() < 6 {
//     //         println!("bad entry: {:#?}", e);
//     //         continue;
//     //     }
//     //
//     //     let name = &res[0];
//     //     let bus_num = &res[5];
//     //     let status = &res[6];
//     //     println!("name: {:#?}", name);
//     //     println!("bus_num: {:#?}", bus_num);
//     //     println!("status: {:#?}", status);
//     // }
//
//     Ok(())
// }


//
// fn main() {
//     println!("Hello, world!");
//
//     let html = r#"
//             <!doctype html>
//             <html lang="en">
//                 <head>
//                     <meta charset="utf-8">
//                     <title>Html parser</title>
//                 </head>
//                 <body>
//                     <h1 id="a" class="b c">Hello world</h1>
//                     </h1> <!-- comments & dangling elements are ignored -->
//                 </body>
//             </html>"#;
//
//     assert!(Dom::parse(html).is_ok());
//
// }

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