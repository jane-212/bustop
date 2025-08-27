use gpui::{
    AnyElement, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement as _, Pixels,
    Render, Styled as _, Window, div, px,
};
use gpui_component::{
    ActiveTheme as _, Selectable as _,
    button::{Button, ButtonVariants as _},
};
use strum::IntoStaticStr;

use super::icon::IconName;

const LEFT_BAR_WIDTH: Pixels = px(50.);

pub struct LeftBar {
    selected_item: LeftBarItem,
    focus_handle: FocusHandle,
}

impl LeftBar {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            selected_item: LeftBarItem::Forum,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn selected_item(&self) -> &LeftBarItem {
        &self.selected_item
    }

    fn render_item(&self, item: &LeftBarItem, cx: &mut Context<Self>) -> impl IntoElement {
        let id: &'static str = item.into();
        let button = Button::new(id)
            .icon(item.icon())
            .ghost()
            .cursor_pointer()
            .selected(&self.selected_item == item)
            .on_click(cx.listener({
                let item = item.clone();
                move |left_bar, _, _, cx| left_bar.on_click(item.clone(), cx)
            }));

        div()
            .w(LEFT_BAR_WIDTH)
            .mt_2()
            .flex()
            .justify_center()
            .items_center()
            .child(button)
    }

    fn on_click(&mut self, item: LeftBarItem, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.selected_item = item;
    }
}

impl Render for LeftBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let all_items = LeftBarItem::all_items()
            .iter()
            .map(|item| self.render_item(item, cx).into_any_element())
            .collect::<Vec<AnyElement>>();
        let theme = cx.theme();

        div()
            .track_focus(&self.focus_handle)
            .h_full()
            .w(LEFT_BAR_WIDTH)
            .border_r_1()
            .border_color(theme.border)
            .children(all_items)
    }
}

#[derive(PartialEq, Eq, IntoStaticStr, Clone)]
pub enum LeftBarItem {
    Forum,
    Find,
}

impl LeftBarItem {
    fn all_items() -> &'static [Self] {
        &[Self::Forum, Self::Find]
    }

    fn icon(&self) -> IconName {
        match self {
            LeftBarItem::Forum => IconName::House,
            LeftBarItem::Find => IconName::BookMarked,
        }
    }
}
