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
            Message::StartRecording => {
                if self.mode == Mode::Playing {
                    return Ok(Task::none());
                }

                self.mode = Mode::Recording;
                self.events.clear();
                self.status = "Recording...".to_string();

                if let Ok(mut state) = self.recorder_state.lock() {
                    state.enabled = true;
                    state.left_down = false;
                    state.right_down = false;
                    state.middle_down = false;
                    state.started_by_click = false;
                    state.last_click_pos = None;
                    state.synthetic_time_ms = 0;

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
                    self.mode = Mode::Idle;
                    self.status = "Playback stopped.".to_string();
                } else if self.mode == Mode::Recording {
                    self.mode = Mode::Idle;
                    self.status = format!("Stopped. {} events recorded.", self.events.len());

                    if let Ok(mut state) = self.recorder_state.lock() {
                        state.enabled = false;
                    }
                }

                Ok(Task::none())
            }
            Message::StartPlayback => {
                if self.mode == Mode::Recording || self.events.is_empty() {
                    return Ok(Task::none());
                }

                self.mode = Mode::Playing;
                self.status = "Playing...".to_string();
                self.playback_active_index = None;
                self.playback_last_scrolled_index = None;

                let cancel = Arc::new(AtomicBool::new(false));
                self.playback_cancel = Some(cancel.clone());

                let progress = Arc::new(AtomicUsize::new(usize::MAX));
                self.playback_progress = Some(progress.clone());

                let events = self.events.clone();
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
                match result {
                    Ok(()) => self.status = "Playback finished".to_string(),
                    Err(err) => self.status = format!("Playback error: {err}"),
                }
                Ok(Task::none())
            }
            Message::Clear => {
                if self.mode == Mode::Playing {
                    return Ok(Task::none());
                }
                self.events.clear();
                self.status = "Cleared".to_string();
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

                    if let Some((x, y)) = get_mouse_pos() {
                        state.last_x = x;
                        state.last_y = y;
                    }

                    #[cfg(windows)]
                    {
                        let left_now = is_vk_down_windows(VK_LBUTTON);
                        let right_now = is_vk_down_windows(VK_RBUTTON);
                        let middle_now = is_vk_down_windows(VK_MBUTTON);

                        if left_now && !state.left_down {
                            state.left_down = true;
                        } else if !left_now && state.left_down {
                            state.left_down = false;
                            let pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            self.push_recorded_button_release(
                                &mut state,
                                &mut pushed,
                                pos,
                                MouseButton::Left,
                            );
                        }

                        if right_now && !state.right_down {
                            state.right_down = true;
                        } else if !right_now && state.right_down {
                            state.right_down = false;
                            let pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            self.push_recorded_button_release(
                                &mut state,
                                &mut pushed,
                                pos,
                                MouseButton::Right,
                            );
                        }

                        if middle_now && !state.middle_down {
                            state.middle_down = true;
                        } else if !middle_now && state.middle_down {
                            state.middle_down = false;
                            let pos = get_mouse_pos().or(Some((state.last_x, state.last_y)));
                            self.push_recorded_button_release(
                                &mut state,
                                &mut pushed,
                                pos,
                                MouseButton::Middle,
                            );
                        }
                    }
                }

                self.events.extend(pushed);
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
                            Some(idx)
                        };

                        if new_active != self.playback_active_index {
                            self.playback_active_index = new_active;
                            if let Some(i) = new_active {
                                let should_snap = match self.playback_last_scrolled_index {
                                    None => true,
                                    Some(prev) => i.abs_diff(prev) >= 3,
                                };

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
                            self.status = "GET (X,Y) cancelled".to_string();
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
                        let normalized = (clamped as f32 + 0.5) / len as f32;
                        let y = (normalized - 0.35).clamp(0.0, 1.0);
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
                    self.status = "Provide a file path".to_string();
                    return Ok(Task::none());
                }
                let events = self.events.clone();
                self.status = format!("Saving to {path}...");
                Ok(Task::perform(
                    async move {
                        save_events_to_file(&path, &events).map(FileOpResult::Saved)
                    },
                    Message::FileOpFinished,
                ))
            }
            Message::LoadFromFile => {
                if self.mode == Mode::Recording || self.mode == Mode::Playing {
                    self.status = "Stop recording/playback before loading".to_string();
                    return Ok(Task::none());
                }

                let path = self.file_path.trim().to_string();
                if path.is_empty() {
                    self.status = "Provide a file path".to_string();
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
                        self.status = format!("Loaded {count} events");
                    }
                    Err(err) => self.status = err,
                }
                Ok(Task::none())
            }
            _ => Err(message),
        }
    }
}
