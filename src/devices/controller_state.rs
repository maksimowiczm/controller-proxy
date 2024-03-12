#[repr(C)]
#[derive(Debug, Default)]
pub struct ControllerState {
    pub left_thumb: i16,
    pub right_thumb: i16,
}
