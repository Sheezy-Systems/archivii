extern crate dotenv;
use std::env;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize, Serialize};
use serde_json::Result;

fn main() {
    // Constants
    let schoology_baseurl = "https://schoology.tesd.net";
    let auth_cookie_name = "SESS61c75f44be1e14cdb172294ad6a89a4e";

    // Make sure our .env file exists and get its SECRET
    dotenv::dotenv().expect("Could not find the .env file in the local directory");
    let secret = env::var("SECRET").expect("Could not find SECRET in the .env");

    // Get our config file, in the future this should be done via the command file
    let config_file = BufReader::new(File::open("config.json").expect("Could not find config.json"));
    let config: Config = serde_json::from_reader(config_file).expect("config.json was improperly formatted");



    println!("{}", config.id)
}


#[derive(Serialize, Deserialize)]
struct Config {
    realm: String,
    id: String,
    start: u32,
    limit: u32,
}