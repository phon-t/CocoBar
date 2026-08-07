#![windows_subsystem = "windows"]

mod cat;
mod config;
mod cosmetics;
mod menu;

use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::Mutex;
use std::time::Instant;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{
    CreateProcessW, WaitForSingleObject, CREATE_NO_WINDOW, INFINITE, PROCESS_INFORMATION, STARTUPINFOW,
};
use windows_sys::Win32::UI::Controls::{InitCommonControlsEx, INITCOMMONCONTROLSEX, ICC_BAR_CLASSES};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MOD_ALT, MOD_CONTROL, RegisterHotKey,
};
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION,
    NOTIFYICONDATAW, NOTIFYICON_VERSION_4, Shell_NotifyIconW, CSIDL_DESKTOPDIRECTORY, CSIDL_STARTUP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, CS_DBLCLKS,
    DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect,
    GWLP_USERDATA, HWND_NOTOPMOST, HWND_TOPMOST, ICONINFO, IDC_ARROW, IDI_APPLICATION,
    LoadCursorW, LoadIconW, MF_CHECKED, MF_POPUP, MF_STRING, MF_UNCHECKED,
    PostQuitMessage, RegisterClassExW, SendMessageW, SetForegroundWindow, SetProcessDPIAware,
    SetTimer, SetWindowLongPtrW, SetWindowPos, SM_CXSCREEN, SM_CYSCREEN,
    SWP_NOMOVE, SWP_NOZORDER, TrackPopupMenu, TranslateMessage, ULW_ALPHA,
    UpdateLayeredWindow, WNDCLASSEXW, WM_APP, WM_COMMAND, WM_DESTROY, WM_ENDSESSION,
    WM_HOTKEY, WM_LBUTTONDOWN,     WM_MOUSEWHEEL, WM_MOUSEMOVE, WM_LBUTTONUP, WM_LBUTTONDBLCLK, WM_MOVE, WM_QUERYENDSESSION,
    WM_RBUTTONUP, WM_TIMER, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE, SWP_NOSIZE,
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

pub(crate) const SIZE_STEP: i32 = 40;
pub(crate) const SIZES: [i32; 3] = [320, 520, 760];

// Cat width range in pixels: 100 .. 500
pub(crate) fn size_limits() -> (i32, i32) {
    (100, 500)
}

pub(crate) const BLACK_FULL: &[u8] = include_bytes!("../assets/blackcatfull.png");
pub(crate) const WHITE_FULL: &[u8] = include_bytes!("../assets/whitecatfull.png");
pub(crate) const ORANGE_FULL: &[u8] = include_bytes!("../assets/orangecatfull.png");

pub(crate) const APP_VERSION: &str = "0.5.0";
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
    pub menu_tab: u32,
    pub note: String,
    pub todos: Vec<(String, bool)>,
    pub drag_active: bool,
    pub drag_win_x: i32,
    pub drag_win_y: i32,
    pub drag_mouse_x: i32,
    pub drag_mouse_y: i32,
    pub annoyed_until: Option<Instant>,
    pub prev_mouse_down: bool,
    pub always_on_top: bool,
    pub cosmetic_bell: Option<usize>,
    pub cosmetic_scarf: Option<usize>,
    pub cosmetic_tie: Option<usize>,
    pub status: String,
    pub menu_timer_id: u32,
}

impl App {
    pub(crate) fn set_color(&mut self, c: usize) {
        if self.color == c {
            return;
        }
        self.color = c;
        self.cat.rebuild_layers(c, self.w, self.h, self.scale);
        self.rebuild_icon();
        self.update_tray();
    }

    pub(crate) fn set_width(&mut self, mut new_w: i32) {
        let (min_w, _max_w) = size_limits();
        new_w = new_w.clamp(min_w, size_limits().1);
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let max_w = (sw - 40).max(min_w);
        let max_h = (sh - 60).max(120);
        let new_w = new_w.min(max_w);
        let mut new_h = ((new_w as f32) * cat::CONTENT_H as f32 / cat::CONTENT_W as f32).round() as i32;
        let old_r = self.pos_x + self.w;
        let old_b = self.pos_y + self.h;
        if new_h > max_h {
            new_h = max_h;
            self.w = ((new_h as f32) * cat::CONTENT_W as f32 / cat::CONTENT_H as f32).round() as i32;
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
            SetWindowPos(self.hwnd, null_mut(), self.pos_x, self.pos_y, self.w, self.h, SWP_NOZORDER);
        }
        self.cat.rebuild_layers(self.color, self.w, self.h, self.scale);
        self.cat.rebuild_cosmetics(self.cosmetic_bell, self.cosmetic_scarf, self.cosmetic_tie, self.scale);
        self.rebuild_dib();
    }

    pub(crate) fn set_size(&mut self, idx: usize) {
        self.size_idx = idx.min(2);
        let (min_w, max_lim) = size_limits();
        let old_r = self.pos_x + self.w;
        let old_b = self.pos_y + self.h;
        let sw = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let sh = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let max_w = (sw - 40).max(min_w);
        let max_h = (sh - 60).max(120);
        let mut w = SIZES[idx].clamp(min_w, max_lim).min(max_w);
        let mut h = ((w as f32) * cat::CONTENT_H as f32 / cat::CONTENT_W as f32).round() as i32;
        if h > max_h {
            h = max_h;
            w = ((h as f32) * cat::CONTENT_W as f32 / cat::CONTENT_H as f32).round() as i32;
        }
        self.w = w;
        self.h = h;
        self.scale = w as f32 / cat::CONTENT_W as f32;
        self.pos_x = old_r - self.w;
        self.pos_y = old_b - self.h;
        if self.pos_x < 0 {
            self.pos_x = 0;
        }
        if self.pos_y < 0 {
            self.pos_y = 0;
        }
        unsafe {
            SetWindowPos(self.hwnd, null_mut(), self.pos_x, self.pos_y, self.w, self.h, SWP_NOZORDER);
        }
        self.cat.rebuild_layers(self.color, self.w, self.h, self.scale);
        self.cat.rebuild_cosmetics(self.cosmetic_bell, self.cosmetic_scarf, self.cosmetic_tie, self.scale);
        self.rebuild_dib();
    }

    pub(crate) fn set_topmost(&mut self, on: bool) {
        self.always_on_top = on;
        unsafe {
            SetWindowPos(
                self.hwnd,
                if on { HWND_TOPMOST } else { HWND_NOTOPMOST },
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE,
            );
        }
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
        let resized = image::imageops::resize(&mut img, 32, 32, image::imageops::FilterType::Triangle);
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
                biSize: std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::BITMAPINFOHEADER>() as u32,
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
                rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0,
            }],
        };
        let mut bits: *mut c_void = null_mut();
        let color_bitmap = unsafe {
            windows_sys::Win32::Graphics::Gdi::CreateDIBSection(
                hdc, &bmi, 0, &mut bits as *mut *mut c_void, null_mut(), 0,
            )
        };
        unsafe {
            if !bits.is_null() {
                std::ptr::copy_nonoverlapping(pixels.as_ptr(), bits as *mut u8, pixels.len());
            }
            let info = ICONINFO {
                fIcon: 1, xHotspot: 0, yHotspot: 0,
                hbmMask: null_mut(), hbmColor: color_bitmap,
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
            Shell_NotifyIconW(NIM_ADD, &nid);
            nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            Shell_NotifyIconW(NIM_SETVERSION, &nid);
        }
    }

    fn popup_menu(&mut self) {
        unsafe {
            let popmenu = CreatePopupMenu();
            AppendMenuW(popmenu, MF_STRING | if self.color == 0 { MF_CHECKED } else { MF_UNCHECKED }, MENU_BLACK, wstr("Black Cat"));
            AppendMenuW(popmenu, MF_STRING | if self.color == 1 { MF_CHECKED } else { MF_UNCHECKED }, MENU_WHITE, wstr("White Cat"));
            AppendMenuW(popmenu, MF_STRING | if self.color == 2 { MF_CHECKED } else { MF_UNCHECKED }, MENU_ORANGE, wstr("Orange Cat"));
            AppendMenuW(popmenu, MF_STRING, 0, wstr(""));
            let size_menu = CreatePopupMenu();
            for (i, label) in ["Small", "Medium", "Large"].iter().enumerate() {
                AppendMenuW(size_menu, MF_STRING | if self.size_idx == i { MF_CHECKED } else { MF_UNCHECKED }, MENU_SIZE_SMALL + i, wstr(label));
            }
            AppendMenuW(popmenu, MF_POPUP, size_menu as usize, wstr("Size"));
            AppendMenuW(popmenu, MF_STRING, 0, wstr(""));
            AppendMenuW(popmenu, MF_STRING, MENU_SHORTCUT_DESKTOP, wstr("Add shortcut to Desktop"));
            AppendMenuW(popmenu, MF_STRING | if self.startup_enabled() { MF_CHECKED } else { MF_UNCHECKED }, MENU_STARTUP, wstr("Start with Windows"));
            AppendMenuW(popmenu, MF_STRING, 0, wstr(""));
            AppendMenuW(popmenu, MF_STRING, MENU_EXIT, wstr("Exit"));
            let mut pt = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pt);
            SetForegroundWindow(self.hwnd);
            let cmd = TrackPopupMenu(popmenu, 0x0100 | 0x0002, pt.x, pt.y, 0, self.hwnd, null());
            if cmd != 0 {
                SendMessageW(self.hwnd, WM_COMMAND, cmd as WPARAM, 0);
            }
            DestroyMenu(popmenu);
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
                DestroyWindow(self.hwnd);
            },
            _ => {}
        }
    }

    fn app_dir(&self) -> PathBuf {
        self.config_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf()
    }

    fn startup_lnk(&self) -> Option<PathBuf> {
        known_folder(CSIDL_STARTUP).map(|p| p.join("cocoBar.lnk"))
    }

    pub(crate) fn startup_enabled(&self) -> bool {
        self.startup_lnk().map(|p| p.exists()).unwrap_or(false)
    }

    pub(crate) fn set_startup(&self, on: bool) {
        let Some(lnk) = self.startup_lnk() else { return };
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
            ps_quote(&lnk.to_string_lossy()), ps_quote(&exe), ps_quote(&dir), ps_quote(&ico_str),
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
            .expect("icon decode").to_rgba8();
        let resized = image::imageops::resize(&mut img, 32, 32, image::imageops::FilterType::Triangle);
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
        let Some(desktop) = known_folder(CSIDL_DESKTOPDIRECTORY) else { return };
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
            ps_quote(&lnk.to_string_lossy()), ps_quote(&exe), ps_quote(&dir), ps_quote(&ico_str),
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

fn wstr(s: &str) -> *const u16 {
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
        if windows_sys::Win32::UI::Shell::SHGetFolderPathW(null_mut(), csidl as i32, null_mut(), 0, buf.as_mut_ptr()) != 0 {
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
        let ok = CreateProcessW(null(), c.as_mut_ptr(), null(), null(), 0, CREATE_NO_WINDOW, null_mut(), null(), &si, &mut pi);
        if ok == 0 { return false; }
        WaitForSingleObject(pi.hProcess, INFINITE);
        windows_sys::Win32::Foundation::CloseHandle(pi.hProcess);
        windows_sys::Win32::Foundation::CloseHandle(pi.hThread);
        true
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
                // Detect single-click on cat
                let mouse_down = GetAsyncKeyState(0x01) as u16 & 0x8000 != 0;
                if mouse_down && !app.prev_mouse_down {
                    let mut pt = POINT { x: 0, y: 0 };
                    if GetCursorPos(&mut pt) != 0 {
                        if pt.x >= app.pos_x && pt.x <= app.pos_x + app.w
                            && pt.y >= app.pos_y && pt.y <= app.pos_y + app.h
                        {
                            app.annoyed_until = Some(Instant::now() + std::time::Duration::from_millis(1500));
                        }
                    }
                }
                app.prev_mouse_down = mouse_down;

                // Render
                let now = Instant::now();
                if let Some(until) = app.annoyed_until {
                    if now < until {
                        if let Some(annoyed) = &app.cat.annoyed {
                            app.cat.buf.fill(0);
                            let dw = app.w as usize;
                            let dh = app.h as usize;
                            if annoyed.w == dw && annoyed.h == dh {
                                app.cat.buf.copy_from_slice(&annoyed.data);
                            } else {
                                cat::blit(&mut app.cat.buf, dw, dh, &annoyed.data, annoyed.w, annoyed.h, 0, 0);
                            }
                            // Keep cosmetics visible on the annoyed face
                            if let Some(s) = &app.cat.scarf {
                                cat::blit(&mut app.cat.buf, dw, dh, &s.data, s.w, s.h, 0, 0);
                            }
                            if let Some(b) = &app.cat.bell {
                                cat::blit(&mut app.cat.buf, dw, dh, &b.data, b.w, b.h, 0, 0);
                            }
                            if let Some(t) = &app.cat.tie {
                                cat::blit(&mut app.cat.buf, dw, dh, &t.data, t.w, t.h, 0, 0);
                            }
                            if !app.bits.is_null() {
                                std::ptr::copy_nonoverlapping(app.cat.buf.as_ptr(), app.bits as *mut u8, app.cat.buf.len());
                                let pt_dst = POINT { x: app.pos_x, y: app.pos_y };
                                let size = SIZE { cx: app.w, cy: app.h };
                                let pt_src = POINT { x: 0, y: 0 };
                                let blend = windows_sys::Win32::Graphics::Gdi::BLENDFUNCTION {
                                    BlendOp: 0, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: 1,
                                };
                                UpdateLayeredWindow(app.hwnd, app.hdc_screen, &pt_dst, &size, app.hdc_mem, &pt_src, 0, &blend, ULW_ALPHA);
                            }
                        }
                        return 0;
                    } else {
                        app.annoyed_until = None;
                    }
                }
                app.cat.render(app.pos_x, app.pos_y, app.w, app.h, app.scale, app.hdc_screen, app.hdc_mem, app.bits, app.hwnd);
                0
            }
            TRAY_CB => {
                match (lparam as u32) & 0xFFFF {
                    WM_RBUTTONUP | WM_LBUTTONDOWN => {
                        // Kill cat timer while menu is open
                        windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_ID);
                        menu::show_menu(app);
                        // Resume cat timer
                        windows_sys::Win32::UI::WindowsAndMessaging::SetTimer(hwnd, TIMER_ID, FRAME_MS, None);
                    }
                    _ => {}
                }
                0
            }
            WM_COMMAND => {
                let id = (wparam as u32 & 0xFFFF) as usize;
                app.command(id);
                if id == MENU_EXIT {
                    let cfg = config::ConfigData {
                        color: app.color, size_idx: app.size_idx, size_px: app.w, pos_x: app.pos_x, pos_y: app.pos_y,
                        always_on_top: app.always_on_top,
                        cosmetic_bell: app.cosmetic_bell, cosmetic_scarf: app.cosmetic_scarf, cosmetic_tie: app.cosmetic_tie,
                    };
                    config::save_config(&app.config_path, &cfg);
                }
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
                    HK_EXIT => {
                        DestroyWindow(app.hwnd);
                    }
                    _ => {}
                }
                0
            }
            WM_LBUTTONDBLCLK => {
                windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_ID);
                menu::show_menu(app);
                windows_sys::Win32::UI::WindowsAndMessaging::SetTimer(hwnd, TIMER_ID, FRAME_MS, None);
                0
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) & 0xFFFF) as i16;
                if delta > 0 { app.set_width(app.w + SIZE_STEP); }
                else if delta < 0 { app.set_width(app.w - SIZE_STEP); }
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
                windows_sys::Win32::UI::Input::KeyboardAndMouse::SetCapture(hwnd);
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
                    SetWindowPos(hwnd, null_mut(), nx, ny, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
                }
                0
            }
            WM_LBUTTONUP => {
                if app.drag_active {
                    app.drag_active = false;
                    windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                }
                0
            }
            WM_RBUTTONUP => {
                app.popup_menu();
                0
            }
            WM_MOVE => {
                let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                if GetWindowRect(hwnd, &mut r) != 0 {
                    app.pos_x = r.left;
                    app.pos_y = r.top;
                }
                0
            }
            WM_DESTROY => {
                let cfg = config::ConfigData {
                    color: app.color, size_idx: app.size_idx, size_px: app.w, pos_x: app.pos_x, pos_y: app.pos_y,
                    always_on_top: app.always_on_top,
                    cosmetic_bell: app.cosmetic_bell, cosmetic_scarf: app.cosmetic_scarf, cosmetic_tie: app.cosmetic_tie,
                };
                config::save_config(&app.config_path, &cfg);
                let data = config::UserData { note: app.note.clone(), todos: app.todos.clone() };
                config::save_user_data(&app.data_path, &data);
                let mut nid = std::mem::zeroed::<NOTIFYICONDATAW>();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = TRAY_ID as u32;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                PostQuitMessage(0);
                0
            }
            WM_QUERYENDSESSION => {
                let cfg = config::ConfigData {
                    color: app.color, size_idx: app.size_idx, size_px: app.w, pos_x: app.pos_x, pos_y: app.pos_y,
                    always_on_top: app.always_on_top,
                    cosmetic_bell: app.cosmetic_bell, cosmetic_scarf: app.cosmetic_scarf, cosmetic_tie: app.cosmetic_tie,
                };
                config::save_config(&app.config_path, &cfg);
                let data = config::UserData { note: app.note.clone(), todos: app.todos.clone() };
                config::save_user_data(&app.data_path, &data);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_ENDSESSION => {
                let cfg = config::ConfigData {
                    color: app.color, size_idx: app.size_idx, size_px: app.w, pos_x: app.pos_x, pos_y: app.pos_y,
                    always_on_top: app.always_on_top,
                    cosmetic_bell: app.cosmetic_bell, cosmetic_scarf: app.cosmetic_scarf, cosmetic_tie: app.cosmetic_tie,
                };
                config::save_config(&app.config_path, &cfg);
                let data = config::UserData { note: app.note.clone(), todos: app.todos.clone() };
                config::save_user_data(&app.data_path, &data);
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

        // Register main window class
        let class_name = wstr("cocoBarWnd");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_DBLCLKS,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0, cbWndExtra: 0,
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
            cbClsExtra: 0, cbWndExtra: 0,
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
            cbClsExtra: 0, cbWndExtra: 0,
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
        let app_dir = PathBuf::from(&appdata).join("cocoBar");
        config::migrate_legacy(&PathBuf::from(&appdata));
        let _ = std::fs::create_dir_all(&app_dir);

        let cfg_path = app_dir.join("config.txt");
        let data_path = app_dir.join("mydata.txt");
        let cfg = config::load_config(&cfg_path);
        let data = config::load_user_data(&data_path);

        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);

        let (min_w, max_lim) = size_limits();
        let mut w = if cfg.size_px > 0 {
            cfg.size_px.clamp(min_w, max_lim)
        } else {
            SIZES[cfg.size_idx.min(2)].clamp(min_w, max_lim)
        };
        if w > sw - 40 { w = sw - 40; }
        if w < min_w { w = min_w; }
        let mut h = ((w as f32) * cat::CONTENT_H as f32 / cat::CONTENT_W as f32).round() as i32;
        if h > sh - 80 {
            h = sh - 80;
            w = ((h as f32) * cat::CONTENT_W as f32 / cat::CONTENT_H as f32).round() as i32;
            if w < min_w { w = min_w; }
        }
        let scale = w as f32 / cat::CONTENT_W as f32;
        let mut pos_x = cfg.pos_x;
        let mut pos_y = cfg.pos_y;
        if pos_x < 0 || pos_y < 0 || pos_x.saturating_add(w) > sw + 50 || pos_y.saturating_add(h) > sh + 50 {
            pos_x = sw - w - 24;
            pos_y = sh - h - 48;
            if pos_y < 0 { pos_y = 24; }
        }

        let mut app = App {
            hwnd: null_mut(),
            cat: cat::CatRenderer::new(),
            w, h, scale, pos_x, pos_y,
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
            menu_tab: 0,
            note: data.note,
            todos: data.todos,
            drag_active: false,
            drag_win_x: 0, drag_win_y: 0, drag_mouse_x: 0, drag_mouse_y: 0,
            annoyed_until: None,
            prev_mouse_down: false,
            always_on_top: cfg.always_on_top,
            cosmetic_bell: cfg.cosmetic_bell,
            cosmetic_scarf: cfg.cosmetic_scarf,
            cosmetic_tie: cfg.cosmetic_tie,
            status: String::new(),
            menu_timer_id: 0,
        };

        app.cat.rebuild_layers(app.color, app.w, app.h, app.scale);
        app.cat.rebuild_cosmetics(app.cosmetic_bell, app.cosmetic_scarf, app.cosmetic_tie, app.scale);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            wstr("cocoBar"),
            WS_POPUP | WS_VISIBLE,
            app.pos_x, app.pos_y, app.w, app.h,
            null_mut(), null_mut(), hinst, null(),
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
        if !app.desktop_shortcut_exists() {
            app.create_desktop_shortcut();
        }

        RegisterHotKey(hwnd, HK_BLACK, MOD_CONTROL | MOD_ALT, 0x42);
        RegisterHotKey(hwnd, HK_WHITE, MOD_CONTROL | MOD_ALT, 0x57);
        RegisterHotKey(hwnd, HK_ORANGE, MOD_CONTROL | MOD_ALT, 0x4F);
        RegisterHotKey(hwnd, HK_SMALL, MOD_CONTROL | MOD_ALT, 0x31);
        RegisterHotKey(hwnd, HK_MEDIUM, MOD_CONTROL | MOD_ALT, 0x32);
        RegisterHotKey(hwnd, HK_LARGE, MOD_CONTROL | MOD_ALT, 0x33);
        RegisterHotKey(hwnd, HK_EXIT, MOD_CONTROL | MOD_ALT, 0x58);

        SetTimer(hwnd, TIMER_ID, FRAME_MS, None);

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::Graphics::Gdi::CreateSolidBrush;
