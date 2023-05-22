use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use reqwest;
use reqwest::blocking::Response;
use reqwest::header;
use soup::prelude::*;
use data_encoding::BASE64;
use crate::{ACCEPT, AUTH_COOKIE_NAME, EMPTY_FEEDPAGE, SCHOOLOGY_BASEURL};

use crate::serializable::*;

pub fn scrape_realm_by_page(realm: &String, id: &String, page: u32, secret: &String) -> (Vec<SchoologyPost>, HashMap<String, SchoologyUser>) {
    let mut posts: Vec<SchoologyPost> = Vec::new();
    let mut authors: HashMap<String, SchoologyUser> = HashMap::new();

    println!("Scanning {} ({}) on page {}...", realm, id, page);

    let response: FeedResponse = make_schoology_request(
        format!("{}/{}/{}/feed?page={}", SCHOOLOGY_BASEURL, realm, id, page),
        secret,
    ).json().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");

    if response.output != EMPTY_FEEDPAGE {
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
                    content = soup.tag("p").find_all().map(|line| line.display() + "\n").collect();
                }
                _ => {}
            }
            content.truncate(content.trim_end_matches(&['\r', '\n'][..]).len());

            let like_count: u32 = match post.attr("class", "like-details-btn").find() {
                None => {0}
                Some(details) => {details.text().split(" ").next().unwrap().parse().unwrap()}
            };

            // Get post comments
            let comment_response: CommentsResponse = make_schoology_request(
                format!("{}/comment/ajax/{}&context=updates", SCHOOLOGY_BASEURL, id),
                secret,
            ).json().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");

            let comment_soup = Soup::new(&*comment_response.comments);
            let comments: Vec<SchoologyComment> = comment_soup.attr("class", "comment-comment").find_all().map(|comment| {
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
                    content:  BASE64.encode(comment.attr("class", "comment-body-wrapper").find().unwrap().text()),
                    timestamp: "".to_string(),
                    like_count, // todo: likes and like count
                    likes: vec![],
                }
            }).collect();


            posts.push(SchoologyPost {
                author: author_id,
                content: BASE64.encode(content.as_ref()),
                like_count,
                likes: vec![], // todo: likes
                comments,
            })
        }
    }

    (posts, authors)
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