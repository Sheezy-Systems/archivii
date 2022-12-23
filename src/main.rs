use std::error::Error;
use std::env;
use std::fmt::format;
use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use serde_json::Result;
use reqwest;
use reqwest::blocking::Response;
use reqwest::header;
use soup::prelude::*;
use tokio::runtime::Handle;

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


    parse_link(config, &secret)
}

fn make_schoology_request(url: String, secret: &String) -> Response {
    // TODO: schoology has a limit of 15 requests per five seconds, add a rate limit
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", header::HeaderValue::from_static(ACCEPT));
    headers.insert("cookie", header::HeaderValue::from_str(&*format!("{}={}", AUTH_COOKIE_NAME, secret)).unwrap());

    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    client.get(url).send().unwrap()
}

fn parse_link(config:Config, secret: &String) {
    let page:u32 = 0;
    let response: FeedResponse = make_schoology_request(
        format!("{}/{}/{}/feed?page={}", SCHOOLOGY_BASEURL, config.realm, config.id, page),
        secret
    ).json().expect("Failed to parse response as json or bad SECRET");

    let soup = Soup::new(&*response.output);
    for post in soup.attr("class", "s-edge-type-update-post").find_all() {

        let id = post.attr("class", "like-btn").find().unwrap()
            .get("ajax").unwrap().rsplit("/").next().unwrap().to_string();

        let author_name = post
            .attr("class", "update-sentence-inner").find().unwrap()
            .tag("a").find().unwrap().text();

        let mut content: String = post.tag("p").find_all().map(|line| line.text() + "\n").collect();

        match post.attr("class", "show-more-link").find() {
            // If there's a "Show More" prompt
            Some(show_more) => {
                let response: ShowMoreResponse = make_schoology_request(
                    format!("{}/{}", SCHOOLOGY_BASEURL, show_more.get("href").unwrap()),
                    secret
                ).json().expect("Failed to parse response as json or bad SECRET");

                let soup = Soup::new(&*response.update);
                content = soup.tag("p").find_all().map(|line| line.text() + "\n").collect();
            }
            _ => {}
        }
        content.truncate(content.trim_end_matches(&['\r', '\n'][..]).len());

        let like_count: u32 = post.attr("class", "like-details-btn").find().unwrap()
            .text().split(" ").next().unwrap().parse().unwrap();
        let k = make_schoology_request(
            format!("{}/comment/ajax/{}&context=updates", SCHOOLOGY_BASEURL, id),
            secret
        );
        let comment_response: CommentsResponse = k.json().expect("Failed to parse comment response as json or bad SECRET");
        let mut comment_list: Vec<(String, String)> = Vec::new();
        let comment_soup = Soup::new(&*comment_response.comments);
        let comments = comment_soup.attr("class", "comment-comment").find_all();
        comment_list = comments.map(|comment| {
            let author = comment.attr("class", "comment-author").find().unwrap().tag("a").find().unwrap().text();
            let text = comment.attr("class", "comment-body-wrapper").find().unwrap().text();
            (author, text)
        }).collect();


        for (author, text) in comment_list {
            println!("Author: {}", author);
            println!("Text: {}", text);
        }
    }

}

#[derive(Serialize, Deserialize)]
struct FeedResponse {
    output: String
}

#[derive(Serialize, Deserialize)]
struct ShowMoreResponse {
    update: String
}

#[derive(Serialize, Deserialize)]
struct CommentsResponse {
    comments: String,
    count: String // Schoology stores count as a string.
}

#[derive(Serialize, Deserialize)]
struct Config {
    realm: String,
    id: String,
    start: u32,
    limit: u32,
}
