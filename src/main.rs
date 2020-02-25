extern crate reqwest;
extern crate select;
#[macro_use]
extern crate serde_derive;

use std::env;

mod types;

use types::GamesData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut steam_api_url = String::from("");
    let mut steam_store_url = String::from("https://store.steampowered.com/app/");
    let mut env_iter = env::args().take(3).filter(|x| !x.contains(".exe"));

    let (steam_key, user_id) = (env_iter.next().unwrap(), env_iter.next().unwrap());

    steam_api_url.push_str(&format!("http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json",steam_key,user_id));

    get_games_data(&steam_api_url).await;

    Ok(())
}

async fn get_games_data(uri: &String) -> GamesData {
    let res = reqwest::get(uri).await.unwrap();
    res.json::<GamesData>().await.unwrap()
}
