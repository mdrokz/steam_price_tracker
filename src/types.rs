extern crate chrono;

use chrono::naive::NaiveDateTime;
use postgres_types::ToSql;
use std::default::Default;
use tokio_postgres::Row;

pub trait ExtractStructs {
    fn extract(_data: &Self) -> Vec<&(dyn ToSql + Sync)> {
        vec![&20, &20]
    }

    fn map_pg_values(&mut self, _pg_row: &Vec<Row>) {}
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct GamesData {
    #[serde(rename = "response")]
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Response {
    #[serde(rename = "game_count")]
    pub game_count: i64,

    #[serde(rename = "games")]
    pub games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct Game {
    #[serde(rename = "appid")]
    pub appid: i64,

    #[serde(rename = "playtime_forever")]
    pub playtime_forever: i64,

    #[serde(rename = "playtime_windows_forever")]
    pub playtime_windows_forever: i64,

    #[serde(rename = "playtime_mac_forever")]
    pub playtime_mac_forever: i64,

    #[serde(rename = "playtime_linux_forever")]
    pub playtime_linux_forever: i64,

    #[serde(rename = "playtime_2weeks")]
    pub playtime_2_weeks: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct GameJsonData {
    #[serde(rename = "appid")]
    pub appid: i64,
    #[serde(rename = "range")]
    pub range: i32,
}

#[derive(ExtractStructs, Debug)]
pub struct PriceData {
    pub appid: i32,
    pub price: i32,
    pub date: NaiveDateTime,
}
