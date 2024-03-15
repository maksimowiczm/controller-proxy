#[repr(C)]
#[derive(Debug, Default)]
pub struct ControllerState {
    pub left_thumb: i16,
    pub right_thumb: i16,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct ArduinoControllerState {
    pub left_thumb: i8,
    pub right_thumb: i8,
}

impl ArduinoControllerState {
    pub fn from_controller_state(state: &ControllerState) -> Self {
        let left_thumb =
            i8::try_from(state.left_thumb.checked_mul(128).unwrap_or(0) / 1256i16).unwrap_or(0);
        let right_thumb =
            i8::try_from(state.right_thumb.checked_mul(128).unwrap_or(0) / 1256i16).unwrap_or(0);

        ArduinoControllerState {
            left_thumb,
            right_thumb,
        }
    }
}
