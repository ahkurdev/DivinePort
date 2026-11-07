pub mod library_tab;
pub mod controls_tab;
pub mod graphics_tab;
pub mod settings_tab;
pub mod controller_canvas;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Library,
    Controls,
    Graphics,
    Settings,
    Logs,
}
