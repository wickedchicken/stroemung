use crate::visualization::ColorType;

#[derive(Debug)]
pub enum MouseState {
    Inspection,
    Boundary,
    Fluid,
}

#[derive(Debug)]
pub struct UiState {
    pub keep_running: bool,
    pub run: bool,
    pub reset: bool,
    pub color_type: ColorType,
    pub speed_multiplier: i32,
    pub mouse_state: MouseState,
}

pub fn initialize_state() -> UiState {
    UiState {
        keep_running: true,
        run: false,
        reset: false,
        color_type: ColorType::Speed,
        speed_multiplier: 20,
        mouse_state: MouseState::Boundary,
    }
}
