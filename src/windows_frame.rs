//! Native resize hit testing for the custom title bar.
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            GetWindowRect, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCLIENT, HTLEFT, HTRIGHT, HTTOP,
            HTTOPLEFT, HTTOPRIGHT, IsZoomed, WM_NCDESTROY, WM_NCHITTEST,
        },
    },
};

pub fn install_resize_border(window: &impl HasWindowHandle) -> anyhow::Result<()> {
    let handle = window.window_handle()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Ok(());
    };
    let hwnd = handle.hwnd.get() as HWND;
    // Creation and destruction run on the owning event-loop thread.
    if unsafe { SetWindowSubclass(hwnd, Some(resize_border), 1, 0) } == 0 {
        anyhow::bail!("Unable to install the native resize border");
    }
    Ok(())
}

unsafe extern "system" fn resize_border(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    _data: usize,
) -> LRESULT {
    if msg == WM_NCDESTROY {
        unsafe { RemoveWindowSubclass(hwnd, Some(resize_border), id) };
    }
    let hit = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
    if msg != WM_NCHITTEST || hit != HTCLIENT as LRESULT || unsafe { IsZoomed(hwnd) } != 0 {
        return hit;
    }
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetWindowRect(hwnd, &mut rect) } == 0 {
        return hit;
    }
    // WM_NCHITTEST carries signed screen coordinates (including secondary monitors).
    let x = lparam as i16 as i32;
    let y = (lparam >> 16) as i16 as i32;
    let axis = |p, min, max| {
        if p < min + 6 {
            -1
        } else if p >= max - 6 {
            1
        } else {
            0
        }
    };
    (match (
        axis(x, rect.left, rect.right),
        axis(y, rect.top, rect.bottom),
    ) {
        (-1, -1) => HTTOPLEFT,
        (1, -1) => HTTOPRIGHT,
        (-1, 1) => HTBOTTOMLEFT,
        (1, 1) => HTBOTTOMRIGHT,
        (-1, _) => HTLEFT,
        (1, _) => HTRIGHT,
        (_, -1) => HTTOP,
        (_, 1) => HTBOTTOM,
        _ => HTCLIENT,
    }) as LRESULT
}
