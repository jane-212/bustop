pub mod assets;
pub mod http_client;
pub mod theme;
pub mod window;

mod find;
mod forum;
mod icon;
mod left_bar;

use find::Find;
use forum::Forum;
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled as _, Window, div,
};
use gpui_component::{ActiveTheme as _, Root, TITLE_BAR_HEIGHT, TitleBar};
use left_bar::{LeftBar, LeftBarItem};

pub struct Bustop {
    left_bar: Entity<LeftBar>,
    forum: Entity<Forum>,
    find: Entity<Find>,
    focus_handle: FocusHandle,
}

impl Bustop {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let left_bar = cx.new(|cx| LeftBar::new(window, cx));
        let forum = cx.new(|cx| Forum::new(window, cx));
        let find = cx.new(|cx| Find::new(window, cx));

        Self {
            left_bar,
            forum,
            find,
            focus_handle: cx.focus_handle(),
        }
    }

    fn titlebar(&self) -> TitleBar {
        TitleBar::new()
    }

    fn content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let main_content = self.main_content(cx);

        div()
            .w_full()
            .h_full()
            .flex()
            .child(self.left_bar.clone())
            .child(main_content)
    }

    fn main_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let container = div().w_full().h_full();

        match self.left_bar.read(cx).selected_item() {
            LeftBarItem::Forum => container.child(self.forum.clone()),
            LeftBarItem::Find => container.child(self.find.clone()),
        }
    }
}

impl Render for Bustop {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let titlebar = self.titlebar();
        let content = self.content(cx);
        let total_height = window.bounds().size.height;
        let content_height = total_height - TITLE_BAR_HEIGHT;
        let content_container = div().w_full().h(content_height).child(content);
        let notification_list = Root::render_notification_layer(window, cx);
        let theme = cx.theme();

        div()
            .track_focus(&self.focus_handle)
            .w_full()
            .h_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(titlebar)
            .child(content_container)
            .children(notification_list)
    }
}
