use super::*;

impl App {
    pub(super) fn handle_editor_message(&mut self, message: Message) -> Result<Task<Message>, Message> {
        match message {
            Message::EditorLeftModeSelected(mode) => {
                if self.editor_click_target != ClickTarget::Left {
                    return Ok(Task::none());
                }
                self.editor_left_mode = mode;
                self.sync_editor_mouse_move_speed_default();
                Ok(Task::none())
            }
            Message::EditorRightModeSelected(mode) => {
                if self.editor_click_target != ClickTarget::Right {
                    return Ok(Task::none());
                }
                self.editor_right_mode = mode;
                self.sync_editor_mouse_move_speed_default();
                Ok(Task::none())
            }
            Message::EditorMiddleModeSelected(mode) => {
                if self.editor_click_target != ClickTarget::Middle {
                    return Ok(Task::none());
                }
                self.editor_middle_mode = mode;
                self.sync_editor_mouse_move_speed_default();
                Ok(Task::none())
            }
            Message::EditorClickTargetSelected(target) => {
                self.editor_click_target = target;
                self.sync_editor_mouse_move_speed_default();
                Ok(Task::none())
            }
            Message::EditorWaitMsChanged(ms) => {
                self.editor_wait_ms = ms.clamp(0, 300);
                Ok(Task::none())
            }
            Message::EditorClickSpeedMsChanged(ms) => {
                let active_mode = match self.editor_click_target {
                    ClickTarget::Left => self.editor_left_mode,
                    ClickTarget::Right => self.editor_right_mode,
                    ClickTarget::Middle => self.editor_middle_mode,
                };

                if matches!(active_mode, ClickEdgeMode::Up | ClickEdgeMode::Down) {
                    return Ok(Task::none());
                }

                self.editor_click_speed_ms = ms.clamp(0, 100);
                Ok(Task::none())
            }
            Message::EditorMouseMoveSpeedMsChanged(ms) => {
                self.editor_mouse_move_speed_ms = ms.clamp(0, 500);
                Ok(Task::none())
            }
            Message::EditorUseFindImageToggled(enabled) => {
                self.editor_use_find_image = enabled;
                Ok(Task::none())
            }
            Message::EditorStartGetXY => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording/playback first".to_string();
                    return Ok(Task::none());
                }

                if let Some((x, y)) = self.current_pos {
                    self.editor_x_text = x.to_string();
                    self.editor_y_text = y.to_string();
                    self.editor_static_preview_patch_b64 = capture_patch_png_base64(x, y, 128).ok();
                }

                #[cfg(windows)]
                {
                    if let Err(err) = ensure_get_capture_hook_thread() {
                        self.status = format!("GET capture unavailable: {err}");
                        return Ok(Task::none());
                    }

                    arm_get_capture_hook();
                    self.editor_capture_armed = true;
                    self.editor_last_capture_button = None;
                    self.status = "GET (X,Y) armed: click Left/Right/Middle to set (ESC cancels)".to_string();
                }

                #[cfg(not(windows))]
                {
                    self.status = "GET (X,Y) capture is Windows-only right now".to_string();
                }

                Ok(Task::none())
            }
            Message::EditorInsertOrApply => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording/playback first".to_string();
                    return Ok(Task::none());
                }

                let Some((x, y)) = self.parse_editor_xy() else {
                    self.status = "Invalid X/Y".to_string();
                    return Ok(Task::none());
                };

                let patch = if self.editor_use_find_image {
                    self.editor_static_preview_patch_b64
                        .clone()
                        .or_else(|| capture_patch_png_base64(x, y, 128).ok())
                } else {
                    capture_patch_png_base64(x, y, 128).ok()
                };
                let mut kinds = self.event_kinds_from_editor_modes(patch);
                if kinds.is_empty() {
                    self.status = if self.editor_use_find_image {
                        "Capture GET target first".to_string()
                    } else {
                        "Choose at least one click mode".to_string()
                    };
                    return Ok(Task::none());
                }

                let click_meta = Some(self.current_click_list_meta());

                if let Some(index) = self.selected_index {
                    if let Some(ev) = self.events.get(index).cloned() {
                        match ev.kind {
                            RecordedEventKind::Move { .. } => {
                                if let Some(cur) = self.events.get_mut(index) {
                                    cur.kind = RecordedEventKind::Move { x, y };
                                    cur.pos = Some((x, y));
                                }
                                self.status = format!("Updated move at row {}", index);
                                return Ok(Task::none());
                            }
                            _ => {}
                        }
                    }

                    if let Some(cur) = self.events.get_mut(index) {
                        cur.kind = kinds.remove(0);
                        cur.pos = Some((x, y));
                        cur.click_meta = click_meta.clone();
                    }

                    let mut inserted = 0usize;
                    let base_ms = self.events.get(index).map(|e| e.ms_from_start).unwrap_or(0);
                    for (offset, kind) in kinds.into_iter().enumerate() {
                        self.events.insert(
                            index + 1 + offset,
                            RecordedEvent {
                                ms_from_start: base_ms + 1 + offset as u128,
                                kind,
                                pos: Some((x, y)),
                                click_meta: click_meta.clone(),
                            },
                        );
                        inserted += 1;
                    }

                    self.status = if inserted > 0 {
                        format!("Updated selected row and inserted {} extra row(s)", inserted)
                    } else {
                        "Updated selected row".to_string()
                    };
                } else {
                    let insert_at = self.events.len();
                    let prev_ms = self.events.last().map(|e| e.ms_from_start).unwrap_or(0);

                    for (offset, kind) in kinds.into_iter().enumerate() {
                        self.events.push(RecordedEvent {
                            ms_from_start: prev_ms + 1 + offset as u128,
                            kind,
                            pos: Some((x, y)),
                            click_meta: click_meta.clone(),
                        });
                    }
                    self.selected_index = Some(insert_at);
                    self.status = "Inserted click row".to_string();
                }

                Ok(Task::none())
            }
            Message::EditorInsertBelowSelected => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording/playback first".to_string();
                    return Ok(Task::none());
                }

                let Some(index) = self.selected_index else {
                    self.status = "Select a row first to insert below".to_string();
                    return Ok(Task::none());
                };

                let Some((x, y)) = self.parse_editor_xy() else {
                    self.status = "Invalid X/Y".to_string();
                    return Ok(Task::none());
                };

                let patch = if self.editor_use_find_image {
                    self.editor_static_preview_patch_b64
                        .clone()
                        .or_else(|| capture_patch_png_base64(x, y, 128).ok())
                } else {
                    capture_patch_png_base64(x, y, 128).ok()
                };
                let kinds = self.event_kinds_from_editor_modes(patch);
                if kinds.is_empty() {
                    self.status = if self.editor_use_find_image {
                        "Capture GET target first".to_string()
                    } else {
                        "Choose at least one click mode".to_string()
                    };
                    return Ok(Task::none());
                }

                let click_meta = Some(self.current_click_list_meta());

                let base_ms = self.events.get(index).map(|e| e.ms_from_start).unwrap_or(0);
                let insert_at = index + 1;
                let inserted = kinds.len();

                for (offset, kind) in kinds.into_iter().enumerate() {
                    self.events.insert(
                        insert_at + offset,
                        RecordedEvent {
                            ms_from_start: base_ms + 1 + offset as u128,
                            kind,
                            pos: Some((x, y)),
                            click_meta: click_meta.clone(),
                        },
                    );
                }

                self.selected_index = Some(insert_at);
                self.status = format!("Inserted {} row(s) below selected", inserted);
                Ok(Task::none())
            }
            Message::SelectRow(index) => {
                self.selected_index = Some(index);
                if let Some(ev) = self.events.get(index) {
                    let event_patch_b64: Option<String> = match &ev.kind {
                        RecordedEventKind::FindTarget { patch_png_base64, .. } => {
                            Some(patch_png_base64.clone())
                        }
                        RecordedEventKind::LeftDown { patch_png_base64 }
                        | RecordedEventKind::LeftUp { patch_png_base64 }
                        | RecordedEventKind::LeftClick { patch_png_base64 }
                        | RecordedEventKind::RightDown { patch_png_base64 }
                        | RecordedEventKind::RightUp { patch_png_base64 }
                        | RecordedEventKind::RightClick { patch_png_base64 }
                        | RecordedEventKind::MiddleDown { patch_png_base64 }
                        | RecordedEventKind::MiddleUp { patch_png_base64 }
                        | RecordedEventKind::MiddleClick { patch_png_base64 } => {
                            patch_png_base64.clone()
                        }
                        _ => None,
                    };

                    if let Some((x, y)) = ev.pos {
                        self.editor_x_text = x.to_string();
                        self.editor_y_text = y.to_string();
                    }

                    self.editor_static_preview_patch_b64 = event_patch_b64;

                    match &ev.kind {
                        RecordedEventKind::LeftDown { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Left;
                            self.editor_left_mode = ClickEdgeMode::Down;
                            self.editor_right_mode = ClickEdgeMode::Auto;
                            self.editor_middle_mode = ClickEdgeMode::Auto;
                        }
                        RecordedEventKind::LeftUp { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Left;
                            self.editor_left_mode = ClickEdgeMode::Up;
                            self.editor_right_mode = ClickEdgeMode::Auto;
                            self.editor_middle_mode = ClickEdgeMode::Auto;
                        }
                        RecordedEventKind::RightDown { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Right;
                            self.editor_left_mode = ClickEdgeMode::Auto;
                            self.editor_right_mode = ClickEdgeMode::Down;
                            self.editor_middle_mode = ClickEdgeMode::Auto;
                        }
                        RecordedEventKind::RightUp { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Right;
                            self.editor_left_mode = ClickEdgeMode::Auto;
                            self.editor_right_mode = ClickEdgeMode::Up;
                            self.editor_middle_mode = ClickEdgeMode::Auto;
                        }
                        RecordedEventKind::MiddleDown { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Middle;
                            self.editor_left_mode = ClickEdgeMode::Auto;
                            self.editor_right_mode = ClickEdgeMode::Auto;
                            self.editor_middle_mode = ClickEdgeMode::Down;
                        }
                        RecordedEventKind::MiddleUp { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Middle;
                            self.editor_left_mode = ClickEdgeMode::Auto;
                            self.editor_right_mode = ClickEdgeMode::Auto;
                            self.editor_middle_mode = ClickEdgeMode::Up;
                        }
                        RecordedEventKind::LeftClick { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Left;
                            self.editor_left_mode = ev
                                .click_meta
                                .as_ref()
                                .map(|m| m.left_mode)
                                .unwrap_or(ClickEdgeMode::Auto);
                            self.editor_right_mode = ClickEdgeMode::Auto;
                            self.editor_middle_mode = ClickEdgeMode::Auto;
                        }
                        RecordedEventKind::RightClick { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Right;
                            self.editor_left_mode = ClickEdgeMode::Auto;
                            self.editor_right_mode = ev
                                .click_meta
                                .as_ref()
                                .map(|m| m.right_mode)
                                .unwrap_or(ClickEdgeMode::Auto);
                            self.editor_middle_mode = ClickEdgeMode::Auto;
                        }
                        RecordedEventKind::MiddleClick { .. } => {
                            self.editor_use_find_image = false;
                            self.editor_click_target = ClickTarget::Middle;
                            self.editor_left_mode = ClickEdgeMode::Auto;
                            self.editor_right_mode = ClickEdgeMode::Auto;
                            self.editor_middle_mode = ev
                                .click_meta
                                .as_ref()
                                .map(|m| m.middle_mode)
                                .unwrap_or(ClickEdgeMode::Auto);
                        }
                        _ => {}
                    }

                    let active_mode = match self.editor_click_target {
                        ClickTarget::Left => self.editor_left_mode,
                        ClickTarget::Right => self.editor_right_mode,
                        ClickTarget::Middle => self.editor_middle_mode,
                    };
                    self.editor_mouse_move_speed_ms =
                        Self::default_mouse_move_speed_for_mode(active_mode);

                    if let Some(meta) = &ev.click_meta {
                        self.editor_left_mode = meta.left_mode;
                        self.editor_right_mode = meta.right_mode;
                        self.editor_middle_mode = meta.middle_mode;
                        self.editor_wait_ms = meta.wait_ms;
                        self.editor_click_speed_ms = meta.click_speed_ms.min(100);
                        self.editor_mouse_move_speed_ms = meta.mouse_move_speed_ms.min(500);
                        self.selected_wait_ms_text = meta.wait_ms.to_string();
                        self.editor_use_find_image = meta.use_find_image;
                    }

                    match &ev.kind {
                        RecordedEventKind::Wait { ms } => {
                            self.selected_wait_ms_text = ms.to_string();
                        }
                        RecordedEventKind::FindTarget {
                            precision,
                            timeout_ms,
                            search_anchor,
                            search_region_size,
                            ..
                        } => {
                            self.selected_precision_text = format!("{precision:.3}");
                            self.selected_timeout_ms_text = timeout_ms.to_string();
                            self.selected_anchor = *search_anchor;
                            self.selected_limit_region = search_region_size.is_some();
                            self.selected_region_size_text = search_region_size
                                .unwrap_or(self.find_image_region_size)
                                .to_string();
                        }
                        _ => {}
                    }
                }
                Ok(Task::none())
            }
            Message::ClearSelection => {
                self.selected_index = None;
                Ok(Task::none())
            }
            _ => Err(message),
        }
    }
}
