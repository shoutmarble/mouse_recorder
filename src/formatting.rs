use super::*;

pub(crate) fn format_event_with_prev(
    ev: &RecordedEvent,
    prev_pos: Option<(i32, i32)>,
) -> (String, String, Option<(i32, i32)>) {
    let wait_value = |meta: Option<&ClickListMeta>| -> String {
        let wait_ms = meta.map(|m| m.wait_ms).unwrap_or(0);
        format!("wait {} ms", wait_ms)
    };

    let move_value = |meta: Option<&ClickListMeta>| -> String {
        if let Some(m) = meta {
            format!("move {} ms | {}", m.mouse_move_speed_ms, wait_value(meta))
        } else {
            wait_value(meta)
        }
    };

    let source_from_meta = |meta: Option<&ClickListMeta>, pos: Option<(i32, i32)>| {
        if meta.map(|m| m.use_find_image).unwrap_or(false) {
            "TARGET".to_string()
        } else if let Some((x, y)) = pos {
            format!("({x},{y})")
        } else {
            "(X,Y)".to_string()
        }
    };

    let mode_tag = |mode: ClickEdgeMode| match mode {
        ClickEdgeMode::Auto => "CLICK",
        ClickEdgeMode::Down => "DOWN",
        ClickEdgeMode::Up => "UP",
        ClickEdgeMode::Double => "DOUBLE",
    };

    let click_action = |button: &str, default_mode: ClickEdgeMode, meta: Option<&ClickListMeta>, source_override: Option<&str>, pos: Option<(i32, i32)>| {
        let source = source_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| source_from_meta(meta, pos));
        let mode = match (button, meta) {
            ("LEFT", Some(m)) => m.left_mode,
            ("RIGHT", Some(m)) => m.right_mode,
            ("MIDDLE", Some(m)) => m.middle_mode,
            _ => default_mode,
        };
        let duration = meta.map(|m| m.click_speed_ms).unwrap_or(20);
        format!("{button}:{}|D:{}ms|{source}", mode_tag(mode), duration)
    };

    match &ev.kind {
        RecordedEventKind::Move { x, y } => {
            let _ = prev_pos.unwrap_or((*x, *y));
            (
                format!("MOVE|({x},{y})"),
                move_value(ev.click_meta.as_ref()),
                Some((*x, *y)),
            )
        }
        RecordedEventKind::Moves { points } => {
            let last = points.last().copied().or(prev_pos);
            (
                "MOVES|(X,Y)".to_string(),
                format!("{} pts | {}", points.len(), move_value(ev.click_meta.as_ref())),
                last,
            )
        }
        RecordedEventKind::Wait { ms } => (
            "WAIT".to_string(),
            format!("wait {} ms", ms),
            prev_pos,
        ),
        RecordedEventKind::FindTarget { .. } => (
            "FIND|TARGET".to_string(),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::LeftClick { .. } => (
            click_action("LEFT", ClickEdgeMode::Auto, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::RightClick { .. } => (
            click_action("RIGHT", ClickEdgeMode::Auto, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::MiddleClick { .. } => (
            click_action("MIDDLE", ClickEdgeMode::Auto, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::LeftDown { .. } => (
            click_action("LEFT", ClickEdgeMode::Down, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::LeftUp { .. } => (
            click_action("LEFT", ClickEdgeMode::Up, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::RightDown { .. } => (
            click_action("RIGHT", ClickEdgeMode::Down, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::RightUp { .. } => (
            click_action("RIGHT", ClickEdgeMode::Up, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::MiddleDown { .. } => (
            click_action("MIDDLE", ClickEdgeMode::Down, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
        RecordedEventKind::MiddleUp { .. } => (
            click_action("MIDDLE", ClickEdgeMode::Up, ev.click_meta.as_ref(), None, ev.pos.or(prev_pos)),
            wait_value(ev.click_meta.as_ref()),
            ev.pos.or(prev_pos),
        ),
    }
}