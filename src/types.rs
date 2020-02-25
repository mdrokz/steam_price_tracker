#[derive(Serialize, Deserialize, Debug)]
pub struct GamesData {
    #[serde(rename = "response")]
    response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "game_count")]
    game_count: i64,

    #[serde(rename = "games")]
    games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    #[serde(rename = "appid")]
    appid: i64,

    #[serde(rename = "playtime_forever")]
    playtime_forever: i64,

    #[serde(rename = "playtime_windows_forever")]
    playtime_windows_forever: i64,

    #[serde(rename = "playtime_mac_forever")]
    playtime_mac_forever: i64,

    #[serde(rename = "playtime_linux_forever")]
    playtime_linux_forever: i64,

    #[serde(rename = "playtime_2weeks")]
    playtime_2_weeks: Option<i64>,
}
