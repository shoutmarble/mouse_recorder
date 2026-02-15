use crate::model::SearchAnchor;

#[derive(Debug, Clone)]
pub(crate) struct FindTargetDraft {
    pub patch_png_base64: Option<String>,
    pub patch_size_text: String,
    pub patch_size: u32,
    pub precision_text: String,
    pub precision: f32,
    pub timeout_ms_text: String,
    pub timeout_ms: u64,

    pub limit_region: bool,
    pub region_size_text: String,
    pub region_size: u32,

    pub anchor: SearchAnchor,

    // Capture flow
    pub capture_waiting: bool,
    pub capture_left_was_down: bool,
    pub captured_pos: Option<(i32, i32)>,

    // Load-from-path flow
    pub image_path: String,

    pub status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct WaitDraft {
    pub wait_ms_text: String,
    pub wait_ms: u64,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Idle,
    Recording,
    Playing,
}

#[derive(Debug)]
pub(crate) struct RecorderState {
    pub enabled: bool,

    pub last_x: i32,
    pub last_y: i32,

    pub left_down: bool,
    pub right_down: bool,
    pub middle_down: bool,

    pub started_by_click: bool,
    pub last_click_pos: Option<(i32, i32)>,
    pub synthetic_time_ms: u128,
}

impl Default for RecorderState {
    fn default() -> Self {
        Self {
            enabled: false,
            last_x: 0,
            last_y: 0,
            left_down: false,
            right_down: false,
            middle_down: false,
            started_by_click: false,
            last_click_pos: None,
            synthetic_time_ms: 0,
        }
    }
}
