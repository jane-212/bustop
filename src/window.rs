use gpui::{App, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point, px, size};

pub fn window_options(cx: &mut App) -> WindowOptions {
    let min_and_default_size = size(px(1080.), px(800.0));
    let bounds = Bounds::centered(None, min_and_default_size, cx);
    let window_bounds = WindowBounds::Windowed(bounds);
    let titlebar_options = TitlebarOptions {
        appears_transparent: true,
        traffic_light_position: Some(point(px(9.), px(9.))),
        ..Default::default()
    };

    WindowOptions {
        window_bounds: Some(window_bounds),
        titlebar: Some(titlebar_options),
        app_id: Some("github.jane-212.bustop".into()),
        window_min_size: Some(min_and_default_size),
        ..Default::default()
    }
}
