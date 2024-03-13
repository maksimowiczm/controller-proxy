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
        let left_thumb = i8::try_from(state.left_thumb * 128 / 256i16).unwrap();
        let right_thumb = i8::try_from(state.right_thumb * 128 / 256i16).unwrap();

        ArduinoControllerState {
            left_thumb,
            right_thumb,
        }
    }
}
