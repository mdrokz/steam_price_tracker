#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate web_api_derive;

use std::env;
use std::fs;
// use std::thread;

use chrono::{DateTime, Local,NaiveDateTime};
use reqwest::header::COOKIE;
use scraper::{Html, Selector};
use tokio_postgres::{Client, NoTls};

mod types;

use types::{ExtractStructs, Game, GameJsonData, GamesData, PriceData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut steam_api_url = String::from("");
    let mut env_iter = env::args()
        .take(4)
        .filter(|x| !x.contains(".exe") && !x.contains("no_steam"));
    let no_steam = env::args().nth(1).filter(|y| y.contains("no_steam"));
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

fn get_current_date() -> NaiveDateTime {
    let local: DateTime<Local> = Local::now();
    // local.format("%Y-%m-%d %H:%M:%S").to_string()
    local.datetime
}

async fn connect(username: &str, password: &str, database: &str) -> Client {
    let credentials = format!(
        "host=localhost port=5432 user={} password={} dbname={}",
        username, password, database
    );
    // match Client::connect(&credentials, NoTls) {
    //     Ok(client) => client,
    //     Err(error) => panic!(error),
    // }
    let (client, connection) = tokio_postgres::connect(&credentials, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
}

async fn get_games_data(uri: &String) -> GamesData {
    let res = reqwest::get(uri).await.unwrap();
    res.json::<GamesData>().await.unwrap()
}

async fn spawn_thread(appid: String, range: Option<i64>) {
    let mut postgres_client = connect(
        "postgres",
        "gear4-snakeman12",
        //&env::var("PG_password").unwrap(),
        "steamPriceData",
    )
    .await;
    let client = reqwest::Client::new();
    let mut steam_store_url = String::from("https://store.steampowered.com/app/");
    steam_store_url.push_str(&appid);
    let res = client
        .get(&steam_store_url)
        .header(COOKIE, "birthtime=1038681001;")
        .send()
        .await
        .unwrap();
    //  println!("memes{}", steam_store_url);
    let body = Html::parse_document(&res.text().await.unwrap());
    let price = Selector::parse(
        "#game_area_purchase>.game_area_purchase_game_wrapper,.discount_final_price",
    )
    .unwrap();
    let s = Selector::parse(".game_purchase_price").unwrap();

    for data in body.select(&price).nth(0) {
        let class = data.value().classes().nth(0).unwrap();

        if class.contains("discount") {
        } else if class.contains("wrapper") {
            let prc = data.select(&s).next().unwrap().text().collect::<Vec<_>>();
            println!("ele:{:?}",prc);
        }

        println!("element:{:?}", data.value().classes().nth(0));
        // println!("ele:{:?}",data.select(&s).next().unwrap().text().collect::<Vec<_>>());
        println!("{:?}", data.text().collect::<Vec<_>>());
        // let story_txt = data.text().collect::<Vec<_>>();
        // if !story_txt[0].trim().replace("₹", "").contains("Free") {
        //     println!("memes2{}",story_txt[0].replace("₹", "").trim());
        //     println!(
        //         "memes{}",
        //         story_txt[0].replace("₹", "").trim().chars().count()
        //     );
        //     let game_price = story_txt[0]
        //         .replace("₹", "")
        //         .trim()
        //         .parse::<i32>()
        //         .unwrap();
        //     println!("price:{}", game_price);
        //     let mut data: PriceData = PriceData {
        //         ..Default::default()
        //     };

        //     data.map_pg_values(
        //         &postgres_client
        //             .query(
        //                 "SELECT appid,price,date FROM public.pricing WHERE appid=$1",
        //                 &[&appid.parse::<i32>().unwrap()],
        //             )
        //             .await
        //             .unwrap(),
        //     );

        // if data.appid == 0 {
        //     postgres_client.query("")
        // } else {

        // }

        //println!("{:?}", data);
        // if let Some(range) = range {
        // } else {

        // }
    }
}
//}

async fn scrape_steam_store(data: Option<GamesData>) {
    println!("memes");
    if let Some(mut game_data) = data {
        let g_data: Vec<GameJsonData> =
            serde_json::from_str(&fs::read_to_string("./json/game.json").unwrap()).unwrap();

        let mut game_data_count = game_data.response.games.len();
        let game_json_data_count = g_data.len();
        for game in g_data {
            game_data.response.games.push(Game {
                appid: game.appid,
                ..Default::default()
            });
        }
        let games = game_data.response.games;
        game_data_count = games.len();
        for (i, game) in games.iter().enumerate() {
            if i >= game_data_count - game_json_data_count {
                println!("{:#?}", games[i]);
            } else {
                spawn_thread(game.appid.to_string(), None).await;
            }
        }
    } else {
        let g_data: Vec<GameJsonData> =
            serde_json::from_str(&fs::read_to_string("./json/game.json").unwrap()).unwrap();
        println!("{:?}", g_data);
        println!("NONE");
    }
    // get headers
    // println!("Headers:{:?}",&res.headers());
}
