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
                self.recorder_wait_ms = self.editor_wait_ms as u64;
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
                let snapped = ((ms.saturating_add(2)) / 5) * 5;
                self.editor_mouse_move_speed_ms = snapped.clamp(5, 500);
                Ok(Task::none())
            }
            Message::MousePathMinDeltaPxChanged(px) => {
                self.recorder_mouse_path_min_delta_px = px.clamp(0, 10);
                Ok(Task::none())
            }
            Message::EditorClickSplitPxChanged(px) => {
                self.editor_click_split_px = px.clamp(0, 20);
                Ok(Task::none())
            }
            Message::EditorClickMaxHoldMsChanged(ms) => {
                self.editor_click_max_hold_ms = ms.clamp(0, 100);
                Ok(Task::none())
            }
            Message::EditorTargetPrecisionChanged(percent) => {
                self.editor_target_precision_percent = percent.clamp(50, 100);
                Ok(Task::none())
            }
            Message::EditorTargetTimeoutMsChanged(ms) => {
                self.editor_target_timeout_ms = ms.clamp(200, 10000);
                Ok(Task::none())
            }
            Message::EditorUseFindImageToggled(enabled) => {
                self.editor_use_find_image = enabled;
                Ok(Task::none())
            }
            Message::EditorStartGetXY => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
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
                        self.status = format!("GET capture failed: {err}");
                        return Ok(Task::none());
                    }

                    arm_get_capture_hook();
                    self.editor_capture_armed = true;
                    self.editor_last_capture_button = None;
                    self.status = "GET (X,Y) armed: click Left, Right, or Middle to set coordinates (ESC cancels).".to_string();
                }

                #[cfg(not(windows))]
                {
                    self.status = "GET (X,Y) capture is currently Windows-only.".to_string();
                }

                Ok(Task::none())
            }
            Message::EditorJumpToXY => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }

                let Some((x, y)) = self.parse_editor_xy() else {
                    self.status = "Invalid X/Y coordinates for jump.".to_string();
                    return Ok(Task::none());
                };

                match jump_mouse_to(x, y) {
                    Ok(()) => {
                        self.status = format!("Jumped mouse to ({x}, {y})");
                    }
                    Err(err) => {
                        self.status = format!("Jump failed: {err}");
                    }
                }

                Ok(Task::none())
            }
            Message::EditorInsertOrApply => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }

                let Some((x, y)) = self.parse_editor_xy() else {
                    self.status = "Invalid X/Y coordinates.".to_string();
                    return Ok(Task::none());
                };

                let patch = if self.editor_use_find_image {
                    let Some(existing_patch) = self.editor_static_preview_patch_b64.clone() else {
                        self.status = "Press GET (X,Y) to capture or select a target image first.".to_string();
                        return Ok(Task::none());
                    };
                    Some(existing_patch)
                } else {
                    capture_patch_png_base64(x, y, 128).ok()
                };
                let kinds = self.event_kinds_from_editor_modes(patch);
                if kinds.is_empty() {
                    self.status = if self.editor_use_find_image {
                        "Capture a GET target first.".to_string()
                    } else {
                        "Choose at least one click mode.".to_string()
                    };
                    return Ok(Task::none());
                }

                let click_meta = Some(self.current_click_list_meta());

                if let Some(index) = self.selected_index {
                    let selected_kind = self.events.get(index).map(|ev| ev.kind.clone());
                    let preserve_selected_patch = matches!(
                        selected_kind.as_ref(),
                        Some(
                            RecordedEventKind::LeftDown { .. }
                                | RecordedEventKind::LeftUp { .. }
                                | RecordedEventKind::LeftClick { .. }
                                | RecordedEventKind::RightDown { .. }
                                | RecordedEventKind::RightUp { .. }
                                | RecordedEventKind::RightClick { .. }
                                | RecordedEventKind::MiddleDown { .. }
                                | RecordedEventKind::MiddleUp { .. }
                                | RecordedEventKind::MiddleClick { .. }
                        )
                    );
                    let selected_patch = selected_kind.as_ref().and_then(|k| match k {
                        RecordedEventKind::LeftDown { patch_png_base64 }
                        | RecordedEventKind::LeftUp { patch_png_base64 }
                        | RecordedEventKind::LeftClick { patch_png_base64 }
                        | RecordedEventKind::RightDown { patch_png_base64 }
                        | RecordedEventKind::RightUp { patch_png_base64 }
                        | RecordedEventKind::RightClick { patch_png_base64 }
                        | RecordedEventKind::MiddleDown { patch_png_base64 }
                        | RecordedEventKind::MiddleUp { patch_png_base64 }
                        | RecordedEventKind::MiddleClick { patch_png_base64 } => patch_png_base64.clone(),
                        _ => None,
                    });

                    if let Some(ev) = self.events.get(index).cloned() {
                        match ev.kind {
                            RecordedEventKind::Move { .. } => {
                                if let Some(cur) = self.events.get_mut(index) {
                                    cur.kind = RecordedEventKind::Move { x, y };
                                    cur.pos = Some((x, y));
                                    cur.click_meta = click_meta.clone();
                                }
                                self.status = format!("Updated move at row {}", index);
                                return Ok(Task::none());
                            }
                            RecordedEventKind::Moves { mut points } => {
                                if let Some(last) = points.last_mut() {
                                    *last = (x, y);
                                } else {
                                    points.push((x, y));
                                }

                                if let Some(cur) = self.events.get_mut(index) {
                                    cur.kind = RecordedEventKind::Moves { points };
                                    cur.pos = Some((x, y));
                                    cur.click_meta = click_meta.clone();
                                }
                                self.status = format!("Updated moves at row {}", index);
                                return Ok(Task::none());
                            }
                            _ => {}
                        }
                    }

                    let replacement_kind = {
                        let kind_matches_selected = |candidate: &RecordedEventKind| -> bool {
                            match (selected_kind.as_ref(), candidate) {
                                (Some(RecordedEventKind::LeftDown { .. }), RecordedEventKind::LeftDown { .. }) => true,
                                (Some(RecordedEventKind::LeftUp { .. }), RecordedEventKind::LeftUp { .. }) => true,
                                (Some(RecordedEventKind::LeftClick { .. }), RecordedEventKind::LeftClick { .. }) => true,
                                (Some(RecordedEventKind::RightDown { .. }), RecordedEventKind::RightDown { .. }) => true,
                                (Some(RecordedEventKind::RightUp { .. }), RecordedEventKind::RightUp { .. }) => true,
                                (Some(RecordedEventKind::RightClick { .. }), RecordedEventKind::RightClick { .. }) => true,
                                (Some(RecordedEventKind::MiddleDown { .. }), RecordedEventKind::MiddleDown { .. }) => true,
                                (Some(RecordedEventKind::MiddleUp { .. }), RecordedEventKind::MiddleUp { .. }) => true,
                                (Some(RecordedEventKind::MiddleClick { .. }), RecordedEventKind::MiddleClick { .. }) => true,
                                _ => false,
                            }
                        };

                        kinds
                            .iter()
                            .find(|k| kind_matches_selected(k))
                            .cloned()
                            .or_else(|| kinds.first().cloned())
                    };

                    let Some(replacement_kind) = replacement_kind else {
                        self.status = "No compatible action to apply".to_string();
                        return Ok(Task::none());
                    };

                    let replacement_kind = if preserve_selected_patch {
                        match replacement_kind {
                            RecordedEventKind::LeftDown { .. } => RecordedEventKind::LeftDown {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::LeftUp { .. } => RecordedEventKind::LeftUp {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::LeftClick { .. } => RecordedEventKind::LeftClick {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::RightDown { .. } => RecordedEventKind::RightDown {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::RightUp { .. } => RecordedEventKind::RightUp {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::RightClick { .. } => RecordedEventKind::RightClick {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::MiddleDown { .. } => RecordedEventKind::MiddleDown {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::MiddleUp { .. } => RecordedEventKind::MiddleUp {
                                patch_png_base64: selected_patch.clone(),
                            },
                            RecordedEventKind::MiddleClick { .. } => RecordedEventKind::MiddleClick {
                                patch_png_base64: selected_patch.clone(),
                            },
                            other => other,
                        }
                    } else {
                        replacement_kind
                    };

                    if let Some(cur) = self.events.get_mut(index) {
                        cur.kind = replacement_kind;
                        cur.pos = Some((x, y));
                        cur.click_meta = click_meta.clone();
                    }
                    self.status = "Updated selected row (single-row apply)".to_string();
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
                    self.status = "Inserted click row.".to_string();
                }

                Ok(Task::none())
            }
            Message::EditorInsertBelowSelected => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }

                let Some(index) = self.selected_index else {
                    self.status = "Select a row first to insert below".to_string();
                    return Ok(Task::none());
                };

                let Some((x, y)) = self.parse_editor_xy() else {
                    self.status = "Invalid X/Y coordinates.".to_string();
                    return Ok(Task::none());
                };

                let patch = if self.editor_use_find_image {
                    let Some(existing_patch) = self.editor_static_preview_patch_b64.clone() else {
                        self.status = "Press GET (X,Y) to capture or select a target image first.".to_string();
                        return Ok(Task::none());
                    };
                    Some(existing_patch)
                } else {
                    capture_patch_png_base64(x, y, 128).ok()
                };
                let kinds = self.event_kinds_from_editor_modes(patch);
                if kinds.is_empty() {
                    self.status = if self.editor_use_find_image {
                        "Capture a GET target first.".to_string()
                    } else {
                        "Choose at least one click mode.".to_string()
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
                self.status = format!("Inserted {} row(s) below selected (multi-row)", inserted);
                Ok(Task::none())
            }
            Message::RowJump(index) => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }
                if index >= self.events.len() {
                    self.status = "Selected row no longer exists".to_string();
                    self.selected_index = None;
                    return Ok(Task::none());
                }
                let _ = self.update(Message::SelectRow(index));
                Ok(self.update(Message::EditorJumpToXY))
            }
            Message::RowClone(index) => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }
                if index >= self.events.len() {
                    self.status = "Selected row no longer exists".to_string();
                    self.selected_index = None;
                    return Ok(Task::none());
                }
                let _ = self.update(Message::SelectRow(index));
                Ok(self.update(Message::EditorInsertBelowSelected))
            }
            Message::RowDelete(index) => {
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }
                if index >= self.events.len() {
                    self.status = "Selected row no longer exists".to_string();
                    self.selected_index = None;
                    return Ok(Task::none());
                }
                let _ = self.update(Message::SelectRow(index));
                Ok(self.update(Message::ClearSelection))
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

                    match &ev.kind {
                        RecordedEventKind::Moves { points } => {
                            if let Some((x, y)) = points.last().copied().or(ev.pos) {
                                self.editor_x_text = x.to_string();
                                self.editor_y_text = y.to_string();
                            }
                        }
                        _ => {
                            if let Some((x, y)) = ev.pos {
                                self.editor_x_text = x.to_string();
                                self.editor_y_text = y.to_string();
                            }
                        }
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
                        self.editor_mouse_move_speed_ms = meta.mouse_move_speed_ms.clamp(5, 500);
                        self.editor_target_precision_percent =
                            (meta.target_precision.clamp(0.5, 1.0) * 100.0).round() as u16;
                        self.editor_target_timeout_ms = meta.target_timeout_ms.clamp(200, 10000) as u16;
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
                if self.mode != Mode::Idle {
                    self.status = "Stop recording or playback first.".to_string();
                    return Ok(Task::none());
                }

                let Some(index) = self.selected_index else {
                    self.status = "Select a row first to delete".to_string();
                    return Ok(Task::none());
                };

                if index >= self.events.len() {
                    self.selected_index = None;
                    self.status = "Selected row no longer exists".to_string();
                    return Ok(Task::none());
                }

                let removed_count = if matches!(self.events[index].kind, RecordedEventKind::Move { .. }) {
                    let mut start = index;
                    while start > 0
                        && matches!(self.events[start - 1].kind, RecordedEventKind::Move { .. })
                    {
                        start -= 1;
                    }

                    let mut end = index;
                    while end + 1 < self.events.len()
                        && matches!(self.events[end + 1].kind, RecordedEventKind::Move { .. })
                    {
                        end += 1;
                    }

                    let count = end - start + 1;
                    self.events.drain(start..=end);
                    count
                } else {
                    self.events.remove(index);
                    1
                };

                if self.events.is_empty() {
                    self.selected_index = None;
                } else {
                    let next_index = index.min(self.events.len() - 1);
                    self.selected_index = Some(next_index);
                    self.status = if removed_count > 1 {
                        format!("Deleted MOVES row ({} points)", removed_count)
                    } else {
                        format!("Deleted row {}", index)
                    };
                    return Ok(self.update(Message::SelectRow(next_index)));
                }

                self.status = if removed_count > 1 {
                    format!("Deleted MOVES row ({} points)", removed_count)
                } else {
                    format!("Deleted row {}", index)
                };
                Ok(Task::none())
            }
            _ => Err(message),
        }
    }
}
