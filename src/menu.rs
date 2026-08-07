use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::SS_LEFT;
use windows_sys::Win32::UI::Controls::BST_CHECKED;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetFocus, VK_ESCAPE};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(crate) const TAB_TODO: u32 = 401;
pub(crate) const TAB_NOTES: u32 = 402;

pub(crate) const ED_TODO_INPUT: u32 = 410;
pub(crate) const BTN_TODO_ADD: u32 = 411;
pub(crate) const BTN_TODO_CLEAR: u32 = 412;

pub(crate) const ED_NOTES: u32 = 420;
pub(crate) const BTN_NOTES_SAVE: u32 = 421;

pub(crate) const BTN_CUSTOMIZE: u32 = 430;
pub(crate) const BTN_EXIT: u32 = 431;
pub(crate) const BTN_SCAN: u32 = 432;

pub(crate) const BTN_COS_BACK: u32 = 440;
pub(crate) const BTN_BELL_NONE: u32 = 441;
pub(crate) const BTN_BELL_0: u32 = 442;
pub(crate) const BTN_BELL_1: u32 = 443;
pub(crate) const BTN_SCARF_NONE: u32 = 450;
pub(crate) const BTN_SCARF_0: u32 = 451;
pub(crate) const BTN_SCARF_1: u32 = 452;
pub(crate) const BTN_SCARF_2: u32 = 453;
pub(crate) const BTN_SCARF_3: u32 = 454;
pub(crate) const BTN_SCARF_4: u32 = 455;
pub(crate) const BTN_TIE_NONE: u32 = 460;
pub(crate) const BTN_TIE_0: u32 = 461;
pub(crate) const BTN_TIE_1: u32 = 462;
pub(crate) const BTN_TIE_2: u32 = 463;
pub(crate) const BTN_COS_CLEAR_ALL: u32 = 464;
pub(crate) const BTN_TOPMOST: u32 = 465;
pub(crate) const BTN_COLOR_BLACK: u32 = 470;
pub(crate) const BTN_COLOR_WHITE: u32 = 471;
pub(crate) const BTN_COLOR_ORANGE: u32 = 472;
pub(crate) const ED_SIZE: u32 = 484;
pub(crate) const BTN_SIZE_APPLY: u32 = 485;
pub(crate) const BTN_DESKTOP: u32 = 490;
pub(crate) const BTN_STARTUP: u32 = 491;

pub(crate) const MENU_W: i32 = 470;
pub(crate) const MENU_H: i32 = 430;
pub(crate) const TIMER_MENU_CLOSE: u32 = 3;

const COS_W: i32 = 400;
const COS_H: i32 = 660;

const CLR_BG: u32 = 0x00F8F9FA;
const CLR_WHITE: u32 = 0x00FFFFFF;
const CLR_SEP: u32 = 0x00E8E8E8;
const CLR_PILL_ON: u32 = 0x004A90D9;
const CLR_PILL_OFF: u32 = 0x00F0F0F0;
const CLR_PILL_BDR: u32 = 0x00D8D8D8;
const CLR_CLOSE_BG: u32 = 0x00FFFFFF;
const CLR_CLOSE_BDR: u32 = 0x00C8C8C8;
const CLR_CLOSE_TXT: u32 = 0x00777777;
const CLR_CARD_BDR: u32 = 0x00E0E0E0;
const CLR_TXT: u32 = 0x00333333;
const CLR_TXT_DIM: u32 = 0x00666666;
const CLR_TXT_DONE: u32 = 0x00AAAAAA;
const CLR_CHK: u32 = 0x00888888;

fn wstr(s: &str) -> *const u16 {
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

fn in_pill(x: i32, y: i32, cx: i32) -> bool {
    x >= cx - 46 && x <= cx + 46 && y >= 8 && y <= 56
}

fn in_bubble(x: i32, y: i32, cx: i32, cy: i32, r: i32) -> bool {
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= r * r
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

unsafe fn get_ctl_text(hwnd: HWND) -> String {
    let n = SendMessageW(hwnd, 0x000E, 0, 0) as usize;
    let mut buf = vec![0u16; n + 1];
    SendMessageW(hwnd, 0x000D, (n + 1) as _, buf.as_mut_ptr() as _);
    String::from_utf16_lossy(&buf[..n])
}

unsafe fn make_ctl(
    parent: HWND,
    id: u32,
    class: &str,
    title: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    style: u32,
    font: *mut core::ffi::c_void,
) -> HWND {
    let hinst = GetModuleHandleW(std::ptr::null());
    let c = CreateWindowExW(
        0,
        wstr(class),
        wstr(title),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | style,
        x,
        y,
        w,
        h,
        parent,
        id as _,
        hinst,
        std::ptr::null(),
    );
    SendMessageW(c, WM_SETFONT, font as WPARAM, 1);
    c
}

pub(crate) fn show_menu(app: &mut super::App) {
    unsafe {
        if !app.menu_hwnd.is_null() {
            SetForegroundWindow(app.menu_hwnd);
            return;
        }
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let mut mx = app.pos_x + app.w + 8;
        if mx + MENU_W > sw - 8 {
            mx = app.pos_x - MENU_W - 8;
        }
        if mx < 8 {
            mx = (sw - MENU_W) / 2;
        }
        let mut my = app.pos_y + (app.h - MENU_H) / 2;
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
            app.hwnd,
            std::ptr::null_mut(),
            GetModuleHandleW(std::ptr::null()),
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return;
        }
        let rgn = CreateRoundRectRgn(0, 0, MENU_W + 1, MENU_H + 1, 18, 18);
        SetWindowRgn(hwnd, rgn, 1);
        app.menu_hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as *mut super::App as isize);
        create_menu_controls(hwnd);
        show_tab_content(app, 0);
        refresh_notes(app);
        ShowWindow(hwnd, 1);
        SetForegroundWindow(hwnd);
    }
}

unsafe fn create_menu_controls(hwnd: HWND) {
    let font = GetStockObject(DEFAULT_GUI_FONT) as _;

    make_ctl(hwnd, ED_TODO_INPUT, "EDIT", "Type your to do here", 20, 344, 310, 26, 0x0080, font);
    make_ctl(hwnd, BTN_TODO_ADD, "BUTTON", "Add", 338, 342, 50, 28, 0, font);
    make_ctl(hwnd, BTN_TODO_CLEAR, "BUTTON", "Clear All", 396, 342, 56, 28, 0, font);

    make_ctl(
        hwnd,
        ED_NOTES,
        "EDIT",
        "",
        20,
        84,
        430,
        280,
        0x0004 | 0x0040 | 0x1000 | 0x00200000 | 0x0100,
        font,
    );
    make_ctl(hwnd, BTN_NOTES_SAVE, "BUTTON", "Save Note", 20, 374, 100, 28, 0, font);

    make_ctl(hwnd, BTN_CUSTOMIZE, "BUTTON", "CUSTOMIZE", 20, 380, 110, 28, 0, font);
    make_ctl(hwnd, BTN_SCAN, "BUTTON", "CHECK UPDATE", 140, 380, 120, 28, 0, font);
    make_ctl(hwnd, BTN_EXIT, "BUTTON", "EXIT", 330, 380, 120, 28, 0, font);
}

pub(crate) fn show_tab_content(app: &super::App, tab: u32) {
    if app.menu_hwnd.is_null() {
        return;
    }
    unsafe {
        let todo_ids = [ED_TODO_INPUT, BTN_TODO_ADD, BTN_TODO_CLEAR];
        let notes_ids = [ED_NOTES, BTN_NOTES_SAVE];
        for &id in &todo_ids {
            let c = GetDlgItem(app.menu_hwnd, id as i32);
            if !c.is_null() {
                ShowWindow(c, if tab == 0 { SW_SHOW as i32 } else { SW_HIDE as i32 });
            }
        }
        for &id in &notes_ids {
            let c = GetDlgItem(app.menu_hwnd, id as i32);
            if !c.is_null() {
                ShowWindow(c, if tab == 1 { SW_SHOW as i32 } else { SW_HIDE as i32 });
            }
        }
        InvalidateRect(app.menu_hwnd, std::ptr::null(), 1);
    }
}

unsafe fn draw_text(
    hdc: *mut core::ffi::c_void,
    s: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    clr: u32,
    size: i32,
    weight: i32,
    flags: u32,
) {
    let font = CreateFontW(
        -size, 0, 0, 0, weight, 0, 0, 0, 1, 0, 0, 0, 0,
        wstr("Segoe UI") as _,
    );
    let old = SelectObject(hdc, font);
    SetBkMode(hdc, 1);
    SetTextColor(hdc, clr);
    let mut rc = RECT { left: x, top: y, right: x + w, bottom: y + h };
    DrawTextW(hdc, wstr(s) as *mut u16, -1, &mut rc, flags);
    SelectObject(hdc, old);
    DeleteObject(font);
}

pub(crate) fn paint_menu(hwnd: HWND, app: &super::App) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, MENU_W, MENU_H);
        SelectObject(mem, bmp);

        let bg = CreateSolidBrush(CLR_BG);
        FillRect(
            mem,
            &RECT { left: 0, top: 0, right: MENU_W, bottom: MENU_H },
            bg,
        );
        DeleteObject(bg);

        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(
            mem,
            &RECT { left: 0, top: 0, right: MENU_W, bottom: 70 },
            hdr,
        );
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 70, std::ptr::null_mut());
        LineTo(mem, MENU_W, 70);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

        // Tab pills: To Do / Notes
        let labels = ["To Do", "Notes"];
        let centers = [140, 300];
        for i in 0..2usize {
            let cx = centers[i];
            let active = (i as u32) == app.menu_tab;
            let pill = CreateRoundRectRgn(cx - 48, 14, cx + 48, 52, 20, 20);
            let fill = if active { CLR_PILL_ON } else { CLR_PILL_OFF };
            let pb = CreateSolidBrush(fill);
            FillRgn(mem, pill, pb);
            DeleteObject(pb);
            if !active {
                let eb = CreateSolidBrush(CLR_PILL_BDR);
                FrameRgn(mem, pill, eb, 1, 1);
                DeleteObject(eb);
            }
            DeleteObject(pill);
            draw_text(
                mem,
                labels[i],
                cx - 46,
                14,
                92,
                38,
                if active { CLR_WHITE } else { CLR_TXT_DIM },
                13,
                if active { 600 } else { 400 },
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        }

        // Close button: a detached bubble poking out of the top-right corner.
        // The window's rounded region clips its right edge, producing a cut-out look.
        let close_brush = CreateSolidBrush(CLR_CLOSE_BG);
        let close_pen = CreatePen(PS_SOLID, 1, CLR_CLOSE_BDR);
        let old_cb = SelectObject(mem, close_brush);
        let old_cp = SelectObject(mem, close_pen);
        Ellipse(mem, MENU_W - 36, 7, MENU_W + 2, 45);
        SelectObject(mem, old_cb);
        SelectObject(mem, old_cp);
        DeleteObject(close_brush);
        DeleteObject(close_pen);

        // A short stem bridging the bubble to the cut corner makes it look deliberate
        let stem_pen = CreatePen(PS_SOLID, 2, CLR_CLOSE_BDR);
        let old_sp = SelectObject(mem, stem_pen);
        MoveToEx(mem, MENU_W - 30, 45, std::ptr::null_mut());
        LineTo(mem, MENU_W - 30, MENU_H);
        SelectObject(mem, old_sp);
        DeleteObject(stem_pen);

        draw_text(
            mem,
            "\u{00D7}",
            MENU_W - 38,
            11,
            36,
            32,
            CLR_CLOSE_TXT,
            15,
            600,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        let card = CreateRoundRectRgn(12, 78, MENU_W - 12, 370, 10, 10);
        let wb = CreateSolidBrush(CLR_WHITE);
        FillRgn(mem, card, wb);
        DeleteObject(wb);
        let fb = CreateSolidBrush(CLR_CARD_BDR);
        FrameRgn(mem, card, fb, 1, 1);
        DeleteObject(fb);
        DeleteObject(card);

        if app.menu_tab == 0 {
            let items_y0 = 88;
            let item_h = 26;
            let max_items = ((340 - items_y0) / item_h) as usize;
            let count = app.todos.len().min(max_items);
            for i in 0..count {
                let (ref text, done) = app.todos[i];
                let y = items_y0 + (i as i32) * item_h;
                let chk_x = 24;
                let chk_y = y + 4;
                let chk_sz = 16;
                let border_pen = CreatePen(PS_SOLID, 1, CLR_CHK);
                let old_p = SelectObject(mem, border_pen);
                let old_b = SelectObject(mem, GetStockObject(5));
                Rectangle(mem, chk_x, chk_y, chk_x + chk_sz, chk_y + chk_sz);
                SelectObject(mem, old_p);
                SelectObject(mem, old_b);
                DeleteObject(border_pen);
                if done {
                    let check_pen = CreatePen(PS_SOLID, 2, CLR_PILL_ON);
                    let old_cp = SelectObject(mem, check_pen);
                    MoveToEx(mem, chk_x + 3, chk_y + 8, std::ptr::null_mut());
                    LineTo(mem, chk_x + 7, chk_y + 12);
                    LineTo(mem, chk_x + 13, chk_y + 3);
                    SelectObject(mem, old_cp);
                    DeleteObject(check_pen);
                }
                draw_text(
                    mem,
                    text,
                    46,
                    y,
                    MENU_W - 70,
                    item_h,
                    if done { CLR_TXT_DONE } else { CLR_TXT },
                    13,
                    400,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                );
                if done {
                    let strike_pen = CreatePen(PS_SOLID, 1, CLR_TXT_DONE);
                    let old_sp = SelectObject(mem, strike_pen);
                    let tlen = (text.len() as i32 * 7).min(MENU_W - 70);
                    MoveToEx(mem, 46, y + item_h / 2, std::ptr::null_mut());
                    LineTo(mem, 46 + tlen, y + item_h / 2);
                    SelectObject(mem, old_sp);
                    DeleteObject(strike_pen);
                }
            }
        }

        // Update status line (check-for-update feedback)
        if !app.status.is_empty() {
            draw_text(
                mem,
                &app.status,
                12,
                410,
                MENU_W - 24,
                16,
                CLR_TXT_DIM,
                11,
                400,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        }

        BitBlt(hdc, 0, 0, MENU_W, MENU_H, mem, 0, 0, SRCCOPY);
        DeleteDC(mem);
        DeleteObject(bmp);
        EndPaint(hwnd, &mut ps);
    }
}

pub(crate) fn handle_command(app: &mut super::App, id: u32) {
    match id {
        TAB_TODO => {
            app.menu_tab = 0;
            show_tab_content(app, 0);
        }
        TAB_NOTES => {
            app.menu_tab = 1;
            show_tab_content(app, 1);
        }
        BTN_TODO_ADD => {
            if app.menu_hwnd.is_null() {
                return;
            }
            unsafe {
                let c = GetDlgItem(app.menu_hwnd, ED_TODO_INPUT as i32);
                if !c.is_null() {
                    let text = get_ctl_text(c);
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        app.todos.push((text, false));
                        super::config::save_user_data(
                            &app.data_path,
                            &super::config::UserData { note: app.note.clone(), todos: app.todos.clone() },
                        );
                        SendMessageW(c, 0x000C, 0, 0);
                        refresh_todo_list(app);
                    }
                }
            }
        }
        BTN_TODO_CLEAR => {
            app.todos.clear();
            super::config::save_user_data(
                &app.data_path,
                &super::config::UserData { note: app.note.clone(), todos: app.todos.clone() },
            );
            refresh_todo_list(app);
        }
        BTN_NOTES_SAVE => {
            if app.menu_hwnd.is_null() {
                return;
            }
            unsafe {
                let c = GetDlgItem(app.menu_hwnd, ED_NOTES as i32);
                if !c.is_null() {
                    app.note = get_ctl_text(c);
                    super::config::save_user_data(
                        &app.data_path,
                        &super::config::UserData { note: app.note.clone(), todos: app.todos.clone() },
                    );
                }
            }
        }
        BTN_CUSTOMIZE => {
            show_customize_panel(app);
        }
        BTN_SCAN => {
            app.status = "Checking for updates...".to_string();
            refresh_todo_list(app);
            app.check_for_update();
        }
        BTN_EXIT => {
            unsafe {
                if !app.menu_hwnd.is_null() {
                    DestroyWindow(app.menu_hwnd);
                }
                DestroyWindow(app.hwnd);
            }
        }
        _ => {}
    }
}

pub(crate) fn refresh_todo_list(app: &super::App) {
    if !app.menu_hwnd.is_null() {
        unsafe {
            InvalidateRect(app.menu_hwnd, std::ptr::null(), 1);
        }
    }
}

pub(crate) fn refresh_notes(app: &super::App) {
    if app.menu_hwnd.is_null() {
        return;
    }
    unsafe {
        let c = GetDlgItem(app.menu_hwnd, ED_NOTES as i32);
        if !c.is_null() {
            SetWindowTextW(c, wstr(&app.note));
        }
    }
}

fn version_tuple(s: &str) -> (u64, u64, u64) {
    let v = s.trim_start_matches('v');
    let mut it = v.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

fn save_cfg(app: &super::App) {
    super::config::save_config(
        &app.config_path,
        &super::config::ConfigData {
            color: app.color,
            size_idx: app.size_idx,
            size_px: app.w,
            pos_x: app.pos_x,
            pos_y: app.pos_y,
            always_on_top: app.always_on_top,
            cosmetic_bell: app.cosmetic_bell,
            cosmetic_scarf: app.cosmetic_scarf,
            cosmetic_tie: app.cosmetic_tie,
        },
    );
}

fn install_update(app: &mut super::App, url: &str) {
    let exe = app.exe.to_string_lossy().to_string();
    let pid = std::process::id();
    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         $dir = Join-Path $env:TEMP 'cocoBar_update'; \
         New-Item -ItemType Directory -Force -Path $dir | Out-Null; \
         Invoke-WebRequest -Uri '{url}' -OutFile (Join-Path $dir 'new.exe') -UseBasicParsing; \
         while (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 300 }}; \
         Start-Sleep -Milliseconds 500; \
         Copy-Item -Force (Join-Path $dir 'new.exe') '{exe}'; \
         Start-Process '{exe}'",
        url = url,
        pid = pid,
        exe = exe
    );
    let _ = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .spawn();
    unsafe {
        DestroyWindow(app.hwnd);
    }
}

pub(crate) fn check_for_update(app: &mut super::App) {
    let out = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "$r = Invoke-RestMethod -Uri 'https://api.github.com/repos/phon-t/CocoBar/releases/latest' -UseBasicParsing; \
             $a = $r.assets | Where-Object { $_.name -like '*.exe' } | Select-Object -First 1; \
             '{0}|{1}' -f $r.tag_name, $a.browser_download_url",
        ])
        .output();
    match out {
        Ok(o) => {
            let line = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if line.is_empty() {
                app.status = "Could not reach GitHub.".to_string();
            } else {
                let mut parts = line.splitn(2, '|');
                let tag = parts.next().unwrap_or("").trim();
                let url = parts.next().unwrap_or("").trim();
                let remote = version_tuple(tag);
                let current = version_tuple(super::APP_VERSION);
                if remote <= current {
                    app.status = format!("Up to date (v{}).", super::APP_VERSION);
                } else if url.is_empty() {
                    app.status = format!(
                        "New v{} available, but release has no installer.",
                        tag.trim_start_matches('v')
                    );
                } else {
                    app.status = format!(
                        "New v{}! Downloading and installing...",
                        tag.trim_start_matches('v')
                    );
                    refresh_todo_list(app);
                    install_update(app, url);
                    return;
                }
            }
        }
        Err(_) => {
            app.status = "Failed to check for updates.".to_string();
        }
    }
    refresh_todo_list(app);
}

pub(crate) unsafe extern "system" fn menu_wnd_proc(
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
        let app = &mut *(app_ptr as *mut super::App);
        match msg {
            WM_COMMAND => {
                let id = wparam as u32 & 0xFFFF;
                handle_command(app, id);
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
                app.menu_hwnd = std::ptr::null_mut();
                KillTimer(hwnd, TIMER_MENU_CLOSE as usize);
                app.menu_timer_id = 0;
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_menu(hwnd, app);
                0
            }
            WM_ACTIVATE => {
                if wparam == 0 {
                    app.menu_timer_id =
                        SetTimer(hwnd, TIMER_MENU_CLOSE as usize, 50, None) as u32;
                }
                0
            }
            WM_TIMER => {
                if wparam == TIMER_MENU_CLOSE as WPARAM {
                    let focus = GetFocus();
                    if focus.is_null() || !is_child_of(hwnd, focus) {
                        KillTimer(hwnd, TIMER_MENU_CLOSE as usize);
                        app.menu_timer_id = 0;
                        DestroyWindow(hwnd);
                    }
                }
                0
            }
            WM_LBUTTONUP => {
                let x = (lparam as u32 & 0xFFFF) as i32;
                let y = ((lparam as u32 >> 16) & 0xFFFF) as i32;
                if in_pill(x, y, 140) {
                    handle_command(app, TAB_TODO);
                } else if in_pill(x, y, 300) {
                    handle_command(app, TAB_NOTES);
                } else if in_bubble(x, y, MENU_W - 17, 26, 21) {
                    KillTimer(hwnd, TIMER_MENU_CLOSE as usize);
                    app.menu_timer_id = 0;
                    DestroyWindow(hwnd);
                } else if app.menu_tab == 0 {
                    let items_y0 = 88i32;
                    let item_h = 26i32;
                    let max_items = ((340 - items_y0) / item_h) as usize;
                    let count = app.todos.len().min(max_items);
                    for i in 0..count {
                        let item_y = items_y0 + (i as i32) * item_h;
                        if x >= 24 && x <= 40 && y >= item_y + 4 && y <= item_y + 20 {
                            app.todos[i].1 = !app.todos[i].1;
                            super::config::save_user_data(
                                &app.data_path,
                                &super::config::UserData { note: app.note.clone(), todos: app.todos.clone() },
                            );
                            InvalidateRect(hwnd, std::ptr::null(), 1);
                            break;
                        }
                    }
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub(crate) fn show_customize_panel(app: &mut super::App) {
    unsafe {
        if !app.customize_hwnd.is_null() {
            SetForegroundWindow(app.customize_hwnd);
            return;
        }
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let mx = ((sw - COS_W) / 2).max(0);
        let my = ((sh - COS_H) / 2).max(0);
        let class_name = wstr("CatCustomizeWnd");
        let hinst = GetModuleHandleW(std::ptr::null());
        let brush = CreateSolidBrush(CLR_BG);
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(customize_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: brush,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name,
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            class_name,
            wstr("Customize"),
            WS_POPUP | WS_VISIBLE,
            mx,
            my,
            COS_W,
            COS_H,
            app.menu_hwnd,
            std::ptr::null_mut(),
            hinst,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return;
        }
        let rgn = CreateRoundRectRgn(0, 0, COS_W + 1, COS_H + 1, 16, 16);
        SetWindowRgn(hwnd, rgn, 1);
        app.customize_hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as *mut super::App as isize);
        create_customize_controls(hwnd, app);
        ShowWindow(hwnd, 1);
        SetForegroundWindow(hwnd);
    }
}

unsafe fn create_customize_controls(hwnd: HWND, app: &super::App) {
    let font = GetStockObject(DEFAULT_GUI_FONT) as _;
    let radio = 0x0009u32; // BS_AUTORADIOBUTTON

    make_ctl(hwnd, 0, "STATIC", "Cat Color", 20, 60, 360, 20, SS_LEFT, font);
    let color_black = make_ctl(hwnd, BTN_COLOR_BLACK, "BUTTON", "Black", 30, 84, 110, 28, radio, font);
    let color_white = make_ctl(hwnd, BTN_COLOR_WHITE, "BUTTON", "White", 148, 84, 110, 28, radio, font);
    let color_orange = make_ctl(hwnd, BTN_COLOR_ORANGE, "BUTTON", "Orange", 266, 84, 110, 28, radio, font);
    let color_sel = if app.color == 0 {
        color_black
    } else if app.color == 1 {
        color_white
    } else {
        color_orange
    };
    SendMessageW(color_sel, BM_SETCHECK, BST_CHECKED as _, 0);

    make_ctl(hwnd, 0, "STATIC", "Scarf (one at a time)", 20, 124, 360, 20, SS_LEFT, font);
    let scarf_sel = app.cosmetic_scarf;
    for i in 0..6usize {
        let id = BTN_SCARF_NONE + i as u32;
        let name = if i == 0 { "None" } else { super::cosmetics::SCARF_ITEMS[i - 1].name };
        let val = if i == 0 { None } else { Some(i - 1) };
        let btn = make_ctl(hwnd, id, "BUTTON", name, 30, 148 + (i as i32) * 32, 340, 28, radio, font);
        if val == scarf_sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }

    make_ctl(hwnd, 0, "STATIC", "Bell OR Tie (on top of scarf)", 20, 346, 360, 20, SS_LEFT, font);
    let bell_rel: [Option<usize>; 3] = [None, Some(0), Some(1)];
    for i in 0..3usize {
        let bell_id = BTN_BELL_NONE + i as u32;
        let bell_name = if i == 0 { "No Bell" } else { super::cosmetics::BELL_ITEMS[i - 1].name };
        let sel = bell_rel[i] == app.cosmetic_bell;
        let btn = make_ctl(hwnd, bell_id, "BUTTON", bell_name, 30, 368 + (i as i32) * 32, 160, 28, radio, font);
        if sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }
    let tie_rel: [Option<usize>; 4] = [None, Some(0), Some(1), Some(2)];
    for i in 0..4usize {
        let tie_id = BTN_TIE_NONE + i as u32;
        let tie_name = if i == 0 { "No Tie" } else { super::cosmetics::TIE_ITEMS[i - 1].name };
        let sel = tie_rel[i] == app.cosmetic_tie;
        let btn = make_ctl(hwnd, tie_id, "BUTTON", tie_name, 205, 368 + (i as i32) * 32, 180, 28, radio, font);
        if sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }

    make_ctl(hwnd, 0, "STATIC", "Cat Size (100-500 px)", 20, 470, 360, 20, SS_LEFT, font);
    let size_edit = make_ctl(hwnd, ED_SIZE, "EDIT", &app.w.to_string(), 30, 494, 200, 28, 0x2000 | 0x0080, font);
    let _ = size_edit;
    make_ctl(hwnd, BTN_SIZE_APPLY, "BUTTON", "Apply", 240, 494, 100, 28, 0, font);

    let top = make_ctl(hwnd, BTN_TOPMOST, "BUTTON", "Always on top of all windows", 30, 532, 340, 26, 0x0003, font);
    if app.always_on_top {
        SendMessageW(top, BM_SETCHECK, BST_CHECKED as _, 0);
    }

    let desk = make_ctl(hwnd, BTN_DESKTOP, "BUTTON", "Create shortcut on desktop", 30, 560, 340, 26, 0x0003, font);
    if app.desktop_shortcut_exists() {
        SendMessageW(desk, BM_SETCHECK, BST_CHECKED as _, 0);
    }

    let start = make_ctl(hwnd, BTN_STARTUP, "BUTTON", "Start with Windows", 30, 588, 340, 26, 0x0003, font);
    if app.startup_enabled() {
        SendMessageW(start, BM_SETCHECK, BST_CHECKED as _, 0);
    }

    make_ctl(hwnd, BTN_COS_BACK, "BUTTON", "Back", 20, 624, 170, 28, 0, font);
    make_ctl(hwnd, BTN_COS_CLEAR_ALL, "BUTTON", "Remove All", 210, 624, 170, 28, 0, font);
}

unsafe fn sync_cos_checks(hwnd: HWND, app: &super::App) {
    let check = |id: u32, on: bool| {
        let c = GetDlgItem(hwnd, id as i32);
        if !c.is_null() {
            SendMessageW(c, BM_SETCHECK, if on { BST_CHECKED as usize } else { 0 }, 0);
        }
    };
        check(BTN_COLOR_BLACK, app.color == 0);
        check(BTN_COLOR_WHITE, app.color == 1);
        check(BTN_COLOR_ORANGE, app.color == 2);
        check(BTN_TOPMOST, app.always_on_top);
        check(BTN_DESKTOP, app.desktop_shortcut_exists());
        check(BTN_STARTUP, app.startup_enabled());
        for i in 0..6u32 {
        let val = if i == 0 { None } else { Some((i - 1) as usize) };
        check(BTN_SCARF_NONE + i, val == app.cosmetic_scarf);
    }
    for i in 0..3u32 {
        let val = if i == 0 { None } else { Some((i - 1) as usize) };
        check(BTN_BELL_NONE + i, val == app.cosmetic_bell);
    }
    for i in 0..4u32 {
        let val = if i == 0 { None } else { Some((i - 1) as usize) };
        check(BTN_TIE_NONE + i, val == app.cosmetic_tie);
    }
}

fn apply_cosmetics(app: &mut super::App) {
    app.cat.rebuild_cosmetics(
        app.cosmetic_bell,
        app.cosmetic_scarf,
        app.cosmetic_tie,
        app.scale,
    );
    app.cat.set_look_down();
    super::config::save_config(
        &app.config_path,
        &super::config::ConfigData {
            color: app.color,
            size_idx: app.size_idx,
            size_px: app.w,
            pos_x: app.pos_x,
            pos_y: app.pos_y,
            always_on_top: app.always_on_top,
            cosmetic_bell: app.cosmetic_bell,
            cosmetic_scarf: app.cosmetic_scarf,
            cosmetic_tie: app.cosmetic_tie,
        },
    );
}

pub(crate) fn paint_customize(hwnd: HWND, app: &super::App) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, COS_W, COS_H);
        SelectObject(mem, bmp);

        let bg = CreateSolidBrush(CLR_BG);
        FillRect(mem, &RECT { left: 0, top: 0, right: COS_W, bottom: COS_H }, bg);
        DeleteObject(bg);

        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(mem, &RECT { left: 0, top: 0, right: COS_W, bottom: 50 }, hdr);
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 50, std::ptr::null_mut());
        LineTo(mem, COS_W, 50);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

        draw_text(
            mem,
            "Customize",
            20,
            10,
            COS_W - 40,
            36,
            CLR_TXT,
            16,
            700,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        // Detached close bubble poking out of the top-right corner
        let close_brush = CreateSolidBrush(CLR_CLOSE_BG);
        let close_pen = CreatePen(PS_SOLID, 1, CLR_CLOSE_BDR);
        let old_cb = SelectObject(mem, close_brush);
        let old_cp = SelectObject(mem, close_pen);
        Ellipse(mem, COS_W - 36, 7, COS_W + 2, 45);
        SelectObject(mem, old_cb);
        SelectObject(mem, old_cp);
        DeleteObject(close_brush);
        DeleteObject(close_pen);
        draw_text(
            mem,
            "\u{00D7}",
            COS_W - 38,
            11,
            36,
            32,
            CLR_CLOSE_TXT,
            15,
            600,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        let _ = app;
        BitBlt(hdc, 0, 0, COS_W, COS_H, mem, 0, 0, SRCCOPY);
        DeleteDC(mem);
        DeleteObject(bmp);
        EndPaint(hwnd, &mut ps);
    }
}

pub(crate) unsafe extern "system" fn customize_wnd_proc(
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
        let app = &mut *(app_ptr as *mut super::App);
        match msg {
            WM_COMMAND => {
                let id = wparam as u32 & 0xFFFF;
                match id {
                    BTN_COS_BACK => {
                        DestroyWindow(hwnd);
                    }
                    BTN_TOPMOST => {
                        app.set_topmost(!app.always_on_top);
                        save_cfg(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_SIZE_APPLY => {
                        let c = GetDlgItem(hwnd, ED_SIZE as i32);
                        if !c.is_null() {
                            let t = get_ctl_text(c);
                            let v: i32 = t.trim().parse().unwrap_or(-1);
                            if v > 0 {
                                let (min_w, max_w) = super::size_limits();
                                let v = v.clamp(min_w, max_w);
                                app.set_width(v);
                                save_cfg(app);
                                SetWindowTextW(c, wstr(&v.to_string()));
                            }
                        }
                    }
                    BTN_DESKTOP => {
                        if app.desktop_shortcut_exists() {
                            app.remove_desktop_shortcut();
                        } else {
                            app.create_desktop_shortcut();
                        }
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_STARTUP => {
                        app.set_startup(!app.startup_enabled());
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_COLOR_BLACK | BTN_COLOR_WHITE | BTN_COLOR_ORANGE => {
                        app.set_color((id - BTN_COLOR_BLACK) as usize);
                        save_cfg(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_COS_CLEAR_ALL => {
                        app.cosmetic_bell = None;
                        app.cosmetic_scarf = None;
                        app.cosmetic_tie = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_BELL_NONE => {
                        app.cosmetic_bell = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_BELL_0 | BTN_BELL_1 => {
                        app.cosmetic_bell = Some((id - BTN_BELL_0) as usize);
                        app.cosmetic_tie = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_SCARF_NONE => {
                        app.cosmetic_scarf = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_SCARF_0 | BTN_SCARF_1 | BTN_SCARF_2 | BTN_SCARF_3 | BTN_SCARF_4 => {
                        app.cosmetic_scarf = Some((id - BTN_SCARF_0) as usize);
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_TIE_NONE => {
                        app.cosmetic_tie = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_TIE_0 | BTN_TIE_1 | BTN_TIE_2 => {
                        app.cosmetic_tie = Some((id - BTN_TIE_0) as usize);
                        app.cosmetic_bell = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    _ => {}
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
                app.customize_hwnd = std::ptr::null_mut();
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_customize(hwnd, app);
                0
            }
            WM_LBUTTONUP => {
                let x = (lparam as u32 & 0xFFFF) as i32;
                let y = ((lparam as u32 >> 16) & 0xFFFF) as i32;
                if in_bubble(x, y, COS_W - 17, 26, 21) {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_HSCROLL => 0,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}