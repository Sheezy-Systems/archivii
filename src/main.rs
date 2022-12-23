use std::error::Error;
use std::{env, fmt, thread, time::Duration};
use std::collections::HashMap;
use std::fmt::{format};
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
static AUTH_COOKIE_NAME: &str = "SESS61c75f44be1e14cdb172294ad6a89a4e";
static ACCEPT: &str = "application/json, text/javascript, */*; q=0.01";

fn main() {
    // Make sure our .env file exists and get its SECRET
    dotenv::dotenv().expect("Could not find the .env file in the local directory");
    let secret = env::var("SECRET").expect("Could not find SECRET in the .env");

    // Get our config file, in the future these should be command line options
    let config_file = BufReader::new(File::open("config.json").expect("Could not find config.json"));
    let config: Config = serde_json::from_reader(config_file).expect("config.json was improperly formatted");

    let mut posts: Vec<SchoologyPost> = Vec::new();
    let mut authors: HashMap<String, SchoologyUser> = HashMap::new();
    for page in (config.start)..(config.start+config.limit) {
        let (mut new_posts, new_authors) = scrape_realm_by_page(&config.realm, &config.id, page, &secret);
        posts.append(&mut new_posts);
        authors.extend(new_authors);
    }
    for (k,v) in authors {
        println!("{} {}", k, v.name);
    }
    for post in posts {
        println!("==\n{}-{}-{}\n{}\n==", post.author, post.like_count, post.comments.len(), post.content);
    }
}

fn make_schoology_request(url: String, secret: &String) -> Response {
    let mut headers = header::HeaderMap::new();
    headers.insert("accept", header::HeaderValue::from_static(ACCEPT));
    headers.insert("cookie", header::HeaderValue::from_str(&*format!("{}={}", AUTH_COOKIE_NAME, secret)).unwrap());

    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();
    let result = client.get(url).send().unwrap();
    // schoology has a limit of 15 requests per five seconds
    // TODO: rewrite this function to instantly send all requests if there are less than 15
    thread::sleep(Duration::from_millis(334));
    result
}

fn scrape_realm_by_page(realm: &String, id: &String, page: u32, secret: &String) -> (Vec<SchoologyPost>, HashMap<String, SchoologyUser>) {
    let mut posts: Vec<SchoologyPost> = Vec::new();
    let mut authors: HashMap<String, SchoologyUser> = HashMap::new();

    let response: FeedResponse = make_schoology_request(
        format!("{}/{}/{}/feed?page={}", SCHOOLOGY_BASEURL, realm, id, page),
        secret,
    ).json().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");

    let soup = Soup::new(&*response.output);
    for post in soup.attr("class", "s-edge-type-update-post").find_all() {

        // Get post ID
        let id = post.attr("class", "like-btn").find().unwrap()
            .get("ajax").unwrap().rsplit("/").next().unwrap().to_string();

        // Get post author
        let author_anchor = post.attr("class", "update-sentence-inner").find().unwrap().tag("a").find().unwrap();
        let author_id = author_anchor.get("href").unwrap().rsplit("/").next().unwrap().to_string();

        // if our author isn't in our list, add it
        if !authors.contains_key(&author_id) {
            authors.insert(author_id.clone(), SchoologyUser {
                id: author_id.clone(),
                name: author_anchor.text(),
                avatar: "".to_string(), // todo: avatar
            });
        }

        // Get post content
        let mut content: String = post.tag("p").find_all().map(|line| line.text() + "\n").collect();

        match post.attr("class", "show-more-link").find() {
            // If there's a "Show More" prompt
            Some(show_more) => {
                let response: ShowMoreResponse = make_schoology_request(
                    format!("{}/{}", SCHOOLOGY_BASEURL, show_more.get("href").unwrap()),
                    secret,
                ).json().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");

                let soup = Soup::new(&*response.update);
                content = soup.tag("p").find_all().map(|line| line.text() + "\n").collect();
            }
            _ => {}
        }

        content.truncate(content.trim_end_matches(&['\r', '\n'][..]).len());

        let like_count: u32 = post.attr("class", "like-details-btn").find().unwrap()
            .text().split(" ").next().unwrap().parse().unwrap();

        // Get post comments
        let mut comments: Vec<SchoologyComment> = Vec::new();

        let comment_response: CommentsResponse = make_schoology_request(
            format!("{}/comment/ajax/{}&context=updates", SCHOOLOGY_BASEURL, id),
            secret,
        ).json().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");

        let comment_soup = Soup::new(&*comment_response.comments);
        comments = comment_soup.attr("class", "comment-comment").find_all().map(|comment| {
            let author_anchor = comment.attr("class", "comment-author").find().unwrap().tag("a").find().unwrap();
            let author_id = author_anchor.get("href").unwrap().rsplit("/").next().unwrap().to_string();

            // if our author isn't in our list, add it
            if !authors.contains_key(&author_id) {
                authors.insert(author_id.clone(), SchoologyUser {
                    id: author_id.clone(),
                    name: author_anchor.text(),
                    avatar: "".to_string(), // todo: avatar
                });
            }

            SchoologyComment {
                author: author_id.clone(),
                content: comment.attr("class", "comment-body-wrapper").find().unwrap().text(),
                timestamp: "".to_string(),
                like_count, // todo: likes and like count
                likes: vec![],
            }
        }).collect();

        posts.push(SchoologyPost {
            author: author_id,
            content,
            like_count,
            likes: vec![], // todo: likes
            comments,
        })
    }

    (posts, authors)
}


#[derive(Deserialize)]
struct SchoologyPost {
    author: String,
    content: String,
    like_count: u32,
    likes: Vec<String>,
    comments: Vec<SchoologyComment>,
}

#[derive(Deserialize)]
struct SchoologyComment {
    author: String,
    content: String,
    timestamp: String,
    like_count: u32,
    likes: Vec<String>,
}

#[derive(Deserialize)]
struct SchoologyUser {
    id: String,
    name: String,
    avatar: String,
    // ... todo: badges, email, groups, schools
}

#[derive(Deserialize)]
struct FeedResponse {
    output: String,
}

#[derive(Deserialize)]
struct ShowMoreResponse {
    update: String,
}

#[derive(Deserialize)]
struct CommentsResponse {
    comments: String,
    count: String, // Schoology stores count as a string.
}

#[derive(Deserialize)]
struct Config {
    realm: String,
    id: String,
    start: u32,
    limit: u32,
}
