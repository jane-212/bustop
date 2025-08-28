use scraper::Selector;

pub struct Selectors {
    pub title: Selector,
    pub page: Selector,
    pub main_author_name: Selector,
    pub main_author_picture: Selector,
    pub main_published_at: Selector,
    pub main_content: Selector,
    pub main_replys: Selector,
    pub reply_name: Selector,
    pub reply_picture: Selector,
    pub reply_published_at: Selector,
    pub reply_published_at_normal: Selector,
    pub reply_content: Selector,
    pub items: Selector,
    pub item_name: Selector,
    pub item_picture: Selector,
    pub item_published_at: Selector,
    pub item_published_at_normal: Selector,
    pub item_count: Selector,
    pub item_content: Selector,
    pub item_replys: Selector,
}

impl Selectors {
    pub fn new() -> Self {
        macro_rules! parse_selector {
            ($s:expr) => {
                Selector::parse($s).expect(concat!("Failed to parse selector: ", $s))
            };
        }

        Self {
            title: parse_selector!("#thread_subject"),
            page: parse_selector!(
                "#ct > div.wp.cl > div.mn > div.pgs.mtm.mbm.cl > div.pg > label > span"
            ),
            main_author_name: parse_selector!(
                "#ct > div.wp.cl > div.sd.sd_allbox > div.viewthread_authorinfo > div.authi > a"
            ),
            main_author_picture: parse_selector!(
                "#ct > div.wp.cl > div.sd.sd_allbox > div.viewthread_authorinfo > div.avatar > a > img"
            ),
            main_published_at: parse_selector!(
                "#postlist > div.nthread_info.cl > div > div > span:nth-child(2)"
            ),
            main_content: parse_selector!(
                "#postlist > div.nthread_firstpostbox > table.nthread_firstpost > tbody > tr:nth-child(1) > td > div > div > div:nth-child(2) > table > tbody > tr > td.t_f"
            ),
            main_replys: parse_selector!(
                "#postlist > div.nthread_firstpostbox > table.nthread_firstpost > tbody > tr:nth-child(1) > td > div > div > div.cm > div.pstl"
            ),
            reply_name: parse_selector!("div.psta.vm > a.xi2.xw1"),
            reply_picture: parse_selector!("div.psta.vm > a:nth-child(1) > img"),
            reply_published_at: parse_selector!("div.psti > span > span"),
            reply_published_at_normal: parse_selector!("div.psti > span"),
            reply_content: parse_selector!("div.psti"),
            items: parse_selector!("#postlist > div.nthread_postbox"),
            item_name: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.plc > div.pi > div > div.authi > a.xw1"
            ),
            item_picture: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.pls > div.pls.favatar > div > div.avatar > a > img"
            ),
            item_published_at: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.plc > div.pi > div > div.authi > em > span"
            ),
            item_published_at_normal: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.plc > div.pi > div > div.authi > em"
            ),
            item_count: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.plc > div.pi > strong > a > em"
            ),
            item_content: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.plc > div.pct > div > div:nth-child(1) > table > tbody > tr > td.t_f"
            ),
            item_replys: parse_selector!(
                "table.plhin > tbody > tr:nth-child(1) > td.plc > div.pct > div.pcb > div.cm > div.pstl.xs1.cl"
            ),
        }
    }
}
