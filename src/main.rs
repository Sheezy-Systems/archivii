mod scrape;
mod serializable;

use std::collections::HashMap;
use std::{env, fs};
use std::fs::File;
use std::io::BufReader;

use crate::serializable::*;
use crate::scrape::scrape_realm_by_page;

static SCHOOLOGY_BASEURL: &str = "https://schoology.tesd.net";
static AUTH_COOKIE_NAME: &str = "SESS61c75f44be1e14cdb172294ad6a89a4e";
static ACCEPT: &str = "application/json, text/javascript, */*; q=0.01";
static EMPTY_FEEDPAGE: &str = "<div class=\"item-list\"><ul class=\"s-edge-feed feed-no-realm\"><li id=\"feed-empty-message\" class=\"first last\"><div class=\"small gray\">There are no posts</div></li>\n</ul></div>";

fn main() {
    // Make sure our .env file exists and get its SECRET
    dotenv::dotenv().expect("Could not find the .env file in the local directory");
    let secret = env::var("SECRET").expect("Could not find SECRET in the .env");

    // Get our config file, in the future these should be command line options
    let config_file = BufReader::new(File::open("config.json").expect("Could not find config.json"));
    let config: Config = serde_json::from_reader(config_file).expect("config.json was improperly formatted");

    // Create fresh output directory
    fs::remove_dir_all("output/").unwrap_or(());
    fs::create_dir("output/").expect("Failed to create /output directory");

    let mut posts: Vec<SchoologyPost> = Vec::new();
    let mut authors: HashMap<String, SchoologyUser> = HashMap::new();
    for page in (config.start)..(config.start + config.limit) {
        let (mut new_posts, new_authors) = scrape_realm_by_page(&config.realm, &config.id, page, &secret);
        if new_posts.is_empty() {
            break;
        }
        posts.append(&mut new_posts);
        authors.extend(new_authors);
    }

    let realm = SchoologyRealm { posts };
    let realm_json = serde_json::to_string(&realm).expect("Failed to realm serialize as JSON");
    fs::write(format!("output/{}_{}.json", &config.realm, &config.id), realm_json).expect("Failed to write to file");

    let users = SchoologyUsers { users: authors };
    let users_json = serde_json::to_string(&users).expect("Failed to users serialize as JSON");
    fs::write(format!("output/users.json"), users_json).expect("Failed to write to file");
}

