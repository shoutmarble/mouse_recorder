use super::*;
use crate::state::PendingClick;

impl App {
    pub(super) fn materialize_moves_grouped_events_with_row_map(&self) -> (Vec<RecordedEvent>, Vec<usize>) {
        let mut out: Vec<RecordedEvent> = Vec::with_capacity(self.events.len());
        let mut row_map: Vec<usize> = Vec::with_capacity(self.events.len());
        let mut i = 0usize;

        while i < self.events.len() {
            match &self.events[i].kind {
                RecordedEventKind::Move { .. } => {
                    let mut j = i;
                    let mut points: Vec<(i32, i32)> = Vec::new();
                    let mut move_meta: Option<ClickListMeta> = None;
                    while j < self.events.len() {
                        match &self.events[j].kind {
                            RecordedEventKind::Move { x, y } => {
                                points.push((*x, *y));
                                if self.events[j].click_meta.is_some() {
                                    move_meta = self.events[j].click_meta.clone();
                                }
                                j += 1;
                            }
                            _ => break,
                        }
                    }

                    if points.len() >= 2 {
                        let end = &self.events[j - 1];
                        let last_pos = points.last().copied();
                        out.push(RecordedEvent {
                            ms_from_start: end.ms_from_start,
                            kind: RecordedEventKind::Moves { points },
                            pos: last_pos,
                            click_meta: move_meta,
                        });
                        row_map.push(j - 1);
                    } else {
                        out.push(self.events[i].clone());
                        row_map.push(i);
                    }

                    i = j;
                }
                _ => {
                    out.push(self.events[i].clone());
                    row_map.push(i);
                    i += 1;
                }
            }
        }

        (out, row_map)
    }

    pub(super) fn materialize_moves_grouped_events(&self) -> Vec<RecordedEvent> {
        self.materialize_moves_grouped_events_with_row_map().0
    }

    pub(super) fn append_recorded_events_compacting_moves(
        &mut self,
        new_events: Vec<RecordedEvent>,
    ) -> bool {
        let mut changed = false;

        for ev in new_events {
            match (&mut self.events.last_mut(), &ev.kind) {
                (
                    Some(last),
                    RecordedEventKind::Move { x, y },
                ) if matches!(last.kind, RecordedEventKind::Move { .. }) => {
                    let is_same_pos = match &last.kind {
                        RecordedEventKind::Move { x: lx, y: ly } => *lx == *x && *ly == *y,
                        _ => false,
                    };

                    if is_same_pos {
                        last.ms_from_start = ev.ms_from_start;
                        changed = true;
                    } else {
                        self.events.push(ev);
                        changed = true;
                    }
                }
                _ => {
                    self.events.push(ev);
                    changed = true;
                }
            }
        }

        changed
    }

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
        down_pos: Option<(i32, i32)>,
        up_pos: Option<(i32, i32)>,
        held_ms: u64,
        button: MouseButton,
    ) {
        let click_time = if state.started_by_click {
            state.synthetic_time_ms = state.synthetic_time_ms.saturating_add(self.recorder_wait_ms as u128);
            state.synthetic_time_ms
        } else {
            state.started_by_click = true;
            state.synthetic_time_ms
        };

        let release_pos = up_pos.or(down_pos);

        let make_down_kind = |patch_png_base64: Option<String>| match button {
            MouseButton::Left => RecordedEventKind::LeftDown { patch_png_base64 },
            MouseButton::Right => RecordedEventKind::RightDown { patch_png_base64 },
            MouseButton::Middle => RecordedEventKind::MiddleDown { patch_png_base64 },
        };

        let make_up_kind = |patch_png_base64: Option<String>| match button {
            MouseButton::Left => RecordedEventKind::LeftUp { patch_png_base64 },
            MouseButton::Right => RecordedEventKind::RightUp { patch_png_base64 },
            MouseButton::Middle => RecordedEventKind::MiddleUp { patch_png_base64 },
        };

        let make_click_kind = |patch_png_base64: Option<String>| match button {
            MouseButton::Left => RecordedEventKind::LeftClick { patch_png_base64 },
            MouseButton::Right => RecordedEventKind::RightClick { patch_png_base64 },
            MouseButton::Middle => RecordedEventKind::MiddleClick { patch_png_base64 },
        };

        let split_deadzone_px = self.editor_click_split_px as i32;
        let click_max_hold_ms = self.editor_click_max_hold_ms as u64;
        let click_speed_ms = self.editor_click_speed_ms as u64;

        let moved_far = matches!((down_pos, up_pos), (Some(d), Some(u)) if {
            let dx = (u.0 - d.0).abs();
            let dy = (u.1 - d.1).abs();
            dx >= split_deadzone_px || dy >= split_deadzone_px
        });

        let split_into_down_up = moved_far || held_ms > click_max_hold_ms;

        if split_into_down_up {
            self.flush_pending_click_for_button(state, pushed, button);

            let down_patch = down_pos.and_then(|(x, y)| {
                capture_patch_png_base64(x, y, self.find_image_patch_size).ok()
            });
            let up_patch = up_pos.and_then(|(x, y)| {
                capture_patch_png_base64(x, y, self.find_image_patch_size).ok()
            });

            let down_meta = self.recorded_click_meta(button, ClickEdgeMode::Down);
            let up_meta = self.recorded_click_meta(button, ClickEdgeMode::Up);

            pushed.push(RecordedEvent {
                ms_from_start: click_time,
                kind: make_down_kind(down_patch),
                pos: down_pos,
                click_meta: Some(down_meta),
            });

            pushed.push(RecordedEvent {
                ms_from_start: click_time,
                kind: make_up_kind(up_patch),
                pos: up_pos,
                click_meta: Some(up_meta),
            });

            state.last_click_pos = release_pos;
            return;
        }

        let pending_slot = match button {
            MouseButton::Left => &mut state.left_pending_click,
            MouseButton::Right => &mut state.right_pending_click,
            MouseButton::Middle => &mut state.middle_pending_click,
        };

        let now = Instant::now();
        if let Some(prev_pending) = pending_slot.take() {
            let within_double_window =
                now.duration_since(prev_pending.up_at).as_millis() as u64 <= click_speed_ms;

            if within_double_window {
                let patch = release_pos
                    .and_then(|(x, y)| capture_patch_png_base64(x, y, self.find_image_patch_size).ok());
                let kind = make_click_kind(patch);
                let click_meta = self.recorded_click_meta(button, ClickEdgeMode::Double);

                pushed.push(RecordedEvent {
                    ms_from_start: click_time,
                    kind,
                    pos: release_pos,
                    click_meta: Some(click_meta),
                });
                state.last_click_pos = release_pos;
                return;
            }

            self.push_pending_single_click(pushed, prev_pending);
        }

        *pending_slot = Some(PendingClick {
            button,
            pos: release_pos,
            up_at: now,
            synthetic_time_ms: click_time,
        });
        state.last_click_pos = release_pos;
    }

    pub(super) fn push_recorded_button_down_for_path(
        &self,
        state: &mut RecorderState,
        pushed: &mut Vec<RecordedEvent>,
        down_pos: Option<(i32, i32)>,
        button: MouseButton,
    ) {
        self.flush_pending_click_for_button(state, pushed, button);

        if state.started_by_click {
            state.synthetic_time_ms = state.synthetic_time_ms.saturating_add(1);
        } else {
            state.started_by_click = true;
        }

        let patch = down_pos
            .and_then(|(x, y)| capture_patch_png_base64(x, y, self.find_image_patch_size).ok());

        let kind = match button {
            MouseButton::Left => RecordedEventKind::LeftDown { patch_png_base64: patch },
            MouseButton::Right => RecordedEventKind::RightDown { patch_png_base64: patch },
            MouseButton::Middle => RecordedEventKind::MiddleDown { patch_png_base64: patch },
        };

        let down_meta = self.recorded_click_meta(button, ClickEdgeMode::Down);
        pushed.push(RecordedEvent {
            ms_from_start: state.synthetic_time_ms,
            kind,
            pos: down_pos,
            click_meta: Some(down_meta),
        });
    }

    pub(super) fn push_recorded_button_up_for_path(
        &self,
        state: &mut RecorderState,
        pushed: &mut Vec<RecordedEvent>,
        up_pos: Option<(i32, i32)>,
        button: MouseButton,
    ) {
        if state.started_by_click {
            state.synthetic_time_ms = state.synthetic_time_ms.saturating_add(1);
        } else {
            state.started_by_click = true;
        }

        let patch = up_pos
            .and_then(|(x, y)| capture_patch_png_base64(x, y, self.find_image_patch_size).ok());

        let kind = match button {
            MouseButton::Left => RecordedEventKind::LeftUp { patch_png_base64: patch },
            MouseButton::Right => RecordedEventKind::RightUp { patch_png_base64: patch },
            MouseButton::Middle => RecordedEventKind::MiddleUp { patch_png_base64: patch },
        };

        let up_meta = self.recorded_click_meta(button, ClickEdgeMode::Up);
        pushed.push(RecordedEvent {
            ms_from_start: state.synthetic_time_ms,
            kind,
            pos: up_pos,
            click_meta: Some(up_meta),
        });

        state.last_click_pos = up_pos;
    }

    fn recorded_click_meta(&self, button: MouseButton, mode: ClickEdgeMode) -> ClickListMeta {
        let mut meta = ClickListMeta {
            left_mode: ClickEdgeMode::Auto,
            right_mode: ClickEdgeMode::Auto,
            middle_mode: ClickEdgeMode::Auto,
            wait_ms: self.recorder_wait_ms as u16,
            click_speed_ms: self.editor_click_speed_ms.clamp(0, 100),
            mouse_move_speed_ms: self.editor_mouse_move_speed_ms.clamp(5, 50),
            use_find_image: self.editor_use_find_image,
            target_precision: (self.editor_target_precision_percent as f32 / 100.0).clamp(0.5, 1.0),
            target_timeout_ms: (self.editor_target_timeout_ms as u64).clamp(200, 10000),
        };

        match button {
            MouseButton::Left => meta.left_mode = mode,
            MouseButton::Right => meta.right_mode = mode,
            MouseButton::Middle => meta.middle_mode = mode,
        }

        meta
    }

    fn push_pending_single_click(&self, pushed: &mut Vec<RecordedEvent>, pending: PendingClick) {
        let patch = pending
            .pos
            .and_then(|(x, y)| capture_patch_png_base64(x, y, self.find_image_patch_size).ok());

        let kind = match pending.button {
            MouseButton::Left => RecordedEventKind::LeftClick { patch_png_base64: patch },
            MouseButton::Right => RecordedEventKind::RightClick { patch_png_base64: patch },
            MouseButton::Middle => RecordedEventKind::MiddleClick { patch_png_base64: patch },
        };

        let click_meta = self.recorded_click_meta(pending.button, ClickEdgeMode::Auto);

        pushed.push(RecordedEvent {
            ms_from_start: pending.synthetic_time_ms,
            kind,
            pos: pending.pos,
            click_meta: Some(click_meta),
        });
    }

    pub(super) fn flush_pending_click_for_button(
        &self,
        state: &mut RecorderState,
        pushed: &mut Vec<RecordedEvent>,
        button: MouseButton,
    ) {
        let pending = match button {
            MouseButton::Left => state.left_pending_click.take(),
            MouseButton::Right => state.right_pending_click.take(),
            MouseButton::Middle => state.middle_pending_click.take(),
        };

        if let Some(pending) = pending {
            self.push_pending_single_click(pushed, pending);
        }
    }

    pub(super) fn flush_expired_pending_clicks(
        &self,
        state: &mut RecorderState,
        pushed: &mut Vec<RecordedEvent>,
    ) {
        let now = Instant::now();
        let click_speed_ms = self.editor_click_speed_ms as u64;

        let mut flush_slot = |slot: &mut Option<PendingClick>| {
            let Some(pending) = slot.take() else {
                return;
            };

            if now.duration_since(pending.up_at).as_millis() as u64 > click_speed_ms {
                self.push_pending_single_click(pushed, pending);
            } else {
                *slot = Some(pending);
            }
        };

        flush_slot(&mut state.left_pending_click);
        flush_slot(&mut state.right_pending_click);
        flush_slot(&mut state.middle_pending_click);
    }

    pub(super) fn flush_all_pending_clicks(
        &self,
        state: &mut RecorderState,
        pushed: &mut Vec<RecordedEvent>,
    ) {
        if let Some(pending) = state.left_pending_click.take() {
            self.push_pending_single_click(pushed, pending);
        }
        if let Some(pending) = state.right_pending_click.take() {
            self.push_pending_single_click(pushed, pending);
        }
        if let Some(pending) = state.middle_pending_click.take() {
            self.push_pending_single_click(pushed, pending);
        }
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
