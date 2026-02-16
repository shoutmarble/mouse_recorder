use iced::widget::{
    button, checkbox, column, container, image, mouse_area, pick_list, radio, row, scrollable, stack, text,
    slider, svg, text_input, toggler, tooltip,
};
use iced::widget::tooltip::Position as TooltipPosition;
use iced::{alignment, Background, Border, Color, Element, Length, Shadow, Subscription, Task};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod model;
mod platform;
mod storage;
mod view;
mod controller;
mod playback;
mod formatting;
mod message;
mod state;

use model::{ClickEdgeMode, ClickListMeta, ClickTarget, MouseButton, RecordedEvent, RecordedEventKind, SearchAnchor};
use message::{FileOpResult, Message};
use state::{FindTargetDraft, Mode, RecorderState, WaitDraft};
use formatting::format_event_with_prev;
use playback::playback;
use platform::{
    arm_get_capture_hook, capture_patch_png_base64, disarm_get_capture_hook, ensure_get_capture_hook_thread,
    get_mouse_pos, is_vk_down_windows, jump_mouse_to, take_get_capture_hook_result, VK_ESCAPE, VK_LBUTTON, VK_MBUTTON,
    VK_RBUTTON,
};
use storage::{load_events_from_file, save_events_to_file};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .scale_factor(App::ui_scale_factor)
        .window(iced::window::Settings {
            // Roughly: 1/2 width and 2/3 height of a typical 1024x768 default.
            size: iced::Size::new(1040.0, 1162.0),
            ..Default::default()
        })
        .subscription(App::subscription)
        .run()
}

struct App {
    ui_scale_percent: u16,

    mode: Mode,
    events: Vec<RecordedEvent>,
    status: String,

    file_path: String,
    current_pos: Option<(i32, i32)>,
    current_left_down: bool,
    current_right_down: bool,
    current_middle_down: bool,
    esc_was_down: bool,

    recorder_wait_ms: u64,
    recorder_mouse_path_enabled: bool,
    recorder_mouse_path_min_delta_px: u16,

    find_image_patch_size: u32,
    find_image_region_size: u32,

    find_target_modal: Option<FindTargetDraft>,
    wait_modal: Option<WaitDraft>,

    editor_x_text: String,
    editor_y_text: String,
    editor_wait_ms: u16,
    editor_click_speed_ms: u16,
    editor_mouse_move_speed_ms: u16,
    editor_click_split_px: u16,
    editor_click_max_hold_ms: u16,
    editor_target_precision_percent: u16,
    editor_target_timeout_ms: u16,
    editor_click_target: ClickTarget,
    editor_left_mode: ClickEdgeMode,
    editor_right_mode: ClickEdgeMode,
    editor_middle_mode: ClickEdgeMode,
    editor_use_find_image: bool,
    editor_static_preview_patch_b64: Option<String>,
    editor_capture_armed: bool,
    editor_last_capture_button: Option<&'static str>,
    editor_last_preview_at: Option<Instant>,
    editor_last_preview_pos: Option<(i32, i32)>,

    selected_index: Option<usize>,
    selected_wait_ms_text: String,
    selected_precision_text: String,
    selected_timeout_ms_text: String,
    selected_limit_region: bool,
    selected_region_size_text: String,
    selected_anchor: SearchAnchor,

    thumb_cache: RefCell<HashMap<u64, iced::widget::image::Handle>>,
    preview_cache: RefCell<HashMap<u64, iced::widget::image::Handle>>,
    events_scroll_id: iced::widget::Id,

    playback_cancel: Option<Arc<AtomicBool>>,
    playback_progress: Option<Arc<AtomicUsize>>,
    playback_active_index: Option<usize>,
    playback_last_scrolled_index: Option<usize>,
    playback_progress_row_map: Vec<usize>,

    window_height_px: f32,

    // Shared recorder state for the background poller
    recorder_state: Arc<Mutex<RecorderState>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ui_scale_percent: 50,
            mode: Mode::Idle,
            events: Vec::new(),
            status: "Ready".to_string(),
            file_path: "recording.yaml".to_string(),
            current_pos: None,
            current_left_down: false,
            current_right_down: false,
            current_middle_down: false,
            esc_was_down: false,
            recorder_wait_ms: 10,
            recorder_mouse_path_enabled: false,
            recorder_mouse_path_min_delta_px: 0,

            find_image_patch_size: 64,
            find_image_region_size: 600,

            find_target_modal: None,
            wait_modal: None,

            editor_x_text: "0".to_string(),
            editor_y_text: "0".to_string(),
            editor_wait_ms: 20,
            editor_click_speed_ms: 20,
            editor_mouse_move_speed_ms: 20,
            editor_click_split_px: 10,
            editor_click_max_hold_ms: 50,
            editor_target_precision_percent: 90,
            editor_target_timeout_ms: 2000,
            editor_click_target: ClickTarget::Left,
            editor_left_mode: ClickEdgeMode::Auto,
            editor_right_mode: ClickEdgeMode::Auto,
            editor_middle_mode: ClickEdgeMode::Auto,
            editor_use_find_image: false,
            editor_static_preview_patch_b64: None,
            editor_capture_armed: false,
            editor_last_capture_button: None,
            editor_last_preview_at: None,
            editor_last_preview_pos: None,

            selected_index: None,
            selected_wait_ms_text: "1000".to_string(),
            selected_precision_text: "0.92".to_string(),
            selected_timeout_ms_text: "2000".to_string(),
            selected_limit_region: true,
            selected_region_size_text: "600".to_string(),
            selected_anchor: SearchAnchor::RecordedClick,
            thumb_cache: RefCell::new(HashMap::new()),
            preview_cache: RefCell::new(HashMap::new()),
            events_scroll_id: iced::widget::Id::new("events-list"),
            playback_cancel: None,
            playback_progress: None,
            playback_active_index: None,
            playback_last_scrolled_index: None,
            playback_progress_row_map: Vec::new(),
            window_height_px: 1162.0,
            recorder_state: Arc::new(Mutex::new(RecorderState::default())),
        }
    }
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn title(&self) -> String {
        "rustautogui - Mouse Recorder (GUI)".to_string()
    }

    fn ui_scale_factor(&self) -> f32 {
        (self.ui_scale_percent as f32 / 100.0).clamp(0.25, 1.0)
    }

    fn active_editor_mode(&self) -> ClickEdgeMode {
        match self.editor_click_target {
            ClickTarget::Left => self.editor_left_mode,
            ClickTarget::Right => self.editor_right_mode,
            ClickTarget::Middle => self.editor_middle_mode,
        }
    }

    fn default_mouse_move_speed_for_mode(mode: ClickEdgeMode) -> u16 {
        let _ = mode;
        20
    }

    fn sync_editor_mouse_move_speed_default(&mut self) {
        self.editor_mouse_move_speed_ms =
            Self::default_mouse_move_speed_for_mode(self.active_editor_mode());
    }

    fn estimated_visible_event_rows(&self) -> usize {
        // Approximate based on current fixed window layout in window logical units.
        // Resize events provide logical size, so do not rescale by ui_scale_factor here.
        // Keep a conservative estimate to avoid under-scrolling the active row.
        let window_h_logical = self.window_height_px;
        let reserved_h_logical = 245.0;
        let row_h_logical = 60.0;

        let rows = ((window_h_logical - reserved_h_logical) / row_h_logical).floor();
        rows.max(3.0) as usize
    }
}


