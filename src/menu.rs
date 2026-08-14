use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::SS_LEFT;
use windows_sys::Win32::UI::Controls::{BST_CHECKED, SetScrollInfo};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetFocus, SetFocus, VK_ESCAPE};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(crate) const TAB_TODO: u32 = 401;
pub(crate) const TAB_NOTES: u32 = 402;

pub(crate) const ED_TODO_INPUT: u32 = 410;
pub(crate) const BTN_TODO_ADD: u32 = 411;
pub(crate) const BTN_TODO_CLEAR: u32 = 412;

pub(crate) const ED_NOTES: u32 = 420;
pub(crate) const BTN_NOTES_SAVE: u32 = 421;

pub(crate) const HINT_TODO: u32 = 700;
pub(crate) const HINT_NOTES: u32 = 701;

pub(crate) const BTN_CUSTOMIZE: u32 = 430;
pub(crate) const BTN_EXIT: u32 = 431;
pub(crate) const BTN_SCAN: u32 = 432;
pub(crate) const BTN_SETTINGS: u32 = 433;

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

#[allow(dead_code)]
pub(crate) const BTN_HK_BLACK: u32 = 500;
#[allow(dead_code)]
pub(crate) const BTN_HK_WHITE: u32 = 501;
#[allow(dead_code)]
pub(crate) const BTN_HK_ORANGE: u32 = 502;
#[allow(dead_code)]
pub(crate) const BTN_HK_SMALL: u32 = 503;
#[allow(dead_code)]
pub(crate) const BTN_HK_MEDIUM: u32 = 504;
#[allow(dead_code)]
pub(crate) const BTN_HK_LARGE: u32 = 505;
#[allow(dead_code)]
pub(crate) const BTN_HK_EXIT: u32 = 506;

pub(crate) const BTN_LBL_WINDOW: u32 = 510;
pub(crate) const BTN_LBL_HOTKEYS: u32 = 511;

pub(crate) const MENU_W: i32 = 470;
pub(crate) const MENU_H: i32 = 430;
pub(crate) const TIMER_MENU_CLOSE: u32 = 3;

const COS_W: i32 = 400;
const COS_H: i32 = 652;

const SET_W: i32 = 400;
const SET_H: i32 = 500;
// Scrollable card: 12,58 .. SET_W-12,330. Content bottom inside the card.
const SET_VIEW_X: i32 = 12;
const SET_VIEW_Y: i32 = 58;
const SET_VIEW_H: i32 = 400;
const SET_CONTENT_BOTTOM: i32 = 364;
const SET_VIEW_ID: usize = 520;
const SET_SB_ID: usize = 521;

const CLR_BG: u32 = 0x00F8F9FA;
const CLR_WHITE: u32 = 0x00FFFFFF;
const CLR_SEP: u32 = 0x00E2E8F0;
const CLR_PILL_ON: u32 = 0x003B82F6;
const CLR_PILL_OFF: u32 = 0x00F1F5F9;
const CLR_PILL_BDR: u32 = 0x00CBD5E1;
const CLR_CLOSE_BG: u32 = 0x00FFFFFF;
const CLR_CLOSE_BDR: u32 = 0x00CBD5E1;
const CLR_CLOSE_TXT: u32 = 0x0064748B;
const CLR_CARD_BDR: u32 = 0x00E2E8F0;
const CLR_TXT: u32 = 0x001E293B;
const CLR_TXT_DIM: u32 = 0x00475569;
const CLR_TXT_DONE: u32 = 0x0094A3B8;
const CLR_CHK: u32 = 0x0094A3B8;

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
    x >= cx - 48 && x <= cx + 48 && y >= 14 && y <= 50
}

fn in_close_btn(x: i32, y: i32, w: i32) -> bool {
    let bx = w - 38;
    let by = 13;
    let bw = 24;
    let bh = 24;
    x >= bx - 5 && x <= bx + bw + 5 && y >= by - 5 && y <= by + bh + 5
}

unsafe fn create_asymmetric_rgn(w: i32, h: i32, r: i32) -> HRGN {
    // Sharp top-left, top-right, and bottom-right corners.
    // Smooth rounded bottom-left corner with radius r.
    let rgn_top = CreateRectRgn(0, 0, w, h - r);
    let rgn_br = CreateRectRgn(r, h - r, w, h);
    let rgn_bl = CreateEllipticRgn(0, h - 2 * r, 2 * r, h);
    CombineRgn(rgn_top, rgn_top, rgn_br, 2 /* RGN_OR */);
    CombineRgn(rgn_top, rgn_top, rgn_bl, 2 /* RGN_OR */);
    DeleteObject(rgn_br);
    DeleteObject(rgn_bl);
    rgn_top
}

unsafe fn draw_close_button(hdc: *mut core::ffi::c_void, w: i32) {
    let bx = w - 38;
    let by = 13;
    let bw = 24;
    let bh = 24;

    // 1. Cutout recess background / slot
    let slot_pen = CreatePen(PS_SOLID, 1, 0x00E2E8F0);
    let slot_brush = CreateSolidBrush(0x00F1F5F9);
    let old_p = SelectObject(hdc, slot_pen);
    let old_b = SelectObject(hdc, slot_brush);
    Rectangle(hdc, bx - 3, by - 3, bx + bw + 3, by + bh + 3);
    SelectObject(hdc, old_b);
    SelectObject(hdc, old_p);
    DeleteObject(slot_brush);
    DeleteObject(slot_pen);

    // 2. Floating rectangular button tile
    let tile_brush = CreateSolidBrush(CLR_CLOSE_BG);
    let tile_pen = CreatePen(PS_SOLID, 1, CLR_CLOSE_BDR);
    let old_tb = SelectObject(hdc, tile_brush);
    let old_tp = SelectObject(hdc, tile_pen);
    Rectangle(hdc, bx, by, bx + bw, by + bh);
    SelectObject(hdc, old_tb);
    SelectObject(hdc, old_tp);
    DeleteObject(tile_brush);
    DeleteObject(tile_pen);

    // 3. Crisp cross symbol centered in the rectangle
    draw_text(
        hdc,
        "\u{00D7}",
        bx,
        by - 1,
        bw,
        bh,
        CLR_CLOSE_TXT,
        14,
        600,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
}

unsafe fn draw_window_border(hdc: *mut core::ffi::c_void, w: i32, h: i32, r: i32) {
    let border_pen = CreatePen(PS_SOLID, 1, 0x00D1D5DB);
    let old_bp = SelectObject(hdc, border_pen);
    // Top edge
    MoveToEx(hdc, 0, 0, std::ptr::null_mut());
    LineTo(hdc, w, 0);
    // Right edge
    LineTo(hdc, w - 1, h);
    // Bottom edge (from right to start of corner curve)
    LineTo(hdc, r, h - 1);
    // Left edge (from top to start of corner curve)
    MoveToEx(hdc, 0, 0, std::ptr::null_mut());
    LineTo(hdc, 0, h - r);
    // Bottom-left arc
    Arc(hdc, 0, h - 2 * r, 2 * r, h, 0, h - r, r, h);
    SelectObject(hdc, old_bp);
    DeleteObject(border_pen);
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
        let rgn = create_asymmetric_rgn(MENU_W + 1, MENU_H + 1, 18);
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

    make_ctl(hwnd, ED_TODO_INPUT, "EDIT", "", 16, 342, 304, 28, 0x0080, font);
    make_ctl(hwnd, BTN_TODO_ADD, "BUTTON", "Add", 328, 342, 58, 28, 0, font);
    make_ctl(hwnd, BTN_TODO_CLEAR, "BUTTON", "Clear All", 392, 342, 62, 28, 0, font);
    make_ctl(hwnd, HINT_TODO, "STATIC", "Type your to do here...", 22, 347, 280, 18, SS_LEFT, font);

    make_ctl(
        hwnd,
        ED_NOTES,
        "EDIT",
        "",
        22,
        82,
        426,
        244,
        0x0004 | 0x0040 | 0x1000 | 0x00200000 | 0x0100,
        font,
    );
    make_ctl(hwnd, HINT_NOTES, "STATIC", "Write your note here...", 28, 88, 414, 18, SS_LEFT, font);
    make_ctl(hwnd, BTN_NOTES_SAVE, "BUTTON", "Save Note", 16, 342, 100, 28, 0, font);

    make_ctl(hwnd, BTN_CUSTOMIZE, "BUTTON", "CUSTOMIZE", 16, 384, 138, 30, 0, font);
    make_ctl(hwnd, BTN_SETTINGS, "BUTTON", "SETTINGS", 166, 384, 138, 30, 0, font);
    make_ctl(hwnd, BTN_EXIT, "BUTTON", "EXIT", 316, 384, 138, 30, 0, font);
}

pub(crate) fn show_tab_content(app: &super::App, tab: u32) {
    if app.menu_hwnd.is_null() {
        return;
    }
    unsafe {
        let todo_ids = [ED_TODO_INPUT, HINT_TODO, BTN_TODO_ADD, BTN_TODO_CLEAR];
        let notes_ids = [ED_NOTES, HINT_NOTES, BTN_NOTES_SAVE];
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

        // Header bar
        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(
            mem,
            &RECT { left: 0, top: 0, right: MENU_W, bottom: 64 },
            hdr,
        );
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 64, std::ptr::null_mut());
        LineTo(mem, MENU_W, 64);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

        // Tab pills: To Do / Notes
        let labels = ["To Do", "Notes"];
        let centers = [84, 194];
        for i in 0..2usize {
            let cx = centers[i];
            let active = (i as u32) == app.menu_tab;
            let pill = CreateRoundRectRgn(cx - 48, 14, cx + 48, 50, 8, 8);
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
                36,
                if active { CLR_WHITE } else { CLR_TXT_DIM },
                13,
                if active { 600 } else { 500 },
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        }

        // Detached rectangular cut-out close button in the top-right
        draw_close_button(mem, MENU_W);

        // Main content card
        let card = CreateRoundRectRgn(16, 76, MENU_W - 16, 332, 8, 8);
        let wb = CreateSolidBrush(CLR_WHITE);
        FillRgn(mem, card, wb);
        DeleteObject(wb);
        let fb = CreateSolidBrush(CLR_CARD_BDR);
        FrameRgn(mem, card, fb, 1, 1);
        DeleteObject(fb);
        DeleteObject(card);

        if app.menu_tab == 0 {
            let items_y0 = 86;
            let item_h = 28;
            let max_items = ((324 - items_y0) / item_h) as usize;
            let count = app.todos.len().min(max_items);
            if count == 0 {
                draw_text(
                    mem,
                    "No to-dos yet! Add a task below.",
                    30,
                    180,
                    MENU_W - 60,
                    30,
                    CLR_TXT_DONE,
                    13,
                    400,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            } else {
                for i in 0..count {
                    let (ref text, done) = app.todos[i];
                    let y = items_y0 + (i as i32) * item_h;
                    let chk_x = 28;
                    let chk_y = y + 5;
                    let chk_sz = 17;
                    let border_pen = CreatePen(PS_SOLID, 1, if done { CLR_PILL_ON } else { CLR_CHK });
                    let old_p = SelectObject(mem, border_pen);
                    let old_b = SelectObject(mem, GetStockObject(5));
                    Rectangle(mem, chk_x, chk_y, chk_x + chk_sz, chk_y + chk_sz);
                    SelectObject(mem, old_p);
                    SelectObject(mem, old_b);
                    DeleteObject(border_pen);
                    if done {
                        let fill_b = CreateSolidBrush(CLR_PILL_ON);
                        FillRect(
                            mem,
                            &RECT {
                                left: chk_x + 1,
                                top: chk_y + 1,
                                right: chk_x + chk_sz - 1,
                                bottom: chk_y + chk_sz - 1,
                            },
                            fill_b,
                        );
                        DeleteObject(fill_b);
                        let check_pen = CreatePen(PS_SOLID, 2, CLR_WHITE);
                        let old_cp = SelectObject(mem, check_pen);
                        MoveToEx(mem, chk_x + 4, chk_y + 8, std::ptr::null_mut());
                        LineTo(mem, chk_x + 7, chk_y + 12);
                        LineTo(mem, chk_x + 13, chk_y + 4);
                        SelectObject(mem, old_cp);
                        DeleteObject(check_pen);
                    }
                    draw_text(
                        mem,
                        text,
                        52,
                        y,
                        MENU_W - 80,
                        item_h,
                        if done { CLR_TXT_DONE } else { CLR_TXT },
                        13,
                        400,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );
                    if done {
                        let strike_pen = CreatePen(PS_SOLID, 1, CLR_TXT_DONE);
                        let old_sp = SelectObject(mem, strike_pen);
                        let tlen = (text.len() as i32 * 7).min(MENU_W - 80);
                        MoveToEx(mem, 52, y + item_h / 2, std::ptr::null_mut());
                        LineTo(mem, 52 + tlen, y + item_h / 2);
                        SelectObject(mem, old_sp);
                        DeleteObject(strike_pen);
                    }
                }
            }
        }

        // Draw crisp perimeter border matching the asymmetric region shape
        draw_window_border(mem, MENU_W, MENU_H, 18);

        // Update status line (check-for-update feedback)
        if !app.status.is_empty() {
            draw_text(
                mem,
                &app.status,
                16,
                416,
                MENU_W - 32,
                12,
                CLR_TXT_DONE,
                10,
                400,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
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
        BTN_SETTINGS => {
            show_settings_panel(app);
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
            hotkeys: app.hotkeys,
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

unsafe fn update_hint(hwnd: HWND, edit_id: u32) {
    let hint_id = if edit_id == ED_TODO_INPUT { HINT_TODO } else { HINT_NOTES };
    let c = GetDlgItem(hwnd, hint_id as i32);
    let e = GetDlgItem(hwnd, edit_id as i32);
    if c.is_null() || e.is_null() {
        return;
    }
    let has_text = get_ctl_text(e).trim().len() > 0;
    let focused = GetFocus() == e;
    ShowWindow(c, if has_text || focused { SW_HIDE as i32 } else { SW_SHOW as i32 });
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
                let code = (wparam >> 16) & 0xFFFF;
                let id = wparam as u32 & 0xFFFF;
                // Placeholder hints: track focus + text changes on both edit boxes
                if (id == ED_TODO_INPUT || id == ED_NOTES)
                    && (code == 0x0100 /* EN_SETFOCUS */
                        || code == 0x0200 /* EN_KILLFOCUS */
                        || code == 0x0300 /* EN_CHANGE */)
                {
                    update_hint(hwnd, id);
                }
                handle_command(app, id);
                0
            }
            WM_CTLCOLORSTATIC => {
                // Gray hint text on the white card background
                let hdc = wparam as isize as *mut std::ffi::c_void;
                SetBkMode(hdc, 1);
                SetTextColor(hdc, CLR_TXT_DONE);
                GetStockObject(5) as isize
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
                if in_pill(x, y, 84) {
                    handle_command(app, TAB_TODO);
                } else if in_pill(x, y, 194) {
                    handle_command(app, TAB_NOTES);
                } else if in_close_btn(x, y, MENU_W) {
                    KillTimer(hwnd, TIMER_MENU_CLOSE as usize);
                    app.menu_timer_id = 0;
                    DestroyWindow(hwnd);
                } else if app.menu_tab == 0 {
                    let items_y0 = 86i32;
                    let item_h = 28i32;
                    let max_items = ((324 - items_y0) / item_h) as usize;
                    let count = app.todos.len().min(max_items);
                    for i in 0..count {
                        let item_y = items_y0 + (i as i32) * item_h;
                        if x >= 24 && x <= 48 && y >= item_y + 2 && y <= item_y + 24 {
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
        // Sit beside the cat (mirroring the main menu) so cosmetic changes are visible live
        let mut mx = app.pos_x + app.w + 8;
        if mx + COS_W > sw - 8 {
            mx = app.pos_x - COS_W - 8;
        }
        if mx < 8 {
            mx = (sw - COS_W) / 2;
        }
        let mut my = app.pos_y + (app.h - COS_H) / 2;
        if my < 8 {
            my = 8;
        }
        if my + COS_H > sh - 8 {
            my = (sh - COS_H - 8).max(8);
        }
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
        let rgn = create_asymmetric_rgn(COS_W + 1, COS_H + 1, 16);
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

    make_ctl(hwnd, 0, "STATIC", "Cat Color", 20, 66, 360, 20, SS_LEFT, font);
    let color_black = make_ctl(hwnd, BTN_COLOR_BLACK, "BUTTON", "Black", 30, 86, 110, 28, radio, font);
    let color_white = make_ctl(hwnd, BTN_COLOR_WHITE, "BUTTON", "White", 148, 86, 110, 28, radio, font);
    let color_orange = make_ctl(hwnd, BTN_COLOR_ORANGE, "BUTTON", "Orange", 266, 86, 110, 28, radio, font);
    let color_sel = if app.color == 0 {
        color_black
    } else if app.color == 1 {
        color_white
    } else {
        color_orange
    };
    SendMessageW(color_sel, BM_SETCHECK, BST_CHECKED as _, 0);

    make_ctl(hwnd, 0, "STATIC", "Scarf (one at a time)", 20, 166, 360, 20, SS_LEFT, font);
    let scarf_sel = app.cosmetic_scarf;
    for i in 0..6usize {
        let id = BTN_SCARF_NONE + i as u32;
        let name = if i == 0 { "None" } else { super::cosmetics::SCARF_ITEMS[i - 1].name };
        let val = if i == 0 { None } else { Some(i - 1) };
        let btn = make_ctl(hwnd, id, "BUTTON", name, 30, 186 + (i as i32) * 28, 340, 28, radio, font);
        if val == scarf_sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }

    make_ctl(hwnd, 0, "STATIC", "Bell OR Tie (on top of scarf)", 20, 374, 360, 20, SS_LEFT, font);
    let bell_rel: [Option<usize>; 3] = [None, Some(0), Some(1)];
    for i in 0..3usize {
        let bell_id = BTN_BELL_NONE + i as u32;
        let bell_name = if i == 0 { "No Bell" } else { super::cosmetics::BELL_ITEMS[i - 1].name };
        let sel = bell_rel[i] == app.cosmetic_bell;
        let btn = make_ctl(hwnd, bell_id, "BUTTON", bell_name, 30, 394 + (i as i32) * 28, 150, 28, radio, font);
        if sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }
    let tie_rel: [Option<usize>; 4] = [None, Some(0), Some(1), Some(2)];
    for i in 0..4usize {
        let tie_id = BTN_TIE_NONE + i as u32;
        let tie_name = if i == 0 { "No Tie" } else { super::cosmetics::TIE_ITEMS[i - 1].name };
        let sel = tie_rel[i] == app.cosmetic_tie;
        let btn = make_ctl(hwnd, tie_id, "BUTTON", tie_name, 205, 394 + (i as i32) * 28, 175, 28, radio, font);
        if sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }

    make_ctl(hwnd, 0, "STATIC", "Cat Size (100-500 px)", 20, 536, 360, 20, SS_LEFT, font);
    let size_edit = make_ctl(hwnd, ED_SIZE, "EDIT", &app.w.to_string(), 30, 558, 200, 28, 0x2000 | 0x0080, font);
    let _ = size_edit;
    make_ctl(hwnd, BTN_SIZE_APPLY, "BUTTON", "Apply", 240, 558, 100, 28, 0, font);

    make_ctl(hwnd, BTN_COS_BACK, "BUTTON", "Back", 30, 612, 170, 28, 0, font);
    make_ctl(hwnd, BTN_COS_CLEAR_ALL, "BUTTON", "Remove All", 210, 612, 170, 28, 0, font);
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
        for i in 0..7u32 {
            check(BTN_HK_BLACK + i, app.hotkey_enabled(i as usize));
        }
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
            hotkeys: app.hotkeys,
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

        // Detached rectangular cut-out close button in the top-right
        draw_close_button(mem, COS_W);

        // Section cards: Cat Color / Scarf / Bell or Tie / Cat Size
        let cards = [
            (12i32, 58i32, 388i32, 146i32),
            (12, 158, 388, 354),
            (12, 366, 388, 516),
            (12, 528, 388, 600),
        ];
        for &(x0, y0, x1, y1) in &cards {
            let card = CreateRoundRectRgn(x0, y0, x1, y1, 10, 10);
            let wb = CreateSolidBrush(CLR_WHITE);
            FillRgn(mem, card, wb);
            DeleteObject(wb);
            let fb = CreateSolidBrush(CLR_CARD_BDR);
            FrameRgn(mem, card, fb, 1, 1);
            DeleteObject(fb);
            DeleteObject(card);
        }

        // Draw crisp perimeter border matching the asymmetric region shape
        draw_window_border(mem, COS_W, COS_H, 16);

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
                if in_close_btn(x, y, COS_W) {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_HSCROLL => 0,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub(crate) fn show_settings_panel(app: &mut super::App) {
    unsafe {
        if !app.settings_hwnd.is_null() {
            SetForegroundWindow(app.settings_hwnd);
            SetFocus(app.settings_hwnd);
            return;
        }
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        let mx = ((sw - SET_W) / 2).max(0);
        let my = ((sh - SET_H) / 2).max(0);
        let class_name = wstr("CatSettingsWnd");
        let hinst = GetModuleHandleW(std::ptr::null());
        let brush = CreateSolidBrush(CLR_BG);
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(settings_wnd_proc),
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
            wstr("Settings"),
            WS_POPUP | WS_VISIBLE,
            mx,
            my,
            SET_W,
            SET_H,
            app.menu_hwnd,
            std::ptr::null_mut(),
            hinst,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return;
        }
        let rgn = create_asymmetric_rgn(SET_W + 1, SET_H + 1, 16);
        SetWindowRgn(hwnd, rgn, 1);
        app.settings_hwnd = hwnd;
        app.settings_scroll = 0;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as *mut super::App as isize);
        let view_class = wstr("CatSettingsViewWnd");
        let vwc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(settings_view_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: view_class,
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&vwc);
        let check_class = wstr("CatCheckWnd");
        let cwc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(check_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 8,
            hInstance: hinst,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: check_class,
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&cwc);
        let view = CreateWindowExW(
            0,
            view_class,
            wstr(""),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL,
            SET_VIEW_X,
            SET_VIEW_Y,
            SET_W - 30,
            SET_VIEW_H,
            hwnd,
            SET_VIEW_ID as _,
            hinst,
            std::ptr::null(),
        );
        create_settings_controls(hwnd, view, app);
        let sb = GetDlgItem(view, SET_SB_ID as i32);
        let _ = sb;
        SetFocus(hwnd);
        init_settings_scroll(hwnd);
        apply_settings_scroll(hwnd, app);
        ShowWindow(hwnd, 1);
        SetForegroundWindow(hwnd);
    }
}

// y positions are relative to the scroll view (which starts at SET_VIEW_Y).
fn settings_base_y(id: u32) -> i32 {
    match id {
        BTN_LBL_WINDOW => 14,
        BTN_TOPMOST => 38,
        BTN_STARTUP => 68,
        BTN_DESKTOP => 98,
        BTN_LBL_HOTKEYS => 134,
        BTN_HK_BLACK..=BTN_HK_EXIT => 158 + (id - BTN_HK_BLACK) as i32 * 30,
        _ => -1,
    }
}

fn settings_max_scroll() -> i32 {
    (SET_CONTENT_BOTTOM - SET_VIEW_H).max(0)
}

unsafe fn init_settings_scroll(hwnd: HWND) {
    let max = settings_max_scroll();
    let page: u32 = if max > 0 { 60 } else { 0 };
    let mut si: SCROLLINFO = std::mem::zeroed();
    si.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
    si.fMask = SIF_RANGE | SIF_PAGE | SIF_POS;
    si.nMin = 0;
    // the system clamps nPos to nMax - nPage + 1, so extend the range so
    // nPos == settings_max_scroll() puts the thumb at the very bottom
    si.nMax = if max > 0 { max + page as i32 - 1 } else { 0 };
    si.nPage = page;
    si.nPos = 0;
    SetScrollInfo(GetDlgItem(hwnd, SET_VIEW_ID as i32), SB_VERT, &si, 1);
}

unsafe fn apply_settings_scroll(hwnd: HWND, app: &mut super::App) {
    let max = settings_max_scroll();
    let pos = app.settings_scroll.clamp(0, max);
    if pos != app.settings_scroll {
        app.settings_scroll = pos;
    }
    let view = GetDlgItem(hwnd, SET_VIEW_ID as i32);
    if !view.is_null() {
        let mut child = GetWindow(view, GW_CHILD);
        while !child.is_null() {
            let id = GetDlgCtrlID(child) as u32;
            let base = settings_base_y(id);
            if base >= 0 {
                SetWindowPos(
                    child,
                    std::ptr::null_mut(),
                    30,
                    base - pos,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            child = GetWindow(child, GW_HWNDNEXT);
        }
    }
    let mut si: SCROLLINFO = std::mem::zeroed();
    si.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
    si.fMask = SIF_POS;
    si.nPos = pos;
    SetScrollInfo(GetDlgItem(hwnd, SET_VIEW_ID as i32), SB_VERT, &si, 1);
    RedrawWindow(
        hwnd,
        std::ptr::null(),
        std::ptr::null_mut(),
        RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_UPDATENOW,
    );
}

unsafe fn create_settings_controls(hwnd: HWND, view: HWND, app: &super::App) {
    let font = GetStockObject(DEFAULT_GUI_FONT) as _;

    make_ctl(view, BTN_LBL_WINDOW, "STATIC", "Window", 30, 14, 320, 20, SS_LEFT, font);
    make_check(view, BTN_TOPMOST, "Always on top of all windows", 30, 38, 320, 26, app.always_on_top);
    make_check(view, BTN_STARTUP, "Start with Windows", 30, 68, 320, 26, app.startup_enabled());
    make_check(view, BTN_DESKTOP, "Shortcut on desktop", 30, 98, 320, 26, app.desktop_shortcut_exists());

    make_ctl(view, BTN_LBL_HOTKEYS, "STATIC", "Keyboard shortcuts (Ctrl+Alt+...)", 30, 134, 320, 20, SS_LEFT, font);
    let hk_names = [
        "Switch to Black cat (Ctrl+Alt+B)",
        "Switch to White cat (Ctrl+Alt+W)",
        "Switch to Orange cat (Ctrl+Alt+O)",
        "Small size (Ctrl+Alt+1)",
        "Medium size (Ctrl+Alt+2)",
        "Large size (Ctrl+Alt+3)",
        "Exit (Ctrl+Alt+X)",
    ];
    for i in 0..7usize {
        make_check(
            view,
            BTN_HK_BLACK + i as u32,
            hk_names[i],
            30,
            158 + (i as i32) * 30,
            320,
            26,
            app.hotkey_enabled(i),
        );
    }

    make_ctl(hwnd, BTN_SCAN, "BUTTON", "CHECK FOR UPDATE", 210, 470, 170, 28, 0, font);
    make_ctl(hwnd, BTN_COS_BACK, "BUTTON", "Back", 30, 470, 170, 28, 0, font);
}

pub(crate) unsafe extern "system" fn settings_view_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND => {
                SendMessageW(GetParent(hwnd), WM_COMMAND, wparam, lparam);
                0
            }
            WM_VSCROLL => {
                SendMessageW(GetParent(hwnd), WM_VSCROLL, wparam, lparam);
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                BeginPaint(hwnd, &mut ps);
                EndPaint(hwnd, &mut ps);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), 256);
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

pub(crate) unsafe extern "system" fn check_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                let checked = GetWindowLongPtrW(hwnd, 0) != 0;
                let text = window_text(hwnd);
                let bg = CreateSolidBrush(CLR_WHITE);
                FillRect(hdc, &rc, bg);
                DeleteObject(bg);
                let sq = RECT {
                    left: 4,
                    top: (rc.bottom - 16) / 2,
                    right: 20,
                    bottom: (rc.bottom - 16) / 2 + 16,
                };
                if checked {
                    let fill = CreateSolidBrush(CLR_PILL_ON);
                    FillRect(hdc, &sq, fill);
                    DeleteObject(fill);
                    let pen = CreatePen(PS_SOLID, 2, CLR_WHITE);
                    let op = SelectObject(hdc, pen);
                    MoveToEx(hdc, sq.left + 3, sq.top + 8, std::ptr::null_mut());
                    LineTo(hdc, sq.left + 6, sq.top + 11);
                    LineTo(hdc, sq.left + 13, sq.top + 4);
                    SelectObject(hdc, op);
                    DeleteObject(pen);
                } else {
                    let pen = CreatePen(PS_SOLID, 1, CLR_CHK);
                    let op = SelectObject(hdc, pen);
                    let ob = SelectObject(hdc, GetStockObject(NULL_BRUSH) as _);
                    Rectangle(hdc, sq.left, sq.top, sq.right, sq.bottom);
                    SelectObject(hdc, ob);
                    SelectObject(hdc, op);
                    DeleteObject(pen);
                }
                if !text.is_empty() {
                    draw_text(
                        hdc,
                        &text,
                        26,
                        0,
                        rc.right - 26,
                        rc.bottom,
                        CLR_TXT,
                        12,
                        400,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                    );
                }
                EndPaint(hwnd, &mut ps);
                0
            }
            WM_LBUTTONUP => {
                let v = GetWindowLongPtrW(hwnd, 0) == 0;
                SetWindowLongPtrW(hwnd, 0, v as isize);
                InvalidateRect(hwnd, std::ptr::null(), 1);
                SendMessageW(
                    GetParent(hwnd),
                    WM_COMMAND,
                    (GetDlgCtrlID(hwnd) as u32) as usize,
                    hwnd as isize,
                );
                0
            }
            BM_SETCHECK => {
                SetWindowLongPtrW(hwnd, 0, (wparam != 0) as isize);
                InvalidateRect(hwnd, std::ptr::null(), 1);
                0
            }
            BM_GETCHECK => GetWindowLongPtrW(hwnd, 0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe fn make_check(
    parent: HWND,
    id: u32,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    checked: bool,
) -> HWND {
    let hwnd = CreateWindowExW(
        0,
        wstr("CatCheckWnd"),
        wstr(text),
        WS_CHILD | WS_VISIBLE,
        x,
        y,
        w,
        h,
        parent,
        id as _,
        GetModuleHandleW(std::ptr::null()),
        std::ptr::null(),
    );
    if checked {
        SendMessageW(hwnd, BM_SETCHECK, 1, 0);
    }
    hwnd
}

pub(crate) fn paint_settings(hwnd: HWND, app: &super::App) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, SET_W, SET_H);
        SelectObject(mem, bmp);

        let bg = CreateSolidBrush(CLR_BG);
        FillRect(mem, &RECT { left: 0, top: 0, right: SET_W, bottom: SET_H }, bg);
        DeleteObject(bg);

        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(mem, &RECT { left: 0, top: 0, right: SET_W, bottom: 50 }, hdr);
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 50, std::ptr::null_mut());
        LineTo(mem, SET_W, 50);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

        draw_text(
            mem,
            "Settings",
            20,
            10,
            SET_W - 40,
            36,
            CLR_TXT,
            16,
            700,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        // Detached rectangular cut-out close button in the top-right
        draw_close_button(mem, SET_W);

        let card = CreateRoundRectRgn(12, 58, SET_W - 12, SET_H - 32, 10, 10);
        let wb = CreateSolidBrush(CLR_WHITE);
        FillRgn(mem, card, wb);
        DeleteObject(wb);
        let fb = CreateSolidBrush(CLR_CARD_BDR);
        FrameRgn(mem, card, fb, 1, 1);
        DeleteObject(fb);
        DeleteObject(card);

        if !app.status.is_empty() {
            draw_text(
                mem,
                &app.status,
                30,
                370,
                SET_W - 60,
                16,
                CLR_TXT_DIM,
                11,
                400,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        }

        // Draw crisp perimeter border matching the asymmetric region shape
        draw_window_border(mem, SET_W, SET_H, 16);

        BitBlt(hdc, 0, 0, SET_W, SET_H, mem, 0, 0, SRCCOPY);
        DeleteDC(mem);
        DeleteObject(bmp);
        EndPaint(hwnd, &mut ps);
    }
}

unsafe fn bm_checked(hwnd: HWND, id: u32) -> bool {
    let view = GetDlgItem(hwnd, SET_VIEW_ID as i32);
    let mut c = if view.is_null() { std::ptr::null_mut() } else { GetDlgItem(view, id as i32) };
    if c.is_null() {
        c = GetDlgItem(hwnd, id as i32);
    }
    !c.is_null() && SendMessageW(c, BM_GETCHECK, 0, 0) != 0
}

pub(crate) unsafe extern "system" fn settings_wnd_proc(
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
                        app.set_topmost(bm_checked(hwnd, id));
                        save_cfg(app);
                        // set_topmost raises the cat window; bring the panel back on top
                        SetForegroundWindow(hwnd);
                    }
                    BTN_STARTUP => {
                        app.set_startup(bm_checked(hwnd, id));
                    }
                    BTN_DESKTOP => {
                        if bm_checked(hwnd, id) {
                            app.create_desktop_shortcut();
                        } else {
                            app.remove_desktop_shortcut();
                        }
                    }
                    BTN_SCAN => {
                        handle_command(app, BTN_SCAN);
                        InvalidateRect(hwnd, std::ptr::null(), 1);
                    }
                    BTN_HK_BLACK..=BTN_HK_EXIT => {
                        let i = (id - BTN_HK_BLACK) as usize;
                        app.set_hotkey(i, bm_checked(hwnd, id));
                        save_cfg(app);
                        sync_cos_checks(hwnd, app);
                    }
                    _ => {}
                }
                0
            }
            WM_VSCROLL => {
                let code = (wparam as u32 & 0xFFFF) as i32;
                let max = settings_max_scroll();
                let mut pos = app.settings_scroll;
                match code {
                    SB_TOP => pos = 0,
                    SB_BOTTOM => pos = max,
                    SB_LINEUP => pos -= 8,
                    SB_LINEDOWN => pos += 8,
                    SB_PAGEUP => pos -= 100,
                    SB_PAGEDOWN => pos += 100,
                    SB_THUMBPOSITION | SB_THUMBTRACK => {
                        let mut si: SCROLLINFO = std::mem::zeroed();
                        si.cbSize = std::mem::size_of::<SCROLLINFO>() as u32;
                        si.fMask = SIF_TRACKPOS;
                        let sb = if lparam != 0 {
                            lparam as HWND
                        } else {
                            GetDlgItem(hwnd, SET_VIEW_ID as i32)
                        };
                        GetScrollInfo(sb, SB_VERT, &mut si);
                        pos = si.nTrackPos;
                    }
                    _ => {}
                }
                app.settings_scroll = pos.clamp(0, max);
                apply_settings_scroll(hwnd, app);
                0
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam >> 16) & 0xFFFF) as i16;
                let max = settings_max_scroll();
                app.settings_scroll =
                    (app.settings_scroll - delta as i32 / 120 * 8).clamp(0, max);
                apply_settings_scroll(hwnd, app);
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
                app.settings_hwnd = std::ptr::null_mut();
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_settings(hwnd, app);
                0
            }
            WM_LBUTTONUP => {
                let x = (lparam as u32 & 0xFFFF) as i32;
                let y = ((lparam as u32 >> 16) & 0xFFFF) as i32;
                if in_close_btn(x, y, SET_W) {
                    DestroyWindow(hwnd);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}