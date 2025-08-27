use gpui::Window;
use gpui_component::Theme;

pub fn sync_with_system(window: &mut Window) {
    window
        .observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        })
        .detach();
}
