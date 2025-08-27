use scraper::Selector;

pub struct Selectors {
    pub items: Selector,
    pub title: Selector,
    pub author_picture: Selector,
    pub author_name: Selector,
    pub published_at: Selector,
    pub published_at_normal: Selector,
    pub view: Selector,
    pub reply: Selector,
    pub preview_images: Selector,
    pub last_reply_name: Selector,
    pub last_reply_published_at: Selector,
    pub last_reply_published_at_normal: Selector,
    pub href: Selector,
}

impl Selectors {
    pub fn new() -> Self {
        macro_rules! parse_selector {
            ($s:expr) => {
                Selector::parse($s).expect(concat!("Failed to parse selector: ", $s))
            };
        }

        Self {
            items: parse_selector!("#threadlisttableid > tbody"),
            title: parse_selector!("tr > th > div.post_inforight > div.post_infolist > div > a.s"),
            author_picture: parse_selector!("tr > th > div.post_avatar > a > img"),
            author_name: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > div:nth-child(1) > span.author > a"
            ),
            published_at: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > div:nth-child(1) > span.dateline > span"
            ),
            published_at_normal: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > div:nth-child(1) > span.dateline"
            ),
            view: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > div.z.nums > span.views"
            ),
            reply: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > div.z.nums > span.reply"
            ),
            preview_images: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist > div > a > img"
            ),
            last_reply_name: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > span > a"
            ),
            last_reply_published_at: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > span > span:nth-child(3) > span"
            ),
            last_reply_published_at_normal: parse_selector!(
                "tr > th > div.post_inforight > div.post_infolist_other > span > span:nth-child(3)"
            ),
            href: parse_selector!("tr > th > div.post_inforight > div.post_infolist > div > a.s"),
        }
    }
}
