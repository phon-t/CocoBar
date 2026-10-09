#![windows_subsystem = "windows"]

mod cat;
mod config;
mod cosmetics;
mod menu;
mod notes;
mod updater;

use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::Mutex;
use std::time::Instant;
use windows_sys::Win32::Foundation::{
    GetLastError, ERROR_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    CreateMutexW, CreateProcessW, WaitForSingleObject, CREATE_NO_WINDOW, INFINITE,
    PROCESS_INFORMATION, STARTUPINFOW,
};
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_BAR_CLASSES, INITCOMMONCONTROLSEX,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, CSIDL_DESKTOPDIRECTORY, CSIDL_STARTUP, NIF_ICON, NIF_MESSAGE, NIF_TIP,
    NIM_ADD, NIM_DELETE, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowLongPtrW,
    GetWindowRect, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassExW, SetProcessDPIAware,
    SetTimer, SetWindowLongPtrW, SetWindowPos, TranslateMessage, UpdateLayeredWindow, CS_DBLCLKS,
    GWLP_USERDATA, HWND_NOTOPMOST, HWND_TOPMOST, ICONINFO, IDC_ARROW, IDI_APPLICATION, SM_CXSCREEN,
    SM_CYSCREEN, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, ULW_ALPHA, WM_APP, WM_CLOSE, WM_COMMAND,
    WM_DESTROY, WM_ENDSESSION, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_MOVE, WM_QUERYENDSESSION, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};

pub(crate) const TRAY_ID: usize = 1;
pub(crate) const TIMER_ID: usize = 2;
pub(crate) const FRAME_MS: u32 = 16;

pub(crate) const MENU_BLACK: usize = 101;
pub(crate) const MENU_WHITE: usize = 102;
pub(crate) const MENU_ORANGE: usize = 103;
pub(crate) const MENU_SIZE_SMALL: usize = 105;
pub(crate) const MENU_SIZE_MEDIUM: usize = 106;
pub(crate) const MENU_SIZE_LARGE: usize = 107;
pub(crate) const MENU_EXIT: usize = 109;
pub(crate) const MENU_SHORTCUT_DESKTOP: usize = 110;
pub(crate) const MENU_STARTUP: usize = 111;

pub(crate) const HK_BLACK: i32 = 1;
pub(crate) const HK_WHITE: i32 = 2;
pub(crate) const HK_ORANGE: i32 = 3;
pub(crate) const HK_SMALL: i32 = 4;
pub(crate) const HK_MEDIUM: i32 = 5;
pub(crate) const HK_LARGE: i32 = 6;
pub(crate) const HK_EXIT: i32 = 7;
pub(crate) const HK_NOTES: i32 = 8;
pub(crate) const HK_TODO: i32 = 9;
const HK_VK: [u32; 9] = [0x42, 0x57, 0x4F, 0x31, 0x32, 0x33, 0x58, 0x4E, 0x54];

pub(crate) const SIZE_STEP: i32 = 40;
pub(crate) const SIZES: [i32; 3] = [200, 320, 500];

// Cat width range in pixels: 100 .. 500
pub(crate) fn size_limits() -> (i32, i32) {
    (100, 500)
}

pub(crate) const BLACK_FULL: &[u8] = include_bytes!("../assets/blackcatfull.png");
pub(crate) const WHITE_FULL: &[u8] = include_bytes!("../assets/whitecatfull.png");
pub(crate) const ORANGE_FULL: &[u8] = include_bytes!("../assets/orangecatfull.png");

pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub(crate) const TRAY_CB: u32 = WM_APP + 2;

pub struct App {
    pub hwnd: HWND,
    pub(crate) cat: cat::CatRenderer,
    pub w: i32,
    pub h: i32,
    pub scale: f32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub color: usize,
    pub size_idx: usize,
    pub hdc_screen: *mut c_void,
    pub hdc_mem: *mut c_void,
    pub hbmp: *mut c_void,
    pub bits: *mut c_void,
    pub icon: *mut c_void,
    pub config_path: PathBuf,
    pub data_path: PathBuf,
    pub exe: PathBuf,
    pub menu_hwnd: HWND,
    pub customize_hwnd: HWND,
    pub settings_hwnd: HWND,
    pub menu_tab: u32,
    pub note: String,
    pub(crate) notes: Vec<notes::Note>,
    pub editing_note: Option<u64>,
    pub notes_editor_open: bool,
    pub notes_page: usize,
    pub(crate) deleted_note: Option<notes::Note>,
    pub data_dirty: bool,
    pub data_status: String,
    pub todo_page: usize,
    pub todos: Vec<(String, bool)>,
    pub drag_active: bool,
    pub drag_win_x: i32,
    pub drag_win_y: i32,
    pub drag_mouse_x: i32,
    pub drag_mouse_y: i32,
    pub press_mouse_x: i32,
    pub press_mouse_y: i32,
    pub tilt_restore_w: i32,
    pub tilt_restore_h: i32,
    pub annoyed_until: Option<Instant>,
    pub annoyed_drawn: bool,
    pub always_on_top: bool,
    pub desktop_shortcut: bool,
    pub auto_update: bool,
    pub cosmetic_bell: Option<usize>,
    pub cosmetic_scarf: Option<usize>,
    pub cosmetic_tie: Option<usize>,
    pub status: String,
    pub menu_timer_id: u32,
    pub keep_panels_open: bool,
    pub hotkeys: u16,
    pub settings_scroll: i32,
    pub mouse_pt: POINT,
    pub(crate) updater: updater::Updater,
}

impl App {
    pub(crate) fn save_data(&mut self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (notes, active) = if self.notes_editor_open {
            notes::edited_notes(&self.notes, self.editing_note, &self.note, now)
        } else {
            (self.notes.clone(), self.editing_note)
        };
        let data = config::UserData {
            note: if self.notes_editor_open {
                self.note.clone()
            } else {
                notes
                    .first()
                    .map(|note| note.text.clone())
                    .unwrap_or_default()
            },
            todos: self.todos.clone(),
            notes,
        };
        match config::save_user_data(&self.data_path, &data) {
            Ok(()) => {
                self.notes = data.notes;
                self.editing_note = active;
                self.data_dirty = false;
                self.data_status = "All changes saved".into();
                true
            }
            Err(e) => {
                self.data_dirty = true;
                self.data_status = format!("Could not save: {e}. Press Ctrl+S to retry.");
                eprintln!("Could not save {}: {e}", self.data_path.display());
                false
            }
        }
    }

    pub(crate) fn save_settings(&mut self) -> bool {
        let cfg = config::ConfigData {
            color: self.color,
            size_idx: self.size_idx,
            size_px: if self.drag_active {
                self.tilt_restore_w
            } else {
                self.w
            },
            pos_x: self.pos_x,
            pos_y: self.pos_y,
            always_on_top: self.always_on_top,
            desktop_shortcut: self.desktop_shortcut,
            auto_update: self.auto_update,
            cosmetic_bell: self.cosmetic_bell,
            cosmetic_scarf: self.cosmetic_scarf,
            cosmetic_tie: self.cosmetic_tie,
            hotkeys: self.hotkeys,
        };
        if let Err(e) = config::save_config(&self.config_path, &cfg) {
            self.status = format!("Could not save settings: {e}");
            return false;
        }
        true
    }

    pub(crate) fn set_color(&mut self, c: usize) {
        if self.color == c {
            return;
        }
        self.color = c.min(2);
        self.annoyed_drawn = false;
        self.cat
            .rebuild_layers(self.color, self.w, self.h, self.scale);
        self.rebuild_icon();
        self.update_tray();
        self.save_settings();
    }

    pub(crate) fn set_width(&mut self, mut new_w: i32) {
        // A wheel/hotkey resize while held must update the normal size, then
        // rebuild the tilted canvas and re-anchor the drag to the current cursor.
        let resume_drag = self.drag_active;
        if resume_drag {
            self.cat.fast_drag = false;
            self.drag_active = false;
            self.exit_tilt();
        }
        let (min_w, _max_w) = size_limits();
        new_w = new_w.clamp(min_w, size_limits().1);
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let max_w = (sw - 40).max(min_w);
        let max_h = (sh - 60).max(120);
        let new_w = new_w.min(max_w);
        let mut new_h =
            ((new_w as f32) * cat::CONTENT_H as f32 / cat::CONTENT_W as f32).round() as i32;
        let old_r = self.pos_x + self.w;
        let old_b = self.pos_y + self.h;
        if new_h > max_h {
            new_h = max_h;
            self.w =
                ((new_h as f32) * cat::CONTENT_W as f32 / cat::CONTENT_H as f32).round() as i32;
        } else {
            self.w = new_w;
        }
        self.h = new_h;
        self.scale = self.w as f32 / cat::CONTENT_W as f32;
        self.pos_x = old_r - self.w;
        self.pos_y = old_b - self.h;
        if self.pos_x < 0 {
            self.pos_x = 0;
        }
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
        self.cat
            .rebuild_layers(self.color, self.w, self.h, self.scale);
        self.cat.rebuild_cosmetics(
            self.cosmetic_bell,
            self.cosmetic_scarf,
            self.cosmetic_tie,
            self.scale,
        );
        // Keep the tilt-pose caches fresh for the current zoom so grabs stay instant
        self.cat.ensure_tilted(cat::tilted_scale(self.scale));
        self.rebuild_dib();
        self.annoyed_drawn = false;
        self.save_settings();
        if resume_drag {
            self.drag_active = true;
            self.enter_tilt();
            let mut cursor = POINT { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut cursor);
            }
            self.drag_mouse_x = cursor.x;
            self.drag_mouse_y = cursor.y;
        }
    }

    pub(crate) fn normal_width(&self) -> i32 {
        if self.drag_active {
            self.tilt_restore_w
        } else {
            self.w
        }
    }

    pub(crate) fn set_size(&mut self, idx: usize) {
        self.size_idx = idx.min(2);
        self.set_width(SIZES[self.size_idx]);
    }

    pub(crate) fn set_topmost(&mut self, on: bool) {
        self.always_on_top = on;
        unsafe {
            SetWindowPos(
                self.hwnd,
                if on { HWND_TOPMOST } else { HWND_NOTOPMOST },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
        }
    }

    fn set_raw_window(&mut self, w: i32, h: i32, content_w: f32) {
        let (min_w, _) = size_limits();
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let max_w = (sw - 40).max(min_w);
        let max_h = (sh - 60).max(120);
        let mut nw = w.max(min_w);
        let mut nh = h.max(120);
        if nw > max_w {
            let k = max_w as f32 / nw as f32;
            nw = max_w;
            nh = (nh as f32 * k).round() as i32;
        }
        if nh > max_h {
            let k = max_h as f32 / nh as f32;
            nh = max_h;
            nw = (nw as f32 * k).round() as i32;
        }
        self.w = nw;
        self.h = nh;
        self.scale = nw as f32 / content_w;
        self.cat
            .rebuild_layers(self.color, self.w, self.h, self.scale);
        self.cat.rebuild_cosmetics(
            self.cosmetic_bell,
            self.cosmetic_scarf,
            self.cosmetic_tie,
            self.scale,
        );
        self.rebuild_dib();
    }

    fn enter_tilt(&mut self) {
        self.annoyed_until = None;
        self.annoyed_drawn = false;
        self.cat.tail.reset();
        self.tilt_restore_w = self.w;
        self.tilt_restore_h = self.h;
        // Draw the tilted pose a touch smaller than the raw canvas so it matches the
        // normal cat's visual size
        let ts = cat::tilted_scale(self.scale);
        let tw = ((cat::TILT_W as f32) * ts).round().max(1.0) as i32;
        let th = ((cat::TILT_CANVAS_H as f32) * ts).round().max(1.0) as i32;
        // Anchor: keep the cat's bottom edge fixed while the window changes size.
        // The tilted pose only needs the pre-warmed tilt layers + the DIB, so skip
        // the expensive normal-pose layer rebuild for a lag-free drag start.
        let old_b = self.pos_y + self.h;
        self.w = tw;
        self.h = th;
        self.scale = tw as f32 / cat::TILT_W as f32;
        self.rebuild_dib();
        self.cat.ensure_tilted(self.scale);
        self.pos_y = old_b - (cat::TILT_H as f32 * self.scale).round() as i32;
        if self.pos_y < 0 {
            self.pos_y = 0;
        }
        self.constrain_drag_position();
        self.cat.drag_active = true;
        self.cat.render(
            self.pos_x,
            self.pos_y,
            self.w,
            self.h,
            self.scale,
            self.hdc_screen,
            self.hdc_mem,
            self.bits,
            self.hwnd,
        );
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
        // Keep the drag offset tracking consistent with the resized window
        self.drag_win_x = self.pos_x;
        self.drag_win_y = self.pos_y;
    }

    fn exit_tilt(&mut self) {
        let nw = self.tilt_restore_w.max(100);
        let nh = self.tilt_restore_h.max(120);
        let old_b = self.pos_y + (cat::TILT_H as f32 * self.scale).round() as i32;
        self.set_raw_window(nw, nh, cat::CONTENT_W as f32);
        self.pos_y = old_b - self.h;
        if self.pos_y < 0 {
            self.pos_y = 0;
        }
        self.cat.drag_active = false;
        self.cat.render(
            self.pos_x,
            self.pos_y,
            self.w,
            self.h,
            self.scale,
            self.hdc_screen,
            self.hdc_mem,
            self.bits,
            self.hwnd,
        );
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
        self.drag_win_x = self.pos_x;
        self.drag_win_y = self.pos_y;
    }

    fn constrain_drag_position(&mut self) {
        unsafe {
            use windows_sys::Win32::Graphics::Gdi::{
                GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
            };
            let monitor = MonitorFromPoint(
                POINT {
                    x: self.pos_x + self.w / 2,
                    y: self.pos_y + self.h / 2,
                },
                MONITOR_DEFAULTTONEAREST,
            );
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) != 0 {
                let work = info.rcWork;
                self.pos_x = self
                    .pos_x
                    .clamp(work.left, (work.right - self.w).max(work.left));
                self.pos_y = self
                    .pos_y
                    .clamp(work.top, (work.bottom - self.h).max(work.top));
            }
        }
    }

    pub(crate) fn draw_annoyed(&mut self) {
        if self.bits.is_null() {
            return;
        }
        let dw = self.w as usize;
        let dh = self.h as usize;
        if !self.cat.compose_annoyed(dw, dh) {
            return;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.cat.buf.as_ptr(),
                self.bits as *mut u8,
                self.cat.buf.len(),
            );
            let pt_dst = POINT {
                x: self.pos_x,
                y: self.pos_y,
            };
            let size = SIZE {
                cx: self.w,
                cy: self.h,
            };
            let pt_src = POINT { x: 0, y: 0 };
            let blend = windows_sys::Win32::Graphics::Gdi::BLENDFUNCTION {
                BlendOp: 0,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: 1,
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
        self.annoyed_drawn = true;
    }

    fn rebuild_dib(&mut self) {
        let (hbmp, bits) = self.cat.rebuild_dib(self.hdc_mem, self.w, self.h);
        unsafe {
            // Select the new bitmap into the mem DC so rendering uses the new size
            windows_sys::Win32::Graphics::Gdi::SelectObject(self.hdc_mem, hbmp);
            if !self.hbmp.is_null() {
                windows_sys::Win32::Graphics::Gdi::DeleteObject(self.hbmp);
            }
        }
        self.hbmp = hbmp;
        self.bits = bits;
    }

    fn rebuild_icon(&mut self) {
        unsafe {
            if !self.icon.is_null() {
                DestroyIcon(self.icon);
            }
        }
        let img_bytes = [BLACK_FULL, WHITE_FULL, ORANGE_FULL][self.color.min(2)];
        let mut img = image::load_from_memory_with_format(img_bytes, image::ImageFormat::Png)
            .expect("icon decode")
            .to_rgba8();
        let resized =
            image::imageops::resize(&mut img, 32, 32, image::imageops::FilterType::Triangle);
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
        let bmi = windows_sys::Win32::Graphics::Gdi::BITMAPINFO {
            bmiHeader: windows_sys::Win32::Graphics::Gdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPINFOHEADER>()
                    as u32,
                biWidth: 32,
                biHeight: -32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [windows_sys::Win32::Graphics::Gdi::RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };
        let mut bits: *mut c_void = null_mut();
        let color_bitmap = unsafe {
            windows_sys::Win32::Graphics::Gdi::CreateDIBSection(
                hdc,
                &bmi,
                0,
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
                windows_sys::Win32::Graphics::Gdi::DeleteObject(color_bitmap);
            }
            ReleaseDC(null_mut(), hdc);
        }
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
            if Shell_NotifyIconW(windows_sys::Win32::UI::Shell::NIM_MODIFY, &nid) == 0 {
                Shell_NotifyIconW(NIM_ADD, &nid);
            }
            nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            Shell_NotifyIconW(NIM_SETVERSION, &nid);
        }
    }

    fn command(&mut self, id: usize) {
        match id {
            MENU_BLACK => self.set_color(0),
            MENU_WHITE => self.set_color(1),
            MENU_ORANGE => self.set_color(2),
            MENU_SIZE_SMALL | MENU_SIZE_MEDIUM | MENU_SIZE_LARGE => {
                self.set_size(id - MENU_SIZE_SMALL)
            }
            MENU_SHORTCUT_DESKTOP => self.create_desktop_shortcut(),
            MENU_STARTUP => {
                self.set_startup(!self.startup_enabled());
            }
            MENU_EXIT => unsafe {
                if menu::flush_data(self) {
                    DestroyWindow(self.hwnd);
                } else {
                    menu::show_menu(self);
                }
            },
            _ => {}
        }
    }

    fn app_dir(&self) -> PathBuf {
        self.config_path
            .parent()
            .unwrap_or(&PathBuf::from("."))
            .to_path_buf()
    }

    pub(crate) fn hotkey_enabled(&self, i: usize) -> bool {
        i < HK_VK.len() && (self.hotkeys & (1 << i)) != 0
    }

    pub(crate) fn set_hotkey(&mut self, i: usize, on: bool) {
        if i >= HK_VK.len() {
            return;
        }
        let bit = 1u16 << i;
        let id = (i + 1) as i32;
        let was = (self.hotkeys & bit) != 0;
        if on == was {
            return;
        }
        unsafe {
            if on {
                if RegisterHotKey(
                    self.hwnd,
                    id,
                    MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
                    HK_VK[i],
                ) == 0
                {
                    self.status = "That shortcut is already used by another application.".into();
                    return;
                }
                self.hotkeys |= bit;
            } else {
                UnregisterHotKey(self.hwnd, id);
                self.hotkeys &= !bit;
            }
        }
    }

    fn startup_lnk(&self) -> Option<PathBuf> {
        known_folder(CSIDL_STARTUP).map(|p| p.join("cocoBar.lnk"))
    }

    pub(crate) fn startup_enabled(&self) -> bool {
        self.startup_lnk().map(|p| p.exists()).unwrap_or(false)
    }

    pub(crate) fn set_startup(&self, on: bool) {
        let Some(lnk) = self.startup_lnk() else {
            return;
        };
        if !on {
            let _ = std::fs::remove_file(&lnk);
            return;
        }
        let ico = self.ensure_icon();
        let exe = self.exe.to_string_lossy().to_string();
        let dir = self
            .exe
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_string_lossy()
            .to_string();
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

    fn ensure_icon(&self) -> PathBuf {
        let ico = self.app_dir().join("cat.ico");
        if ico.exists() {
            return ico;
        }
        let _ = std::fs::create_dir_all(self.app_dir());
        let mut img = image::load_from_memory_with_format(BLACK_FULL, image::ImageFormat::Png)
            .expect("icon decode")
            .to_rgba8();
        let resized =
            image::imageops::resize(&mut img, 32, 32, image::imageops::FilterType::Triangle);
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

    pub(crate) fn create_desktop_shortcut(&self) {
        let Some(desktop) = known_folder(CSIDL_DESKTOPDIRECTORY) else {
            return;
        };
        let ico = self.ensure_icon();
        let lnk = desktop.join("cocoBar.lnk");
        let exe = self.exe.to_string_lossy().to_string();
        let dir = self
            .exe
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_string_lossy()
            .to_string();
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

    pub(crate) fn desktop_shortcut_exists(&self) -> bool {
        known_folder(CSIDL_DESKTOPDIRECTORY)
            .map(|p| p.join("cocoBar.lnk").exists())
            .unwrap_or(false)
    }

    pub(crate) fn remove_desktop_shortcut(&self) {
        if let Some(desktop) = known_folder(CSIDL_DESKTOPDIRECTORY) {
            let _ = std::fs::remove_file(desktop.join("cocoBar.lnk"));
        }
    }

    fn check_for_update(&mut self) {
        menu::check_for_update(self);
    }
}

fn wstr(s: &'static str) -> *const u16 {
    use std::collections::HashMap;
    use std::sync::OnceLock;
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

fn known_folder(csidl: u32) -> Option<PathBuf> {
    let mut buf = [0u16; 260];
    unsafe {
        if windows_sys::Win32::UI::Shell::SHGetFolderPathW(
            null_mut(),
            csidl as i32,
            null_mut(),
            0,
            buf.as_mut_ptr(),
        ) != 0
        {
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
        windows_sys::Win32::Foundation::CloseHandle(pi.hProcess);
        windows_sys::Win32::Foundation::CloseHandle(pi.hThread);
        true
    }
}

unsafe extern "system" fn wnd_proc(
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
            WM_TIMER if wparam == TIMER_ID as WPARAM => {
                menu::poll_update(app);
                if windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(hwnd) == 0 {
                    return 0;
                }
                // No annoyed face while the cat is being dragged
                if app.drag_active {
                    app.annoyed_until = None;
                }

                // Render
                app.cat.drag_active = app.drag_active;
                let now = Instant::now();
                let mut restore_expression = false;
                if let Some(until) = app.annoyed_until {
                    if now < until {
                        if !app.annoyed_drawn {
                            app.draw_annoyed();
                        }
                        return 0;
                    } else {
                        app.annoyed_until = None;
                        restore_expression = app.annoyed_drawn;
                        app.annoyed_drawn = false;
                    }
                }

                // Skip the frame entirely when nothing needs repainting (idle cat)
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                let moved = pt.x != app.mouse_pt.x || pt.y != app.mouse_pt.y;
                app.mouse_pt = pt;
                if !app.cat.tick(moved) && !restore_expression {
                    return 0;
                }
                app.cat.render(
                    app.pos_x,
                    app.pos_y,
                    app.w,
                    app.h,
                    app.scale,
                    app.hdc_screen,
                    app.hdc_mem,
                    app.bits,
                    app.hwnd,
                );
                0
            }
            TRAY_CB => {
                match (lparam as u32) & 0xFFFF {
                    WM_RBUTTONUP | WM_LBUTTONDOWN => {
                        // Kill cat timer while menu is open
                        windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_ID);
                        menu::show_menu(app);
                        // Resume cat timer
                        windows_sys::Win32::UI::WindowsAndMessaging::SetTimer(
                            hwnd, TIMER_ID, FRAME_MS, None,
                        );
                    }
                    _ => {}
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam as u32 & 0xFFFF) as usize;
                app.command(id);
                0
            }
            WM_HOTKEY => {
                match wparam as i32 {
                    HK_BLACK => app.set_color(0),
                    HK_WHITE => app.set_color(1),
                    HK_ORANGE => app.set_color(2),
                    HK_SMALL => app.set_size(0),
                    HK_MEDIUM => app.set_size(1),
                    HK_LARGE => app.set_size(2),
                    HK_NOTES => menu::open_tab(app, 1),
                    HK_TODO => menu::open_tab(app, 0),
                    HK_EXIT => {
                        if menu::flush_data(app) {
                            DestroyWindow(app.hwnd);
                        } else {
                            menu::show_menu(app);
                        }
                    }
                    _ => {}
                }
                0
            }
            WM_LBUTTONDBLCLK => {
                windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_ID);
                menu::show_menu(app);
                windows_sys::Win32::UI::WindowsAndMessaging::SetTimer(
                    hwnd, TIMER_ID, FRAME_MS, None,
                );
                0
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) & 0xFFFF) as i16;
                if delta > 0 {
                    app.set_width(app.normal_width() + SIZE_STEP);
                } else if delta < 0 {
                    app.set_width(app.normal_width() - SIZE_STEP);
                }
                0
            }
            WM_LBUTTONDOWN => {
                let mut p = POINT { x: 0, y: 0 };
                GetCursorPos(&mut p);
                app.press_mouse_x = p.x;
                app.press_mouse_y = p.y;
                app.drag_active = false;
                // Instant feedback: the annoyed face appears immediately on any press;
                // if this turns into a drag, enter_tilt swaps to the tilted pose right away
                app.annoyed_until = Some(Instant::now() + std::time::Duration::from_millis(1500));
                app.draw_annoyed();
                windows_sys::Win32::UI::Input::KeyboardAndMouse::SetCapture(hwnd);
                0
            }
            WM_MOUSEMOVE => {
                if (wparam as u32 & 1) != 0 {
                    let mut p = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut p);
                    if !app.drag_active {
                        let d = (p.x - app.press_mouse_x).abs() + (p.y - app.press_mouse_y).abs();
                        if d > 5 {
                            app.drag_active = true;
                            app.drag_win_x = app.pos_x;
                            app.drag_win_y = app.pos_y;
                            app.drag_mouse_x = p.x;
                            app.drag_mouse_y = p.y;
                            // Switch to the tilted pose immediately (no lag from the timer)
                            app.enter_tilt();
                        }
                    }
                    if app.drag_active {
                        let nx = app.drag_win_x + (p.x - app.drag_mouse_x);
                        let ny = app.drag_win_y + (p.y - app.drag_mouse_y);
                        app.pos_x = nx;
                        app.pos_y = ny;
                        app.constrain_drag_position();
                        SetWindowPos(
                            hwnd,
                            null_mut(),
                            app.pos_x,
                            app.pos_y,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOZORDER,
                        );
                    }
                }
                0
            }
            WM_LBUTTONUP => {
                let was_drag = app.drag_active;
                app.drag_active = false;
                if was_drag {
                    // Restore the normal pose immediately (no resize-lag glitch)
                    app.exit_tilt();
                    app.save_settings();
                }
                windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                0
            }
            windows_sys::Win32::UI::WindowsAndMessaging::WM_CAPTURECHANGED => {
                if app.drag_active {
                    app.drag_active = false;
                    app.exit_tilt();
                    app.save_settings();
                }
                0
            }
            WM_RBUTTONUP => {
                menu::show_menu(app);
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
            WM_CLOSE => {
                if menu::flush_data(app) {
                    DestroyWindow(hwnd);
                } else {
                    menu::show_menu(app);
                }
                0
            }
            WM_DESTROY => {
                menu::flush_data(app);
                app.save_settings();
                let mut nid = std::mem::zeroed::<NOTIFYICONDATAW>();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = TRAY_ID as u32;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                for i in 1..=HK_VK.len() as i32 {
                    UnregisterHotKey(hwnd, i);
                }
                windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_ID);
                PostQuitMessage(0);
                0
            }
            WM_QUERYENDSESSION => {
                menu::flush_data(app);
                app.save_settings();
                1
            }
            WM_ENDSESSION => {
                if wparam != 0 {
                    menu::flush_data(app);
                    app.save_settings();
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}\n", info);
        let panic_log = std::env::temp_dir().join("cocobar_panic.log");
        std::fs::write(panic_log, &msg).ok();
        eprintln!("{}", msg);
    }));
    unsafe {
        // Single instance: another running copy must not spawn (two cats = lag + crashes)
        let _mutex = CreateMutexW(null_mut(), 1, wstr("cocoBar_SingleInstance"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
        SetProcessDPIAware();
        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_BAR_CLASSES,
        });
        let hinst = GetModuleHandleW(null());

        // Register main window class
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

        // Register menu window class
        let menu_class = wstr("CatMenuWnd");
        let menu_brush = CreateSolidBrush(0x00F8F9FA);
        let mwc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(menu::menu_wnd_proc),
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

        // Register customize panel window class
        let customize_class = wstr("CatCustomizeWnd");
        let cwc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(menu::customize_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: menu_brush,
            lpszMenuName: null(),
            lpszClassName: customize_class,
            hIconSm: null_mut(),
        };
        RegisterClassExW(&cwc);

        // Init app
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        let app_dir = config::app_dir();
        if std::env::var_os("COCOBAR_DATA_DIR").is_none() {
            config::migrate_legacy(&PathBuf::from(&appdata));
        }
        let _ = std::fs::create_dir_all(&app_dir);

        let cfg_path = app_dir.join("config.txt");
        let data_path = app_dir.join("mydata.txt");
        let cfg = config::load_config(&cfg_path);
        let data = match config::load_user_data(&data_path) {
            Ok(data) => data,
            Err(e) => {
                let message: Vec<u16> = format!("Could not read {}: {e}\n\nYour saved data has been left intact. Check the file and try again.", data_path.display()).encode_utf16().chain(Some(0)).collect();
                windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                    null_mut(),
                    message.as_ptr(),
                    wstr("cocoBar"),
                    0x10,
                );
                return;
            }
        };

        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);

        let (min_w, max_lim) = size_limits();
        let mut w = if cfg.size_px > 0 {
            cfg.size_px.clamp(min_w, max_lim)
        } else {
            SIZES[cfg.size_idx.min(2)].clamp(min_w, max_lim)
        };
        if w > sw - 40 {
            w = sw - 40;
        }
        if w < min_w {
            w = min_w;
        }
        let mut h = ((w as f32) * cat::CONTENT_H as f32 / cat::CONTENT_W as f32).round() as i32;
        if h > sh - 80 {
            h = sh - 80;
            w = ((h as f32) * cat::CONTENT_W as f32 / cat::CONTENT_H as f32).round() as i32;
            if w < min_w {
                w = min_w;
            }
        }
        let scale = w as f32 / cat::CONTENT_W as f32;
        let mut pos_x = cfg.pos_x;
        let mut pos_y = cfg.pos_y;
        if pos_x < 0
            || pos_y < 0
            || pos_x.saturating_add(w) > sw + 50
            || pos_y.saturating_add(h) > sh + 50
        {
            pos_x = sw - w - 24;
            pos_y = sh - h - 48;
            if pos_y < 0 {
                pos_y = 24;
            }
        }

        let mut app = App {
            hwnd: null_mut(),
            cat: cat::CatRenderer::new(),
            w,
            h,
            scale,
            pos_x,
            pos_y,
            color: cfg.color,
            size_idx: cfg.size_idx.min(2),
            hdc_screen: null_mut(),
            hdc_mem: null_mut(),
            hbmp: null_mut(),
            bits: null_mut(),
            icon: null_mut(),
            config_path: cfg_path,
            data_path,
            exe: std::env::current_exe().unwrap_or_default(),
            menu_hwnd: null_mut(),
            customize_hwnd: null_mut(),
            settings_hwnd: null_mut(),
            menu_tab: 0,
            note: data.note,
            notes: data.notes,
            editing_note: None,
            notes_editor_open: false,
            notes_page: 0,
            deleted_note: None,
            data_dirty: false,
            data_status: "All changes saved".into(),
            todo_page: 0,
            todos: data.todos,
            drag_active: false,
            drag_win_x: 0,
            drag_win_y: 0,
            drag_mouse_x: 0,
            drag_mouse_y: 0,
            press_mouse_x: 0,
            press_mouse_y: 0,
            tilt_restore_w: 0,
            tilt_restore_h: 0,
            annoyed_until: None,
            annoyed_drawn: false,
            always_on_top: cfg.always_on_top,
            desktop_shortcut: cfg.desktop_shortcut,
            auto_update: cfg.auto_update,
            cosmetic_bell: cfg.cosmetic_bell,
            cosmetic_scarf: cfg.cosmetic_scarf,
            cosmetic_tie: cfg.cosmetic_tie,
            status: std::fs::read_to_string(config::app_dir().join("update-result.txt"))
                .unwrap_or_default(),
            menu_timer_id: 0,
            keep_panels_open: std::env::var("COCOBAR_UI_TEST").as_deref() == Ok("1")
                && std::env::var_os("COCOBAR_DATA_DIR").is_some(),
            hotkeys: cfg.hotkeys,
            settings_scroll: 0,
            mouse_pt: POINT {
                x: i32::MIN,
                y: i32::MIN,
            },
            updater: updater::Updater::new(),
        };

        app.cat.rebuild_layers(app.color, app.w, app.h, app.scale);
        app.cat.rebuild_cosmetics(
            app.cosmetic_bell,
            app.cosmetic_scarf,
            app.cosmetic_tie,
            app.scale,
        );
        // Pre-warm the tilted-pose layers/cosmetics so the first grab is instant
        app.cat.ensure_tilted(cat::tilted_scale(app.scale));

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

        if !app.always_on_top {
            app.set_topmost(false);
        }

        app.hdc_screen = GetDC(null_mut());
        app.hdc_mem = windows_sys::Win32::Graphics::Gdi::CreateCompatibleDC(app.hdc_screen);
        app.rebuild_dib();
        windows_sys::Win32::Graphics::Gdi::SelectObject(app.hdc_mem, app.hbmp);
        app.rebuild_icon();
        app.update_tray();

        // Desktop shortcut is created by default (first run)
        if app.desktop_shortcut
            && std::env::var_os("COCOBAR_DATA_DIR").is_none()
            && !app.desktop_shortcut_exists()
        {
            app.create_desktop_shortcut();
        }

        for (i, &key) in HK_VK.iter().enumerate() {
            if app.hotkey_enabled(i) {
                if RegisterHotKey(
                    hwnd,
                    (i + 1) as i32,
                    MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
                    key,
                ) == 0
                {
                    app.hotkeys &= !(1 << i);
                    app.status = "A shortcut is in use by another app. Enable it again in Settings after freeing it.".into();
                }
            }
        }

        SetTimer(hwnd, TIMER_ID, FRAME_MS, None);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if menu::process_keyboard(&mut app, &msg) {
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        windows_sys::Win32::Graphics::Gdi::DeleteDC(app.hdc_mem);
        windows_sys::Win32::Graphics::Gdi::DeleteObject(app.hbmp);
        ReleaseDC(null_mut(), app.hdc_screen);
        DestroyIcon(app.icon);
        windows_sys::Win32::Foundation::CloseHandle(_mutex);
    }
}

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::CreateSolidBrush;
