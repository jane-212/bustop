use chrono::NaiveDateTime;
use gpui::SharedString;

pub struct TalkPage {
    pub total_page: u32,
    pub title: SharedString,
    pub href: SharedString,
    pub talks: Vec<Talk>,
}

pub struct Talk {
    pub author_name: SharedString,
    pub author_picture: SharedString,
    pub published_at: NaiveDateTime,
    pub count: u32,
    pub content: Vec<Content>,
    pub replys: Vec<Reply>,
}

pub enum Content {
    Text(SharedString),
    Image(SharedString),
    Quote(SharedString, NaiveDateTime, SharedString),
}

pub struct Reply {
    pub author_name: SharedString,
    pub author_picture: SharedString,
    pub published_at: NaiveDateTime,
    pub content: SharedString,
}
