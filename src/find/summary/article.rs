use chrono::{NaiveDate, NaiveDateTime};
use gpui::SharedString;

pub struct Article {
    pub title: SharedString,
    pub author: Author,
    pub published_at: NaiveDate,
    pub view: u32,
    pub reply: u32,
    pub last_reply: LastReply,
    pub preview_images: Vec<SharedString>,
    pub href: SharedString,
}

pub struct LastReply {
    pub name: SharedString,
    pub published_at: NaiveDateTime,
}

pub struct Author {
    pub name: SharedString,
    pub picture: SharedString,
}
