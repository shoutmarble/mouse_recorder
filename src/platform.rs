use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

#[cfg(windows)]
fn win_cursor_pos() -> Option<(i32, i32)> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    unsafe {
        let mut p = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut p).is_ok() {
            return Some((p.x, p.y));
        }
    }
    None
}

#[cfg(windows)]
fn win_key_is_down(vk: i32) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    unsafe { (GetAsyncKeyState(vk) as i16) < 0 }
}

#[cfg(windows)]
fn win_screen_size() -> Option<(i32, i32)> {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    if screen_w <= 0 || screen_h <= 0 {
        return None;
    }
    Some((screen_w, screen_h))
}

pub fn get_mouse_pos() -> Option<(i32, i32)> {
    #[cfg(windows)]
    {
        win_cursor_pos()
    }

    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
pub const VK_LBUTTON: i32 = 0x01;
#[cfg(windows)]
pub const VK_RBUTTON: i32 = 0x02;
#[cfg(windows)]
pub const VK_MBUTTON: i32 = 0x04;
#[cfg(windows)]
pub const VK_ESCAPE: i32 = 0x1B;

#[cfg(windows)]
#[derive(Default)]
struct GetCaptureHookState {
    armed: bool,
    captured: Option<(i32, i32, &'static str)>,
    swallow_up_message: Option<u32>,
}

#[cfg(windows)]
static GET_CAPTURE_HOOK_STATE: OnceLock<Arc<Mutex<GetCaptureHookState>>> = OnceLock::new();
#[cfg(windows)]
static GET_CAPTURE_HOOK_STARTED: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
fn get_capture_hook_state() -> &'static Arc<Mutex<GetCaptureHookState>> {
    GET_CAPTURE_HOOK_STATE.get_or_init(|| Arc::new(Mutex::new(GetCaptureHookState::default())))
}

#[cfg(windows)]
pub fn arm_get_capture_hook() {
    if let Ok(mut state) = get_capture_hook_state().lock() {
        state.armed = true;
        state.captured = None;
        state.swallow_up_message = None;
    }
}

#[cfg(windows)]
pub fn disarm_get_capture_hook() {
    if let Ok(mut state) = get_capture_hook_state().lock() {
        state.armed = false;
        state.captured = None;
        state.swallow_up_message = None;
    }
}

#[cfg(windows)]
pub fn take_get_capture_hook_result() -> Option<(i32, i32, &'static str)> {
    let mut state = get_capture_hook_state().lock().ok()?;
    state.captured.take()
}

#[cfg(windows)]
pub fn ensure_get_capture_hook_thread() -> Result<(), String> {
    use std::sync::mpsc;
    use std::thread;
    use windows::Win32::Foundation::{HINSTANCE, HWND};
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
        WH_MOUSE_LL, MSG,
    };

    if GET_CAPTURE_HOOK_STARTED.load(Ordering::Relaxed) {
        return Ok(());
    }

    let (tx, rx) = mpsc::channel::<Result<(), String>>();

    thread::spawn(move || unsafe {
        let hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(get_capture_mouse_hook_proc), HINSTANCE(0), 0) {
            Ok(hook) if !hook.is_invalid() => hook,
            _ => {
                let _ = tx.send(Err("SetWindowsHookExW(WH_MOUSE_LL) failed".to_string()));
                return;
            }
        };

        let _ = tx.send(Ok(()));

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(hook);
    });

    match rx.recv_timeout(Duration::from_millis(800)) {
        Ok(Ok(())) => {
            GET_CAPTURE_HOOK_STARTED.store(true, Ordering::Relaxed);
            Ok(())
        }
        Ok(Err(err)) => Err(err),
        Err(_) => Err("Timed out initializing mouse capture hook".to_string()),
    }
}

#[cfg(windows)]
unsafe extern "system" fn get_capture_mouse_hook_proc(
    code: i32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, HC_ACTION, HHOOK, MSLLHOOKSTRUCT,
        WM_LBUTTONDOWN, WM_LBUTTONUP,
        WM_RBUTTONDOWN, WM_RBUTTONUP,
        WM_MBUTTONDOWN, WM_MBUTTONUP,
    };

    if code == HC_ACTION as i32 {
        let message = wparam.0 as u32;

        if let Ok(mut state) = get_capture_hook_state().lock() {
            if let Some(up_message) = state.swallow_up_message {
                if message == up_message {
                    state.swallow_up_message = None;
                    return LRESULT(1);
                }
            }

            if state.armed {
                let click = match message {
                    WM_LBUTTONDOWN => Some(("Left", WM_LBUTTONUP)),
                    WM_RBUTTONDOWN => Some(("Right", WM_RBUTTONUP)),
                    WM_MBUTTONDOWN => Some(("Middle", WM_MBUTTONUP)),
                    _ => None,
                };

                if let Some((button_name, up_message)) = click {
                    let data = &*(lparam.0 as *const MSLLHOOKSTRUCT);
                    state.captured = Some((data.pt.x, data.pt.y, button_name));
                    state.armed = false;
                    state.swallow_up_message = Some(up_message);
                    return LRESULT(1);
                }
            }
        }
    }

    CallNextHookEx(HHOOK(0), code, wparam, lparam)
}

#[cfg(windows)]
pub fn is_vk_down_windows(vk: i32) -> bool {
    win_key_is_down(vk)
}

#[cfg(windows)]
pub fn capture_patch_png_base64(center_x: i32, center_y: i32, patch_size: u32) -> Result<String, String> {
    use base64::engine::general_purpose;
    use base64::Engine;
    use ::image::{DynamicImage, ImageFormat, RgbaImage};
    use std::io::Cursor;

    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::*;
    let Some((screen_w, screen_h)) = win_screen_size() else {
        return Err("GetSystemMetrics returned invalid screen size".to_string());
    };

    let size = patch_size.max(16) as i32;
    let half = size / 2;
    let mut left = center_x - half;
    let mut top = center_y - half;
    left = left.clamp(0, screen_w - size);
    top = top.clamp(0, screen_h - size);

    unsafe {
        let hdc_screen = GetDC(HWND(0));
        if hdc_screen.is_invalid() {
            return Err("GetDC failed".to_string());
        }
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            let _ = ReleaseDC(HWND(0), hdc_screen);
            return Err("CreateCompatibleDC failed".to_string());
        }

        let hbmp = CreateCompatibleBitmap(hdc_screen, size, size);
        if hbmp.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND(0), hdc_screen);
            return Err("CreateCompatibleBitmap failed".to_string());
        }

        let old = SelectObject(hdc_mem, hbmp);
        if old.is_invalid() {
            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND(0), hdc_screen);
            return Err("SelectObject failed".to_string());
        }

        let ok = BitBlt(
            hdc_mem,
            0,
            0,
            size,
            size,
            hdc_screen,
            left,
            top,
            SRCCOPY,
        )
        .is_ok();
        if !ok {
            let _ = SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND(0), hdc_screen);
            return Err("BitBlt failed".to_string());
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };

        let mut bgra = vec![0u8; (size * size * 4) as usize];

        let lines = GetDIBits(
            hdc_mem,
            hbmp,
            0,
            size as u32,
            Some(bgra.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        if lines == 0 {
            let _ = SelectObject(hdc_mem, old);
            let _ = DeleteObject(hbmp);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND(0), hdc_screen);
            return Err("GetDIBits failed".to_string());
        }

        for px in bgra.chunks_exact_mut(4) {
            let b = px[0];
            let g = px[1];
            let r = px[2];
            let a = px[3];
            px[0] = r;
            px[1] = g;
            px[2] = b;
            px[3] = a;
        }

        let rgba: RgbaImage =
            RgbaImage::from_raw(size as u32, size as u32, bgra).ok_or("Image buffer failed")?;

        let mut png = Vec::new();
        DynamicImage::ImageRgba8(rgba)
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .map_err(|e| e.to_string())?;

        let _ = SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbmp);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND(0), hdc_screen);

        Ok(general_purpose::STANDARD.encode(png))
    }
}

#[cfg(not(windows))]
pub fn capture_patch_png_base64(_center_x: i32, _center_y: i32, _patch_size: u32) -> Result<String, String> {
    Err("Image-search click recording is currently Windows-only".to_string())
}
