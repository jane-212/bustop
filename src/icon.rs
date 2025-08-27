use gpui::SharedString;
use gpui_component::{Icon, IconNamed};
use strum::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum IconName {
    House,
    BookMarked,
    LoaderCircle,
    Reply,
    MessageCircle,
    ChevronLeft,
    ChevronRight,
    Plus,
    Minus,
    Eye,
}

impl IconNamed for IconName {
    fn path(&self) -> SharedString {
        let file_stem: &'static str = self.into();
        format!("icons/{file_stem}.svg").into()
    }
}

impl From<IconName> for Icon {
    fn from(val: IconName) -> Self {
        Icon::default().path(val.path())
    }
}
