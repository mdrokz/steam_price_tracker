#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate web_api_derive;

use std::cmp::Ordering;
use std::env;
use std::fs;

use chrono::{DateTime, Local, NaiveDateTime};
use lettre::smtp::authentication::IntoCredentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;
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
    // send_mail();
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

async fn connect(username: &str, password: &str, database: &str) -> Client {
    let credentials = format!(
        "host=localhost port=5432 user={} password={} dbname={}",
        username, password, database
    );
    let (client, connection) = tokio_postgres::connect(&credentials, NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
}

fn send_mail(text: &str) {
    let smtp_address = "smtp.gmail.com";
    let username = env::var("From_Email").unwrap();
    let password = env::var("Email_Secret").unwrap();
    let email = EmailBuilder::new()
        .to(env::var("To_Email").unwrap().as_str())
        .from(username.as_str())
        .subject("Steam Sale")
        .text(text)
        .build()
        .unwrap()
        .into();
    let credentials = (username.as_str(), password.as_str()).into_credentials();
    let mut client = SmtpClient::new_simple(smtp_address)
        .unwrap()
        .credentials(credentials)
        .transport();
    let result = client.send(email);

    println!("result:{:?}", result.err());
}

async fn get_games_data(uri: &String) -> GamesData {
    let res = reqwest::get(uri).await.unwrap();
    res.json::<GamesData>().await.unwrap()
}

fn get_current_date() -> NaiveDateTime {
    let local: DateTime<Local> = Local::now();
    // local.format("%Y-%m-%d %H:%M:%S").to_string()
    local.naive_local()
}

async fn spawn_thread(appid: String, range: Option<i32>) {
    println!("date:{}", get_current_date());
    let postgres_client = connect(
        "postgres",
        &env::var("PG_password").unwrap(),
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
            let prc = data.text().collect::<Vec<_>>();

            let mut price_data: PriceData = PriceData {
                appid: 0,
                price: 0,
                date: get_current_date(),
            };

            price_data.map_pg_values(
                &postgres_client
                    .query(
                        "SELECT * FROM public.pricing WHERE appid=$1 ORDER BY date ASC limit 1",
                        &[&appid.parse::<i32>().unwrap()],
                    )
                    .await
                    .unwrap(),
            );

            if price_data.appid == 0 {
                let p_data: PriceData = PriceData {
                    appid: appid.parse::<i32>().unwrap(),
                    price: prc[0].replace("₹", "").trim().parse::<i32>().unwrap(),
                    date: get_current_date(),
                };
                let data_slice = PriceData::extract(&p_data);
                postgres_client
                    .query(
                        "INSERT INTO public.pricing(
                    appid, price, date)
                    VALUES ($1, $2, $3);",
                        &data_slice[..],
                    )
                    .await
                    .unwrap();
            } else {
                let c_price = prc[0].replace("₹", "").trim().parse::<i32>().unwrap();
                if let Ordering::Equal = price_data.price.cmp(&c_price) {
                } else {
                    let result = c_price - price_data.price;
                    if let Some(range) = range {
                        if result > range {
                            send_mail(&format!("A game is gone on sale its price is {}", c_price));
                        }
                    } else {
                        send_mail(&format!("A game is gone on sale its price is {}", c_price));
                    }
                }
                let p_data: PriceData = PriceData {
                    appid: appid.parse::<i32>().unwrap(),
                    price: prc[0].replace("₹", "").trim().parse::<i32>().unwrap(),
                    date: get_current_date(),
                };
                let data_slice = PriceData::extract(&p_data);
                postgres_client
                    .query(
                        "INSERT INTO public.pricing(
                    appid, price, date)
                    VALUES ($1, $2, $3);",
                        &data_slice[..],
                    )
                    .await
                    .unwrap();
            }
        } else if class.contains("wrapper") {
            let prc = data.select(&s).next().unwrap().text().collect::<Vec<_>>();
            println!("ele:{:?}", prc);

            let mut price_data: PriceData = PriceData {
                appid: 0,
                price: 0,
                date: get_current_date(),
            };

            price_data.map_pg_values(
                &postgres_client
                    .query(
                        "SELECT * FROM public.pricing WHERE appid=$1 ORDER BY date ASC limit 1",
                        &[&appid.parse::<i32>().unwrap()],
                    )
                    .await
                    .unwrap(),
            );

            if price_data.appid == 0 {
                let p_data: PriceData = PriceData {
                    appid: appid.parse::<i32>().unwrap(),
                    price: prc[0].replace("₹", "").trim().parse::<i32>().unwrap(),
                    date: get_current_date(),
                };
                let data_slice = PriceData::extract(&p_data);
                postgres_client
                    .query(
                        "INSERT INTO public.pricing(
                    appid, price, date)
                    VALUES ($1, $2, $3);",
                        &data_slice[..],
                    )
                    .await
                    .unwrap();
            } else {
                let c_price = prc[0].replace("₹", "").trim().parse::<i32>().unwrap();
                if let Ordering::Equal = price_data.price.cmp(&c_price) {
                } else {
                    let result = c_price - price_data.price;
                    if let Some(range) = range {
                        if result > range {
                            send_mail(&format!("A game is gone on sale its price is {}", c_price));
                        }
                    } else {
                        send_mail(&format!("A game is gone on sale its price is {}", c_price));
                    }
                }
                let p_data: PriceData = PriceData {
                    appid: appid.parse::<i32>().unwrap(),
                    price: prc[0].replace("₹", "").trim().parse::<i32>().unwrap(),
                    date: get_current_date(),
                };
                let data_slice = PriceData::extract(&p_data);
                postgres_client
                    .query(
                        "INSERT INTO public.pricing(
                    appid, price, date)
                    VALUES ($1, $2, $3);",
                        &data_slice[..],
                    )
                    .await
                    .unwrap();
            }
        }
    }
}

async fn scrape_steam_store(data: Option<GamesData>) {
    println!("memes");
    if let Some(mut game_data) = data {
        let g_data: Vec<GameJsonData> =
            serde_json::from_str(&fs::read_to_string("./json/game.json").unwrap()).unwrap();

        let mut game_data_count = game_data.response.games.len();
        let game_json_data_count = g_data.len();
        for game in &g_data {
            game_data.response.games.push(Game {
                appid: game.appid,
                ..Default::default()
            });
        }
        let games = game_data.response.games;
        let mut y = 0;
        game_data_count = games.len();
        for (i, game) in games.iter().enumerate() {
            if i >= game_data_count - game_json_data_count {
                spawn_thread(g_data[y].appid.to_string(), Some(g_data[y].range)).await;
                y = y + 1;
            } else {
                spawn_thread(game.appid.to_string(), None).await;
            }
        }
    } else {
        let g_data: Vec<GameJsonData> =
            serde_json::from_str(&fs::read_to_string("./json/game.json").unwrap()).unwrap();

        for game in g_data {
            spawn_thread(game.appid.to_string(), Some(game.range)).await;
        }
    }
}
