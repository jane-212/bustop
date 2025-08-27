use bustop::assets::Assets;
use bustop::{Bustop, http_client, theme, window};
use gpui::{App, AppContext as _, Application};
use gpui_component::theme as gpui_theme;
use gpui_component::{Root, input};

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_theme::init(cx);
        http_client::init(cx);
        input::init(cx);

        let window_options = window::window_options(cx);
        cx.open_window(window_options, |window, cx| {
            theme::sync_with_system(window);
            let bustop = cx.new(|cx| Bustop::new(window, cx));

            cx.new(|cx| Root::new(bustop.into(), window, cx))
        })
        .expect("failed to open window");

        cx.on_window_closed(|cx| {
            cx.quit();
        })
        .detach();

        cx.activate(true);
    });
}
