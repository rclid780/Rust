use strum::{Display, EnumIter, FromRepr};

pub enum Event {
    KeyInput(crossterm::event::KeyEvent),
    BackgroundTask(f64),
}

#[derive(PartialEq, Eq)]
pub enum HostState {
    Running,
    ShuttingDown,
    Completed,
}

#[derive(Default, Display, PartialEq, Eq, FromRepr, Clone, Copy, EnumIter)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "Tab 1")]
    Tab1,
    #[strum(to_string = "Tab 2")]
    Tab2,
    #[strum(to_string = "Tab 3")]
    Tab3,
    #[strum(to_string = "Tab 4")]
    Tab4,
}
