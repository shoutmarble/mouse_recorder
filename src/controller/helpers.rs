use super::*;

impl App {
    pub(super) fn parse_editor_xy(&self) -> Option<(i32, i32)> {
        let x = self.editor_x_text.trim().parse::<i32>().ok()?;
        let y = self.editor_y_text.trim().parse::<i32>().ok()?;
        Some((x, y))
    }

    pub(super) fn current_click_list_meta(&self) -> ClickListMeta {
        ClickListMeta {
            left_mode: self.editor_left_mode,
            right_mode: self.editor_right_mode,
            middle_mode: self.editor_middle_mode,
            wait_ms: self.editor_wait_ms,
            click_speed_ms: self.editor_click_speed_ms,
            mouse_move_speed_ms: self.editor_mouse_move_speed_ms,
            use_find_image: self.editor_use_find_image,
            target_precision: self.editor_target_precision_percent as f32 / 100.0,
            target_timeout_ms: self.editor_target_timeout_ms as u64,
        }
    }

    pub(super) fn push_recorded_button_release(
        &self,
        state: &mut RecorderState,
        pushed: &mut Vec<RecordedEvent>,
        pos: Option<(i32, i32)>,
        button: MouseButton,
    ) {
        let click_time = if state.started_by_click {
            state.synthetic_time_ms = state.synthetic_time_ms.saturating_add(self.recorder_wait_ms as u128);
            state.synthetic_time_ms
        } else {
            state.started_by_click = true;
            state.synthetic_time_ms
        };

        if let (Some(prev), Some(curr)) = (state.last_click_pos, pos) {
            if prev != curr {
                pushed.push(RecordedEvent {
                    ms_from_start: click_time,
                    kind: RecordedEventKind::Move {
                        x: curr.0,
                        y: curr.1,
                    },
                    pos: Some(curr),
                    click_meta: None,
                });
            }
        }

        let patch = pos.and_then(|(x, y)| capture_patch_png_base64(x, y, self.find_image_patch_size).ok());
        let kind = match button {
            MouseButton::Left => RecordedEventKind::LeftClick { patch_png_base64: patch },
            MouseButton::Right => RecordedEventKind::RightClick { patch_png_base64: patch },
            MouseButton::Middle => RecordedEventKind::MiddleClick { patch_png_base64: patch },
        };

        pushed.push(RecordedEvent {
            ms_from_start: click_time,
            kind,
            pos,
            click_meta: Some(ClickListMeta {
                left_mode: ClickEdgeMode::Auto,
                right_mode: ClickEdgeMode::Auto,
                middle_mode: ClickEdgeMode::Auto,
                wait_ms: self.recorder_wait_ms as u16,
                click_speed_ms: self.editor_click_speed_ms,
                mouse_move_speed_ms: self.editor_mouse_move_speed_ms,
                use_find_image: self.editor_use_find_image,
                target_precision: self.editor_target_precision_percent as f32 / 100.0,
                target_timeout_ms: self.editor_target_timeout_ms as u64,
            }),
        });

        state.last_click_pos = pos;
    }

    pub(super) fn event_kinds_from_editor_modes(&self, patch: Option<String>) -> Vec<RecordedEventKind> {
        let mut out = Vec::new();

        // Keep click-row patch even in find-target mode so existing thumbnails
        // and reference image data remain stable unless user explicitly captures
        // a new patch via GET (X,Y).
        let click_patch = patch.clone();

        if self.editor_left_mode == ClickEdgeMode::Auto
            && self.editor_right_mode == ClickEdgeMode::Auto
            && self.editor_middle_mode == ClickEdgeMode::Auto
        {
            out.push(match self.editor_click_target {
                ClickTarget::Left => RecordedEventKind::LeftClick {
                    patch_png_base64: click_patch,
                },
                ClickTarget::Right => RecordedEventKind::RightClick {
                    patch_png_base64: click_patch,
                },
                ClickTarget::Middle => RecordedEventKind::MiddleClick {
                    patch_png_base64: click_patch,
                },
            });
            return out;
        }

        if self.editor_left_mode != ClickEdgeMode::Auto {
            let kind = match self.editor_left_mode {
                ClickEdgeMode::Auto => unreachable!(),
                ClickEdgeMode::Down => RecordedEventKind::LeftDown {
                    patch_png_base64: click_patch.clone(),
                },
                ClickEdgeMode::Up => RecordedEventKind::LeftUp {
                    patch_png_base64: click_patch.clone(),
                },
                ClickEdgeMode::Double => RecordedEventKind::LeftClick {
                    patch_png_base64: click_patch.clone(),
                },
            };
            out.push(kind);
        }

        if self.editor_right_mode != ClickEdgeMode::Auto {
            let kind = match self.editor_right_mode {
                ClickEdgeMode::Auto => unreachable!(),
                ClickEdgeMode::Down => RecordedEventKind::RightDown {
                    patch_png_base64: click_patch.clone(),
                },
                ClickEdgeMode::Up => RecordedEventKind::RightUp {
                    patch_png_base64: click_patch.clone(),
                },
                ClickEdgeMode::Double => RecordedEventKind::RightClick {
                    patch_png_base64: click_patch.clone(),
                },
            };
            out.push(kind);
        }

        if self.editor_middle_mode != ClickEdgeMode::Auto {
            let kind = match self.editor_middle_mode {
                ClickEdgeMode::Auto => unreachable!(),
                ClickEdgeMode::Down => RecordedEventKind::MiddleDown {
                    patch_png_base64: click_patch.clone(),
                },
                ClickEdgeMode::Up => RecordedEventKind::MiddleUp {
                    patch_png_base64: click_patch.clone(),
                },
                ClickEdgeMode::Double => RecordedEventKind::MiddleClick {
                    patch_png_base64: click_patch.clone(),
                },
            };
            out.push(kind);
        }

        out
    }
}
