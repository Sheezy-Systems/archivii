use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// TODO: create a macro that adds pub to the start of everything, lol

#[derive(Serialize, Deserialize)]
pub struct SchoologyUsers {
    pub users: HashMap<String, SchoologyUser>,
}

#[derive(Serialize, Deserialize)]
pub struct SchoologyRealm {
    // todo: id, name, banner, information, category, members
    pub posts: Vec<SchoologyPost>,
}

#[derive(Serialize, Deserialize)]
pub struct SchoologyPost {
    pub author: String,
    pub content: String,
    pub like_count: u32,
    pub likes: Vec<String>,
    pub comments: Vec<SchoologyComment>,
}

#[derive(Serialize, Deserialize)]
pub struct SchoologyComment {
    pub author: String,
    pub content: String,
    pub timestamp: String,
    pub like_count: u32,
    pub likes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SchoologyUser {
    pub id: String,
    pub name: String,
    pub avatar: String,
    // ... todo: badges, email, groups, schools
}

#[derive(Deserialize)]
pub struct FeedResponse {
    pub output: String,
}

#[derive(Deserialize)]
pub struct ShowMoreResponse {
    pub update: String,
}

#[derive(Deserialize)]
pub struct CommentsResponse {
    pub comments: String,
    pub count: String, // Schoology stores the amount of comments as a string.
}

#[derive(Deserialize)]
pub struct Config {
    pub realm: String,
    pub id: String,
    pub start: u32,
    pub limit: u32,
}