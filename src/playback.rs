use super::*;

use base64::engine::general_purpose;
use base64::Engine;
use rustautogui::{MatchMode, MouseClick, RustAutoGui};

pub(crate) fn playback(
    events: Vec<RecordedEvent>,
    cancel: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
) -> anyhow::Result<()> {
    let mut gui = RustAutoGui::new(false)?;

    let mut last_smart_found: Option<(i32, i32)> = None;
    let mut current_pos: Option<(i32, i32)> = get_mouse_pos();

    let sleep_with_cancel = |cancel: &Arc<AtomicBool>, total_ms: u64| -> anyhow::Result<()> {
        let mut remaining = Duration::from_millis(total_ms);
        while remaining > Duration::from_millis(0) {
            if cancel.load(Ordering::Relaxed) {
                anyhow::bail!("Cancelled");
            }
            let step = remaining.min(Duration::from_millis(10));
            std::thread::sleep(step);
            remaining = remaining.saturating_sub(step);
        }
        Ok(())
    };

    let click_once_with_speed = |gui: &mut RustAutoGui, button: MouseButton, speed_ms: u64| {
        match button {
            MouseButton::Left => {
                let _ = gui.click_down(MouseClick::LEFT);
                if speed_ms > 0 {
                    std::thread::sleep(Duration::from_millis(speed_ms));
                }
                let _ = gui.click_up(MouseClick::LEFT);
            }
            MouseButton::Right => {
                let _ = gui.click_down(MouseClick::RIGHT);
                if speed_ms > 0 {
                    std::thread::sleep(Duration::from_millis(speed_ms));
                }
                let _ = gui.click_up(MouseClick::RIGHT);
            }
            MouseButton::Middle => {
                let _ = gui.click_down(MouseClick::MIDDLE);
                if speed_ms > 0 {
                    std::thread::sleep(Duration::from_millis(speed_ms));
                }
                let _ = gui.click_up(MouseClick::MIDDLE);
            }
        }
    };

    for (index, ev) in events.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("Cancelled");
        }

        progress.store(index, Ordering::Relaxed);

        let RecordedEvent {
            kind,
            pos,
            click_meta,
            ..
        } = ev;

        let wait_ms = click_meta.as_ref().map(|m| m.wait_ms as u64).unwrap_or(0);
        let click_speed_ms = click_meta
            .as_ref()
            .map(|m| m.click_speed_ms as u64)
            .unwrap_or(20)
            .min(100);
        let default_move_speed_ms = if matches!(
            &kind,
            RecordedEventKind::LeftUp { .. }
                | RecordedEventKind::RightUp { .. }
                | RecordedEventKind::MiddleUp { .. }
        ) {
            250
        } else {
            150
        };
        let mouse_move_speed_ms = click_meta
            .as_ref()
            .map(|m| m.mouse_move_speed_ms as u64)
            .unwrap_or(default_move_speed_ms)
            .min(500);

        if wait_ms > 0 && !matches!(&kind, RecordedEventKind::Wait { .. }) {
            sleep_with_cancel(&cancel, wait_ms)?;
        }

        match kind {
            RecordedEventKind::Move { x, y } => {
                move_mouse_with_speed(
                    &mut gui,
                    &cancel,
                    &mut current_pos,
                    (x, y),
                    mouse_move_speed_ms,
                )?;
            }
            RecordedEventKind::Moves { points } => {
                for (x, y) in points {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
            }
            RecordedEventKind::Wait { ms } => {
                sleep_with_cancel(&cancel, ms)?;
            }
            RecordedEventKind::FindTarget {
                patch_png_base64,
                patch_size: _,
                precision,
                timeout_ms,
                search_anchor,
                search_region_size,
            } => {
                let anchor_pos = match search_anchor {
                    SearchAnchor::RecordedClick => pos,
                    SearchAnchor::CurrentMouse => get_mouse_pos().or(pos).or(last_smart_found),
                    SearchAnchor::LastFound => last_smart_found.or(pos).or(get_mouse_pos()),
                };
                let found = find_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    &patch_png_base64,
                    precision,
                    timeout_ms,
                    search_region_size,
                    anchor_pos,
                )?;

                move_mouse_with_speed(
                    &mut gui,
                    &cancel,
                    &mut current_pos,
                    found,
                    mouse_move_speed_ms,
                )?;
            }
            RecordedEventKind::LeftDown { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some((x, y)) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
                let _ = gui.click_down(MouseClick::LEFT);
            }
            RecordedEventKind::LeftUp { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some(target) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        target,
                        mouse_move_speed_ms,
                    )?;
                }
                let _ = gui.click_up(MouseClick::LEFT);
            }
            RecordedEventKind::LeftClick { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some((x, y)) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
                click_once_with_speed(&mut gui, MouseButton::Left, click_speed_ms);
                if click_meta
                    .as_ref()
                    .map(|m| m.left_mode == ClickEdgeMode::Double)
                    .unwrap_or(false)
                {
                    if click_speed_ms > 0 {
                        std::thread::sleep(Duration::from_millis(click_speed_ms));
                    }
                    click_once_with_speed(&mut gui, MouseButton::Left, click_speed_ms);
                }
            }
            RecordedEventKind::RightDown { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some((x, y)) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
                let _ = gui.click_down(MouseClick::RIGHT);
            }
            RecordedEventKind::RightUp { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some(target) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        target,
                        mouse_move_speed_ms,
                    )?;
                }
                let _ = gui.click_up(MouseClick::RIGHT);
            }
            RecordedEventKind::RightClick { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some((x, y)) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
                click_once_with_speed(&mut gui, MouseButton::Right, click_speed_ms);
                if click_meta
                    .as_ref()
                    .map(|m| m.right_mode == ClickEdgeMode::Double)
                    .unwrap_or(false)
                {
                    if click_speed_ms > 0 {
                        std::thread::sleep(Duration::from_millis(click_speed_ms));
                    }
                    click_once_with_speed(&mut gui, MouseButton::Right, click_speed_ms);
                }
            }
            RecordedEventKind::MiddleDown { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some((x, y)) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
                let _ = gui.click_down(MouseClick::MIDDLE);
            }
            RecordedEventKind::MiddleUp { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some(target) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        target,
                        mouse_move_speed_ms,
                    )?;
                }
                let _ = gui.click_up(MouseClick::MIDDLE);
            }
            RecordedEventKind::MiddleClick { patch_png_base64 } => {
                let target_pos = resolve_click_target_position(
                    &mut gui,
                    &cancel,
                    &mut last_smart_found,
                    click_meta.as_ref(),
                    patch_png_base64.as_deref(),
                    pos,
                )?;

                if let Some((x, y)) = target_pos {
                    move_mouse_with_speed(
                        &mut gui,
                        &cancel,
                        &mut current_pos,
                        (x, y),
                        mouse_move_speed_ms,
                    )?;
                }
                click_once_with_speed(&mut gui, MouseButton::Middle, click_speed_ms);
                if click_meta
                    .as_ref()
                    .map(|m| m.middle_mode == ClickEdgeMode::Double)
                    .unwrap_or(false)
                {
                    if click_speed_ms > 0 {
                        std::thread::sleep(Duration::from_millis(click_speed_ms));
                    }
                    click_once_with_speed(&mut gui, MouseButton::Middle, click_speed_ms);
                }
            }
        }
    }

    progress.store(usize::MAX, Ordering::Relaxed);

    Ok(())
}

fn move_mouse_with_speed(
    gui: &mut RustAutoGui,
    cancel: &Arc<AtomicBool>,
    current_pos: &mut Option<(i32, i32)>,
    target: (i32, i32),
    total_ms: u64,
) -> anyhow::Result<()> {
    let start = current_pos
        .or_else(get_mouse_pos)
        .unwrap_or(target);
    if start == target {
        *current_pos = Some(target);
        return Ok(());
    }

    if total_ms == 0 {
        let _ = gui.move_mouse_to_pos(target.0.max(0) as u32, target.1.max(0) as u32, 0.0);
        *current_pos = Some(target);
        return Ok(());
    }

    let steps = ((total_ms.max(1) + 9) / 10).clamp(1, 60) as i32;
    let step_ms = (total_ms.max(1) / steps as u64).max(1);

    for i in 1..=steps {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("Cancelled");
        }

        let t = i as f32 / steps as f32;
        let nx = start.0 as f32 + (target.0 - start.0) as f32 * t;
        let ny = start.1 as f32 + (target.1 - start.1) as f32 * t;
        let _ = gui.move_mouse_to_pos(nx.max(0.0) as u32, ny.max(0.0) as u32, 0.0);
        std::thread::sleep(Duration::from_millis(step_ms));
    }

    *current_pos = Some(target);
    Ok(())
}

fn resolve_click_target_position(
    gui: &mut RustAutoGui,
    cancel: &Arc<AtomicBool>,
    last_smart_found: &mut Option<(i32, i32)>,
    click_meta: Option<&ClickListMeta>,
    patch_png_base64: Option<&str>,
    pos: Option<(i32, i32)>,
) -> anyhow::Result<Option<(i32, i32)>> {
    let Some(meta) = click_meta else {
        anyhow::bail!("Click row is missing click metadata");
    };

    let use_find_image = meta.use_find_image;
    if !use_find_image {
        return Ok(pos);
    }

    let Some(patch_b64) = patch_png_base64 else {
        anyhow::bail!("Target click row is missing patch image data");
    };

    let precision = meta.target_precision.clamp(0.5, 1.0);
    let timeout_ms = meta.target_timeout_ms.clamp(200, 10000);

    let found = find_target_position(
        gui,
        cancel,
        last_smart_found,
        patch_b64,
        precision,
        timeout_ms,
        None,
        pos.or(*last_smart_found).or(get_mouse_pos()),
    )?;

    Ok(Some(found))
}

fn find_target_position(
    gui: &mut RustAutoGui,
    cancel: &Arc<AtomicBool>,
    last_smart_found: &mut Option<(i32, i32)>,
    patch_png_base64: &str,
    precision: f32,
    timeout_ms: u64,
    search_region_size: Option<u32>,
    anchor_pos: Option<(i32, i32)>,
) -> anyhow::Result<(i32, i32)> {
    let patch_png = general_purpose::STANDARD
        .decode(patch_png_base64)
        .map_err(|e| anyhow::anyhow!("FindTarget decode failed: {e}"))?;

    let region = match (search_region_size, anchor_pos) {
        (Some(size), Some((x, y))) => {
            let (sw, sh) = gui.get_screen_size();
            compute_region_around_point(sw, sh, x, y, size)
        }
        _ => None,
    };

    gui.prepare_template_from_raw_encoded(&patch_png, region, MatchMode::FFT)?;

    let started_at = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    loop {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("Cancelled");
        }
        if started_at.elapsed() > timeout {
            anyhow::bail!("FindTarget timed out ({} ms)", timeout_ms);
        }

        match gui.find_image_on_screen(precision)? {
            Some(locations) if !locations.is_empty() => {
                let (x, y, _corr) = locations[0];
                let found = (x as i32, y as i32);
                *last_smart_found = Some(found);
                return Ok(found);
            }
            _ => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn compute_region_around_point(
    screen_w: i32,
    screen_h: i32,
    center_x: i32,
    center_y: i32,
    region_size: u32,
) -> Option<(u32, u32, u32, u32)> {
    if screen_w <= 0 || screen_h <= 0 {
        return None;
    }

    let size = region_size.max(100) as i32;
    let half = size / 2;

    let sw = screen_w as i32;
    let sh = screen_h as i32;

    let cx = center_x.clamp(0, sw.saturating_sub(1));
    let cy = center_y.clamp(0, sh.saturating_sub(1));

    let mut left = cx - half;
    let mut top = cy - half;
    left = left.clamp(0, sw.saturating_sub(size));
    top = top.clamp(0, sh.saturating_sub(size));

    Some((left as u32, top as u32, size as u32, size as u32))
}
