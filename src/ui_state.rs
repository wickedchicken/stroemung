use crate::visualization::ColorType;
#[derive(Debug)]
pub struct UiState {
    pub keep_running: bool,
    pub run: bool,
    pub color_type: ColorType,
    pub speed_multiplier: i32,
}

pub fn initialize_state() -> UiState {
    UiState {
        keep_running: false,
        run: false,
        color_type: ColorType::Speed,
        speed_multiplier: 20,
    }
}
