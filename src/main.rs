#[macro_use]
extern crate serde_derive;

use std::env;

use reqwest::header::COOKIE;
use scraper::{Html, Selector};

mod types;

use types::GamesData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut steam_api_url = String::from("");
    let mut steam_store_url = String::from("https://store.steampowered.com/app/");
    let mut env_iter = env::args()
        .take(4)
        .filter(|x| !x.contains(".exe") && !x.contains("no_steam"));
    let no_steam = env::args().nth(1);
    let env_pairs = (env_iter.next(), env_iter.next());
    println!("{:?}", no_steam);
    if no_steam != None {
        scrape_steam_store(None).await;
    } else {
        match env_pairs {
            (Some(steam_key), Some(user_id)) => {
                steam_api_url.push_str(&format!("http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json",steam_key,user_id));
            }
            (None, Some(_)) => {
                panic!("Steam API Key is not specified in the arguments,please provide it.")
            }
            (Some(_), None) => {
                panic!("Steam User Id is not specified in the arguments,please provide it.")
            }
            (None, None) => panic!(
                "Both Steam Api Key And User Id are not specified in the arguments,please provide them."
            ),
        }
        let data = get_games_data(&steam_api_url).await;
        scrape_steam_store(Some(data)).await;
    }
    Ok(())
}

async fn get_games_data(uri: &String) -> GamesData {
    let res = reqwest::get(uri).await.unwrap();
    res.json::<GamesData>().await.unwrap()
}

async fn scrape_steam_store(data: Option<GamesData>) {
    let client = reqwest::Client::new();

    if let Some(game_data) = data {
        for game in game_data.response.games {

        }
    } else {
        println!("NONE");
    }

    let res = client
        .get("https://store.steampowered.com/app/39210")
        .header(COOKIE, "birthtime=1038681001;")
        .send()
        .await
        .unwrap();

    // get headers
    // println!("Headers:{:?}",&res.headers());
    let s = "                             s           ";

    println!("{}",s);
    println!("{}",s.trim_start());

    let body = Html::parse_document(&res.text().await.unwrap());

    let price = Selector::parse(".game_purchase_price").unwrap();

    for data in body.select(&price).nth(0) {
        let story_txt = data.text().collect::<Vec<_>>();
        println!("hehe:{}", story_txt[0].trim_start());
    }
}
