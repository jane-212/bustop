mod selector;
mod talk;

use std::sync::Arc;

use chrono::NaiveDateTime;
use ego_tree::NodeRef;
use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, ListAlignment, ListState, ParentElement as _,
    Pixels, Render, SharedString, Styled as _, Window, div, img, list, prelude::FluentBuilder as _,
    px,
};
use gpui_component::avatar::Avatar;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::indicator::Indicator;
use gpui_component::input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction};
use gpui_component::label::Label;
use gpui_component::{
    ActiveTheme as _, ContextModal as _, Disableable, Sizable as _, StyledExt as _,
};
use http_client::{AsyncBody, HttpClient, Request};
use scraper::{ElementRef, Html, Node};
use selector::Selectors;
use smol::io::AsyncReadExt as _;
use talk::{Content, Reply, Talk, TalkPage};

use crate::icon::IconName;

const PAGER_HEIGHT: Pixels = px(50.);

pub struct Detail {
    selectors: Arc<Selectors>,
    list_state: ListState,
    page: u32,
    page_state: Entity<InputState>,
    page_input_value: u32,
    is_loading: bool,
    talk: Option<TalkPage>,
    focus_handle: FocusHandle,
    window_handle: AnyWindowHandle,
}

impl Detail {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        cx.subscribe(&entity, Self::on_event).detach();
        let page_state = cx.new(|cx| InputState::new(window, cx).placeholder(""));
        cx.subscribe_in(&page_state, window, Self::on_input_event)
            .detach();
        cx.subscribe_in(&page_state, window, Self::on_number_input_event)
            .detach();

        Self {
            selectors: Arc::new(Selectors::new()),
            list_state: ListState::new(0, ListAlignment::Top, px(1000.)),
            page: 0,
            page_state,
            page_input_value: 0,
            is_loading: false,
            talk: None,
            focus_handle: cx.focus_handle(),
            window_handle: window.window_handle(),
        }
    }

    fn on_input_event(
        &mut self,
        this: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ref talk) = self.talk else {
            return;
        };
        match event {
            InputEvent::PressEnter { secondary: _ } => {
                let page = self.page_input_value;
                cx.emit(DetailEvent::Load(talk.href.clone(), page));
                cx.focus_self(window);
                cx.notify();
            }
            InputEvent::Change(text) => {
                if this == &self.page_state {
                    if let Ok(page) = text.parse::<u32>() {
                        let max_page = self.talk.as_ref().map(|talk| talk.total_page).unwrap_or(1);
                        if page != 0 && page <= max_page {
                            self.page_input_value = page;
                            return;
                        }

                        if page == 0 {
                            self.page_input_value = 1;
                        }

                        if page > max_page {
                            self.page_input_value = max_page;
                        }
                    }

                    this.update(cx, |input, cx| {
                        input.set_value(self.page_input_value.to_string(), window, cx);
                    });
                }
            }
            _ => {}
        }
    }

    fn on_number_input_event(
        &mut self,
        this: &Entity<InputState>,
        event: &NumberInputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NumberInputEvent::Step(step_action) => match step_action {
                StepAction::Decrement => {
                    if this == &self.page_state {
                        if self.page_input_value <= 1 {
                            return;
                        }

                        self.page_input_value = self.page_input_value - 1;
                        this.update(cx, |input, cx| {
                            input.set_value(self.page_input_value.to_string(), window, cx);
                        });
                    }
                }
                StepAction::Increment => {
                    if this == &self.page_state {
                        let max_page = self.talk.as_ref().map(|talk| talk.total_page).unwrap_or(1);
                        if self.page_input_value == max_page {
                            return;
                        }

                        self.page_input_value = self.page_input_value + 1;
                        this.update(cx, |input, cx| {
                            input.set_value(self.page_input_value.to_string(), window, cx);
                        });
                    }
                }
            },
        }
    }

    fn on_event(&mut self, _: Entity<Self>, evt: &DetailEvent, cx: &mut Context<Self>) {
        match evt {
            DetailEvent::Load(detail_url, page) => self.event_load(detail_url, *page, cx),
        }
    }

    fn event_load(&mut self, url: &SharedString, page: u32, cx: &mut Context<Self>) {
        if self.is_loading {
            return;
        }
        self.is_loading = true;
        cx.notify();

        let client = cx.http_client();
        let selectors = self.selectors.clone();
        let url = url.clone();
        cx.spawn(async move |this, cx| {
            let talk = Self::load_detail(client, &selectors, url, page).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| match talk {
                    Ok(update) => this.load_success(update, page, cx),
                    Err(error) => this.load_failure(error, cx),
                })
                .ok();
            }
        })
        .detach();
    }

    fn load_failure(&mut self, error: anyhow::Error, cx: &mut Context<Self>) {
        cx.update_window(self.window_handle, |_, window, cx| {
            window.push_notification(error.to_string(), cx);
        })
        .ok();
        self.is_loading = false;
        cx.notify();
    }

    fn load_success(&mut self, update: Update, page: u32, cx: &mut Context<Self>) {
        self.page = page;
        self.page_input_value = page;
        cx.update_window(self.window_handle, |_, window, cx| {
            self.page_state.update(cx, |this, cx| {
                this.set_value(self.page_input_value.to_string(), window, cx);
            });
        })
        .ok();
        match update {
            Update::All(talk_page) => {
                self.talk = Some(talk_page);
            }
            Update::Talk(talks) => {
                if let Some(talk) = &mut self.talk {
                    talk.talks.clear();
                    talk.talks.extend(talks);
                }
            }
        }
        if let Some(ref talk_page) = self.talk {
            self.list_state.reset(talk_page.talks.len() + 2);
        }
        self.is_loading = false;
        cx.notify();
    }

    async fn load_detail(
        http_client: Arc<dyn HttpClient>,
        selectors: &Selectors,
        href: SharedString,
        page: u32,
    ) -> anyhow::Result<Update> {
        let url = format!("{href}&page={page}");
        let request = Request::builder()
            .method("GET")
            .uri(&url)
            .header("Cookie", "existmag=mag")
            .header("Accept-Language", "zh-CN,zh-Hans;q=0.9")
            .body(AsyncBody::empty())
            .map_err(|error| anyhow::anyhow!("构建请求失败 - {error}"))?;
        let response = http_client.send(request).await?;
        anyhow::ensure!(response.status().is_success(), "加载页面失败 - {url}");

        let mut text = String::new();
        let mut body = response.into_body();
        body.read_to_string(&mut text)
            .await
            .map_err(|error| anyhow::anyhow!("读取内容失败 - {error}"))?;

        let update = Self::parse_page(href, &text, selectors, page == 1)
            .ok_or_else(|| anyhow::anyhow!("解析失败"))?;

        Ok(update)
    }

    fn parse_page(
        href: SharedString,
        text: &str,
        selectors: &Selectors,
        is_first_page: bool,
    ) -> Option<Update> {
        let html = Html::parse_document(text);

        if is_first_page {
            Self::parse_first_page(href, html, selectors)
        } else {
            Self::parse_normal_page(html, selectors)
        }
    }

    fn parse_first_page(href: SharedString, html: Html, selectors: &Selectors) -> Option<Update> {
        let title = html
            .select(&selectors.title)
            .next()
            .and_then(|span| span.text().next())
            .map(|title| title.trim().to_string())
            .map(SharedString::from)?;
        let page = html
            .select(&selectors.page)
            .next()
            .and_then(|span| span.attr("title"))
            .map(|title| title.trim_matches(&['共', '頁', ' ']))
            .and_then(|page| page.parse::<u32>().ok())
            .unwrap_or(1);
        let main_author_name = html
            .select(&selectors.main_author_name)
            .next()
            .map(|name| name.text().collect::<String>())
            .map(SharedString::from)?;
        let main_author_picture = html
            .select(&selectors.main_author_picture)
            .next()
            .and_then(|img| img.attr("src"))
            .map(|src| src.to_string())
            .map(SharedString::from)?;
        let main_published_at = html
            .select(&selectors.main_published_at)
            .next()
            .map(|span| span.text().collect::<String>())
            .and_then(|date_time| {
                NaiveDateTime::parse_from_str(date_time.trim(), "%Y-%m-%d %H:%M:%S").ok()
            })?;
        let main_content = html
            .select(&selectors.main_content)
            .next()
            .map(|content| Self::parse_content(content))?;
        let main_replys = html
            .select(&selectors.main_replys)
            .into_iter()
            .flat_map(|item| Self::parse_reply(item, selectors))
            .collect();
        let talk = Talk {
            author_name: main_author_name,
            author_picture: main_author_picture,
            published_at: main_published_at,
            count: 1,
            content: main_content,
            replys: main_replys,
        };
        let mut talk_page = TalkPage {
            total_page: page,
            title,
            href,
            talks: vec![talk],
        };
        let talks = html
            .select(&selectors.items)
            .into_iter()
            .flat_map(|item| Self::parse_item(item, selectors))
            .collect::<Vec<_>>();
        talk_page.talks.extend(talks);

        Some(Update::All(talk_page))
    }

    fn parse_normal_page(html: Html, selectors: &Selectors) -> Option<Update> {
        let talks = html
            .select(&selectors.items)
            .into_iter()
            .flat_map(|item| Self::parse_item(item, selectors))
            .collect();

        Some(Update::Talk(talks))
    }

    fn parse_item(item: ElementRef, selectors: &Selectors) -> Option<Talk> {
        let name = item
            .select(&selectors.item_name)
            .next()
            .map(|a| a.text().collect::<String>())
            .map(SharedString::from)?;
        let picture = item
            .select(&selectors.item_picture)
            .next()
            .and_then(|img| img.attr("src"))
            .map(|src| src.to_string())
            .map(SharedString::from)?;
        let published_at = item
            .select(&selectors.item_published_at)
            .next()
            .and_then(|span| span.attr("title"))
            .and_then(|date_time| {
                NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H:%M:%S").ok()
            })
            .or_else(|| {
                item.select(&selectors.item_published_at_normal)
                    .next()
                    .map(|em| em.text().collect::<String>())
                    .and_then(|date_time| {
                        let date_time = date_time.trim_start_matches(&[' ', '發', '表', '於']);
                        NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H:%M:%S").ok()
                    })
            })?;
        let count = item
            .select(&selectors.item_count)
            .next()
            .map(|em| em.text().collect::<String>())
            .and_then(|text| text.parse::<u32>().ok())?;
        let content = item
            .select(&selectors.item_content)
            .next()
            .map(|content| Self::parse_content(content))?;
        let replys = item
            .select(&selectors.item_replys)
            .into_iter()
            .flat_map(|item| Self::parse_reply(item, selectors))
            .collect();
        let talk = Talk {
            author_name: name,
            author_picture: picture,
            published_at,
            count,
            content,
            replys,
        };

        Some(talk)
    }

    fn parse_content(content: ElementRef) -> Vec<Content> {
        let mut contents = Vec::new();
        Self::parse_inner(*content, &mut contents);

        contents
    }

    fn parse_inner(node: NodeRef<Node>, contents: &mut Vec<Content>) {
        for node in node.children() {
            let value = node.value();
            match value {
                Node::Text(text) => {
                    let text = text.trim();
                    if text.is_empty() {
                        continue;
                    }
                    let text = text.to_string();
                    contents.push(Content::Text(SharedString::from(text)));
                }
                Node::Element(element) => {
                    let name = element.name();
                    match name {
                        "img" => {
                            let Some(src) = element.attr("src") else {
                                continue;
                            };
                            if !src.starts_with("http") {
                                continue;
                            }

                            let src = src.to_string();
                            contents.push(Content::Image(SharedString::from(src)));
                        }
                        "blockquote" => {
                            let Some(quote) = Self::parse_blockquote(node) else {
                                continue;
                            };
                            contents.push(quote);
                        }
                        _ => {
                            Self::parse_inner(node, contents);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_blockquote(node: NodeRef<Node>) -> Option<Content> {
        let mut texts = Vec::new();
        for child in node.descendants() {
            match child.value().as_text() {
                Some(text) => {
                    texts.push(text.to_string());
                }
                None => continue,
            }
        }

        let info = texts.get(0)?.split(' ').collect::<Vec<_>>();
        let content = texts.get(1)?.trim().to_string();
        let name = info.get(0)?.to_string();
        let date = info.get(2)?;
        let time = info.get(3)?;
        let date_time = format!("{date} {time}");
        let date_time = NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M").ok()?;

        Some(Content::Quote(
            SharedString::from(name),
            date_time,
            SharedString::from(content),
        ))
    }

    fn parse_reply(item: ElementRef, selectors: &Selectors) -> Option<Reply> {
        let reply_name = item
            .select(&selectors.reply_name)
            .next()
            .map(|a| a.text().collect::<String>())
            .map(SharedString::from)?;
        let reply_picture = item
            .select(&selectors.reply_picture)
            .next()
            .and_then(|img| img.attr("src"))
            .map(|src| src.to_string())
            .map(SharedString::from)?;
        let reply_published_at = item
            .select(&selectors.reply_published_at)
            .next()
            .and_then(|span| span.attr("title"))
            .and_then(|date_time| NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H:%M").ok())
            .or_else(|| {
                item.select(&selectors.reply_published_at_normal)
                    .next()
                    .map(|span| span.text().collect::<String>())
                    .and_then(|date_time| {
                        let date_time = date_time.trim_start_matches(&[' ', '發', '表', '於']);
                        NaiveDateTime::parse_from_str(date_time, "%Y-%m-%d %H:%M").ok()
                    })
            })?;
        let reply_content = item
            .select(&selectors.reply_content)
            .next()
            .and_then(|div| div.text().next())
            .map(|content| content.trim().to_string())
            .map(SharedString::from)?;
        let reply = Reply {
            author_name: reply_name,
            author_picture: reply_picture,
            published_at: reply_published_at,
            content: reply_content,
        };

        Some(reply)
    }

    fn load_circle() -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .justify_center()
            .items_center()
            .child(Indicator::new().large().icon(IconName::LoaderCircle))
    }

    fn render_content(&self, _: &Window, cx: &Context<Self>) -> impl IntoElement {
        list(
            self.list_state.clone(),
            cx.processor(|this, idx, window, cx| this.render_item(idx, window, cx)),
        )
        .size_full()
    }

    fn render_content_item(content: &Content, cx: &Context<Self>) -> AnyElement {
        let theme = cx.theme();

        match content {
            Content::Text(text) => Label::new(text).pt_2().into_any_element(),
            Content::Image(src) => div()
                .pt_2()
                .child(img(src.clone()).max_w_full().rounded_md())
                .into_any_element(),
            Content::Quote(name, date_time, content) => div()
                .pt_2()
                .child(
                    div()
                        .p_2()
                        .gap_1()
                        .rounded_md()
                        .bg(theme.secondary_active)
                        .child(
                            div()
                                .flex()
                                .gap_1()
                                .child(
                                    Label::new(name)
                                        .text_color(theme.blue)
                                        .font_light()
                                        .text_sm(),
                                )
                                .child(
                                    Label::new(date_time.format("@ %Y-%m-%d %H:%M").to_string())
                                        .text_color(theme.yellow)
                                        .font_light()
                                        .text_sm(),
                                ),
                        )
                        .child(Label::new(content)),
                )
                .into_any_element(),
        }
    }

    fn render_reply(reply: &Reply, cx: &Context<Self>, is_first: bool) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .items_center()
            .gap_1()
            .when(!is_first, |this| this.pt_2())
            .child(Avatar::new().src(reply.author_picture.clone()).small())
            .child(
                Label::new(reply.author_name.clone())
                    .text_color(theme.blue)
                    .font_light()
                    .text_sm(),
            )
            .child(
                Label::new(reply.published_at.format("@ %Y-%m-%d %H:%M").to_string())
                    .text_color(theme.yellow)
                    .font_light()
                    .text_sm(),
            )
            .child(Label::new(reply.content.clone()).overflow_hidden())
    }

    fn render_talk(&self, talk: &Talk, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let contents = talk
            .content
            .iter()
            .map(|item| Self::render_content_item(item, cx))
            .collect::<Vec<_>>();
        let replys = talk
            .replys
            .iter()
            .enumerate()
            .map(|(idx, reply)| Self::render_reply(reply, cx, idx == 0))
            .collect::<Vec<_>>();

        div()
            .p_2()
            .rounded_md()
            .bg(theme.secondary_hover)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .gap_1()
                    .child(Avatar::new().src(talk.author_picture.clone()))
                    .child(
                        div()
                            .child(
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        Label::new(talk.author_name.clone())
                                            .text_color(theme.blue)
                                            .font_light()
                                            .text_sm(),
                                    )
                                    .child(
                                        Label::new(
                                            talk.published_at
                                                .format("@ %Y-%m-%d %H:%M:%S")
                                                .to_string(),
                                        )
                                        .text_color(theme.yellow)
                                        .font_light()
                                        .text_sm(),
                                    ),
                            )
                            .child(
                                Label::new(format!("#{}", talk.count))
                                    .text_color(theme.primary_hover)
                                    .font_light()
                                    .text_sm(),
                            ),
                    ),
            )
            .children(contents)
            .when(!replys.is_empty(), |this| {
                this.child(
                    div()
                        .p_2()
                        .mt_2()
                        .rounded_md()
                        .bg(theme.secondary_active)
                        .children(replys),
                )
            })
    }

    fn render_item(&self, idx: usize, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(ref talk) = self.talk else {
            return div().into_any_element();
        };

        let item = if idx == 0 {
            self.render_title(window, cx).into_any_element()
        } else if idx == talk.talks.len() + 1 {
            self.render_pager(cx).into_any_element()
        } else {
            let talk = &talk.talks[idx - 1];
            self.render_talk(talk, cx).into_any_element()
        };

        div()
            .pt_2()
            .px_2()
            .when(idx == talk.talks.len() + 1, |div| div.pb_2())
            .child(item)
            .into_any_element()
    }

    fn render_title(&self, _: &Window, cx: &Context<Self>) -> impl IntoElement {
        let Some(ref talk) = self.talk else {
            return div();
        };
        let theme = cx.theme();

        div()
            .p_2()
            .rounded_md()
            .bg(theme.secondary_hover)
            .border_1()
            .border_color(theme.border)
            .child(Label::new(talk.title.clone()).font_semibold().text_lg())
    }

    fn render_pager(&self, cx: &Context<Self>) -> impl IntoElement {
        let max_page = self.talk.as_ref().map(|talk| talk.total_page).unwrap_or(1);

        div()
            .w_full()
            .h(PAGER_HEIGHT)
            .flex()
            .items_center()
            .justify_between()
            .p_2()
            .child(
                Button::new("ForumPrevious")
                    .icon(IconName::ChevronLeft)
                    .ghost()
                    .disabled(self.page <= 1)
                    .when(self.page > 1, |div| div.cursor_pointer())
                    .on_click(cx.listener(|this, _, _, cx| {
                        let Some(ref talk) = this.talk else {
                            return;
                        };

                        let page = this.page - 1;
                        cx.emit(DetailEvent::Load(talk.href.clone(), page));
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .h_full()
                    .w(px(200.))
                    .flex()
                    .justify_center()
                    .items_center()
                    .gap_1()
                    .child(Label::new("第"))
                    .child(NumberInput::new(&self.page_state))
                    .child(Label::new("页,"))
                    .child(Label::new(format!("共 {max_page} 页"))),
            )
            .child(
                Button::new("ForumNext")
                    .icon(IconName::ChevronRight)
                    .ghost()
                    .cursor_pointer()
                    .disabled(self.page >= max_page)
                    .when(self.page < max_page, |div| div.cursor_pointer())
                    .on_click(cx.listener(|this, _, _, cx| {
                        let Some(ref talk) = this.talk else {
                            return;
                        };

                        let page = this.page + 1;
                        cx.emit(DetailEvent::Load(talk.href.clone(), page));
                        cx.notify();
                    })),
            )
    }
}

impl Render for Detail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.render_content(window, cx);

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .when(self.is_loading, |div| div.child(Self::load_circle()))
            .when(!self.is_loading, |div| div.child(content))
    }
}

impl Focusable for Detail {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub enum DetailEvent {
    Load(SharedString, u32),
}

enum Update {
    All(TalkPage),
    Talk(Vec<Talk>),
}

impl EventEmitter<DetailEvent> for Detail {}
