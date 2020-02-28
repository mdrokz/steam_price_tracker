use postgres_types::ToSql;
use postgres::Row;
use chrono::NaiveDateTime;

pub trait ExtractStructs {
    fn extract(_data:&Self) -> Vec<&(dyn ToSql + Sync)>  {
        vec![&20,&20]
    }

    fn map_pg_values(&mut self,_pg_row:&Vec<Row>) {
        
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GamesData {
    #[serde(rename = "response")]
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "game_count")]
    pub game_count: i64,

    #[serde(rename = "games")]
    pub games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
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
    pub range: i64,
}

#[derive(ExtractStructs,Default,Debug)]
pub struct PriceData {
    pub appid: i32,
    pub price: i32,
    pub date: NaiveDateTime
}