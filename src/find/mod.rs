mod detail;
mod summary;

use detail::{Detail, DetailEvent};
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement as _, Render, Styled as _, Window, div,
};
use summary::{Summary, SummaryEvent};

pub struct Find {
    summary: Entity<Summary>,
    detail: Entity<Detail>,
    focus_handle: FocusHandle,
}

impl Find {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let summary = cx.new(|cx| Summary::new(window, cx));
        summary.update(cx, |_, cx| {
            cx.emit(SummaryEvent::Load(1));
            cx.notify();
        });
        cx.subscribe(&summary, |this, _, event, cx| match event {
            SummaryEvent::LoadDetail(detail_url) => {
                this.detail.update(cx, |_, cx| {
                    cx.emit(DetailEvent::Load(detail_url.clone(), 1));
                    cx.notify();
                });
            }
            _ => {}
        })
        .detach();
        let detail = cx.new(|cx| Detail::new(window, cx));

        Self {
            summary,
            detail,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for Find {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .w_full()
            .h_full()
            .flex()
            .child(self.summary.clone())
            .child(self.detail.clone())
    }
}
