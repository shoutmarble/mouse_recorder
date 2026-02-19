use crate::model::{ClickEdgeMode, ClickTarget, RecordedEvent, SearchAnchor};

#[derive(Debug, Clone)]
pub(crate) enum FileOpResult {
    Saved(String),
    Loaded(Vec<RecordedEvent>),
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    UiScaleChanged(u16),

    StartRecording,
    StopRecording,
    StartPlayback,
    Clear,
    SetMousePathEnabled(bool),
    MousePathMinDeltaPxChanged(u16),
    WindowResized(f32, f32),
    Tick,
    PlaybackFinished(Result<(), String>),
    PosTick,

    FilePathChanged(String),
    SaveToFile,
    LoadFromFile,
    FileOpFinished(Result<FileOpResult, String>),

    CloseModal,
    FindTargetOk,

    WaitOk,
    WaitMsChanged(String),

    FindTargetPatchSizeChanged(String),
    FindTargetPrecisionChanged(String),
    FindTargetTimeoutChanged(String),
    FindTargetLimitRegionToggled(bool),
    FindTargetRegionSizeChanged(String),
    FindTargetAnchorSelected(SearchAnchor),

    FindTargetStartCapture,
    FindTargetCaptureTick,

    FindTargetPathChanged(String),
    FindTargetLoadFromPath,

    EditorLeftModeSelected(ClickEdgeMode),
    EditorRightModeSelected(ClickEdgeMode),
    EditorMiddleModeSelected(ClickEdgeMode),
    EditorClickTargetSelected(ClickTarget),
    EditorWaitMsChanged(u16),
    EditorClickSpeedMsChanged(u16),
    EditorMouseMoveSpeedMsChanged(u16),
    EditorClickSplitPxChanged(u16),
    EditorClickMaxHoldMsChanged(u16),
    EditorTargetPrecisionChanged(u16),
    EditorTargetTimeoutMsChanged(u16),
    EditorUseFindImageToggled(bool),
    EditorStartGetXY,
    EditorJumpToXY,
    EditorInsertOrApply,
    EditorInsertBelowSelected,

    RowJump(usize),
    RowClone(usize),
    RowDelete(usize),

    SelectRow(usize),
    ClearSelection,

}
