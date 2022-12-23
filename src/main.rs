use std::error::Error;
use std::env;
use std::fmt::format;
use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use reqwest;
use reqwest::header;
use soup::prelude::*;

static SCHOOLOGY_BASEURL: &str = "https://schoology.tesd.net";
static AUTH_COOKIE_NAME: &str= "SESS61c75f44be1e14cdb172294ad6a89a4e";
static ACCEPT: &str = "application/json, text/javascript, */*; q=0.01";

fn main() {
    // Make sure our .env file exists and get its SECRET
    dotenv::dotenv().expect("Could not find the .env file in the local directory");
    let secret = env::var("SECRET").expect("Could not find SECRET in the .env");

    // Get our config file, in the future these should be command line options
    let config_file = BufReader::new(File::open("config.json").expect("Could not find config.json"));
    let config: Config = serde_json::from_reader(config_file).expect("config.json was improperly formatted");


    parse_link(config, secret)
}

fn parse_link(config:Config, secret:String) {
    let page:u32 = 0;
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", header::HeaderValue::from_static(ACCEPT));
    headers.insert("cookie", header::HeaderValue::from_str(&*format!("{}={}", AUTH_COOKIE_NAME, secret)).unwrap());

    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();
    let response: Response = client.get(
        format!("{}/{}/{}/feed?page={}", SCHOOLOGY_BASEURL, config.realm, config.id, page)
    ).send().unwrap().json().expect("Failed to parse response as json or bad SECRET");
    let soup = Soup::new(&*response.output);
    for post in soup.attr("class", "s-edge-type-update-post").find_all() {
        let author_element = post
            .attr("class", "update-sentence-inner").find().unwrap()
            .tag("a").find().unwrap();
        println!("{}", author_element.text())
    }

}

#[derive(Serialize, Deserialize)]
struct Response {
    output: String
}

#[derive(Serialize, Deserialize)]
struct Config {
    realm: String,
    id: String,
    start: u32,
    limit: u32,
}