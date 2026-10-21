use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub(crate) fn rect(x: i32, y: i32, w: i32, h: i32) -> RECT {
    RECT {
        left: x,
        top: y,
        right: x.saturating_add(w),
        bottom: y.saturating_add(h),
    }
}

pub(crate) fn work_area(bounds: &RECT) -> RECT {
    unsafe { monitor_work_area(MonitorFromRect(bounds, MONITOR_DEFAULTTONEAREST)) }
}

pub(crate) fn cursor_work_area(cursor: POINT) -> RECT {
    unsafe { monitor_work_area(MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST)) }
}

unsafe fn monitor_work_area(monitor: windows_sys::Win32::Graphics::Gdi::HMONITOR) -> RECT {
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(monitor, &mut info) != 0 {
        info.rcWork
    } else {
        rect(
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        )
    }
}

pub(crate) fn constrain(x: i32, y: i32, w: i32, h: i32, work: &RECT) -> (i32, i32) {
    (
        x.clamp(work.left, work.right.saturating_sub(w).max(work.left)),
        y.clamp(work.top, work.bottom.saturating_sub(h).max(work.top)),
    )
}

pub(crate) fn fit_size(w: i32, h: i32, work: &RECT) -> (i32, i32) {
    let max_w = (work.right - work.left).max(1);
    let max_h = (work.bottom - work.top).max(1);
    let ratio = (max_w as f64 / w.max(1) as f64)
        .min(max_h as f64 / h.max(1) as f64)
        .min(1.0);
    (
        (w as f64 * ratio).round().max(1.0) as i32,
        (h as f64 * ratio).round().max(1.0) as i32,
    )
}

// Carrying changes the canvas, but placement belongs to the standing cat.
// Share this offset for dragging, dropping, and saving so the pose cannot
// change the chosen position. Tail padding is deliberately outside this anchor.
pub(crate) fn carry_offset(normal_w: i32, normal_h: i32, tilt_w: i32, art_h: i32) -> (i32, i32) {
    ((tilt_w - normal_w) / 2, art_h - normal_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_corners_on_monitors_left_above_and_right_of_primary() {
        for work in [
            rect(-1920, 0, 1920, 1040),
            rect(0, -1080, 1920, 1080),
            rect(1920, 120, 1280, 984),
        ] {
            assert_eq!(
                constrain(i32::MIN, i32::MIN, 200, 293, &work),
                (work.left, work.top)
            );
            assert_eq!(
                constrain(i32::MAX, i32::MAX, 200, 293, &work),
                (work.right - 200, work.bottom - 293)
            );
            let inside = (work.left + 100, work.top + 100);
            assert_eq!(constrain(inside.0, inside.1, 200, 293, &work), inside);
        }
    }

    #[test]
    fn carrying_does_not_reserve_extra_space_at_any_corner() {
        let work = rect(-1920, -120, 1920, 1040);
        for (w, h, tw, th) in [(200, 293, 281, 277), (500, 733, 702, 693)] {
            let (dx, dy) = carry_offset(w, h, tw, th);
            for standing in [
                (work.left, work.top),
                (work.right - w, work.top),
                (work.left, work.bottom - h),
                (work.right - w, work.bottom - h),
            ] {
                let carrying = (standing.0 - dx, standing.1 - dy);
                let drop = constrain(carrying.0 + dx, carrying.1 + dy, w, h, &work);
                assert_eq!(drop, standing);
            }
        }
    }

    #[test]
    fn fit_uses_destination_work_area_and_keeps_aspect_ratio() {
        assert_eq!(fit_size(500, 733, &rect(-2560, 0, 2560, 1400)), (500, 733));
        let (w, h) = fit_size(500, 733, &rect(1920, 0, 800, 600));
        assert_eq!(h, 600);
        assert!((w as f64 / h as f64 - 500.0 / 733.0).abs() < 0.002);
    }
}
