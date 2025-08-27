mod article;
mod selector;

use std::sync::Arc;

use article::{Article, Author, LastReply};
use chrono::{NaiveDate, NaiveDateTime};
use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement, IntoElement, ListAlignment, ListState, MouseButton,
    ParentElement as _, Pixels, Render, SharedString, Styled as _, Window, div, img, list,
    prelude::FluentBuilder as _, px,
};
use gpui_component::button::ButtonVariants as _;
use gpui_component::input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction};
use gpui_component::{
    ActiveTheme as _, ContextModal as _, Disableable, Icon, Sizable, StyledExt, avatar::Avatar,
    button::Button, indicator::Indicator, label::Label,
};
use http_client::{AsyncBody, HttpClient, Request};
use scraper::{ElementRef, Html};
use selector::Selectors;
use smol::io::AsyncReadExt as _;

use crate::icon::IconName;

const SUMMARY_WIDTH: Pixels = px(700.);
const PAGER_HEIGHT: Pixels = px(50.);

pub struct Summary {
    selectors: Arc<Selectors>,
    articles: Vec<Article>,
    list_state: ListState,
    page: u32,
    page_state: Entity<InputState>,
    page_input_value: u32,
    is_loading: bool,
    focus_handle: FocusHandle,
    window_handle: AnyWindowHandle,
}

impl Summary {
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
            articles: Vec::new(),
            list_state: ListState::new(0, ListAlignment::Top, px(1000.)),
            page: 0,
            page_state,
            page_input_value: 0,
            is_loading: false,
            focus_handle: cx.focus_handle(),
            window_handle: window.window_handle(),
        }
    }

    fn on_event(&mut self, _: Entity<Self>, evt: &SummaryEvent, cx: &mut Context<Self>) {
        match evt {
            SummaryEvent::Load(page) => self.event_load(*page, cx),
            _ => {}
        }
    }

    fn event_load(&mut self, page: u32, cx: &mut Context<Self>) {
        if self.is_loading {
            return;
        }
        self.is_loading = true;
        cx.notify();

        let client = cx.http_client();
        let selectors = self.selectors.clone();
        cx.spawn(async move |this, cx| {
            let articles = Self::load_page(client, &selectors, page).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| match articles {
                    Ok(articles) => this.load_success(articles, page, cx),
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

    fn load_success(&mut self, articles: Vec<Article>, page: u32, cx: &mut Context<Self>) {
        self.page = page;
        self.page_input_value = page;
        cx.update_window(self.window_handle, |_, window, cx| {
            self.page_state.update(cx, |this, cx| {
                this.set_value(self.page_input_value.to_string(), window, cx);
            });
        })
        .ok();
        self.articles.clear();
        self.articles.extend(articles);
        self.list_state.reset(self.articles.len() + 1);
        self.is_loading = false;
        cx.notify();
    }

    async fn load_page(
        http_client: Arc<dyn HttpClient>,
        selectors: &Selectors,
        page: u32,
    ) -> anyhow::Result<Vec<Article>> {
        let url =
            format!("https://www.javbus.com/forum/forum.php?mod=forumdisplay&fid=2&page={page}");
        let request = Request::builder()
            .method("GET")
            .uri(url)
            .header("Cookie", "existmag=mag")
            .header("Accept-Language", "zh-CN,zh-Hans;q=0.9")
            .body(AsyncBody::empty())
            .map_err(|error| anyhow::anyhow!("构建请求失败 - {error}"))?;
        let response = http_client.send(request).await?;
        anyhow::ensure!(response.status().is_success(), "加载页面失败 - {page}");

        let mut text = String::new();
        let mut body = response.into_body();
        body.read_to_string(&mut text)
            .await
            .map_err(|error| anyhow::anyhow!("读取内容失败 - {error}"))?;

        let articles = Self::parse_page(&text, selectors);

        Ok(articles)
    }

    fn parse_page(text: &str, selectors: &Selectors) -> Vec<Article> {
        let html = Html::parse_document(&text);

        html.select(&selectors.items)
            .into_iter()
            .flat_map(|item| Self::parse_single_article(item, selectors))
            .collect()
    }

    fn parse_single_article(item: ElementRef, selectors: &Selectors) -> Option<Article> {
        let title = item
            .select(&selectors.title)
            .next()
            .map(|title| title.text())
            .map(|title| title.collect::<String>())
            .map(SharedString::from)?;
        let author_picture = item
            .select(&selectors.author_picture)
            .next()
            .and_then(|img| img.attr("src"))
            .map(String::from)
            .map(SharedString::from)?;
        let author_name = item
            .select(&selectors.author_name)
            .next()
            .map(|name| name.text())
            .map(|name| name.collect::<String>())
            .map(SharedString::from)?;
        let author = Author {
            name: author_name,
            picture: author_picture,
        };
        let published_at = item
            .select(&selectors.published_at)
            .next()
            .and_then(|span| span.attr("title"))
            .and_then(|time| NaiveDate::parse_from_str(time, "%Y-%m-%d").ok())
            .or_else(|| {
                item.select(&selectors.published_at_normal)
                    .next()
                    .map(|span| span.text().collect::<String>())
                    .and_then(|time| NaiveDate::parse_from_str(&time, "%Y-%m-%d").ok())
            })?;
        let view = item
            .select(&selectors.view)
            .next()
            .map(|view| view.text())
            .map(|view| view.collect::<String>())
            .and_then(|view| view.parse::<u32>().ok())?;
        let reply = item
            .select(&selectors.reply)
            .next()
            .map(|reply| reply.text())
            .map(|reply| reply.collect::<String>())
            .and_then(|reply| reply.parse::<u32>().ok())?;
        let last_reply_name = item
            .select(&selectors.last_reply_name)
            .next()
            .map(|name| name.text())
            .map(|name| name.collect::<String>())
            .map(SharedString::from)?;
        let last_reply_published_at = item
            .select(&selectors.last_reply_published_at)
            .next()
            .and_then(|span| span.attr("title"))
            .and_then(|time| NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M").ok())
            .or_else(|| {
                item.select(&selectors.last_reply_published_at_normal)
                    .next()
                    .map(|span| span.text().collect::<String>())
                    .and_then(|time| NaiveDateTime::parse_from_str(&time, "%Y-%m-%d %H:%M").ok())
            })?;
        let last_reply = LastReply {
            name: last_reply_name,
            published_at: last_reply_published_at,
        };
        let preview_images = item
            .select(&selectors.preview_images)
            .into_iter()
            .flat_map(|img| {
                img.attr("src").and_then(|src| match src {
                    "template/javbus/images/folder_lock.gif"
                    | "template/javbus/images/pollsmall.gif" => None,
                    _ => Some(src),
                })
            })
            .map(|img| format!("https://www.javbus.com/forum/{img}"))
            .map(SharedString::from)
            .collect();
        let href = item
            .select(&selectors.href)
            .next()
            .and_then(|href| href.attr("href"))
            .map(|href| format!("https://www.javbus.com/forum/{href}"))
            .map(SharedString::from)?;

        let article = Article {
            title,
            author,
            published_at,
            view,
            reply,
            last_reply,
            preview_images,
            href,
        };

        Some(article)
    }

    fn on_item_click(detail_url: SharedString, cx: &mut Context<Self>) {
        cx.emit(SummaryEvent::LoadDetail(detail_url));
        cx.notify();
    }

    fn render_article(&self, article: &Article, cx: &mut Context<Self>) -> impl IntoElement {
        let href = article.href.clone();
        let theme = cx.theme();

        div()
            .p_2()
            .rounded_md()
            .bg(theme.secondary_hover)
            .border_1()
            .border_color(theme.border)
            .hover(|style| style.bg(theme.secondary_active))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_, _, _, cx| Self::on_item_click(href.clone(), cx)),
            )
            .child(Label::new(article.title.clone()).font_semibold().text_lg())
            .child(
                div()
                    .flex()
                    .pt_1()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(Avatar::new().xsmall().src(article.author.picture.clone()))
                            .child(
                                Label::new(article.author.name.clone())
                                    .text_color(theme.blue)
                                    .font_light()
                                    .text_sm(),
                            )
                            .child(
                                Label::new(article.published_at.format("@ %Y-%m-%d").to_string())
                                    .text_color(theme.yellow)
                                    .font_light()
                                    .text_sm(),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .child(Icon::new(IconName::Eye).small())
                            .child(
                                Label::new(article.view.to_string())
                                    .text_color(theme.primary_hover)
                                    .font_light()
                                    .text_sm(),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .child(Icon::new(IconName::MessageCircle).small())
                            .child(
                                Label::new(article.reply.to_string())
                                    .text_color(theme.primary_hover)
                                    .font_light()
                                    .text_sm(),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .py_1()
                    .gap_1()
                    .items_center()
                    .justify_end()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .child(Icon::new(IconName::Reply).small())
                            .child(
                                Label::new(article.last_reply.name.clone())
                                    .text_color(theme.blue)
                                    .font_light()
                                    .text_sm(),
                            ),
                    )
                    .child(
                        Label::new(
                            article
                                .last_reply
                                .published_at
                                .format("@ %Y-%m-%d %H:%M")
                                .to_string(),
                        )
                        .text_color(theme.yellow)
                        .font_light()
                        .text_sm(),
                    ),
            )
            .when(!article.preview_images.is_empty(), |this| {
                this.child(
                    div()
                        .pt_1()
                        .h(px(100.))
                        .w_full()
                        .flex()
                        .justify_between()
                        .items_center()
                        .children(article.preview_images.iter().map(|preview| {
                            img(preview.clone())
                                .w(px(120.))
                                .rounded_md()
                                .bg(theme.background)
                        })),
                )
            })
    }

    fn render_item(&self, idx: usize, cx: &mut Context<Self>) -> AnyElement {
        let item = if idx == self.articles.len() {
            self.render_pager(cx).into_any_element()
        } else {
            let article = &self.articles[idx];
            self.render_article(article, cx).into_any_element()
        };

        div()
            .pt_2()
            .px_2()
            .when(idx == self.articles.len(), |div| div.pb_2())
            .child(item)
            .into_any_element()
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
            cx.processor(|this, idx, _, cx| this.render_item(idx, cx)),
        )
        .size_full()
    }

    fn render_pager(&self, cx: &Context<Self>) -> impl IntoElement {
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
                        let page = this.page - 1;
                        cx.emit(SummaryEvent::Load(page));
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
                    .child(Label::new("页")),
            )
            .child(
                Button::new("ForumNext")
                    .icon(IconName::ChevronRight)
                    .ghost()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        let page = this.page + 1;
                        cx.emit(SummaryEvent::Load(page));
                        cx.notify();
                    })),
            )
    }

    fn on_input_event(
        &mut self,
        this: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::PressEnter { secondary: _ } => {
                let page = self.page_input_value;
                cx.emit(SummaryEvent::Load(page));
                cx.focus_self(window);
                cx.notify();
            }
            InputEvent::Change(text) => {
                if this == &self.page_state {
                    if let Ok(page) = text.parse::<u32>() {
                        if page != 0 {
                            self.page_input_value = page;
                            return;
                        }

                        self.page_input_value = 1;
                    };

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
                        if self.page_input_value == std::u32::MAX {
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
}

impl Render for Summary {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.render_content(window, cx);
        let theme = cx.theme();

        div()
            .track_focus(&self.focus_handle)
            .h_full()
            .w(SUMMARY_WIDTH)
            .border_r_1()
            .border_color(theme.border)
            .when(self.is_loading, |div| div.child(Self::load_circle()))
            .when(!self.is_loading, |div| div.child(content))
    }
}

impl Focusable for Summary {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub enum SummaryEvent {
    Load(u32),
    LoadDetail(SharedString),
}

impl EventEmitter<SummaryEvent> for Summary {}
