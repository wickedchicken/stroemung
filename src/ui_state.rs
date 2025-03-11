use crate::visualization::ColorType;

use strum_macros::EnumString;

use thiserror::Error;

#[derive(Debug)]
pub enum MouseState {
    Inspection,
    Boundary,
    Fluid,
}

#[derive(Error, Debug)]
pub enum PresetError {
    #[error("An error occurred while parsing the Preset enum: `{0}`")]
    PresetParsingError(String),
}

#[derive(Debug, Copy, Clone, PartialEq, EnumString, strum_macros::VariantNames)]
pub enum Preset {
    Obstacle,
    #[strum(serialize = "Empty")]
    Inflow,
}

impl TryFrom<usize> for Preset {
    type Error = PresetError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Preset::Obstacle),
            1 => Ok(Preset::Inflow),
            _ => Err(PresetError::PresetParsingError(format!(
                "{:?} does not match to a known Preset",
                value
            ))),
        }
    }
}

#[derive(Debug)]
pub struct UiState {
    pub keep_running: bool,
    pub run: bool,
    pub reset: bool,
    pub color_type: ColorType,
    pub speed_multiplier: i32,
    pub mouse_state: MouseState,
    pub preset: Preset,
}

pub fn initialize_state() -> UiState {
    UiState {
        keep_running: true,
        run: false,
        reset: false,
        color_type: ColorType::Speed,
        speed_multiplier: 20,
        mouse_state: MouseState::Boundary,
        preset: Preset::Obstacle,
    }
}
