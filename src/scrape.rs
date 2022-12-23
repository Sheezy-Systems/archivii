use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use reqwest;
use reqwest::blocking::Response;
use reqwest::header;
use soup::prelude::*;
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
            let comment_response: CommentsResponse = make_schoology_request(
                format!("{}/comment/ajax/{}&context=updates", SCHOOLOGY_BASEURL, id),
                secret,
            ).json().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");

            let comment_soup = Soup::new(&*comment_response.comments);
            let comments: Vec<SchoologyComment> = comment_soup.attr("class", "discussion-card").find_all().map(|comment| {
                let author_anchor = comment.attr("class", "comment-author").find().unwrap().tag("a").find().unwrap();
                let author_id = author_anchor.get("href").unwrap().rsplit("/").next().unwrap().to_string();
                let comment_id = comment.attr("class", "like-btn").find().unwrap().get("ajax").unwrap().rsplit("/").next().unwrap().to_string();
                let likes_url = format!("{}/likes/c/{}", SCHOOLOGY_BASEURL, comment_id);
                let likes_response = make_schoology_request(likes_url, secret).text().expect("Failed to parse response as JSON: Possibly bad SECRET or rate-limiting");
                let likes_soup = Soup::new(&*likes_response);
                let _likes_list = likes_soup.tag("ul").find().unwrap();
                // println!("{}", likes_list.text());

                let likes_list: Vec<String> = likes_soup.tag("ul").find().iter().map(|like| {
                    let name = like.attr("class", "sExtlink-processed").find().unwrap().get("title").unwrap();
                    let id = like.get("href").unwrap().rsplit("/").next().unwrap().to_string();
                    if !authors.contains_key(&author_id) {
                        authors.insert(author_id.clone(), SchoologyUser {
                            id: id.clone(),
                            name
                        });
                    }

                    id
                }).collect();

                // if our author isn't in our list, add it
                if !authors.contains_key(&author_id) {
                    authors.insert(author_id.clone(), SchoologyUser {
                        id: author_id.clone(),
                        name: author_anchor.text(),
                    });
                }


                SchoologyComment {
                    author: author_id.clone(),
                    content: comment.attr("class", "comment-body-wrapper").find().unwrap().text(),
                    id: comment_id.clone(),
                    timestamp: "".to_string(),
                    like_count, // todo: likes and like count
                    likes: likes_list
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