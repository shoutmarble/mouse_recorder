#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClickEdgeMode {
    Auto,
    Down,
    Up,
    Double,
}

impl ClickEdgeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "single",
            Self::Down => "down",
            Self::Up => "up",
            Self::Double => "double",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickTarget {
    Left,
    Right,
    Middle,
}

impl ClickTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "Left click",
            Self::Right => "Right click",
            Self::Middle => "Middle click",
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SearchAnchor {
    RecordedClick,
    CurrentMouse,
    LastFound,
}

impl std::fmt::Display for SearchAnchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

impl SearchAnchor {
    pub fn label(self) -> &'static str {
        match self {
            Self::RecordedClick => "Recorded",
            Self::CurrentMouse => "Mouse",
            Self::LastFound => "Last found",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecordedEventKind {
    Move { x: i32, y: i32 },
    Moves { points: Vec<(i32, i32)> },
    Wait { ms: u64 },
    FindTarget {
        patch_png_base64: String,
        patch_size: u32,
        precision: f32,
        timeout_ms: u64,
        search_anchor: SearchAnchor,
        search_region_size: Option<u32>,
    },
    LeftDown {
        patch_png_base64: Option<String>,
    },
    LeftUp {
        patch_png_base64: Option<String>,
    },
    LeftClick {
        patch_png_base64: Option<String>,
    },
    RightDown {
        patch_png_base64: Option<String>,
    },
    RightUp {
        patch_png_base64: Option<String>,
    },
    RightClick {
        patch_png_base64: Option<String>,
    },
    MiddleDown {
        patch_png_base64: Option<String>,
    },
    MiddleUp {
        patch_png_base64: Option<String>,
    },
    MiddleClick {
        patch_png_base64: Option<String>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedEvent {
    pub ms_from_start: u128,
    pub kind: RecordedEventKind,
    pub pos: Option<(i32, i32)>,
    pub click_meta: Option<ClickListMeta>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClickListMeta {
    pub left_mode: ClickEdgeMode,
    pub right_mode: ClickEdgeMode,
    pub middle_mode: ClickEdgeMode,
    pub wait_ms: u16,
    pub click_speed_ms: u16,
    pub mouse_move_speed_ms: u16,
    pub use_find_image: bool,
    pub target_precision: f32,
    pub target_timeout_ms: u64,
}

impl Default for ClickListMeta {
    fn default() -> Self {
        Self {
            left_mode: ClickEdgeMode::Auto,
            right_mode: ClickEdgeMode::Auto,
            middle_mode: ClickEdgeMode::Auto,
            wait_ms: 20,
            click_speed_ms: 20,
            mouse_move_speed_ms: 150,
            use_find_image: false,
            target_precision: 0.90,
            target_timeout_ms: 2000,
        }
    }
}
