use super::*;

impl App {
    pub(super) fn handle_modal_message(&mut self, message: Message) -> Result<Task<Message>, Message> {
        match message {
            Message::CloseModal => {
                self.find_target_modal = None;
                self.wait_modal = None;
                Ok(Task::none())
            }
            Message::WaitMsChanged(txt) => {
                if let Some(draft) = &mut self.wait_modal {
                    draft.wait_ms_text = txt;
                    if let Ok(v) = draft.wait_ms_text.trim().parse::<u64>() {
                        draft.wait_ms = v.clamp(0, 600_000);
                    }
                }
                Ok(Task::none())
            }
            Message::WaitOk => {
                if self.mode != Mode::Idle {
                    return Ok(Task::none());
                }
                let Some(draft) = self.wait_modal.take() else {
                    return Ok(Task::none());
                };

                let ms_from_start = self
                    .events
                    .last()
                    .map(|e| e.ms_from_start)
                    .unwrap_or(0);

                self.events.push(RecordedEvent {
                    ms_from_start,
                    kind: RecordedEventKind::Wait { ms: draft.wait_ms },
                    pos: None,
                    click_meta: None,
                });
                self.status = "Added Wait row".to_string();
                Ok(Task::none())
            }
            Message::FindTargetOk => {
                if self.mode != Mode::Idle {
                    return Ok(Task::none());
                }
                let Some(draft) = self.find_target_modal.take() else {
                    return Ok(Task::none());
                };
                let Some(patch_png_base64) = draft.patch_png_base64 else {
                    self.find_target_modal = Some(FindTargetDraft { status: "No image selected".to_string(), ..draft });
                    return Ok(Task::none());
                };

                let ms_from_start = self
                    .events
                    .last()
                    .map(|e| e.ms_from_start)
                    .unwrap_or(0);
                let pos = draft.captured_pos.or(self.current_pos);

                self.events.push(RecordedEvent {
                    ms_from_start,
                    kind: RecordedEventKind::FindTarget {
                        patch_png_base64,
                        patch_size: draft.patch_size,
                        precision: draft.precision,
                        timeout_ms: draft.timeout_ms,
                        search_anchor: draft.anchor,
                        search_region_size: draft.limit_region.then_some(draft.region_size),
                    },
                    pos,
                    click_meta: None,
                });

                self.status = "Added Find target row (move only)".to_string();
                Ok(Task::none())
            }
            Message::FindTargetPatchSizeChanged(txt) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.patch_size_text = txt;
                    if let Ok(v) = draft.patch_size_text.trim().parse::<u32>() {
                        draft.patch_size = v.clamp(16, 512);
                    }
                }
                Ok(Task::none())
            }
            Message::FindTargetPrecisionChanged(txt) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.precision_text = txt;
                    if let Ok(v) = draft.precision_text.trim().parse::<f32>() {
                        draft.precision = v.clamp(0.50, 0.999);
                    }
                }
                Ok(Task::none())
            }
            Message::FindTargetTimeoutChanged(txt) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.timeout_ms_text = txt;
                    if let Ok(v) = draft.timeout_ms_text.trim().parse::<u64>() {
                        draft.timeout_ms = v.clamp(100, 60_000);
                    }
                }
                Ok(Task::none())
            }
            Message::FindTargetLimitRegionToggled(v) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.limit_region = v;
                }
                Ok(Task::none())
            }
            Message::FindTargetRegionSizeChanged(txt) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.region_size_text = txt;
                    if let Ok(v) = draft.region_size_text.trim().parse::<u32>() {
                        draft.region_size = v.clamp(100, 5000);
                    }
                }
                Ok(Task::none())
            }
            Message::FindTargetAnchorSelected(anchor) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.anchor = anchor;
                }
                Ok(Task::none())
            }
            Message::FindTargetStartCapture => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.patch_png_base64 = None;
                    draft.captured_pos = None;

                    #[cfg(windows)]
                    {
                        draft.capture_waiting = true;
                        draft.capture_left_was_down = is_vk_down_windows(VK_LBUTTON);
                        draft.status = "Click on the screen to capture".to_string();
                    }

                    #[cfg(not(windows))]
                    {
                        draft.status = "Capture is Windows-only right now".to_string();
                    }
                }
                Ok(Task::none())
            }
            Message::FindTargetCaptureTick => {
                #[cfg(windows)]
                {
                    if let Some(draft) = &mut self.find_target_modal {
                        if !draft.capture_waiting {
                            return Ok(Task::none());
                        }

                        let left_now = is_vk_down_windows(VK_LBUTTON);
                        if draft.capture_left_was_down && !left_now {
                            if let Some((x, y)) = get_mouse_pos() {
                                match capture_patch_png_base64(x, y, draft.patch_size) {
                                    Ok(b64) => {
                                        draft.patch_png_base64 = Some(b64);
                                        draft.captured_pos = Some((x, y));
                                        draft.capture_waiting = false;
                                        draft.status = "Captured".to_string();
                                    }
                                    Err(err) => {
                                        draft.capture_waiting = false;
                                        draft.status = format!("Capture failed: {err}");
                                    }
                                }
                            } else {
                                draft.capture_waiting = false;
                                draft.status = "Could not read mouse position".to_string();
                            }
                        }
                        draft.capture_left_was_down = left_now;
                    }
                }
                Ok(Task::none())
            }
            Message::FindTargetPathChanged(txt) => {
                if let Some(draft) = &mut self.find_target_modal {
                    draft.image_path = txt;
                }
                Ok(Task::none())
            }
            Message::FindTargetLoadFromPath => {
                if let Some(draft) = &mut self.find_target_modal {
                    let path = draft.image_path.trim();
                    if path.is_empty() {
                        draft.status = "Provide a path".to_string();
                        return Ok(Task::none());
                    }

                    match std::fs::read(path) {
                        Ok(bytes) => {
                            use base64::engine::general_purpose;
                            use base64::Engine;
                            draft.patch_png_base64 = Some(general_purpose::STANDARD.encode(bytes));
                            draft.captured_pos = self.current_pos;
                            draft.capture_waiting = false;
                            draft.status = "Loaded".to_string();
                        }
                        Err(err) => {
                            draft.status = format!("Load failed: {err}");
                        }
                    }
                }
                Ok(Task::none())
            }
            _ => Err(message),
        }
    }
}
