use crate::model::{ClickEdgeMode, ClickTarget, RecordedEvent, SearchAnchor};

#[derive(Debug, Clone)]
pub(crate) enum FileOpResult {
    Saved(String),
    Loaded(Vec<RecordedEvent>),
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Noop,
    UiScaleChanged(u16),

    StartRecording,
    StopRecording,
    StartPlayback,
    Clear,
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
    EditorUseFindImageToggled(bool),
    EditorStartGetXY,
    EditorInsertOrApply,
    EditorInsertBelowSelected,

    SelectRow(usize),
    ClearSelection,

}
