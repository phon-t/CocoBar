#![windows_subsystem = "windows"]

use image::imageops::FilterType;
use image::{load_from_memory_with_format, ImageFormat};
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::Mutex;
use std::time::Instant;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, BeginPaint, BitBlt,
    CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, CreatePen,
    CreateRoundRectRgn, CreateSolidBrush, DeleteObject, DIB_RGB_COLORS, DrawTextW, Ellipse, EndPaint, FillRect,
    GetDC, GetStockObject, InvalidateRect, LineTo, MoveToEx, PAINTSTRUCT, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
    DeleteDC, FillRgn, FrameRgn,
    SetWindowRgn, DEFAULT_GUI_FONT, RGBQUAD, PS_SOLID, SRCCOPY, DT_CENTER,
    DT_SINGLELINE, DT_VCENTER,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW};
use windows_sys::Win32::System::SystemServices::{SS_CENTERIMAGE, SS_LEFT};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, WaitForSingleObject, CREATE_NO_WINDOW, INFINITE, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows_sys::Win32::UI::Controls::{
    BST_CHECKED, InitCommonControlsEx, INITCOMMONCONTROLSEX, ICC_BAR_CLASSES, TBM_SETLINESIZE, TBM_SETPAGESIZE,
    TBM_SETPOS, TBM_SETRANGEMAX, TBM_SETRANGEMIN, TBM_SETTICFREQ, TBS_AUTOTICKS, TBS_HORZ,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetFocus, MOD_ALT, MOD_CONTROL, RegisterHotKey, ReleaseCapture, SetCapture, VK_ESCAPE,
};
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFYICONDATAW, NOTIFYICON_VERSION_4, SHGetFolderPathW, Shell_NotifyIconW, CSIDL_DESKTOPDIRECTORY,
    CSIDL_STARTUP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, BM_SETCHECK, BS_AUTOCHECKBOX, CreateIconIndirect, CreatePopupMenu,
    CreateWindowExW, CS_DBLCLKS, DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetDlgItem, GetParent, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect,
    GWLP_USERDATA, ICONINFO, IDC_ARROW, IDI_APPLICATION, KillTimer, LoadCursorW, LoadIconW,
    MF_CHECKED, MF_POPUP, MF_STRING, MF_UNCHECKED, PostQuitMessage, RegisterClassExW, SendMessageW,
    SetForegroundWindow, SetProcessDPIAware, SetTimer, SetWindowLongPtrW, SetWindowPos,
    SetWindowTextW, ShowWindow, SM_CXSCREEN, SM_CYSCREEN, SWP_NOZORDER, TPM_RETURNCMD, TPM_RIGHTBUTTON,
    TrackPopupMenu, TranslateMessage, UpdateLayeredWindow, ULW_ALPHA, WNDCLASSEXW, WM_ACTIVATE, WM_APP,
    WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_ENDSESSION, WM_ERASEBKGND, WM_HOTKEY, WM_HSCROLL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_QUERYENDSESSION,
    WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_MOVE, WM_PAINT,
    WM_RBUTTONUP, WM_SETFONT, WM_TIMER, WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_TABSTOP, WS_VISIBLE, SWP_NOSIZE, SWP_NOACTIVATE,
};

const TRAY_ID: usize = 1;
const TIMER_ID: usize = 2;
const TIMER_MENU_CLOSE: usize = 3;
const FRAME_MS: u32 = 16;

const MENU_BLACK: usize = 101;
const MENU_WHITE: usize = 102;
const MENU_SIZE_SMALL: usize = 103;
const MENU_SIZE_MEDIUM: usize = 104;
const MENU_SIZE_LARGE: usize = 105;
const MENU_EXIT: usize = 107;
const MENU_SHORTCUT_DESKTOP: usize = 108;
const MENU_STARTUP: usize = 109;

const HK_BLACK: i32 = 1;
const HK_WHITE: i32 = 2;
const HK_SMALL: i32 = 3;
const HK_MEDIUM: i32 = 4;
const HK_LARGE: i32 = 5;

const CTRL_SIZE: usize = 201;
const CTRL_SIZE_VAL: usize = 202;
const CTRL_BLACK: usize = 203;
const CTRL_WHITE: usize = 204;
const CTRL_EXIT: usize = 208;

const ED_REM_MSG: usize = 304;
const ED_REM_TIME: usize = 305;
const BTN_REM_SET: usize = 306;
const LB_REM: usize = 307;
const BTN_REM_DEL: usize = 308;
const CTRL_STATUS: usize = 309;
const CTRL_STARTUP: usize = 206;
const ED_NOTE: usize = 310;
const BTN_NOTE_SAVE: usize = 311;
const ED_TODO: usize = 312;
const BTN_TODO_ADD: usize = 313;
const LB_TODO: usize = 314;
const BTN_TODO_TOGGLE: usize = 315;
const BTN_TODO_DEL: usize = 316;

const MENU_W: i32 = 470;
const MENU_H: i32 = 430;
const SIZE_MIN: i32 = 100;
const SIZE_MAX: i32 = 760;
const SIZE_STEP: i32 = 40;
const TBM_GETPOS: u32 = 1024;

const SIZES: [i32; 3] = [320, 520, 760];
const CONTENT_X: i32 = 364;
const CONTENT_Y: i32 = 148;
const CONTENT_W: i32 = 1192;
const CONTENT_H: i32 = 1748;
const MAX_OFF_X: f32 = 77.0;
const MAX_OFF_Y: f32 = 87.0;
const TRAY_CB: u32 = WM_APP + 2;

const BLACK_SOCKET: &[u8] = include_bytes!("../assets/blackcatwitheyesocket.png");
const WHITE_SOCKET: &[u8] = include_bytes!("../assets/whitecatwitheyesocket.png");
const BLACK_CLOSED: &[u8] = include_bytes!("../assets/blackcateyesclossed.png");
const WHITE_CLOSED: &[u8] = include_bytes!("../assets/whitecateyesclossed.png");
const EYES: &[u8] = include_bytes!("../assets/eyes.png");
const HIGHLIGHT: &[u8] = include_bytes!("../assets/highlight.png");
const BLACK_FULL: &[u8] = include_bytes!("../assets/blackcatfull.png");
const WHITE_FULL: &[u8] = include_bytes!("../assets/whitecatfull.png");
const BLACK_ANNOYED: &[u8] = include_bytes!("../assets/blackcatannoyed.png");
const WHITE_ANNOYED: &[u8] = include_bytes!("../assets/whitecatannoyed.png");

struct Layer {
    w: usize,
    h: usize,
    data: Vec<u8>,
}

impl Layer {
    fn new(w: usize, h: usize) -> Self {
        Layer {
            w,
            h,
            data: vec![0; w * h * 4],
        }
    }
}

struct App {
    hwnd: HWND,
    base: Layer,
    closed: Layer,
    eyes: Layer,
    hl: Layer,
    buf: Vec<u8>,
    w: i32,
    h: i32,
    scale: f32,
    pos_x: i32,
    pos_y: i32,
    off_x: f32,
    off_y: f32,
    color: usize,
    size_idx: usize,
    hdc_screen: *mut c_void,
    hdc_mem: *mut c_void,
    hbmp: *mut c_void,
    bits: *mut c_void,
    blink_start: Option<Instant>,
    blink_next: Instant,
    wander_t: f32,
    cursor_last: POINT,
    cursor_still: f32,
    icon: *mut c_void,
    config: PathBuf,
    menu_hwnd: HWND,
    exe: PathBuf,
    menu_tab: u32,
    reminders: Vec<(String, u64)>,
    note: String,
    todos: Vec<(String, bool)>,
    drag_active: bool,
    drag_win_x: i32,
    drag_win_y: i32,
    drag_mouse_x: i32,
    drag_mouse_y: i32,
    annoyed_until: Option<Instant>,
    prev_mouse_down: bool,
    cached_rgba: Vec<u8>,
    cached_w: u32,
    cached_h: u32,
    menu_timer_id: u32,
}

fn rgba_to_layer(rgba: Vec<u8>, w: usize, h: usize) -> Layer {
    let mut l = Layer::new(w, h);
    let mut i = 0usize;
    while i + 3 < rgba.len() {
        let r = rgba[i] as u32;
        let g = rgba[i + 1] as u32;
        let b = rgba[i + 2] as u32;
        let a = rgba[i + 3] as u32;
        l.data[i] = (b * a / 255) as u8;
        l.data[i + 1] = (g * a / 255) as u8;
        l.data[i + 2] = (r * a / 255) as u8;
        l.data[i + 3] = a as u8;
        i += 4;
    }
    l
}

fn load_layer(bytes: &[u8], crop: (i32, i32, i32, i32), scale: f32) -> Layer {
    let mut img = load_from_memory_with_format(bytes, ImageFormat::Png)
        .expect("png decode")
        .to_rgba8();
    let mut cropped = image::imageops::crop(
        &mut img,
        crop.0 as u32,
        crop.1 as u32,
        crop.2 as u32,
        crop.3 as u32,
    )
    .to_image();
    let dw = ((crop.2 as f32) * scale).round().max(1.0) as u32;
    let dh = ((crop.3 as f32) * scale).round().max(1.0) as u32;
    let resized = image::imageops::resize(&mut cropped, dw, dh, FilterType::Triangle);
    let (w, h) = resized.dimensions();
    rgba_to_layer(resized.into_raw(), w as usize, h as usize)
}

fn decode_and_crop(
    bytes: &[u8],
    crop: (i32, i32, i32, i32),
) -> (Vec<u8>, u32, u32) {
    let img = load_from_memory_with_format(bytes, ImageFormat::Png)
        .expect("png decode")
        .to_rgba8();
    let cropped = image::imageops::crop(
        &mut img.clone(),
        crop.0 as u32,
        crop.1 as u32,
        crop.2 as u32,
        crop.3 as u32,
    )
    .to_image();
    let (w, h) = cropped.dimensions();
    (cropped.into_raw(), w, h)
}

fn load_layer_cached(
    cached_rgba: &[u8],
    cached_w: u32,
    cached_h: u32,
    scale: f32,
) -> Layer {
    let dw = (cached_w as f32 * scale).round().max(1.0) as u32;
    let dh = (cached_h as f32 * scale).round().max(1.0) as u32;
    let mut img = image::RgbaImage::from_raw(cached_w, cached_h, cached_rgba.to_vec()).unwrap();
    let resized = image::imageops::resize(&mut img, dw, dh, FilterType::Triangle);
    let (w, h) = resized.dimensions();
    rgba_to_layer(resized.into_raw(), w as usize, h as usize)
}

fn blit(dst: &mut [u8], dw: usize, dh: usize, src: &[u8], sw: usize, sh: usize, dx: i32, dy: i32) {
    let x0 = dx.max(0) as usize;
    let y0 = dy.max(0) as usize;
    let x1 = ((dx + sw as i32).min(dw as i32)).max(0) as usize;
    let y1 = ((dy + sh as i32).min(dh as i32)).max(0) as usize;
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let sx0 = (x0 as i32 - dx).max(0) as usize;
    let sy0 = (y0 as i32 - dy).max(0) as usize;
    for y in y0..y1 {
        let sy = sy0 + (y - y0);
        let row_d = y * dw;
        let row_s = sy * sw;
        for x in x0..x1 {
            let sx = sx0 + (x - x0);
            let di = (row_d + x) * 4;
            let si = (row_s + sx) * 4;
            let a = src[si + 3] as u32;
            if a == 0 {
                continue;
            }
            if a == 255 {
                dst[di..di + 4].copy_from_slice(&src[si..si + 4]);
            } else {
                let inv = 255 - a;
                dst[di] = (src[si] as u32 + dst[di] as u32 * inv / 255) as u8;
                dst[di + 1] = (src[si + 1] as u32 + dst[di + 1] as u32 * inv / 255) as u8;
                dst[di + 2] = (src[si + 2] as u32 + dst[di + 2] as u32 * inv / 255) as u8;
                dst[di + 3] = (src[si + 3] as u32 + dst[di + 3] as u32 * inv / 255) as u8;
            }
        }
    }
}

fn crossfade(buf: &mut [u8], src: &[u8], t: f32) {
    let tb = (t.clamp(0.0, 1.0) * 255.0) as u32;
    let inv = 255 - tb;
    let mut i = 0usize;
    while i + 3 < buf.len() {
        buf[i] = (buf[i] as u32 * inv / 255 + src[i] as u32 * tb / 255) as u8;
        buf[i + 1] = (buf[i + 1] as u32 * inv / 255 + src[i + 1] as u32 * tb / 255) as u8;
        buf[i + 2] = (buf[i + 2] as u32 * inv / 255 + src[i + 2] as u32 * tb / 255) as u8;
        buf[i + 3] = (buf[i + 3] as u32 * inv / 255 + src[i + 3] as u32 * tb / 255) as u8;
        i += 4;
    }
}

impl App {
    fn rebuild_layers(&mut self) {
        let s = self.scale;
        let crop = (CONTENT_X, CONTENT_Y, CONTENT_W, CONTENT_H);
        let (base_bytes, closed_bytes) = if self.color == 0 {
            (BLACK_SOCKET, BLACK_CLOSED)
        } else {
            (WHITE_SOCKET, WHITE_CLOSED)
        };
        if self.cached_rgba.is_empty() || self.cached_w == 0 || self.cached_h == 0 {
            let (rgba, w, h) = decode_and_crop(base_bytes, crop);
            self.cached_rgba = rgba;
            self.cached_w = w;
            self.cached_h = h;
        }
        self.base = load_layer_cached(&self.cached_rgba, self.cached_w, self.cached_h, s);
        self.closed = load_layer(closed_bytes, crop, s);
        self.eyes = load_layer(EYES, crop, s);
        self.hl = load_layer(HIGHLIGHT, crop, s);
        self.buf = vec![0; (self.w * self.h * 4) as usize];
    }

    fn rebuild_dib(&mut self) {
        unsafe {
            if !self.hbmp.is_null() {
                DeleteObject(self.hbmp);
            }
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: self.w,
                    biHeight: -self.h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB,
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
                }],
            };
            self.hbmp = CreateDIBSection(
                self.hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut self.bits as *mut *mut c_void,
                null_mut(),
                0,
            );
            SelectObject(self.hdc_mem, self.hbmp);
        }
    }

    fn rebuild_icon(&mut self) {
        unsafe {
            if !self.icon.is_null() {
                DestroyIcon(self.icon);
            }
        }
        let img_bytes = if self.color == 0 { BLACK_FULL } else { WHITE_FULL };
        let mut img = load_from_memory_with_format(img_bytes, ImageFormat::Png)
            .expect("icon decode")
            .to_rgba8();
        let resized = image::imageops::resize(&mut img, 32, 32, FilterType::Triangle);
        let raw = resized.into_raw();
        let mut pixels = Vec::with_capacity(32 * 32 * 4);
        let mut i = 0usize;
        while i + 3 < raw.len() {
            pixels.push(raw[i + 2]);
            pixels.push(raw[i + 1]);
            pixels.push(raw[i]);
            pixels.push(raw[i + 3]);
            i += 4;
        }
        let hdc = unsafe { GetDC(null_mut()) };
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 32,
                biHeight: -32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
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
            }],
        };
        let mut bits: *mut c_void = null_mut();
        let color_bitmap = unsafe {
            CreateDIBSection(
                hdc,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits as *mut *mut c_void,
                null_mut(),
                0,
            )
        };
        unsafe {
            if !bits.is_null() {
                std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());
            }
            let info = ICONINFO {
                fIcon: 1,
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: null_mut(),
                hbmColor: color_bitmap,
            };
            self.icon = CreateIconIndirect(&info);
            if !color_bitmap.is_null() {
                DeleteObject(color_bitmap);
            }
            ReleaseDC(null_mut(), hdc);
        }
    }

    fn set_width(&mut self, mut w: i32) {
        w = w.clamp(SIZE_MIN, SIZE_MAX);
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let max_w = (sw - 40).max(SIZE_MIN);
        let max_h = (sh - 60).max(120);
        let w = w.min(max_w);
        let mut h = ((w as f32) * CONTENT_H as f32 / CONTENT_W as f32).round() as i32;
        let old_r = self.pos_x + self.w;
        let old_b = self.pos_y + self.h;
        if h > max_h {
            h = max_h;
            let w_prop = ((h as f32) * CONTENT_W as f32 / CONTENT_H as f32).round() as i32;
            self.w = w_prop;
        } else {
            self.w = w;
        }
        self.h = h;
        self.scale = self.w as f32 / CONTENT_W as f32;
        self.size_idx = nearest_preset_index(self.w);
        self.pos_x = old_r - self.w;
        self.pos_y = old_b - self.h;
        if self.pos_y < 0 {
            self.pos_y = 0;
        }
        unsafe {
            SetWindowPos(
                self.hwnd,
                null_mut(),
                self.pos_x,
                self.pos_y,
                self.w,
                self.h,
                SWP_NOZORDER,
            );
        }
        self.rebuild_layers();
        self.rebuild_dib();
        self.sync_menu_size();
    }

    fn set_size(&mut self, idx: usize) {
        self.size_idx = idx.min(2);
        let old_r = self.pos_x + self.w;
        let old_b = self.pos_y + self.h;
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let max_w = (sw - 40).max(120);
        let max_h = (sh - 60).max(120);
        let mut w = SIZES[idx].min(max_w);
        let mut h = ((w as f32) * CONTENT_H as f32 / CONTENT_W as f32).round() as i32;
        if h > max_h {
            h = max_h;
            w = ((h as f32) * CONTENT_W as f32 / CONTENT_H as f32).round() as i32;
        }
        self.w = w;
        self.h = h;
        self.scale = w as f32 / CONTENT_W as f32;
        self.pos_x = old_r - self.w;
        self.pos_y = old_b - self.h;
        if self.pos_y < 0 {
            self.pos_y = 0;
        }
        unsafe {
            SetWindowPos(
                self.hwnd,
                null_mut(),
                self.pos_x,
                self.pos_y,
                self.w,
                self.h,
                SWP_NOZORDER,
            );
        }
        self.rebuild_layers();
        self.rebuild_dib();
        self.sync_menu_size();
    }

    fn set_color(&mut self, c: usize) {
        if self.color == c {
            return;
        }
        self.color = c;
        self.cached_rgba.clear();
        self.cached_w = 0;
        self.cached_h = 0;
        self.rebuild_layers();
        self.rebuild_icon();
        self.update_tray();
    }

    fn show_menu(&mut self) {
        unsafe {
            if !self.menu_hwnd.is_null() {
                SetForegroundWindow(self.menu_hwnd);
                return;
            }
            let sw = GetSystemMetrics(SM_CXSCREEN);
            let sh = GetSystemMetrics(SM_CYSCREEN);
            let mut mx = self.pos_x + self.w + 8;
            if mx + MENU_W > sw - 8 {
                mx = self.pos_x - MENU_W - 8;
            }
            if mx < 8 {
                mx = (sw - MENU_W) / 2;
            }
            let mut my = self.pos_y + (self.h - MENU_H) / 2;
            if my < 8 {
                my = 8;
            }
            if my + MENU_H > sh - 8 {
                my = (sh - MENU_H - 8).max(8);
            }
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
                wstr("CatMenuWnd"),
                wstr("cocoBar"),
                WS_POPUP | WS_VISIBLE,
                mx,
                my,
                MENU_W,
                MENU_H,
                self.hwnd,
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            );
            if hwnd.is_null() {
                return;
            }
            let rgn = CreateRoundRectRgn(0, 0, MENU_W + 1, MENU_H + 1, 30, 30);
            SetWindowRgn(hwnd, rgn, 1);
            self.menu_hwnd = hwnd;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, self as *mut App as isize);
            self.create_menu_controls(hwnd);
            self.show_tab_controls();
            self.sync_menu_states();
            ShowWindow(hwnd, 1);
            SetForegroundWindow(hwnd);
        }
    }

    fn create_menu_controls(&self, hwnd: HWND) {
        unsafe {
            let font = GetStockObject(DEFAULT_GUI_FONT) as _;
            let hinst = GetModuleHandleW(null());
            let ctl = |id: usize,
                       class: &str,
                       title: &str,
                       x: i32,
                       y: i32,
                       w: i32,
                       h: i32,
                       style: u32|
             -> HWND {
                let c = CreateWindowExW(
                    0,
                    wstr(class),
                    wstr(title),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | style,
                    x,
                    y,
                    w,
                    h,
                    hwnd,
                    id as _,
                    hinst,
                    null(),
                );
                SendMessageW(c, WM_SETFONT, font, 1);
                c
            };
            // ---- Reminders tab ----
            ctl(ED_REM_MSG, "EDIT", "Reminder message", 16, 120, 238, 28, 0x0080);
            ctl(ED_REM_TIME, "EDIT", "14:30", 262, 120, 54, 28, 0x0080);
            ctl(BTN_REM_SET, "BUTTON", "Set Reminder", 324, 120, 122, 28, 0);
            ctl(LB_REM, "LISTBOX", "", 16, 158, 430, 120, 0x0001 | 0x00800000 | 0x00200000);
            ctl(BTN_REM_DEL, "BUTTON", "Remove selected", 16, 288, 150, 30, 0);
            // ---- Notes tab ----
            ctl(ED_NOTE, "EDIT", "", 16, 120, 430, 150, 0x0004 | 0x0040 | 0x1000 | 0x00200000 | 0x0100 /*ES_WANTRETURN*/);
            ctl(BTN_NOTE_SAVE, "BUTTON", "Save Note", 16, 282, 120, 30, 0);
            // ---- To-do tab ----
            ctl(ED_TODO, "EDIT", "New task...", 16, 120, 296, 28, 0x0080);
            ctl(BTN_TODO_ADD, "BUTTON", "Add", 320, 118, 60, 30, 0);
            ctl(LB_TODO, "LISTBOX", "", 16, 158, 430, 120, 0x0001 | 0x00800000 | 0x00200000);
            ctl(BTN_TODO_TOGGLE, "BUTTON", "Mark done / undo", 16, 288, 150, 30, 0);
            ctl(BTN_TODO_DEL, "BUTTON", "Remove", 176, 288, 90, 30, 0);
            // ---- shared status line ----
            ctl(CTRL_STATUS, "STATIC", "", 16, 326, 438, 20, SS_LEFT);
            // ---- footer (inside card, no more black gap) ----
            ctl(0, "STATIC", "Size", 16, 360, 40, 22, SS_CENTERIMAGE);
            let tb = ctl(CTRL_SIZE, "msctls_trackbar32", "", 60, 356, 150, 24, TBS_HORZ | TBS_AUTOTICKS);
            SendMessageW(tb, TBM_SETRANGEMIN, 1, SIZE_MIN as _);
            SendMessageW(tb, TBM_SETRANGEMAX, 1, SIZE_MAX as _);
            SendMessageW(tb, TBM_SETTICFREQ, 2, 0);
            SendMessageW(tb, TBM_SETPAGESIZE, 1, (SIZE_STEP * 2) as _);
            SendMessageW(tb, TBM_SETLINESIZE, 0, SIZE_STEP as _);
            SendMessageW(tb, TBM_SETPOS, 1, self.w.max(SIZE_MIN).min(SIZE_MAX) as _);
            ctl(CTRL_SIZE_VAL, "STATIC", "", 218, 360, 60, 22, SS_CENTERIMAGE);
            ctl(CTRL_BLACK, "BUTTON", "Black", 286, 356, 62, 26, 0);
            ctl(CTRL_WHITE, "BUTTON", "White", 352, 356, 62, 26, 0);
            ctl(CTRL_STARTUP, "BUTTON", "Start with Windows", 16, 390, 180, 26, BS_AUTOCHECKBOX as u32);
            ctl(CTRL_EXIT, "BUTTON", "Exit", 390, 390, 62, 26, 0);
            // fill controls with saved state
            self.refresh_lists();
        }
    }

    fn refresh_lists(&self) {
        if self.menu_hwnd.is_null() {
            return;
        }
        unsafe {
            let fill_lb = |id: usize, items: &[(String, Option<u64>)]| {
                let lb = GetDlgItem(self.menu_hwnd, id as i32);
                if lb.is_null() {
                    return;
                }
                SendMessageW(lb, 0x018B, 0, 0); // LB_RESETCONTENT
                for (text, ts) in items {
                    let line = match ts {
                        Some(t) => format!("{}  \u{2013}  {}", fmt_time(*t), text),
                        None => format!("{}", text),
                    };
                    SendMessageW(lb, 0x0180, 0, wstr(&line) as _); // LB_ADDSTRING
                }
            };
            let rems: Vec<(String, Option<u64>)> = self
                .reminders
                .iter()
                .map(|(m, t)| (m.clone(), Some(*t)))
                .collect();
            fill_lb(LB_REM, &rems);
            let todos: Vec<(String, Option<u64>)> = self
                .todos
                .iter()
                .map(|(t, d)| (format!("[{}] {}", if *d { "x" } else { " " }, t), None))
                .collect();
            fill_lb(LB_TODO, &todos);
            let ed = GetDlgItem(self.menu_hwnd, ED_NOTE as i32);
            if !ed.is_null() {
                SetWindowTextW(ed, wstr(&self.note));
            }
            let st = GetDlgItem(self.menu_hwnd, CTRL_STARTUP as i32);
            if !st.is_null() {
                SendMessageW(st, BM_SETCHECK, if self.startup_enabled() { BST_CHECKED as _ } else { 0 }, 0);
            }
        }
    }

    fn sync_menu_size(&self) {
        if self.menu_hwnd.is_null() {
            return;
        }
        unsafe {
            let tb = GetDlgItem(self.menu_hwnd, CTRL_SIZE as i32);
            if !tb.is_null() {
                SendMessageW(tb, TBM_SETPOS, 1, self.w.max(SIZE_MIN).min(SIZE_MAX) as _);
            }
            let val = GetDlgItem(self.menu_hwnd, CTRL_SIZE_VAL as i32);
            if !val.is_null() {
                SetWindowTextW(val, wstr(&format!("{} px", self.w)));
            }
        }
    }

    fn sync_menu_states(&self) {
        if self.menu_hwnd.is_null() {
            return;
        }
        unsafe {
            let set_chk = |id: usize, on: bool| {
                let c = GetDlgItem(self.menu_hwnd, id as i32);
                if !c.is_null() {
                    SendMessageW(c, BM_SETCHECK, if on { BST_CHECKED as _ } else { 0 }, 0);
                }
            };
            set_chk(CTRL_STARTUP, self.startup_enabled());
        }
        self.sync_menu_size();
    }

    fn app_dir(&self) -> PathBuf {
        self.config.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    }

    fn ensure_icon(&self) -> PathBuf {
        let ico = self.app_dir().join("cat.ico");
        if ico.exists() {
            return ico;
        }
        let _ = std::fs::create_dir_all(self.app_dir());
        let mut img = load_from_memory_with_format(BLACK_FULL, ImageFormat::Png)
            .expect("icon decode")
            .to_rgba8();
        let resized = image::imageops::resize(&mut img, 32, 32, FilterType::Triangle);
        let raw = resized.into_raw();
        let mut out: Vec<u8> = Vec::with_capacity(22 + 40 + 4096 + 128);
        out.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
        out.extend_from_slice(&[32, 32, 0, 0, 1, 0, 32, 0]);
        let xor_len = 4096u32;
        let size = 40u32 + xor_len + 128;
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&22u32.to_le_bytes());
        out.extend_from_slice(&40u32.to_le_bytes());
        out.extend_from_slice(&32i32.to_le_bytes());
        out.extend_from_slice(&64i32.to_le_bytes());
        out.extend_from_slice(&[1, 0]);
        out.extend_from_slice(&[32, 0]);
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&xor_len.to_le_bytes());
        out.extend_from_slice(&[0u8; 16]);
        for row in (0..32).rev() {
            for col in 0..32 {
                let i = (row * 32 + col) * 4;
                out.extend_from_slice(&[raw[i + 2], raw[i + 1], raw[i], raw[i + 3]]);
            }
        }
        out.extend_from_slice(&[0u8; 128]);
        let _ = std::fs::write(&ico, out);
        ico
    }

    fn create_desktop_shortcut(&self) {
        let Some(desktop) = known_folder(CSIDL_DESKTOPDIRECTORY) else {
            return;
        };
        let ico = self.ensure_icon();
        let lnk = desktop.join("cocoBar.lnk");
        let exe = self.exe.to_string_lossy().to_string();
        let dir = self.exe.parent().unwrap().to_string_lossy().to_string();
        let ico_str = format!("{},0", ico.to_string_lossy());
        let script = format!(
            "$ws = New-Object -ComObject WScript.Shell; \
             $s = $ws.CreateShortcut({}); \
             $s.TargetPath = {}; \
             $s.WorkingDirectory = {}; \
             $s.IconLocation = {}; \
             $s.Description = 'cocoBar'; \
             $s.Save()",
            ps_quote(&lnk.to_string_lossy()),
            ps_quote(&exe),
            ps_quote(&dir),
            ps_quote(&ico_str),
        );
        run_powershell(&script);
    }

    fn startup_lnk(&self) -> Option<PathBuf> {
        known_folder(CSIDL_STARTUP).map(|p| p.join("cocoBar.lnk"))
    }

    fn startup_enabled(&self) -> bool {
        self.startup_lnk().map(|p| p.exists()).unwrap_or(false)
    }

    fn set_startup(&self, on: bool) {
        let Some(lnk) = self.startup_lnk() else {
            return;
        };
        if !on {
            let _ = std::fs::remove_file(&lnk);
            return;
        }
        let ico = self.ensure_icon();
        let exe = self.exe.to_string_lossy().to_string();
        let dir = self.exe.parent().unwrap().to_string_lossy().to_string();
        let ico_str = format!("{},0", ico.to_string_lossy());
        let script = format!(
            "$ws = New-Object -ComObject WScript.Shell; \
             $s = $ws.CreateShortcut({}); \
             $s.TargetPath = {}; \
             $s.WorkingDirectory = {}; \
             $s.IconLocation = {}; \
             $s.Description = 'cocoBar'; \
             $s.Save()",
            ps_quote(&lnk.to_string_lossy()),
            ps_quote(&exe),
            ps_quote(&dir),
            ps_quote(&ico_str),
        );
        run_powershell(&script);
    }


    fn update_tray(&self) {
        unsafe {
            let mut nid = std::mem::zeroed::<NOTIFYICONDATAW>();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = TRAY_ID as u32;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = TRAY_CB;
            nid.hIcon = self.icon;
            let tip: Vec<u16> = "cocoBar".encode_utf16().collect();
            for (i, c) in tip.iter().enumerate().take(127) {
                nid.szTip[i] = *c;
            }
            Shell_NotifyIconW(NIM_ADD, &nid);
            nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            Shell_NotifyIconW(NIM_SETVERSION, &nid);
        }
    }

    fn popup_menu(&mut self) {
        unsafe {
            let menu = CreatePopupMenu();
            AppendMenuW(
                menu,
                MF_STRING | if self.color == 0 { MF_CHECKED } else { MF_UNCHECKED },
                MENU_BLACK,
                wstr("Black Cat"),
            );
            AppendMenuW(
                menu,
                MF_STRING | if self.color == 1 { MF_CHECKED } else { MF_UNCHECKED },
                MENU_WHITE,
                wstr("White Cat"),
            );
            AppendMenuW(menu, MF_STRING, 0, wstr(""));
            let size_menu = CreatePopupMenu();
            for (i, label) in ["Small", "Medium", "Large"].iter().enumerate() {
                AppendMenuW(
                    size_menu,
                    MF_STRING | if self.size_idx == i { MF_CHECKED } else { MF_UNCHECKED },
                    MENU_SIZE_SMALL + i,
                    wstr(label),
                );
            }
            AppendMenuW(menu, MF_POPUP, size_menu as usize, wstr("Size"));
            AppendMenuW(menu, MF_STRING, 0, wstr(""));
            AppendMenuW(
                menu,
                MF_STRING,
                MENU_SHORTCUT_DESKTOP,
                wstr("Add shortcut to Desktop"),
            );
            AppendMenuW(
                menu,
                MF_STRING | if self.startup_enabled() { MF_CHECKED } else { MF_UNCHECKED },
                MENU_STARTUP,
                wstr("Start with Windows"),
            );
            AppendMenuW(menu, MF_STRING, 0, wstr(""));
            AppendMenuW(menu, MF_STRING, MENU_EXIT, wstr("Exit"));
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            SetForegroundWindow(self.hwnd);
            let cmd = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, pt.x, pt.y, 0, self.hwnd, null());
            if cmd != 0 {
                SendMessageW(self.hwnd, WM_COMMAND, cmd as WPARAM, 0);
            }
            DestroyMenu(menu);
        }
    }

    fn command(&mut self, id: usize) {
        match id {
            MENU_BLACK => self.set_color(0),
            MENU_WHITE => self.set_color(1),
            MENU_SIZE_SMALL | MENU_SIZE_MEDIUM | MENU_SIZE_LARGE => {
                self.set_size(id - MENU_SIZE_SMALL)
            }
            MENU_SHORTCUT_DESKTOP => self.create_desktop_shortcut(),
            MENU_STARTUP => {
                self.set_startup(!self.startup_enabled());
                self.sync_menu_states();
            }
            MENU_EXIT => unsafe {
                DestroyWindow(self.hwnd);
            },
            _ => {}
        }
    }

    fn render(&mut self) {
        let now = Instant::now();

        // Detect single-click on cat (GetAsyncKeyState since WS_EX_NOACTIVATE blocks WM_LBUTTONDOWN)
        let mouse_down = unsafe { GetAsyncKeyState(0x01) } as u16 & 0x8000 != 0;
        if mouse_down && !self.prev_mouse_down {
            let mut pt = POINT { x: 0, y: 0 };
            if unsafe { GetCursorPos(&mut pt) } != 0 {
                if pt.x >= self.pos_x
                    && pt.x <= self.pos_x + self.w
                    && pt.y >= self.pos_y
                    && pt.y <= self.pos_y + self.h
                {
                    self.annoyed_until = Some(now + std::time::Duration::from_millis(1500));
                }
            }
        }
        self.prev_mouse_down = mouse_down;

        // If annoyed, render the annoyed full-image instead of normal composite
        if let Some(until) = self.annoyed_until {
            if now < until {
                let annoyed_bytes = if self.color == 0 { BLACK_ANNOYED } else { WHITE_ANNOYED };
                let crop = (CONTENT_X, CONTENT_Y, CONTENT_W, CONTENT_H);
                let layer = load_layer(annoyed_bytes, crop, self.scale);
                self.buf.fill(0);
                let dw = self.w as usize;
                let dh = self.h as usize;
                if layer.w == dw && layer.h == dh {
                    self.buf.copy_from_slice(&layer.data);
                } else {
                    blit(&mut self.buf, dw, dh, &layer.data, layer.w, layer.h, 0, 0);
                }
                unsafe {
                    if !self.bits.is_null() {
                        std::ptr::copy_nonoverlapping(self.buf.as_ptr(), self.bits as *mut u8, self.buf.len());
                        let pt_dst = POINT { x: self.pos_x, y: self.pos_y };
                        let size = SIZE { cx: self.w, cy: self.h };
                        let pt_src = POINT { x: 0, y: 0 };
                        let blend = BLENDFUNCTION {
                            BlendOp: AC_SRC_OVER as u8,
                            BlendFlags: 0,
                            SourceConstantAlpha: 255,
                            AlphaFormat: AC_SRC_ALPHA as u8,
                        };
                        UpdateLayeredWindow(self.hwnd, self.hdc_screen, &pt_dst, &size, self.hdc_mem, &pt_src, 0, &blend, ULW_ALPHA);
                    }
                }
                return;
            } else {
                self.annoyed_until = None;
            }
        }
        let mut pt = POINT { x: 0, y: 0 };
        let mut cursor_moved = false;
        if unsafe { GetCursorPos(&mut pt) } != 0 {
            if pt.x != self.cursor_last.x || pt.y != self.cursor_last.y {
                self.cursor_still = 0.0;
                self.cursor_last = pt;
                cursor_moved = true;
            } else {
                self.cursor_still += FRAME_MS as f32 / 1000.0;
            }
        }
        let cx = self.pos_x + self.w / 2;
        let cy = self.pos_y + self.h / 2;
        let rx = (pt.x - cx) as f32 / (self.w as f32 / 2.0);
        let ry = (pt.y - cy) as f32 / (self.h as f32 / 2.0);
        let tx = rx.clamp(-1.0, 1.0) * MAX_OFF_X * self.scale;
        let ty = ry.clamp(-1.0, 1.0) * MAX_OFF_Y * self.scale;
        let wander_target;
        if self.cursor_still > 1.5 {
            self.wander_t += FRAME_MS as f32 / 1000.0;
            wander_target = (
                (self.wander_t * 0.9).sin() * 14.0 * self.scale,
                (self.wander_t * 1.3).cos() * 10.0 * self.scale,
            );
        } else {
            self.wander_t = 0.0;
            wander_target = (0.0, 0.0);
        }
        let k = if cursor_moved || self.cursor_still < 0.3 {
            0.25
        } else {
            0.06
        };
        self.off_x += ((tx + wander_target.0) - self.off_x) * k;
        self.off_y += ((ty + wander_target.1) - self.off_y) * k;

        let mut blink_a = 0.0f32;
        if let Some(start) = self.blink_start {
            let t = now.saturating_duration_since(start).as_secs_f32();
            let total = self.blink_total();
            if t >= total {
                self.blink_start = None;
                self.schedule_blink();
            } else {
                let fade = 0.11f32;
                if t < fade {
                    blink_a = t / fade;
                } else if t < fade + 0.05 {
                    blink_a = 1.0;
                } else {
                    blink_a = 1.0 - (t - fade - 0.05) / fade;
                }
            }
        } else if now >= self.blink_next {
            self.blink_start = Some(now);
        }

        self.buf.fill(0);
        let dw = self.w as usize;
        let dh = self.h as usize;
        if self.base.w == dw && self.base.h == dh {
            self.buf.copy_from_slice(&self.base.data);
        } else {
            blit(
                &mut self.buf,
                dw,
                dh,
                &self.base.data,
                self.base.w,
                self.base.h,
                0,
                0,
            );
        }
        let ox = self.off_x.round() as i32;
        let oy = self.off_y.round() as i32;
        blit(&mut self.buf, dw, dh, &self.eyes.data, self.eyes.w, self.eyes.h, ox, oy);
        blit(&mut self.buf, dw, dh, &self.hl.data, self.hl.w, self.hl.h, ox, oy);
        if blink_a > 0.0 && self.closed.w == dw && self.closed.h == dh {
            crossfade(&mut self.buf, &self.closed.data, blink_a);
        }
        unsafe {
            if !self.bits.is_null() {
                std::ptr::copy_nonoverlapping(self.buf.as_ptr(), self.bits as *mut u8, self.buf.len());
                let pt_dst = POINT { x: self.pos_x, y: self.pos_y };
                let size = SIZE {
                    cx: self.w,
                    cy: self.h,
                };
                let pt_src = POINT { x: 0, y: 0 };
                let blend = BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: AC_SRC_ALPHA as u8,
                };
                UpdateLayeredWindow(
                    self.hwnd,
                    self.hdc_screen,
                    &pt_dst,
                    &size,
                    self.hdc_mem,
                    &pt_src,
                    0,
                    &blend,
                    ULW_ALPHA,
                );
            }
        }
    }

    fn blink_total(&self) -> f32 {
        0.27
    }

    fn schedule_blink(&mut self) {
        let secs = 3.0
            + (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos() as f32
                / 1e9
                % 4.5);
        self.blink_next = Instant::now() + std::time::Duration::from_millis((secs * 1000.0) as u64);
    }

    fn save_config(&self) {
        let text = format!(
            "{}\n{}\n{}\n{}",
            self.color, self.size_idx, self.pos_x, self.pos_y
        );
        let _ = std::fs::create_dir_all(self.config.parent().unwrap());
        let _ = std::fs::write(&self.config, text);
    }

    fn load_config(&mut self) {
        if let Ok(text) = std::fs::read_to_string(&self.config) {
            let parts: Vec<&str> = text.lines().collect();
            if parts.len() >= 4 {
                self.color = parts[0].parse().unwrap_or(0);
                self.size_idx = parts[1].parse().unwrap_or(1);
                self.pos_x = parts[2].parse().unwrap_or(-1);
                self.pos_y = parts[3].parse().unwrap_or(-1);
            }
        }
        self.size_idx = self.size_idx.min(2);
        self.color = self.color.min(1);
    }
}

fn wstr(s: &str) -> *const u16 {
    use std::sync::OnceLock;
    use std::collections::HashMap;
    static CACHE: OnceLock<Mutex<HashMap<String, Box<[u16]>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache.lock().unwrap();
    if let Some(ptr) = map.get(s) {
        return ptr.as_ptr();
    }
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    let boxed = v.into_boxed_slice();
    let ptr = boxed.as_ptr();
    map.insert(s.to_string(), boxed);
    ptr
}

fn nearest_preset_index(w: i32) -> usize {
    let mut best = 0;
    let mut best_d = i32::MAX;
    for (i, s) in SIZES.iter().enumerate() {
        let d = (w - s).abs();
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

fn known_folder(csidl: u32) -> Option<PathBuf> {
    let mut buf = [0u16; 260];
    unsafe {
        if SHGetFolderPathW(null_mut(), csidl as i32, null_mut(), 0, buf.as_mut_ptr()) != 0 {
            return None;
        }
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn run_powershell(script: &str) -> bool {
    unsafe {
        let cmd = format!(
            "powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -WindowStyle Hidden -Command \"{}\"",
            script
        );
        let mut c: Vec<u16> = cmd.encode_utf16().collect();
        c.push(0);
        let mut si = std::mem::zeroed::<STARTUPINFOW>();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi = std::mem::zeroed::<PROCESS_INFORMATION>();
        let ok = CreateProcessW(
            null(),
            c.as_mut_ptr(),
            null(),
            null(),
            0,
            CREATE_NO_WINDOW,
            null_mut(),
            null(),
            &si,
            &mut pi,
        );
        if ok == 0 {
            return false;
        }
        WaitForSingleObject(pi.hProcess, INFINITE);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        true
    }
}

unsafe extern "system" fn menu_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if app_ptr == 0 {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *(app_ptr as *mut App);
        match msg {
            WM_COMMAND => {
                let id = (wparam as u32 & 0xFFFF) as usize;
                app.bubble_menu_command(id);
                0
            }
            WM_HSCROLL => {
                if lparam != 0 {
                    let pos = SendMessageW(lparam as HWND, TBM_GETPOS, 0, 0) as i32;
                    app.set_width(pos);
                }
                0
            }
            WM_KEYDOWN => {
                if wparam as u16 == VK_ESCAPE {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                if app.menu_timer_id != 0 {
                    KillTimer(hwnd, app.menu_timer_id as usize);
                    app.menu_timer_id = 0;
                }
                app.menu_hwnd = null_mut();
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_menu(app, hwnd);
                0
            }
            WM_ACTIVATE => {
                if wparam == 0 {
                    app.menu_timer_id = SetTimer(hwnd, TIMER_MENU_CLOSE as usize, 50, None) as u32;
                }
                0
            }
            WM_TIMER => {
                if wparam == TIMER_MENU_CLOSE as WPARAM {
                    let focus = GetFocus();
                    if focus.is_null() || !is_child_of(hwnd, focus) {
                        DestroyWindow(hwnd);
                    }
                }
                0
            }
            WM_LBUTTONUP => {
                let x = (lparam as u32 & 0xFFFF) as i32;
                let y = ((lparam as u32 >> 16) & 0xFFFF) as i32;
                if in_pill(x, y, 105) {
                    app.switch_menu_tab(0);
                } else if in_pill(x, y, 235) {
                    app.switch_menu_tab(1);
                } else if in_pill(x, y, 365) {
                    app.switch_menu_tab(2);
                } else if in_bubble(x, y, MENU_W - 36, 26, 18) {
                    DestroyWindow(hwnd);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let app_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if app_ptr == 0 {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let app = &mut *(app_ptr as *mut App);
        match msg {
            WM_TIMER if wparam == TIMER_ID as WPARAM => {
                app.render();
                app.fire_reminders();
                0
            }
            TRAY_CB => {
                match (lparam as u32) & 0xFFFF {
                    WM_RBUTTONUP | WM_LBUTTONDOWN => app.popup_menu(),
                    _ => {}
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam as u32 & 0xFFFF) as usize;
                app.command(id);
                if id == MENU_EXIT {
                    app.save_config();
                }
                0
            }
            WM_HOTKEY => {
                match wparam as i32 {
                    HK_BLACK => app.set_color(0),
                    HK_WHITE => app.set_color(1),
                    HK_SMALL => app.set_size(0),
                    HK_MEDIUM => app.set_size(1),
                    HK_LARGE => app.set_size(2),
                    _ => {}
                }
                0
            }
            WM_LBUTTONDOWN => {
                let mut p = POINT { x: 0, y: 0 };
                GetCursorPos(&mut p);
                app.drag_win_x = app.pos_x;
                app.drag_win_y = app.pos_y;
                app.drag_mouse_x = p.x;
                app.drag_mouse_y = p.y;
                app.drag_active = true;
                SetCapture(hwnd);
                0
            }
            WM_MOUSEMOVE => {
                if app.drag_active && (wparam as u32 & 1) != 0 {
                    let mut p = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut p);
                    let nx = app.drag_win_x + (p.x - app.drag_mouse_x);
                    let ny = app.drag_win_y + (p.y - app.drag_mouse_y);
                    app.pos_x = nx;
                    app.pos_y = ny;
                    SetWindowPos(
                        hwnd,
                        null_mut(),
                        nx,
                        ny,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
                0
            }
            WM_LBUTTONUP => {
                if app.drag_active {
                    app.drag_active = false;
                    ReleaseCapture();
                }
                0
            }
            WM_LBUTTONDBLCLK => {
                app.drag_active = false;
                ReleaseCapture();
                app.show_menu();
                0
            }
            WM_MOVE => {
                let mut r = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                if GetWindowRect(hwnd, &mut r) != 0 {
                    app.pos_x = r.left;
                    app.pos_y = r.top;
                }
                0
            }
            WM_RBUTTONUP => {
                app.show_menu();
                0
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) & 0xFFFF) as i16;
                if delta > 0 {
                    app.set_width(app.w + SIZE_STEP);
                } else if delta < 0 {
                    app.set_width(app.w - SIZE_STEP);
                }
                0
            }
            WM_DESTROY => {
                app.save_config();
                app.save_user_data();
                let mut nid = std::mem::zeroed::<NOTIFYICONDATAW>();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = TRAY_ID as u32;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                PostQuitMessage(0);
                0
            }
            WM_QUERYENDSESSION => {
                app.save_config();
                app.save_user_data();
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ENDSESSION => {
                app.save_config();
                app.save_user_data();
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn main() {
    unsafe {
        SetProcessDPIAware();
        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_BAR_CLASSES,
        });
        let hinst = GetModuleHandleW(null());
        let class_name = wstr("cocoBarWnd");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_DBLCLKS,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: class_name,
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);
        let menu_class = wstr("CatMenuWnd");
        let menu_brush = CreateSolidBrush(0x00F8F9FA);
        let mwc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(menu_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: menu_brush,
            lpszMenuName: null(),
            lpszClassName: menu_class,
            hIconSm: null_mut(),
        };
        RegisterClassExW(&mwc);

        let mut app = App {
            hwnd: null_mut(),
            base: Layer::new(1, 1),
            closed: Layer::new(1, 1),
            eyes: Layer::new(1, 1),
            hl: Layer::new(1, 1),
            buf: Vec::new(),
            w: SIZES[1],
            h: ((SIZES[1] as f32) * CONTENT_H as f32 / CONTENT_W as f32).round() as i32,
            scale: SIZES[1] as f32 / CONTENT_W as f32,
            pos_x: -1,
            pos_y: -1,
            off_x: 0.0,
            off_y: 0.0,
            color: 0,
            size_idx: 1,
            hdc_screen: null_mut(),
            hdc_mem: null_mut(),
            hbmp: null_mut(),
            bits: null_mut(),
            blink_start: None,
            blink_next: Instant::now() + std::time::Duration::from_secs(3),
            wander_t: 0.0,
            cursor_last: POINT { x: 0, y: 0 },
            cursor_still: 0.0,
            icon: null_mut(),
            config: PathBuf::new(),
            menu_hwnd: null_mut(),
            exe: PathBuf::new(),
            menu_tab: 0,
            reminders: Vec::new(),
            note: String::new(),
            todos: Vec::new(),
            drag_active: false,
            drag_win_x: 0,
            drag_win_y: 0,
            drag_mouse_x: 0,
            drag_mouse_y: 0,
            annoyed_until: None,
            prev_mouse_down: false,
            cached_rgba: Vec::new(),
            cached_w: 0,
            cached_h: 0,
            menu_timer_id: 0,
        };
        app.exe = std::env::current_exe().unwrap_or_default();
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        app.config = PathBuf::from(&appdata)
            .join("cocoBar")
            .join("config.txt");
        let legacy = PathBuf::from(&appdata).join("CatCompanion");
        if !app.config.exists() && legacy.join("config.txt").exists() {
            let _ = std::fs::create_dir_all(app.config.parent().unwrap());
            let _ = std::fs::copy(legacy.join("config.txt"), &app.config);
            let _ = std::fs::copy(
                legacy.join("cat.ico"),
                app.config.parent().unwrap().join("cat.ico"),
            );
            let _ = std::fs::copy(
                legacy.join("mydata.txt"),
                app.config.parent().unwrap().join("mydata.txt"),
            );
        }
        app.load_config();
        app.load_user_data();

        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let mut idx = app.size_idx;
        loop {
            let w = SIZES[idx];
            let h = ((w as f32) * CONTENT_H as f32 / CONTENT_W as f32).round() as i32;
            if idx == 0 || (w < sw - 40 && h < sh - 80) {
                break;
            }
            idx -= 1;
        }
        if idx != app.size_idx {
            app.size_idx = idx;
            app.w = SIZES[idx];
            app.h = ((SIZES[idx] as f32) * CONTENT_H as f32 / CONTENT_W as f32).round() as i32;
            app.scale = SIZES[idx] as f32 / CONTENT_W as f32;
        }
        if app.pos_x < 0 || app.pos_y < 0 || app.pos_x + app.w > sw + 50 || app.pos_y + app.h > sh + 50 {
            app.pos_x = sw - app.w - 24;
            app.pos_y = sh - app.h - 48;
            if app.pos_y < 0 {
                app.pos_y = 24;
            }
        }

        app.rebuild_layers();

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            wstr("cocoBar"),
            WS_POPUP | WS_VISIBLE,
            app.pos_x,
            app.pos_y,
            app.w,
            app.h,
            null_mut(),
            null_mut(),
            hinst,
            null(),
        );
        app.hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, &mut app as *mut App as isize);

        app.hdc_screen = GetDC(null_mut());
        app.hdc_mem = CreateCompatibleDC(app.hdc_screen);
        app.rebuild_dib();
        app.rebuild_icon();
        app.update_tray();

        RegisterHotKey(hwnd, HK_BLACK, MOD_CONTROL | MOD_ALT, 0x42);
        RegisterHotKey(hwnd, HK_WHITE, MOD_CONTROL | MOD_ALT, 0x57);
        RegisterHotKey(hwnd, HK_SMALL, MOD_CONTROL | MOD_ALT, 0x31);
        RegisterHotKey(hwnd, HK_MEDIUM, MOD_CONTROL | MOD_ALT, 0x32);
        RegisterHotKey(hwnd, HK_LARGE, MOD_CONTROL | MOD_ALT, 0x33);

        SetTimer(hwnd, TIMER_ID, FRAME_MS, None);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

impl App {
    fn show_tab_controls(&self) {
        if self.menu_hwnd.is_null() {
            return;
        }
        unsafe {
            let all: [(usize, u32); 13] = [
                (ED_REM_MSG, 0),
                (ED_REM_TIME, 0),
                (BTN_REM_SET, 0),
                (LB_REM, 0),
                (BTN_REM_DEL, 0),
                (ED_NOTE, 1),
                (BTN_NOTE_SAVE, 1),
                (ED_TODO, 2),
                (BTN_TODO_ADD, 2),
                (LB_TODO, 2),
                (BTN_TODO_TOGGLE, 2),
                (BTN_TODO_DEL, 2),
                (CTRL_STATUS, 9),
            ];
            for (id, tab) in all.iter() {
                let c = GetDlgItem(self.menu_hwnd, *id as i32);
                if !c.is_null() {
                    let vis = *tab == 9 || *tab == self.menu_tab;
                    ShowWindow(c, if vis { 5 } else { 0 });
                }
            }
        }
    }

    fn switch_menu_tab(&mut self, tab: u32) {
        self.menu_tab = tab.min(2);
        self.show_tab_controls();
        self.sync_menu_states();
        unsafe {
            InvalidateRect(self.menu_hwnd, null(), 1);
        }
    }

    fn set_status(&self, text: &str) {
        if self.menu_hwnd.is_null() {
            return;
        }
        unsafe {
            let c = GetDlgItem(self.menu_hwnd, CTRL_STATUS as i32);
            if !c.is_null() {
                SetWindowTextW(c, wstr(text));
            }
        }
    }

    fn ctl_text(&self, id: usize) -> String {
        unsafe {
            let c = GetDlgItem(self.menu_hwnd, id as i32);
            if c.is_null() {
                return String::new();
            }
            let n = SendMessageW(c, 0x000E, 0, 0) as usize;
            let mut buf = vec![0u16; n + 1];
            SendMessageW(c, 0x000D, (n + 1) as _, buf.as_mut_ptr() as _);
            String::from_utf16_lossy(&buf[..n])
        }
    }

    fn set_ctl_text(&self, id: usize, text: &str) {
        unsafe {
            let c = GetDlgItem(self.menu_hwnd, id as i32);
            if !c.is_null() {
                SetWindowTextW(c, wstr(text));
            }
        }
    }

    fn lb_selected(&self, id: usize) -> Option<usize> {
        unsafe {
            let lb = GetDlgItem(self.menu_hwnd, id as i32);
            if lb.is_null() {
                return None;
            }
            let sel = SendMessageW(lb, 0x0188, 0, 0) as i32;
            if sel < 0 {
                None
            } else {
                Some(sel as usize)
            }
        }
    }

    fn add_reminder_ui(&mut self) {
        let msg = self.ctl_text(ED_REM_MSG);
        let time_str = self.ctl_text(ED_REM_TIME);
        let msg = msg.trim().to_string();
        let time_str = time_str.trim().to_string();
        if msg.is_empty() || time_str.is_empty() {
            self.set_status("Type a message and a time like 14:30 first.");
            return;
        }
        match parse_hhmm(&time_str) {
            Some(ts) => {
                self.reminders.push((msg.clone(), ts));
                self.save_user_data();
                self.refresh_lists();
                self.set_ctl_text(ED_REM_MSG, "");
                self.set_ctl_text(ED_REM_TIME, "");
                self.set_status(&format!("Reminder set for {}.", fmt_time(ts)));
            }
            None => {
                self.set_status("Time must look like 14:30 (24 hour format).");
            }
        }
    }

    fn delete_reminder_ui(&mut self) {
        if let Some(i) = self.lb_selected(LB_REM) {
            if i < self.reminders.len() {
                self.reminders.remove(i);
                self.save_user_data();
            }
        }
    }

    fn save_note_ui(&mut self) {
        self.note = self.ctl_text(ED_NOTE);
        self.save_user_data();
        self.set_status("Note saved.");
    }

    fn add_todo_ui(&mut self) {
        let t = self.ctl_text(ED_TODO);
        let t = t.trim().to_string();
        if t.is_empty() {
            self.set_status("Type a task first.");
            return;
        }
        self.todos.push((t, false));
        self.save_user_data();
        self.set_ctl_text(ED_TODO, "");
        self.set_status("Task added.");
    }

    fn toggle_todo_ui(&mut self) {
        if let Some(i) = self.lb_selected(LB_TODO) {
            if i < self.todos.len() {
                self.todos[i].1 = !self.todos[i].1;
                self.save_user_data();
            }
        }
    }

    fn delete_todo_ui(&mut self) {
        if let Some(i) = self.lb_selected(LB_TODO) {
            if i < self.todos.len() {
                self.todos.remove(i);
                self.save_user_data();
            }
        }
    }

    fn data_path(&self) -> PathBuf {
        self.app_dir().join("mydata.txt")
    }

    fn save_user_data(&self) {
        let mut out = String::new();
        let esc = |s: &str| -> String { s.replace("\\", "\\\\").replace("\n", "\\n") };
        out.push_str(&format!("N\t{}\n", esc(&self.note)));
        for (msg, ts) in self.reminders.iter() {
            out.push_str(&format!("R\t{}\t{}\n", ts, esc(msg)));
        }
        for (text, done) in self.todos.iter() {
            out.push_str(&format!("T\t{}\t{}\n", if *done { 1 } else { 0 }, esc(text)));
        }
        let _ = std::fs::create_dir_all(self.app_dir());
        let _ = std::fs::write(self.data_path(), out);
    }

    fn load_user_data(&mut self) {
        if let Ok(text) = std::fs::read_to_string(self.data_path()) {
            for line in text.lines() {
                let mut it = line.splitn(3, '\t');
                let unesc = |s: &str| -> String { s.replace("\\n", "\n").replace("\\\\", "\\") };
                match it.next() {
                    Some("N") => {
                        self.note = unesc(it.next().unwrap_or(""));
                    }
                    Some("R") => {
                        let ts: u64 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                        let msg = it.next().unwrap_or("");
                        if !msg.is_empty() {
                            self.reminders.push((unesc(msg), ts));
                        }
                    }
                    Some("T") => {
                        let done = it.next().unwrap_or("0") == "1";
                        let text = it.next().unwrap_or("");
                        if !text.is_empty() {
                            self.todos.push((unesc(text), done));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn notify_reminder(&self, text: &str) {
        unsafe {
            let mut nid = std::mem::zeroed::<NOTIFYICONDATAW>();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = TRAY_ID as u32;
            nid.uFlags = NIF_INFO;
            let title: Vec<u16> = "cocoBar Reminder".encode_utf16().collect();
            for (i, c) in title.iter().enumerate().take(127) {
                nid.szInfoTitle[i] = *c;
            }
            let body: Vec<u16> = text.encode_utf16().collect();
            for (i, c) in body.iter().enumerate().take(255) {
                nid.szInfo[i] = *c;
            }
nid.dwInfoFlags = NIIF_INFO;
            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    fn fire_reminders(&mut self) {
        let now_s = unix_now();
        let mut fired: Vec<String> = Vec::new();
        let mut keep: Vec<(String, u64)> = Vec::new();
        for (msg, ts) in self.reminders.drain(..) {
            if ts <= now_s {
                fired.push(msg);
            } else {
                keep.push((msg, ts));
            }
        }
        self.reminders = keep;
        if !fired.is_empty() {
            self.save_user_data();
            self.notify_reminder(&fired.join("  |  "));
        }
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn parse_hhmm(s: &str) -> Option<u64> {
    let (h, m) = s.trim().split_once(':')?;
    let h: u64 = h.trim().parse().ok()?;
    let m: u64 = m.trim().parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    let now = unix_now();
    let mut ts = (now / 86400) * 86400 + h * 3600 + m * 60;
    if ts <= now {
        ts += 86400;
    }
    Some(ts)
}

fn fmt_time(ts: u64) -> String {
    format!("{:02}:{:02}", (ts % 86400) / 3600, (ts % 3600) / 60)
}
impl App {
    fn bubble_menu_command(&mut self, id: usize) {
        match id {
            CTRL_BLACK => self.set_color(0),
            CTRL_WHITE => self.set_color(1),
            CTRL_EXIT => unsafe {
                DestroyWindow(self.hwnd);
                return;
            },
            CTRL_STARTUP => {
                self.set_startup(!self.startup_enabled());
            }
            BTN_REM_SET => self.add_reminder_ui(),
            BTN_REM_DEL => self.delete_reminder_ui(),
            BTN_NOTE_SAVE => self.save_note_ui(),
            BTN_TODO_ADD => self.add_todo_ui(),
            BTN_TODO_TOGGLE => self.toggle_todo_ui(),
            BTN_TODO_DEL => self.delete_todo_ui(),
            _ => {}
        }
        self.sync_menu_states();
    }
}

fn in_bubble(x: i32, y: i32, cx: i32, cy: i32, r: i32) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= r * r
}

fn in_pill(x: i32, y: i32, cx: i32) -> bool {
    x >= cx - 46 && x <= cx + 46 && y >= 8 && y <= 56
}

unsafe fn is_child_of(parent: HWND, child: HWND) -> bool {
    let mut p = child;
    while !p.is_null() {
        if p == parent {
            return true;
        }
        p = GetParent(p);
    }
    false
}

fn paint_menu(app: &App, hwnd: HWND) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, MENU_W, MENU_H);
        SelectObject(mem, bmp);

        let bg = CreateSolidBrush(0x00F8F9FA);
        FillRect(
            mem,
            &RECT { left: 0, top: 0, right: MENU_W, bottom: MENU_H },
            bg,
        );
        DeleteObject(bg);

        let hdr = CreateSolidBrush(0x00FFFFFF);
        FillRect(
            mem,
            &RECT { left: 0, top: 0, right: MENU_W, bottom: 84 },
            hdr,
        );
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, 0x00E8E8E8);
        let old_pen = SelectObject(mem, sep_pen);
        let old_brush = SelectObject(mem, GetStockObject(5));
        MoveToEx(mem, 0, 84, null_mut());
        LineTo(mem, MENU_W, 84);
        SelectObject(mem, old_pen);
        SelectObject(mem, old_brush);
        DeleteObject(sep_pen);

        let card = CreateRoundRectRgn(12, 92, MENU_W - 12, 350, 10, 10);
        let wb = CreateSolidBrush(0x00FFFFFF);
        FillRgn(mem, card, wb);
        DeleteObject(wb);
        let fb = CreateSolidBrush(0x00E0E0E0);
        FrameRgn(mem, card, fb, 1, 1);
        DeleteObject(fb);
        DeleteObject(card);

        let labels = ["Reminders", "Notes", "To-do"];
        for i in 0..3usize {
            let cx = 105 + (i as i32) * 130;
            let active = (i as u32) == app.menu_tab;
            let pill = CreateRoundRectRgn(cx - 48, 14, cx + 48, 52, 20, 20);
            let fill = if active { 0x004A90D9 } else { 0x00F0F0F0 };
            let pb = CreateSolidBrush(fill);
            FillRgn(mem, pill, pb);
            DeleteObject(pb);
            if !active {
                let eb = CreateSolidBrush(0x00D8D8D8);
                FrameRgn(mem, pill, eb, 1, 1);
                DeleteObject(eb);
            }
            DeleteObject(pill);
            let font = CreateFontW(-13, 0, 0, 0, if active { 600 } else { 400 }, 0, 0, 0, 1, 0, 0, 0, 0, wstr("Segoe UI") as _);
            let old = SelectObject(mem, font);
            SetBkMode(mem, 1);
            SetTextColor(mem, if active { 0x00FFFFFF } else { 0x00666666 });
            let mut rc = RECT {
                left: cx - 46,
                top: 14,
                right: cx + 46,
                bottom: 52,
            };
            DrawTextW(mem, wstr(labels[i]) as *mut u16, -1, &mut rc, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            SelectObject(mem, old);
            DeleteObject(font);
        }

        let cb = CreateSolidBrush(0x00F0F0F0);
        let cp = CreatePen(PS_SOLID, 1, 0x00D0D0D0);
        SelectObject(mem, cb);
        SelectObject(mem, cp);
        Ellipse(mem, MENU_W - 44, 14, MENU_W - 18, 40);
        DeleteObject(cb);
        DeleteObject(cp);
        let cf = CreateFontW(-14, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, wstr("Segoe UI") as _);
        let oldf = SelectObject(mem, cf);
        SetBkMode(mem, 1);
        SetTextColor(mem, 0x00888888);
        let mut rcx = RECT {
            left: MENU_W - 44,
            top: 14,
            right: MENU_W - 18,
            bottom: 40,
        };
        DrawTextW(mem, wstr("\u{00D7}") as *mut u16, -1, &mut rcx, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        SelectObject(mem, oldf);
        DeleteObject(cf);

        BitBlt(hdc, 0, 0, MENU_W, MENU_H, mem, 0, 0, SRCCOPY);
        DeleteDC(mem);
        DeleteObject(bmp);
        EndPaint(hwnd, &ps);
    }
}
