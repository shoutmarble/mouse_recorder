use super::*;

impl App {
    pub(super) fn handle_runtime_message(&mut self, message: Message) -> Result<Task<Message>, Message> {
        match message {
            Message::UiScaleChanged(percent) => {
                let clamped = percent.clamp(25, 100);
                self.ui_scale_percent = if clamped < 38 {
                    25
                } else if clamped < 63 {
                    50
                } else if clamped < 88 {
                    75
                } else {
                    100
                };
                Ok(Task::none())
            }
            Message::WindowResized(w, h) => {
                if w.is_finite() {
                    self.window_width_px = w.max(640.0);
                }
                if h.is_finite() {
                    self.window_height_px = h.max(240.0);
                }
                Ok(Task::none())
            }
            Message::StartRecording => {
                if self.mode == Mode::Playing {
                    return Ok(Task::none());
                }

                self.recorder_wait_ms = self.editor_wait_ms as u64;

                self.mode = Mode::Recording;
                self.events.clear();
                self.status = "Recording...".to_string();

                if let Ok(mut state) = self.recorder_state.lock() {
                    state.enabled = true;
                    state.left_down = false;
                    state.right_down = false;
                    state.middle_down = false;
                    state.left_down_pos = None;
                    state.right_down_pos = None;
                    state.middle_down_pos = None;
                    state.left_down_at = None;
                    state.right_down_at = None;
                    state.middle_down_at = None;
                    state.started_by_click = false;
                    state.last_click_pos = None;
                    state.synthetic_time_ms = 0;
                    state.left_pending_click = None;
                    state.right_pending_click = None;
                    state.middle_pending_click = None;

                    if let Some((x, y)) = get_mouse_pos() {
                        state.last_x = x;
                        state.last_y = y;
                    }
                }

                Ok(Task::none())
            }
            Message::StopRecording => {
                if self.mode == Mode::Playing {
                    if let Some(token) = &self.playback_cancel {
                        token.store(true, Ordering::Relaxed);
                    }
                    self.playback_cancel = None;
                    self.playback_progress = None;
                    self.playback_active_index = None;
                    self.playback_last_scrolled_index = None;
                    self.playback_progress_row_map.clear();
                    self.mode = Mode::Idle;
                    self.status = "Playback stopped.".to_string();
                } else if self.mode == Mode::Recording {
                    let mut pushed = Vec::new();
                    if let Ok(mut state) = self.recorder_state.lock() {
                        self.flush_all_pending_clicks(&mut state, &mut pushed);
                        state.enabled = false;
                    }

                    self.append_recorded_events_compacting_moves(pushed);
                    self.mode = Mode::Idle;
                    self.status = format!("Stopped. {} events recorded.", self.events.len());
                }

                Ok(Task::none())
            }
            Message::StartPlayback => {
                if self.mode == Mode::Recording || self.events.is_empty() {
                    return Ok(Task::none());
                }

                self.mode = Mode::Playing;
                self.status = "Playing (materialized MOVES)...".to_string();
                self.playback_active_index = None;
                self.playback_last_scrolled_index = None;

                let cancel = Arc::new(AtomicBool::new(false));
                self.playback_cancel = Some(cancel.clone());

                let progress = Arc::new(AtomicUsize::new(usize::MAX));
                self.playback_progress = Some(progress.clone());

                let (events, row_map) = self.materialize_moves_grouped_events_with_row_map();
                self.playback_progress_row_map = row_map;
                Ok(Task::perform(
                    async move { playback(events, cancel, progress).map_err(|e| e.to_string()) },
                    Message::PlaybackFinished,
                ))
            }
            Message::PlaybackFinished(result) => {
                self.mode = Mode::Idle;
                self.playback_cancel = None;
                self.playback_progress = None;
                self.playback_active_index = None;
                self.playback_last_scrolled_index = None;
                self.playback_progress_row_map.clear();
                match result {
                    Ok(()) => self.status = "Playback finished.".to_string(),
                    Err(err) => self.status = format!("Playback failed: {err}"),
                }
                Ok(Task::none())
            }
            Message::Clear => {
                if self.mode == Mode::Playing {
                    return Ok(Task::none());
                }
                self.events.clear();
                self.status = "Cleared all events.".to_string();
                Ok(Task::none())
            }
            Message::SetMousePathEnabled(enabled) => {
                self.recorder_mouse_path_enabled = enabled;
                self.status = if self.recorder_mouse_path_enabled {
                    "Mouse path recording: ON".to_string()
                } else {
                    "Mouse path recording: OFF".to_string()
                };
                Ok(Task::none())
            }
            Message::Tick => {
                if self.mode != Mode::Recording {
                    return Ok(Task::none());
                }

                let mut pushed = Vec::new();
                if let Ok(mut state) = self.recorder_state.lock() {
                    if !state.enabled {
                        return Ok(Task::none());
                    }

                    self.flush_expired_pending_clicks(&mut state, &mut pushed);

                    let previous_pos = (state.last_x, state.last_y);
                    let mut current_pos = previous_pos;
                    if let Some((x, y)) = get_mouse_pos() {
                        current_pos = (x, y);
                    }

                    #[cfg(windows)]
                    {
                        let left_now = is_vk_down_windows(VK_LBUTTON);
                        let right_now = is_vk_down_windows(VK_RBUTTON);
                        let middle_now = is_vk_down_windows(VK_MBUTTON);

                        if left_now && !state.left_down {
                            state.left_down = true;
                            state.left_down_pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            state.left_down_at = Some(Instant::now());
                            if self.recorder_mouse_path_enabled {
                                let down_pos = state.left_down_pos;
                                self.push_recorded_button_down_for_path(
                                    &mut state,
                                    &mut pushed,
                                    down_pos,
                                    MouseButton::Left,
                                );
                            }
                        } else if !left_now && state.left_down {
                            state.left_down = false;
                            let down_pos = state.left_down_pos.take().or(Some((state.last_x, state.last_y)));
                            let up_pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            let held_ms = state
                                .left_down_at
                                .take()
                                .map(|t| t.elapsed().as_millis() as u64)
                                .unwrap_or(0);
                            if self.recorder_mouse_path_enabled {
                                self.push_recorded_button_up_for_path(
                                    &mut state,
                                    &mut pushed,
                                    up_pos,
                                    MouseButton::Left,
                                );
                            } else {
                                self.push_recorded_button_release(
                                    &mut state,
                                    &mut pushed,
                                    down_pos,
                                    up_pos,
                                    held_ms,
                                    MouseButton::Left,
                                );
                            }
                        }

                        if right_now && !state.right_down {
                            state.right_down = true;
                            state.right_down_pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            state.right_down_at = Some(Instant::now());
                            if self.recorder_mouse_path_enabled {
                                let down_pos = state.right_down_pos;
                                self.push_recorded_button_down_for_path(
                                    &mut state,
                                    &mut pushed,
                                    down_pos,
                                    MouseButton::Right,
                                );
                            }
                        } else if !right_now && state.right_down {
                            state.right_down = false;
                            let down_pos = state.right_down_pos.take().or(Some((state.last_x, state.last_y)));
                            let up_pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            let held_ms = state
                                .right_down_at
                                .take()
                                .map(|t| t.elapsed().as_millis() as u64)
                                .unwrap_or(0);
                            if self.recorder_mouse_path_enabled {
                                self.push_recorded_button_up_for_path(
                                    &mut state,
                                    &mut pushed,
                                    up_pos,
                                    MouseButton::Right,
                                );
                            } else {
                                self.push_recorded_button_release(
                                    &mut state,
                                    &mut pushed,
                                    down_pos,
                                    up_pos,
                                    held_ms,
                                    MouseButton::Right,
                                );
                            }
                        }

                        if middle_now && !state.middle_down {
                            state.middle_down = true;
                            state.middle_down_pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            state.middle_down_at = Some(Instant::now());
                            if self.recorder_mouse_path_enabled {
                                let down_pos = state.middle_down_pos;
                                self.push_recorded_button_down_for_path(
                                    &mut state,
                                    &mut pushed,
                                    down_pos,
                                    MouseButton::Middle,
                                );
                            }
                        } else if !middle_now && state.middle_down {
                            state.middle_down = false;
                            let down_pos = state.middle_down_pos.take().or(Some((state.last_x, state.last_y)));
                            let up_pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            let held_ms = state
                                .middle_down_at
                                .take()
                                .map(|t| t.elapsed().as_millis() as u64)
                                .unwrap_or(0);
                            if self.recorder_mouse_path_enabled {
                                self.push_recorded_button_up_for_path(
                                    &mut state,
                                    &mut pushed,
                                    up_pos,
                                    MouseButton::Middle,
                                );
                            } else {
                                self.push_recorded_button_release(
                                    &mut state,
                                    &mut pushed,
                                    down_pos,
                                    up_pos,
                                    held_ms,
                                    MouseButton::Middle,
                                );
                            }
                        }
                    }

                    if self.recorder_mouse_path_enabled {
                        let dx = (current_pos.0 - previous_pos.0).abs();
                        let dy = (current_pos.1 - previous_pos.1).abs();
                        let min_delta = self.recorder_mouse_path_min_delta_px as i32;

                        if dx >= min_delta || dy >= min_delta {
                            state.synthetic_time_ms = state.synthetic_time_ms.saturating_add(1);
                            pushed.push(RecordedEvent {
                                ms_from_start: state.synthetic_time_ms,
                                kind: RecordedEventKind::Move {
                                    x: current_pos.0,
                                    y: current_pos.1,
                                },
                                pos: Some(current_pos),
                                click_meta: None,
                            });
                        }
                    }

                    state.last_x = current_pos.0;
                    state.last_y = current_pos.1;

                    self.flush_expired_pending_clicks(&mut state, &mut pushed);
                }

                let changed = self.append_recorded_events_compacting_moves(pushed);

                if changed {
                    return Ok(iced::widget::operation::snap_to(
                        self.events_scroll_id.clone(),
                        iced::widget::operation::RelativeOffset { x: 0.0, y: 1.0 },
                    ));
                }

                Ok(Task::none())
            }
            Message::PosTick => {
                const LIVE_PREVIEW_DEADZONE_PX: i32 = 2;

                self.current_pos = get_mouse_pos();

                let mut scroll_to_playback_row: Option<usize> = None;

                if self.mode == Mode::Playing {
                    if let Some(progress) = &self.playback_progress {
                        let idx = progress.load(Ordering::Relaxed);
                        let new_active = if idx == usize::MAX {
                            None
                        } else {
                            Some(
                                self.playback_progress_row_map
                                    .get(idx)
                                    .copied()
                                    .unwrap_or(idx),
                            )
                        };

                        if new_active != self.playback_active_index {
                            self.playback_active_index = new_active;
                            if let Some(i) = new_active {
                                let should_snap = self.playback_last_scrolled_index != Some(i);

                                if should_snap {
                                    self.playback_last_scrolled_index = Some(i);
                                    scroll_to_playback_row = Some(i);
                                }
                            }
                        }
                    }
                }

                if self.mode == Mode::Idle {
                    let should_refresh_preview = self
                        .editor_last_preview_at
                        .map(|t| t.elapsed() >= Duration::from_millis(180))
                        .unwrap_or(true);

                    if should_refresh_preview {
                        if let Some((x, y)) = self.current_pos {
                            let moved_enough = match self.editor_last_preview_pos {
                                Some((lx, ly)) => {
                                    let dx = (x - lx).abs();
                                    let dy = (y - ly).abs();
                                    dx >= LIVE_PREVIEW_DEADZONE_PX || dy >= LIVE_PREVIEW_DEADZONE_PX
                                }
                                None => true,
                            };

                            if moved_enough {
                                if self.editor_capture_armed {
                                    self.editor_x_text = x.to_string();
                                    self.editor_y_text = y.to_string();

                                    if let Ok(new_patch) = capture_patch_png_base64(x, y, 128) {
                                        let patch_changed = self
                                            .editor_static_preview_patch_b64
                                            .as_deref()
                                            != Some(new_patch.as_str());
                                        if patch_changed {
                                            self.editor_static_preview_patch_b64 = Some(new_patch);
                                        }
                                    }
                                }

                                self.editor_last_preview_pos = Some((x, y));
                            }
                        } else {
                            self.editor_last_preview_pos = None;
                        }
                        self.editor_last_preview_at = Some(Instant::now());
                    }
                }

                #[cfg(windows)]
                {
                    self.current_left_down = is_vk_down_windows(VK_LBUTTON);
                    self.current_right_down = is_vk_down_windows(VK_RBUTTON);
                    self.current_middle_down = is_vk_down_windows(VK_MBUTTON);

                    if self.editor_capture_armed {
                        if let Some((x, y, button_name)) = take_get_capture_hook_result() {
                            self.editor_x_text = x.to_string();
                            self.editor_y_text = y.to_string();
                            self.editor_static_preview_patch_b64 = capture_patch_png_base64(x, y, 128).ok();
                            self.editor_capture_armed = false;
                            self.editor_last_capture_button = Some(button_name);
                            self.status = format!("Captured {button_name} click at ({x}, {y})");
                        }
                    }

                    let esc_now = is_vk_down_windows(VK_ESCAPE);
                    if esc_now && !self.esc_was_down {
                        if self.editor_capture_armed {
                            disarm_get_capture_hook();
                            self.editor_capture_armed = false;
                            self.status = "GET (X,Y) canceled.".to_string();
                            return Ok(Task::none());
                        }

                        match self.mode {
                            Mode::Recording | Mode::Playing => {
                                return Ok(self.update(Message::StopRecording));
                            }
                            Mode::Idle => {}
                        }
                    }
                    self.esc_was_down = esc_now;
                }

                if let Some(active_index) = scroll_to_playback_row {
                    let len = self.events.len();
                    if len > 0 {
                        let clamped = active_index.min(len.saturating_sub(1));
                        let visible_rows = self.estimated_visible_event_rows();

                        let y = if len <= visible_rows {
                            0.0
                        } else {
                            let half = visible_rows / 2;
                            let max_top = len.saturating_sub(visible_rows);
                            let top_index = clamped.saturating_sub(half).min(max_top);
                            top_index as f32 / max_top as f32
                        };

                        return Ok(iced::widget::operation::snap_to(
                            self.events_scroll_id.clone(),
                            iced::widget::operation::RelativeOffset { x: 0.0, y },
                        ));
                    }
                }

                Ok(Task::none())
            }
            Message::FilePathChanged(path) => {
                self.file_path = path;
                Ok(Task::none())
            }
            Message::SaveToFile => {
                let path = self.file_path.trim().to_string();
                if path.is_empty() {
                    self.status = "Please provide a file path.".to_string();
                    return Ok(Task::none());
                }
                let events = self.materialize_moves_grouped_events();

                self.status = format!("Saving to {path} (materializing MOVES from MOVE samples)...");
                Ok(Task::perform(
                    async move {
                        save_events_to_file(&path, &events).map(FileOpResult::Saved)
                    },
                    Message::FileOpFinished,
                ))
            }
            Message::LoadFromFile => {
                if self.mode == Mode::Recording || self.mode == Mode::Playing {
                    self.status = "Stop recording or playback before loading.".to_string();
                    return Ok(Task::none());
                }

                let path = self.file_path.trim().to_string();
                if path.is_empty() {
                    self.status = "Please provide a file path.".to_string();
                    return Ok(Task::none());
                }
                self.status = format!("Loading from {path}...");
                Ok(Task::perform(
                    async move { load_events_from_file(&path).map(FileOpResult::Loaded) },
                    Message::FileOpFinished,
                ))
            }
            Message::FileOpFinished(result) => {
                match result {
                    Ok(FileOpResult::Saved(msg)) => {
                        self.status = msg;
                    }
                    Ok(FileOpResult::Loaded(events)) => {
                        let count = events.len();
                        self.events = events;
                        self.status = format!("Loaded {count} events.");
                    }
                    Err(err) => self.status = err,
                }
                Ok(Task::none())
            }
            _ => Err(message),
        }
    }
}
