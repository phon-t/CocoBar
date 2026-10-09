use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::{SS_CENTER, SS_LEFT};
use windows_sys::Win32::UI::Controls::{
    SetScrollInfo, BST_CHECKED, DRAWITEMSTRUCT, EM_SETLIMITTEXT, ODS_DISABLED, ODS_FOCUS,
    ODS_SELECTED,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetFocus, GetKeyState, SetFocus, VK_CONTROL, VK_ESCAPE, VK_MENU, VK_RETURN,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub(crate) const TAB_TODO: u32 = 401;
pub(crate) const TAB_NOTES: u32 = 402;

pub(crate) const ED_TODO_INPUT: u32 = 410;
pub(crate) const BTN_TODO_ADD: u32 = 411;
pub(crate) const BTN_TODO_CLEAR: u32 = 412;
const BTN_TODO_PREV: u32 = 413;
const BTN_TODO_NEXT: u32 = 414;
const LBL_TODO_PAGE: u32 = 415;

pub(crate) const ED_NOTES: u32 = 420;
pub(crate) const BTN_NOTES_SAVE: u32 = 421;
const BTN_NOTE_NEW: u32 = 422;
const BTN_NOTE_BACK: u32 = 423;
const LBL_NOTE_META: u32 = 425;
const BTN_NOTE_UNDO: u32 = 426;
const BTN_NOTES_PREV: u32 = 427;
const BTN_NOTES_NEXT: u32 = 428;
const LBL_NOTES_PAGE: u32 = 429;
const BTN_NOTE_EDIT: u32 = 600;
const BTN_NOTE_DELETE: u32 = 610;
const NOTES_PAGE_SIZE: usize = 3;

pub(crate) const HINT_TODO: u32 = 700;
pub(crate) const HINT_NOTES: u32 = 701;

pub(crate) const BTN_CUSTOMIZE: u32 = 430;
pub(crate) const BTN_EXIT: u32 = 431;
pub(crate) const BTN_SCAN: u32 = 432;
pub(crate) const BTN_SETTINGS: u32 = 433;
const BTN_CLOSE: u32 = 434;

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
const BTN_AUTO_UPDATE: u32 = 492;
const BTN_LBL_UPDATES: u32 = 512;
const LBL_UPDATE_HINT: u32 = 513;
const LBL_UPDATE_STATUS: u32 = 514;

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
const BTN_HK_NOTES: u32 = 507;
const BTN_HK_TODO: u32 = 508;
const HOTKEY_CHOICES: [(u32, &str, &str); 9] = [
    (BTN_HK_NOTES, "Open Notes", "N"),
    (BTN_HK_TODO, "Open To Do", "T"),
    (BTN_HK_BLACK, "Black cat", "B"),
    (BTN_HK_WHITE, "White cat", "W"),
    (BTN_HK_ORANGE, "Orange cat", "O"),
    (BTN_HK_SMALL, "Small · 200 px", "1"),
    (BTN_HK_MEDIUM, "Medium · 320 px", "2"),
    (BTN_HK_LARGE, "Large · 500 px", "3"),
    (BTN_HK_EXIT, "Quit cocoBar", "X"),
];

pub(crate) const BTN_LBL_WINDOW: u32 = 510;
pub(crate) const BTN_LBL_HOTKEYS: u32 = 511;

pub(crate) const MENU_W: i32 = 470;
pub(crate) const MENU_H: i32 = 454;
pub(crate) const TIMER_MENU_CLOSE: u32 = 3;
const TIMER_AUTOSAVE: usize = 4;
const TODO_PAGE_SIZE: usize = 8;
const TODO_ROW_H: i32 = 26;

const COS_W: i32 = MENU_W;
const COS_H: i32 = 612;

const SET_W: i32 = MENU_W;
const SET_H: i32 = 638;
const CLOSE_GAP: i32 = 3;
const CLOSE_SIZE: i32 = 32;
const CLOSE_Y: i32 = 0;
const SET_VIEW_X: i32 = 16;
const SET_VIEW_Y: i32 = 76;
const SET_VIEW_H: i32 = 478;
const SET_CONTENT_BOTTOM: i32 = 598;
const SET_VIEW_ID: usize = 520;

const fn rgb(hex: u32) -> u32 {
    ((hex & 0xff) << 16) | (hex & 0xff00) | ((hex >> 16) & 0xff)
}
const CLR_BG: u32 = rgb(0xF5F7FB);
const CLR_WHITE: u32 = 0x00FFFFFF;
const CLR_SEP: u32 = rgb(0xE2E8F0);
const CLR_PILL_ON: u32 = rgb(0x4967D9);
const CLR_PILL_BDR: u32 = rgb(0xCBD5E1);
const CLR_CARD_BDR: u32 = rgb(0xE2E8F0);
const CLR_TXT: u32 = rgb(0x1E293B);
const CLR_TXT_DIM: u32 = rgb(0x475569);
const CLR_TXT_DONE: u32 = rgb(0x64748B);
const CLR_CHK: u32 = rgb(0x94A3B8);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn ui_font(size: i32, weight: i32) -> HFONT {
    static FONTS: OnceLock<Mutex<HashMap<(i32, i32), usize>>> = OnceLock::new();
    let mut fonts = FONTS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    *fonts.entry((size, weight)).or_insert_with(|| unsafe {
        CreateFontW(
            -size,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            wstr("Segoe UI"),
        ) as usize
    }) as HFONT
}

unsafe fn draw_button(lparam: LPARAM) -> LRESULT {
    let item = &*(lparam as *const DRAWITEMSTRUCT);
    let primary = matches!(
        item.CtlID,
        BTN_TODO_ADD | BTN_NOTES_SAVE | BTN_NOTE_NEW | BTN_SIZE_APPLY
    ) || (item.CtlID == TAB_TODO || item.CtlID == TAB_NOTES) && {
        let ptr = GetWindowLongPtrW(GetParent(item.hwndItem), GWLP_USERDATA);
        ptr != 0 && (*(ptr as *const super::App)).menu_tab == item.CtlID - TAB_TODO
    };
    let disabled = item.itemState & ODS_DISABLED != 0;
    let pressed = item.itemState & ODS_SELECTED != 0;
    let fill = if disabled {
        CLR_BG
    } else if pressed {
        rgb(0xDCE3F7)
    } else if primary {
        CLR_PILL_ON
    } else {
        CLR_WHITE
    };
    let brush = CreateSolidBrush(CLR_BG);
    FillRect(item.hDC, &item.rcItem, brush);
    DeleteObject(brush);
    let pen = CreatePen(
        PS_SOLID,
        1,
        if primary { CLR_PILL_ON } else { CLR_PILL_BDR },
    );
    let brush = CreateSolidBrush(fill);
    let op = SelectObject(item.hDC, pen);
    let ob = SelectObject(item.hDC, brush);
    let rc = item.rcItem;
    if item.CtlID == BTN_CLOSE {
        Rectangle(item.hDC, rc.left, rc.top, rc.right, rc.bottom);
    } else {
        RoundRect(item.hDC, rc.left, rc.top, rc.right, rc.bottom, 10, 10);
    }
    SelectObject(item.hDC, ob);
    SelectObject(item.hDC, op);
    DeleteObject(brush);
    DeleteObject(pen);
    let color = if disabled {
        CLR_CHK
    } else if primary && !pressed {
        CLR_WHITE
    } else {
        CLR_TXT
    };
    draw_text(
        item.hDC,
        &get_ctl_text(item.hwndItem),
        rc.left + 4,
        rc.top,
        rc.right - rc.left - 8,
        rc.bottom - rc.top,
        color,
        if item.CtlID == BTN_CLOSE { 20 } else { 13 },
        if item.CtlID == BTN_CLOSE { 400 } else { 600 },
        DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
    );
    if item.itemState & ODS_FOCUS != 0 {
        let focus = RECT {
            left: rc.left + 4,
            top: rc.top + 4,
            right: rc.right - 4,
            bottom: rc.bottom - 4,
        };
        DrawFocusRect(item.hDC, &focus);
    }
    1
}

unsafe fn control_colors(msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let hdc = wparam as HDC;
    SetBkMode(hdc, TRANSPARENT as i32);
    let id = GetDlgCtrlID(lparam as HWND) as u32;
    if id == LBL_UPDATE_STATUS || id == LBL_NOTES_PAGE {
        SetTextColor(hdc, CLR_TXT_DIM);
        SetBkColor(hdc, CLR_BG);
        SetDCBrushColor(hdc, CLR_BG);
        return GetStockObject(DC_BRUSH) as LRESULT;
    }
    SetTextColor(
        hdc,
        if matches!(id, HINT_TODO | HINT_NOTES | LBL_TODO_PAGE) {
            CLR_TXT_DIM
        } else {
            CLR_TXT
        },
    );
    SetBkColor(hdc, CLR_WHITE);
    let _ = msg;
    GetStockObject(WHITE_BRUSH) as LRESULT
}

fn wstr(s: &'static str) -> *const u16 {
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
    (w - CLOSE_SIZE..w).contains(&x) && (CLOSE_Y..CLOSE_Y + CLOSE_SIZE).contains(&y)
}

unsafe fn create_body_rgn(w: i32, h: i32, r: i32) -> HRGN {
    let body = CreateRectRgn(0, 0, w, h - r);
    let bottom = CreateRectRgn(r, h - r, w, h);
    let corner = CreateEllipticRgn(0, h - 2 * r, 2 * r, h);
    CombineRgn(body, body, bottom, RGN_OR);
    CombineRgn(body, body, corner, RGN_OR);
    DeleteObject(bottom);
    DeleteObject(corner);
    let cutout = CreateRectRgn(w - CLOSE_SIZE - CLOSE_GAP, 0, w, CLOSE_SIZE + CLOSE_GAP);
    CombineRgn(body, body, cutout, RGN_DIFF);
    DeleteObject(cutout);
    body
}

unsafe fn create_asymmetric_rgn(w: i32, h: i32, r: i32) -> HRGN {
    let body = create_body_rgn(w, h, r);
    let close = CreateRectRgn(w - CLOSE_SIZE, CLOSE_Y, w, CLOSE_Y + CLOSE_SIZE);
    CombineRgn(body, body, close, RGN_OR);
    DeleteObject(close);
    body
}

unsafe fn draw_window_border(hdc: *mut core::ffi::c_void, w: i32, h: i32, r: i32) {
    let body = create_body_rgn(w, h, r);
    let border = CreateSolidBrush(CLR_PILL_BDR);
    FrameRgn(hdc, body, border, 1, 1);
    DeleteObject(border);
    DeleteObject(body);
}

unsafe fn make_close_button(hwnd: HWND, body_width: i32) {
    make_ctl(
        hwnd,
        BTN_CLOSE,
        "BUTTON",
        "×",
        body_width - CLOSE_SIZE,
        CLOSE_Y,
        CLOSE_SIZE,
        CLOSE_SIZE,
        0,
        ui_font(18, 400),
    );
}

unsafe fn paint_card(hdc: HDC, x: i32, y: i32, right: i32, bottom: i32) {
    let card = CreateRoundRectRgn(x, y, right, bottom, 10, 10);
    let fill = CreateSolidBrush(CLR_WHITE);
    let border = CreateSolidBrush(CLR_CARD_BDR);
    FillRgn(hdc, card, fill);
    FrameRgn(hdc, card, border, 1, 1);
    DeleteObject(fill);
    DeleteObject(border);
    DeleteObject(card);
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

unsafe fn panel_position(app: &super::App, width: i32, height: i32) -> (i32, i32) {
    let monitor = MonitorFromWindow(app.hwnd, MONITOR_DEFAULTTONEAREST);
    let mut info: MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    let work = if GetMonitorInfoW(monitor, &mut info) != 0 {
        info.rcWork
    } else {
        RECT {
            left: 0,
            top: 0,
            right: GetSystemMetrics(SM_CXSCREEN),
            bottom: GetSystemMetrics(SM_CYSCREEN),
        }
    };
    let mut x = app.pos_x + app.w + 8;
    if x + width > work.right - 8 {
        x = app.pos_x - width - 8;
    }
    let max_x = (work.right - width - 8).max(work.left + 8);
    let max_y = (work.bottom - height - 8).max(work.top + 8);
    (
        x.clamp(work.left + 8, max_x),
        (app.pos_y + (app.h - height) / 2).clamp(work.top + 8, max_y),
    )
}

unsafe fn get_ctl_text(hwnd: HWND) -> String {
    let n = GetWindowTextLengthW(hwnd).max(0) as usize;
    let mut buf = vec![0u16; n + 1];
    let copied = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32).max(0) as usize;
    String::from_utf16_lossy(&buf[..copied])
}

unsafe fn make_ctl(
    parent: HWND,
    id: u32,
    class: &'static str,
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
        wide(title).as_ptr(),
        WS_CHILD
            | WS_VISIBLE
            | if class == "STATIC" { 0 } else { WS_TABSTOP }
            | if class == "BUTTON" && style == 0 {
                BS_OWNERDRAW as u32
            } else {
                style
            },
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
    if class == "EDIT" {
        SendMessageW(c, EM_SETLIMITTEXT, 0x7ffffffe, 0);
    }
    c
}

pub(crate) fn show_menu(app: &mut super::App) {
    unsafe {
        if !app.menu_hwnd.is_null() && IsWindow(app.menu_hwnd) != 0 {
            SetForegroundWindow(app.menu_hwnd);
            return;
        }
        app.menu_hwnd = std::ptr::null_mut();
        let (mx, my) = panel_position(app, MENU_W, MENU_H);
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            wstr("CatMenuWnd"),
            wstr("cocoBar"),
            WS_POPUP | WS_VISIBLE | WS_CLIPCHILDREN,
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
        let rgn = create_asymmetric_rgn(MENU_W, MENU_H, 18);
        SetWindowRgn(hwnd, rgn, 1);
        app.menu_hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as *mut super::App as isize);
        create_menu_controls(hwnd);
        refresh_notes(app);
        show_tab_content(app, app.menu_tab);
        ShowWindow(hwnd, 1);
        SetForegroundWindow(hwnd);
    }
}

pub(crate) fn open_tab(app: &mut super::App, tab: u32) {
    unsafe {
        if app.drag_active {
            app.drag_active = false;
            app.exit_tilt();
            windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
            app.save_settings();
        }
        for panel in [app.customize_hwnd, app.settings_hwnd] {
            if !panel.is_null() {
                DestroyWindow(panel);
            }
        }
        app.menu_tab = tab;
        show_menu(app);
        show_tab_content(app, tab);
        let focus_id = if tab == 0 {
            ED_TODO_INPUT
        } else if app.notes_editor_open {
            ED_NOTES
        } else {
            BTN_NOTE_NEW
        };
        SetFocus(GetDlgItem(app.menu_hwnd, focus_id as i32));
    }
}

unsafe fn create_menu_controls(hwnd: HWND) {
    let font = ui_font(13, 400);
    make_ctl(hwnd, TAB_TODO, "BUTTON", "To do", 36, 14, 96, 36, 0, font);
    make_ctl(hwnd, TAB_NOTES, "BUTTON", "Notes", 146, 14, 96, 36, 0, font);
    make_close_button(hwnd, MENU_W);
    make_ctl(
        hwnd,
        BTN_TODO_PREV,
        "BUTTON",
        "‹",
        300,
        298,
        28,
        26,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_TODO_NEXT,
        "BUTTON",
        "›",
        416,
        298,
        28,
        26,
        0,
        font,
    );
    make_ctl(
        hwnd,
        LBL_TODO_PAGE,
        "STATIC",
        "",
        332,
        302,
        80,
        18,
        SS_CENTER,
        font,
    );

    make_ctl(
        hwnd,
        ED_TODO_INPUT,
        "EDIT",
        "",
        16,
        342,
        304,
        28,
        0x0080,
        font,
    );
    make_ctl(
        hwnd,
        BTN_TODO_ADD,
        "BUTTON",
        "Add",
        328,
        342,
        58,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_TODO_CLEAR,
        "BUTTON",
        "Clear",
        392,
        342,
        62,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        HINT_TODO,
        "STATIC",
        "Type your to do here...",
        22,
        347,
        280,
        18,
        SS_LEFT,
        font,
    );

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
    make_ctl(
        hwnd,
        HINT_NOTES,
        "STATIC",
        "Write your note here...",
        28,
        88,
        414,
        18,
        SS_LEFT,
        font,
    );
    make_ctl(
        hwnd,
        BTN_NOTES_SAVE,
        "BUTTON",
        "Save Note",
        16,
        342,
        100,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_NOTE_NEW,
        "BUTTON",
        "+ New note",
        16,
        342,
        112,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_NOTE_BACK,
        "BUTTON",
        "All notes",
        128,
        342,
        112,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        LBL_NOTE_META,
        "STATIC",
        "",
        250,
        339,
        204,
        36,
        SS_LEFT,
        ui_font(11, 400),
    );
    make_ctl(
        hwnd,
        BTN_NOTE_UNDO,
        "BUTTON",
        "Undo delete",
        140,
        342,
        112,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_NOTES_PREV,
        "BUTTON",
        "‹",
        300,
        342,
        28,
        28,
        0,
        font,
    );
    make_ctl(
        hwnd,
        LBL_NOTES_PAGE,
        "STATIC",
        "",
        332,
        347,
        80,
        18,
        SS_CENTER,
        font,
    );
    make_ctl(
        hwnd,
        BTN_NOTES_NEXT,
        "BUTTON",
        "›",
        416,
        342,
        28,
        28,
        0,
        font,
    );
    for row in 0..NOTES_PAGE_SIZE {
        let y = 92 + row as i32 * 80;
        make_ctl(
            hwnd,
            BTN_NOTE_EDIT + row as u32,
            "BUTTON",
            "Edit",
            318,
            y,
            54,
            24,
            0,
            font,
        );
        make_ctl(
            hwnd,
            BTN_NOTE_DELETE + row as u32,
            "BUTTON",
            "Delete",
            380,
            y,
            60,
            24,
            0,
            font,
        );
    }

    make_ctl(
        hwnd,
        BTN_CUSTOMIZE,
        "BUTTON",
        "Customize",
        16,
        384,
        138,
        30,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_SETTINGS,
        "BUTTON",
        "Settings",
        166,
        384,
        138,
        30,
        0,
        font,
    );
    make_ctl(hwnd, BTN_EXIT, "BUTTON", "Exit", 316, 384, 138, 30, 0, font);
}

pub(crate) fn show_tab_content(app: &super::App, tab: u32) {
    if app.menu_hwnd.is_null() {
        return;
    }
    unsafe {
        let todo_ids = [
            ED_TODO_INPUT,
            HINT_TODO,
            BTN_TODO_ADD,
            BTN_TODO_CLEAR,
            BTN_TODO_PREV,
            BTN_TODO_NEXT,
            LBL_TODO_PAGE,
        ];
        let notes_ids = [
            ED_NOTES,
            HINT_NOTES,
            BTN_NOTES_SAVE,
            BTN_NOTE_BACK,
            LBL_NOTE_META,
        ];
        for &id in &todo_ids {
            let c = GetDlgItem(app.menu_hwnd, id as i32);
            if !c.is_null() {
                ShowWindow(
                    c,
                    if tab == 0 {
                        SW_SHOW as i32
                    } else {
                        SW_HIDE as i32
                    },
                );
            }
        }
        for &id in &notes_ids {
            let c = GetDlgItem(app.menu_hwnd, id as i32);
            if !c.is_null() {
                ShowWindow(
                    c,
                    if tab == 1 && app.notes_editor_open {
                        SW_SHOW as i32
                    } else {
                        SW_HIDE as i32
                    },
                );
            }
        }
        update_hint(
            app.menu_hwnd,
            if tab == 0 { ED_TODO_INPUT } else { ED_NOTES },
        );
        update_todo_page(app);
        sync_notes_controls(app);
        SetFocus(GetDlgItem(
            app.menu_hwnd,
            if tab == 0 {
                ED_TODO_INPUT
            } else if app.notes_editor_open {
                ED_NOTES
            } else {
                BTN_NOTE_NEW
            } as i32,
        ));
        RedrawWindow(
            app.menu_hwnd,
            std::ptr::null(),
            std::ptr::null_mut(),
            RDW_INVALIDATE | RDW_ALLCHILDREN,
        );
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
    let font = ui_font(size, weight);
    let old = SelectObject(hdc, font);
    SetBkMode(hdc, 1);
    SetTextColor(hdc, clr);
    let mut rc = RECT {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    };
    DrawTextW(hdc, wide(s).as_ptr(), -1, &mut rc, flags | DT_NOPREFIX);
    SelectObject(hdc, old);
}

fn note_date(seconds: u64) -> String {
    if seconds == 0 {
        return "Imported from previous notes".into();
    }
    unsafe {
        use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
        use windows_sys::Win32::System::Time::{
            FileTimeToSystemTime, SystemTimeToTzSpecificLocalTime,
        };
        let Some(ticks) = seconds
            .checked_add(11_644_473_600)
            .and_then(|value| value.checked_mul(10_000_000))
        else {
            return "Date unavailable".into();
        };
        let filetime = FILETIME {
            dwLowDateTime: ticks as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        };
        let mut utc: SYSTEMTIME = std::mem::zeroed();
        let mut local: SYSTEMTIME = std::mem::zeroed();
        if FileTimeToSystemTime(&filetime, &mut utc) == 0
            || SystemTimeToTzSpecificLocalTime(std::ptr::null(), &utc, &mut local) == 0
        {
            return "Date unavailable".into();
        }
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month = months
            .get(local.wMonth.saturating_sub(1) as usize)
            .copied()
            .unwrap_or("?");
        format!(
            "{:02} {month} {} · {:02}:{:02}",
            local.wDay, local.wYear, local.wHour, local.wMinute
        )
    }
}

fn sync_notes_controls(app: &super::App) {
    if app.menu_hwnd.is_null() {
        return;
    }
    unsafe {
        let listing = app.menu_tab == 1 && !app.notes_editor_open;
        let count = app
            .notes
            .len()
            .saturating_sub(app.notes_page * NOTES_PAGE_SIZE)
            .min(NOTES_PAGE_SIZE);
        let show = |id: u32, visible: bool| {
            ShowWindow(
                GetDlgItem(app.menu_hwnd, id as i32),
                if visible { SW_SHOW } else { SW_HIDE },
            );
        };
        show(BTN_NOTE_NEW, listing);
        show(BTN_NOTE_UNDO, listing && app.deleted_note.is_some());
        for row in 0..NOTES_PAGE_SIZE {
            show(BTN_NOTE_EDIT + row as u32, listing && row < count);
            show(BTN_NOTE_DELETE + row as u32, listing && row < count);
        }
        let pages = app.notes.len().div_ceil(NOTES_PAGE_SIZE).max(1);
        for id in [BTN_NOTES_PREV, BTN_NOTES_NEXT, LBL_NOTES_PAGE] {
            show(id, listing && pages > 1);
        }
        SetWindowTextW(
            GetDlgItem(app.menu_hwnd, LBL_NOTES_PAGE as i32),
            wide(&format!("Page {} of {}", app.notes_page + 1, pages)).as_ptr(),
        );
        EnableWindow(
            GetDlgItem(app.menu_hwnd, BTN_NOTES_PREV as i32),
            i32::from(app.notes_page > 0),
        );
        EnableWindow(
            GetDlgItem(app.menu_hwnd, BTN_NOTES_NEXT as i32),
            i32::from(app.notes_page + 1 < pages),
        );
        let metadata = if let Some(note) = app
            .editing_note
            .and_then(|id| app.notes.iter().find(|note| note.id == id))
        {
            if note.created_at == 0 {
                format!("Imported note\r\nSaved {}", note_date(note.updated_at))
            } else {
                format!(
                    "Created {}\r\nSaved {}",
                    note_date(note.created_at),
                    note_date(note.updated_at)
                )
            }
        } else {
            "New note · autosaves as you type".into()
        };
        SetWindowTextW(
            GetDlgItem(app.menu_hwnd, LBL_NOTE_META as i32),
            wide(&metadata).as_ptr(),
        );
        if !app.notes_editor_open {
            ShowWindow(GetDlgItem(app.menu_hwnd, HINT_NOTES as i32), SW_HIDE);
        }
    }
}

fn open_note(app: &mut super::App, id: Option<u64>) {
    if !flush_data(app) {
        refresh_todo_list(app);
        return;
    }
    app.notes_editor_open = false;
    app.editing_note = id;
    app.note = id
        .and_then(|id| app.notes.iter().find(|note| note.id == id))
        .map(|note| note.text.clone())
        .unwrap_or_default();
    refresh_notes(app);
    app.notes_editor_open = true;
    app.menu_tab = 1;
    show_tab_content(app, 1);
}

unsafe fn paint_note_cards(hdc: HDC, app: &super::App) {
    if app.notes.is_empty() {
        draw_text(
            hdc,
            "Keep your thoughts together",
            30,
            160,
            MENU_W - 60,
            26,
            CLR_TXT,
            15,
            600,
            DT_CENTER | DT_SINGLELINE,
        );
        draw_text(
            hdc,
            "Create a note below. Each one gets its own card.",
            30,
            191,
            MENU_W - 60,
            26,
            CLR_TXT_DIM,
            12,
            400,
            DT_CENTER | DT_SINGLELINE,
        );
        return;
    }
    let start = app.notes_page * NOTES_PAGE_SIZE;
    for (row, note) in app
        .notes
        .iter()
        .skip(start)
        .take(NOTES_PAGE_SIZE)
        .enumerate()
    {
        let y = 84 + row as i32 * 80;
        let card = CreateRoundRectRgn(22, y, MENU_W - 22, y + 72, 10, 10);
        let brush = CreateSolidBrush(CLR_WHITE);
        FillRgn(hdc, card, brush);
        DeleteObject(brush);
        let border = CreateSolidBrush(CLR_CARD_BDR);
        FrameRgn(hdc, card, border, 1, 1);
        DeleteObject(border);
        DeleteObject(card);
        draw_text(
            hdc,
            note.title(),
            32,
            y + 7,
            275,
            23,
            CLR_TXT,
            13,
            600,
            DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_text(
            hdc,
            &note.preview(),
            32,
            y + 30,
            404,
            18,
            CLR_TXT_DIM,
            12,
            400,
            DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        let stamp = if note.updated_at == 0 {
            note_date(0)
        } else {
            format!("Saved {}", note_date(note.updated_at))
        };
        draw_text(
            hdc,
            &stamp,
            32,
            y + 50,
            404,
            17,
            CLR_TXT_DONE,
            11,
            400,
            DT_LEFT | DT_SINGLELINE,
        );
    }
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
            &RECT {
                left: 0,
                top: 0,
                right: MENU_W,
                bottom: MENU_H,
            },
            bg,
        );
        DeleteObject(bg);

        // Header bar
        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(
            mem,
            &RECT {
                left: 0,
                top: 0,
                right: MENU_W,
                bottom: 64,
            },
            hdr,
        );
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 64, std::ptr::null_mut());
        LineTo(mem, MENU_W, 64);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

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
            let item_h = TODO_ROW_H;
            let max_items = TODO_PAGE_SIZE;
            let start = app.todo_page * TODO_PAGE_SIZE;
            let count = app.todos.len().saturating_sub(start).min(max_items);
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
                    let (ref text, done) = app.todos[start + i];
                    let y = items_y0 + (i as i32) * item_h;
                    let chk_x = 28;
                    let chk_y = y + 5;
                    let chk_sz = 17;
                    let border_pen =
                        CreatePen(PS_SOLID, 1, if done { CLR_PILL_ON } else { CLR_CHK });
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
                        MENU_W - 110,
                        item_h,
                        if done { CLR_TXT_DONE } else { CLR_TXT },
                        13,
                        400,
                        DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                    );
                    draw_text(
                        mem,
                        "×",
                        MENU_W - 48,
                        y,
                        24,
                        item_h,
                        CLR_TXT_DIM,
                        14,
                        400,
                        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                    );
                    if done {
                        let strike_pen = CreatePen(PS_SOLID, 1, CLR_TXT_DONE);
                        let old_sp = SelectObject(mem, strike_pen);
                        let old_font = SelectObject(mem, ui_font(13, 400));
                        let encoded = wide(text);
                        let mut size = windows_sys::Win32::Foundation::SIZE { cx: 0, cy: 0 };
                        GetTextExtentPoint32W(
                            mem,
                            encoded.as_ptr(),
                            (encoded.len() - 1) as i32,
                            &mut size,
                        );
                        SelectObject(mem, old_font);
                        let tlen = size.cx.min(MENU_W - 110);
                        MoveToEx(mem, 52, y + item_h / 2, std::ptr::null_mut());
                        LineTo(mem, 52 + tlen, y + item_h / 2);
                        SelectObject(mem, old_sp);
                        DeleteObject(strike_pen);
                    }
                }
            }
        }

        if app.menu_tab == 1 && !app.notes_editor_open {
            paint_note_cards(mem, app);
        }

        // Draw crisp perimeter border matching the asymmetric region shape
        draw_window_border(mem, MENU_W, MENU_H, 18);

        if !app.data_status.is_empty() {
            draw_text(
                mem,
                &app.data_status,
                16,
                423,
                MENU_W - 32,
                25,
                if app.data_dirty {
                    rgb(0xB45309)
                } else {
                    CLR_TXT_DIM
                },
                11,
                400,
                DT_LEFT | DT_WORDBREAK,
            );
        }
        // Update status line (check-for-update feedback)
        if !app.status.is_empty() && app.data_status.is_empty() {
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
        BTN_NOTE_NEW => {
            if flush_data(app) {
                app.deleted_note = None;
                open_note(app, None);
            }
        }
        BTN_NOTE_BACK => {
            if flush_data(app) {
                app.notes_editor_open = false;
                app.editing_note = None;
                app.notes_page = 0;
                show_tab_content(app, app.menu_tab);
            } else {
                refresh_todo_list(app);
            }
        }
        BTN_NOTES_PREV | BTN_NOTES_NEXT => {
            let pages = app.notes.len().div_ceil(NOTES_PAGE_SIZE).max(1);
            app.notes_page = if id == BTN_NOTES_PREV {
                app.notes_page.saturating_sub(1)
            } else {
                (app.notes_page + 1).min(pages - 1)
            };
            sync_notes_controls(app);
            refresh_todo_list(app);
        }
        id if (BTN_NOTE_EDIT..BTN_NOTE_EDIT + NOTES_PAGE_SIZE as u32).contains(&id) => {
            let index = app.notes_page * NOTES_PAGE_SIZE + (id - BTN_NOTE_EDIT) as usize;
            if let Some(note) = app.notes.get(index) {
                open_note(app, Some(note.id));
            }
        }
        id if (BTN_NOTE_DELETE..BTN_NOTE_DELETE + NOTES_PAGE_SIZE as u32).contains(&id) => {
            let index = app.notes_page * NOTES_PAGE_SIZE + (id - BTN_NOTE_DELETE) as usize;
            if index < app.notes.len() {
                app.deleted_note = Some(app.notes.remove(index));
                app.notes_page = app
                    .notes_page
                    .min(app.notes.len().saturating_sub(1) / NOTES_PAGE_SIZE);
                app.save_data();
                sync_notes_controls(app);
                refresh_todo_list(app);
            }
        }
        BTN_NOTE_UNDO => {
            if let Some(note) = app.deleted_note.take() {
                app.notes.push(note);
                app.notes
                    .sort_by_key(|note| std::cmp::Reverse((note.updated_at, note.id)));
                app.notes_page = 0;
                app.save_data();
                sync_notes_controls(app);
                refresh_todo_list(app);
            }
        }
        BTN_TODO_PREV | BTN_TODO_NEXT => {
            let pages = app.todos.len().div_ceil(TODO_PAGE_SIZE).max(1);
            app.todo_page = if id == BTN_TODO_PREV {
                app.todo_page.saturating_sub(1)
            } else {
                (app.todo_page + 1).min(pages - 1)
            };
            update_todo_page(app);
            refresh_todo_list(app);
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
                        app.todo_page = (app.todos.len() - 1) / TODO_PAGE_SIZE;
                        app.save_data();
                        SetWindowTextW(c, wstr(""));
                        SetFocus(c);
                        update_todo_page(app);
                        refresh_todo_list(app);
                    }
                }
            }
        }
        BTN_TODO_CLEAR => {
            app.todos.clear();
            app.todo_page = 0;
            app.save_data();
            update_todo_page(app);
            refresh_todo_list(app);
        }
        BTN_NOTES_SAVE => {
            if app.menu_hwnd.is_null() {
                return;
            }
            unsafe {
                let c = GetDlgItem(app.menu_hwnd, ED_NOTES as i32);
                if !c.is_null() {
                    if app.notes_editor_open {
                        app.note = get_ctl_text(c);
                    }
                    KillTimer(app.menu_hwnd, TIMER_AUTOSAVE);
                    app.save_data();
                    refresh_todo_list(app);
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
        BTN_CLOSE => unsafe {
            close_menu(app);
        },
        BTN_EXIT => {
            if !flush_data(app) {
                refresh_todo_list(app);
                return;
            }
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
        sync_notes_controls(app);
        update_todo_page(app);
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
            let text = app.note.replace("\r\n", "\n").replace('\n', "\r\n");
            SetWindowTextW(c, wide(&text).as_ptr());
        }
    }
}

fn update_todo_page(app: &super::App) {
    unsafe {
        if app.menu_hwnd.is_null() {
            return;
        }
        let pages = app.todos.len().div_ceil(TODO_PAGE_SIZE).max(1);
        let label = format!("Page {} of {}", app.todo_page + 1, pages);
        for id in [BTN_TODO_PREV, BTN_TODO_NEXT, LBL_TODO_PAGE] {
            ShowWindow(
                GetDlgItem(app.menu_hwnd, id as i32),
                if app.menu_tab == 0 && pages > 1 {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
        SetWindowTextW(
            GetDlgItem(app.menu_hwnd, LBL_TODO_PAGE as i32),
            wide(&label).as_ptr(),
        );
        EnableWindow(
            GetDlgItem(app.menu_hwnd, BTN_TODO_PREV as i32),
            i32::from(app.todo_page > 0),
        );
        EnableWindow(
            GetDlgItem(app.menu_hwnd, BTN_TODO_NEXT as i32),
            i32::from(app.todo_page + 1 < pages),
        );
        EnableWindow(
            GetDlgItem(app.menu_hwnd, BTN_TODO_CLEAR as i32),
            i32::from(!app.todos.is_empty()),
        );
    }
}

pub(crate) fn flush_data(app: &mut super::App) -> bool {
    // EN_CHANGE copies each draft to app.note immediately, including while the
    // editor is hidden. Never read controls here: children may be destroying.
    if app.data_dirty {
        app.save_data()
    } else {
        true
    }
}

unsafe fn close_menu(app: &mut super::App) {
    if !flush_data(app) {
        KillTimer(app.menu_hwnd, TIMER_MENU_CLOSE as usize);
        SetForegroundWindow(app.menu_hwnd);
        refresh_todo_list(app);
        return;
    }
    DestroyWindow(app.menu_hwnd);
}

unsafe fn close_child_panel(hwnd: HWND, app: &mut super::App) {
    KillTimer(app.menu_hwnd, TIMER_MENU_CLOSE as usize);
    if hwnd == app.customize_hwnd {
        app.customize_hwnd = std::ptr::null_mut();
    }
    if hwnd == app.settings_hwnd {
        app.settings_hwnd = std::ptr::null_mut();
    }
    DestroyWindow(hwnd);
    show_menu(app);
    if !app.menu_hwnd.is_null() {
        // Returning from a child panel must restore focus before the menu's
        // lost-focus timer decides the user has moved to another app.
        KillTimer(app.menu_hwnd, TIMER_MENU_CLOSE as usize);
        app.menu_timer_id = 0;
        SetForegroundWindow(app.menu_hwnd);
        let id = if app.menu_tab == 0 {
            ED_TODO_INPUT
        } else if app.notes_editor_open {
            ED_NOTES
        } else {
            BTN_NOTE_NEW
        };
        SetFocus(GetDlgItem(app.menu_hwnd, id as i32));
    }
}

pub(crate) fn process_keyboard(app: &mut super::App, msg: &MSG) -> bool {
    unsafe {
        let panels = [app.customize_hwnd, app.settings_hwnd, app.menu_hwnd];
        for panel in panels {
            if panel.is_null() || !is_child_of(panel, msg.hwnd) {
                continue;
            }
            if msg.message == WM_KEYDOWN {
                let ctrl = GetKeyState(VK_CONTROL as i32) < 0 && GetKeyState(VK_MENU as i32) >= 0;
                if msg.wParam == VK_ESCAPE as usize {
                    if panel == app.menu_hwnd {
                        close_menu(app);
                    } else {
                        close_child_panel(panel, app);
                    }
                    return true;
                }
                if panel == app.menu_hwnd && ctrl && msg.wParam == b'S' as usize {
                    handle_command(app, BTN_NOTES_SAVE);
                    return true;
                }
                if panel == app.menu_hwnd
                    && app.menu_tab == 1
                    && ctrl
                    && msg.wParam == b'N' as usize
                {
                    handle_command(app, BTN_NOTE_NEW);
                    return true;
                }
                if msg.wParam == VK_RETURN as usize
                    && GetDlgCtrlID(msg.hwnd) == ED_TODO_INPUT as i32
                {
                    handle_command(app, BTN_TODO_ADD);
                    return true;
                }
                if msg.wParam == VK_RETURN as usize && GetDlgCtrlID(msg.hwnd) == ED_SIZE as i32 {
                    SendMessageW(panel, WM_COMMAND, BTN_SIZE_APPLY as usize, 0);
                    return true;
                }
            }
            return IsDialogMessageW(panel, msg) != 0;
        }
        false
    }
}

fn save_cfg(app: &mut super::App) {
    app.save_settings();
}

pub(crate) fn check_for_update(app: &mut super::App) {
    if app.updater.start(false) {
        app.status = "Checking for updates…".into();
    }
}

pub(crate) fn poll_update(app: &mut super::App) {
    use super::updater::Event;
    if app
        .updater
        .automatic_due(app.auto_update, std::time::Instant::now())
    {
        app.updater.start(true);
        app.status = "Checking for updates automatically…".into();
    }
    let Some((automatic, result)) = app.updater.poll() else {
        return;
    };
    match result {
        Err(error) => app.status = error,
        Ok(Event::Message(message) | Event::Progress(message)) => app.status = message,
        Ok(Event::Downloaded(stage)) => {
            app.status = "Update verified. Preparing a safe restart…".into();
            app.updater.prepare(
                stage,
                automatic,
                app.exe.clone(),
                super::config::app_dir().join("update-result.txt"),
            );
        }
        Ok(Event::Ready(mut install)) => {
            if !flush_data(app) || !app.save_settings() {
                app.status = "Update paused because your changes could not be saved.".into();
            } else if let Err(error) = install.commit() {
                app.status = error;
            } else {
                unsafe {
                    DestroyWindow(app.hwnd);
                }
                return;
            }
        }
    }
    refresh_todo_list(app);
    unsafe {
        if !app.settings_hwnd.is_null() {
            InvalidateRect(app.settings_hwnd, std::ptr::null(), 1);
        }
    }
}

unsafe fn update_hint(hwnd: HWND, edit_id: u32) {
    let hint_id = if edit_id == ED_TODO_INPUT {
        HINT_TODO
    } else {
        HINT_NOTES
    };
    let c = GetDlgItem(hwnd, hint_id as i32);
    let e = GetDlgItem(hwnd, edit_id as i32);
    if c.is_null() || e.is_null() {
        return;
    }
    let has_text = GetWindowTextLengthW(e) > 0;
    let focused = GetFocus() == e;
    ShowWindow(
        c,
        if IsWindowVisible(e) == 0 || has_text || focused {
            SW_HIDE as i32
        } else {
            SW_SHOW as i32
        },
    );
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
                        || code == 0x0300/* EN_CHANGE */)
                {
                    update_hint(hwnd, id);
                    if id == ED_NOTES
                        && code == EN_CHANGE as usize
                        && app.notes_editor_open
                        && app.menu_hwnd == hwnd
                    {
                        let note = get_ctl_text(lparam as HWND);
                        if app.note != note {
                            app.note = note;
                            app.data_dirty = true;
                            app.data_status = "Saving changes…".into();
                            SetTimer(hwnd, TIMER_AUTOSAVE, 700, None);
                            refresh_todo_list(app);
                        }
                    }
                }
                if code == BN_CLICKED as usize {
                    handle_command(app, id);
                }
                0
            }
            WM_DRAWITEM => draw_button(lparam),
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
                control_colors(msg, wparam, lparam)
            }
            WM_KEYDOWN => {
                if wparam as u16 == VK_ESCAPE {
                    close_menu(app);
                }
                0
            }
            WM_CLOSE => {
                close_menu(app);
                0
            }
            WM_DESTROY => {
                flush_data(app);
                app.notes_editor_open = false;
                app.editing_note = None;
                app.menu_hwnd = std::ptr::null_mut();
                KillTimer(hwnd, TIMER_MENU_CLOSE as usize);
                KillTimer(hwnd, TIMER_AUTOSAVE);
                app.menu_timer_id = 0;
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                paint_menu(hwnd, app);
                0
            }
            WM_ACTIVATE => {
                if wparam == 0 && !app.keep_panels_open {
                    app.menu_timer_id = SetTimer(hwnd, TIMER_MENU_CLOSE as usize, 50, None) as u32;
                }
                0
            }
            WM_TIMER => {
                if wparam == TIMER_AUTOSAVE {
                    KillTimer(hwnd, TIMER_AUTOSAVE);
                    flush_data(app);
                    refresh_todo_list(app);
                    return 0;
                }
                if wparam == TIMER_MENU_CLOSE as WPARAM {
                    let foreground = GetForegroundWindow();
                    if foreground.is_null() || !is_child_of(hwnd, foreground) {
                        KillTimer(hwnd, TIMER_MENU_CLOSE as usize);
                        app.menu_timer_id = 0;
                        close_menu(app);
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
                    close_menu(app);
                } else if app.menu_tab == 1 && !app.notes_editor_open {
                    if x >= 22 && x <= MENU_W - 22 {
                        for row in 0..NOTES_PAGE_SIZE {
                            let top = 84 + row as i32 * 80;
                            if y >= top && y <= top + 72 {
                                let index = app.notes_page * NOTES_PAGE_SIZE + row;
                                if let Some(note) = app.notes.get(index) {
                                    open_note(app, Some(note.id));
                                }
                                break;
                            }
                        }
                    }
                } else if app.menu_tab == 0 {
                    let items_y0 = 86i32;
                    let item_h = TODO_ROW_H;
                    let max_items = TODO_PAGE_SIZE;
                    let start = app.todo_page * TODO_PAGE_SIZE;
                    let count = app.todos.len().saturating_sub(start).min(max_items);
                    for i in 0..count {
                        let item_y = items_y0 + (i as i32) * item_h;
                        if x >= 24 && x <= 48 && y >= item_y + 2 && y <= item_y + 24 {
                            app.todos[start + i].1 = !app.todos[start + i].1;
                            app.save_data();
                            InvalidateRect(hwnd, std::ptr::null(), 1);
                            break;
                        }
                        if x >= MENU_W - 48
                            && x <= MENU_W - 24
                            && y >= item_y
                            && y <= item_y + item_h
                        {
                            app.todos.remove(start + i);
                            app.todo_page = app
                                .todo_page
                                .min(app.todos.len().saturating_sub(1) / TODO_PAGE_SIZE);
                            app.save_data();
                            update_todo_page(app);
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
        if !app.customize_hwnd.is_null() && IsWindow(app.customize_hwnd) != 0 {
            SetForegroundWindow(app.customize_hwnd);
            return;
        }
        app.customize_hwnd = std::ptr::null_mut();
        let (mx, my) = panel_position(app, COS_W, COS_H);
        let class_name = wstr("CatCustomizeWnd");
        let hinst = GetModuleHandleW(std::ptr::null());
        let brush = GetStockObject(WHITE_BRUSH);
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
            WS_POPUP | WS_VISIBLE | WS_CLIPCHILDREN,
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
        let rgn = create_asymmetric_rgn(COS_W, COS_H, 18);
        SetWindowRgn(hwnd, rgn, 1);
        app.customize_hwnd = hwnd;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as *mut super::App as isize);
        create_customize_controls(hwnd, app);
        ShowWindow(hwnd, 1);
        SetForegroundWindow(hwnd);
        SetFocus(GetDlgItem(
            hwnd,
            (BTN_COLOR_BLACK + app.color as u32) as i32,
        ));
    }
}

unsafe fn create_customize_controls(hwnd: HWND, app: &super::App) {
    let font = ui_font(13, 400);
    // Checkbox appearance is shared with Settings. The model keeps each choice
    // group exclusive, with scarf independent of the bell/tie group.
    let selection = BS_CHECKBOX as u32;
    make_close_button(hwnd, COS_W);

    make_ctl(
        hwnd,
        0,
        "STATIC",
        "Cat color",
        28,
        88,
        414,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    let color_black = make_ctl(
        hwnd,
        BTN_COLOR_BLACK,
        "BUTTON",
        "Black",
        28,
        116,
        130,
        26,
        selection,
        font,
    );
    let color_white = make_ctl(
        hwnd,
        BTN_COLOR_WHITE,
        "BUTTON",
        "White",
        174,
        116,
        130,
        26,
        selection,
        font,
    );
    let color_orange = make_ctl(
        hwnd,
        BTN_COLOR_ORANGE,
        "BUTTON",
        "Orange",
        320,
        116,
        122,
        26,
        selection,
        font,
    );
    let color_sel = if app.color == 0 {
        color_black
    } else if app.color == 1 {
        color_white
    } else {
        color_orange
    };
    SendMessageW(color_sel, BM_SETCHECK, BST_CHECKED as _, 0);

    make_ctl(
        hwnd,
        0,
        "STATIC",
        "Scarf",
        28,
        180,
        414,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    let scarf_sel = app.cosmetic_scarf;
    for i in 0..6usize {
        let id = BTN_SCARF_NONE + i as u32;
        let name = if i == 0 {
            "None"
        } else {
            super::cosmetics::SCARF_ITEMS[i - 1].name
        };
        let val = if i == 0 { None } else { Some(i - 1) };
        let btn = make_ctl(
            hwnd,
            id,
            "BUTTON",
            name,
            28 + (i as i32 % 3) * 146,
            208 + (i as i32 / 3) * 32,
            122,
            26,
            selection,
            font,
        );
        if val == scarf_sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }

    make_ctl(
        hwnd,
        0,
        "STATIC",
        "Bell",
        28,
        312,
        190,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    make_ctl(
        hwnd,
        0,
        "STATIC",
        "Tie",
        246,
        312,
        196,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    let bell_rel: [Option<usize>; 3] = [None, Some(0), Some(1)];
    for i in 0..3usize {
        let bell_id = BTN_BELL_NONE + i as u32;
        let bell_name = if i == 0 {
            "None"
        } else {
            super::cosmetics::BELL_ITEMS[i - 1].name
        };
        let sel = bell_rel[i] == app.cosmetic_bell;
        let btn = make_ctl(
            hwnd,
            bell_id,
            "BUTTON",
            bell_name,
            28,
            356 + (i as i32) * 26,
            190,
            24,
            selection,
            font,
        );
        if sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }
    let tie_rel: [Option<usize>; 4] = [None, Some(0), Some(1), Some(2)];
    for i in 0..4usize {
        let tie_id = BTN_TIE_NONE + i as u32;
        let tie_name = if i == 0 {
            "None"
        } else {
            super::cosmetics::TIE_ITEMS[i - 1].name
        };
        let sel = tie_rel[i] == app.cosmetic_tie;
        let btn = make_ctl(
            hwnd,
            tie_id,
            "BUTTON",
            tie_name,
            246,
            356 + (i as i32) * 26,
            196,
            24,
            selection,
            font,
        );
        if sel {
            SendMessageW(btn, BM_SETCHECK, BST_CHECKED as _, 0);
        }
    }

    make_ctl(
        hwnd,
        0,
        "STATIC",
        "Cat size · 100–500 px",
        28,
        488,
        414,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    let size_edit = make_ctl(
        hwnd,
        ED_SIZE,
        "EDIT",
        &app.normal_width().to_string(),
        28,
        516,
        304,
        26,
        ES_NUMBER as u32 | ES_AUTOHSCROLL as u32 | WS_BORDER,
        font,
    );
    let _ = size_edit;
    make_ctl(
        hwnd,
        BTN_SIZE_APPLY,
        "BUTTON",
        "Apply",
        344,
        514,
        98,
        30,
        0,
        font,
    );

    make_ctl(
        hwnd,
        BTN_COS_BACK,
        "BUTTON",
        "Back",
        16,
        564,
        213,
        32,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_COS_CLEAR_ALL,
        "BUTTON",
        "Remove accessories",
        241,
        564,
        213,
        32,
        0,
        font,
    );
}

unsafe fn sync_cos_checks(hwnd: HWND, app: &super::App) {
    let check = |id: u32, on: bool| {
        let view = GetDlgItem(hwnd, SET_VIEW_ID as i32);
        let c = if view.is_null() {
            GetDlgItem(hwnd, id as i32)
        } else {
            GetDlgItem(view, id as i32)
        };
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
    for i in 0..9u32 {
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
    app.annoyed_drawn = false;
    app.save_settings();
}

pub(crate) fn paint_customize(hwnd: HWND, app: &super::App) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mem = CreateCompatibleDC(hdc);
        let bmp = CreateCompatibleBitmap(hdc, COS_W, COS_H);
        SelectObject(mem, bmp);

        let bg = CreateSolidBrush(CLR_BG);
        FillRect(
            mem,
            &RECT {
                left: 0,
                top: 0,
                right: COS_W,
                bottom: COS_H,
            },
            bg,
        );
        DeleteObject(bg);

        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(
            mem,
            &RECT {
                left: 0,
                top: 0,
                right: COS_W,
                bottom: 64,
            },
            hdr,
        );
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 64, std::ptr::null_mut());
        LineTo(mem, COS_W, 64);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

        draw_text(
            mem,
            "Customize",
            28,
            8,
            COS_W - 56,
            28,
            CLR_TXT,
            16,
            700,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        draw_text(
            mem,
            "Pick a color and dress up your cat.",
            28,
            36,
            COS_W - 56,
            18,
            CLR_TXT_DIM,
            11,
            400,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        let cards = [
            (16i32, 76i32, COS_W - 16, 156i32),
            (16, 168, COS_W - 16, 288),
            (16, 300, COS_W - 16, 464),
            (16, 476, COS_W - 16, 552),
        ];
        for &(x0, y0, x1, y1) in &cards {
            paint_card(mem, x0, y0, x1, y1);
        }
        draw_text(
            mem,
            "Choose either a bell or a tie.",
            28,
            334,
            COS_W - 56,
            18,
            CLR_TXT_DIM,
            11,
            400,
            DT_LEFT | DT_SINGLELINE,
        );

        // Draw crisp perimeter border matching the asymmetric region shape
        draw_window_border(mem, COS_W, COS_H, 18);

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
                    BTN_COS_BACK | BTN_CLOSE => {
                        close_child_panel(hwnd, app);
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
                                SetWindowTextW(c, wide(&app.w.to_string()).as_ptr());
                            } else {
                                SetWindowTextW(c, wide(&app.w.to_string()).as_ptr());
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
                        let selected = Some((id - BTN_BELL_0) as usize);
                        app.cosmetic_bell = if app.cosmetic_bell == selected {
                            None
                        } else {
                            selected
                        };
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
                        let selected = Some((id - BTN_SCARF_0) as usize);
                        app.cosmetic_scarf = if app.cosmetic_scarf == selected {
                            None
                        } else {
                            selected
                        };
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_TIE_NONE => {
                        app.cosmetic_tie = None;
                        apply_cosmetics(app);
                        sync_cos_checks(hwnd, app);
                    }
                    BTN_TIE_0 | BTN_TIE_1 | BTN_TIE_2 => {
                        let selected = Some((id - BTN_TIE_0) as usize);
                        app.cosmetic_tie = if app.cosmetic_tie == selected {
                            None
                        } else {
                            selected
                        };
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
                    close_child_panel(hwnd, app);
                }
                0
            }
            WM_CLOSE => {
                close_child_panel(hwnd, app);
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
                    close_child_panel(hwnd, app);
                }
                0
            }
            WM_HSCROLL => 0,
            WM_DRAWITEM => draw_button(lparam),
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
                control_colors(msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub(crate) fn show_settings_panel(app: &mut super::App) {
    unsafe {
        if !app.settings_hwnd.is_null() && IsWindow(app.settings_hwnd) != 0 {
            SetForegroundWindow(app.settings_hwnd);
            SetFocus(app.settings_hwnd);
            return;
        }
        app.settings_hwnd = std::ptr::null_mut();
        let (mx, my) = panel_position(app, SET_W, SET_H);
        let class_name = wstr("CatSettingsWnd");
        let hinst = GetModuleHandleW(std::ptr::null());
        let brush = GetStockObject(WHITE_BRUSH);
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
            WS_POPUP | WS_VISIBLE | WS_CLIPCHILDREN,
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
        let rgn = create_asymmetric_rgn(SET_W, SET_H, 18);
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
        let view = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            view_class,
            wstr(""),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_CLIPCHILDREN,
            SET_VIEW_X,
            SET_VIEW_Y,
            SET_W - SET_VIEW_X * 2,
            SET_VIEW_H,
            hwnd,
            SET_VIEW_ID as _,
            hinst,
            std::ptr::null(),
        );
        create_settings_controls(hwnd, view, app);
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
        BTN_LBL_UPDATES => 12,
        BTN_AUTO_UPDATE => 42,
        LBL_UPDATE_HINT => 76,
        BTN_LBL_WINDOW => 132,
        BTN_TOPMOST => 160,
        BTN_STARTUP => 190,
        BTN_DESKTOP => 220,
        BTN_LBL_HOTKEYS => 284,
        BTN_HK_BLACK..=BTN_HK_TODO => {
            316 + HOTKEY_CHOICES
                .iter()
                .position(|choice| choice.0 == id)
                .unwrap() as i32
                * 30
        }
        _ => -1,
    }
}

fn settings_max_scroll() -> i32 {
    (SET_CONTENT_BOTTOM - SET_VIEW_H).max(0)
}

unsafe fn init_settings_scroll(hwnd: HWND) {
    let max = settings_max_scroll();
    let page: u32 = if max > 0 { SET_VIEW_H as u32 } else { 0 };
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
                    12,
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
    let font = ui_font(13, 400);
    make_close_button(hwnd, SET_W);
    make_ctl(
        hwnd,
        LBL_UPDATE_STATUS,
        "STATIC",
        &app.status,
        16,
        606,
        SET_W - 32,
        27,
        SS_LEFT,
        ui_font(11, 400),
    );

    make_ctl(
        view,
        BTN_LBL_UPDATES,
        "STATIC",
        "Updates",
        12,
        12,
        390,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    make_check(
        view,
        BTN_AUTO_UPDATE,
        "Automatically install GitHub updates",
        12,
        42,
        390,
        26,
        app.auto_update,
    );
    make_ctl(
        view,
        LBL_UPDATE_HINT,
        "STATIC",
        "Checks at startup and every 6 hours. Saves before restart.",
        12,
        76,
        390,
        20,
        SS_LEFT,
        ui_font(11, 400),
    );

    make_ctl(
        view,
        BTN_LBL_WINDOW,
        "STATIC",
        "Window",
        12,
        132,
        390,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    make_check(
        view,
        BTN_TOPMOST,
        "Keep cat above other windows",
        12,
        160,
        390,
        26,
        app.always_on_top,
    );
    make_check(
        view,
        BTN_STARTUP,
        "Start with Windows",
        12,
        190,
        390,
        26,
        app.startup_enabled(),
    );
    make_check(
        view,
        BTN_DESKTOP,
        "Show a desktop shortcut",
        12,
        220,
        390,
        26,
        app.desktop_shortcut_exists(),
    );

    make_ctl(
        view,
        BTN_LBL_HOTKEYS,
        "STATIC",
        "Keyboard shortcuts",
        12,
        284,
        390,
        20,
        SS_LEFT,
        ui_font(13, 600),
    );
    for (i, &(id, name, _)) in HOTKEY_CHOICES.iter().enumerate() {
        make_check(
            view,
            id,
            name,
            12,
            316 + (i as i32) * 30,
            262,
            26,
            app.hotkey_enabled((id - BTN_HK_BLACK) as usize),
        );
    }

    make_ctl(
        hwnd,
        BTN_SCAN,
        "BUTTON",
        "Check for updates",
        241,
        566,
        213,
        32,
        0,
        font,
    );
    make_ctl(
        hwnd,
        BTN_COS_BACK,
        "BUTTON",
        "Back",
        16,
        566,
        213,
        32,
        0,
        font,
    );
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
            WM_VSCROLL | WM_MOUSEWHEEL => {
                SendMessageW(GetParent(hwnd), msg, wparam, lparam);
                0
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
                control_colors(msg, wparam, lparam)
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                let mut rc = std::mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                let bg = CreateSolidBrush(CLR_BG);
                FillRect(hdc, &rc, bg);
                DeleteObject(bg);
                let app_ptr = GetWindowLongPtrW(GetParent(hwnd), GWLP_USERDATA);
                let scroll = if app_ptr == 0 {
                    0
                } else {
                    (*(app_ptr as *const super::App)).settings_scroll
                };
                paint_card(hdc, 0, -scroll, rc.right, 108 - scroll);
                paint_card(hdc, 0, 120 - scroll, rc.right, 260 - scroll);
                paint_card(hdc, 0, 272 - scroll, rc.right, SET_CONTENT_BOTTOM - scroll);
                for (i, &(_, _, key)) in HOTKEY_CHOICES.iter().enumerate() {
                    draw_text(
                        hdc,
                        &format!("Ctrl + Alt + {key}"),
                        284,
                        316 + i as i32 * 30 - scroll,
                        rc.right - 296,
                        26,
                        CLR_TXT_DIM,
                        11,
                        400,
                        DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
                    );
                }
                EndPaint(hwnd, &mut ps);
                0
            }
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
    let hwnd = make_ctl(
        parent,
        id,
        "BUTTON",
        text,
        x,
        y,
        w,
        h,
        BS_AUTOCHECKBOX as u32,
        ui_font(13, 400),
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
        FillRect(
            mem,
            &RECT {
                left: 0,
                top: 0,
                right: SET_W,
                bottom: SET_H,
            },
            bg,
        );
        DeleteObject(bg);

        let hdr = CreateSolidBrush(CLR_WHITE);
        FillRect(
            mem,
            &RECT {
                left: 0,
                top: 0,
                right: SET_W,
                bottom: 64,
            },
            hdr,
        );
        DeleteObject(hdr);

        let sep_pen = CreatePen(PS_SOLID, 1, CLR_SEP);
        let old_pen = SelectObject(mem, sep_pen);
        MoveToEx(mem, 0, 64, std::ptr::null_mut());
        LineTo(mem, SET_W, 64);
        SelectObject(mem, old_pen);
        DeleteObject(sep_pen);

        draw_text(
            mem,
            "Settings",
            28,
            8,
            SET_W - 56,
            28,
            CLR_TXT,
            16,
            700,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        draw_text(
            mem,
            "Updates, window behavior and keyboard shortcuts.",
            28,
            36,
            SET_W - 56,
            18,
            CLR_TXT_DIM,
            11,
            400,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        let status = GetDlgItem(hwnd, LBL_UPDATE_STATUS as i32);
        SetWindowTextW(status, wide(&app.status).as_ptr());

        // Draw crisp perimeter border matching the asymmetric region shape
        draw_window_border(mem, SET_W, SET_H, 18);

        BitBlt(hdc, 0, 0, SET_W, SET_H, mem, 0, 0, SRCCOPY);
        DeleteDC(mem);
        DeleteObject(bmp);
        EndPaint(hwnd, &mut ps);
    }
}

unsafe fn bm_checked(hwnd: HWND, id: u32) -> bool {
    let view = GetDlgItem(hwnd, SET_VIEW_ID as i32);
    let mut c = if view.is_null() {
        std::ptr::null_mut()
    } else {
        GetDlgItem(view, id as i32)
    };
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
                    BTN_COS_BACK | BTN_CLOSE => {
                        close_child_panel(hwnd, app);
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
                        app.desktop_shortcut = bm_checked(hwnd, id);
                        if app.desktop_shortcut {
                            app.create_desktop_shortcut();
                        } else {
                            app.remove_desktop_shortcut();
                        }
                        app.save_settings();
                    }
                    BTN_AUTO_UPDATE => {
                        let previous = app.auto_update;
                        app.auto_update = bm_checked(hwnd, id);
                        if app.save_settings() {
                            app.updater.set_enabled(app.auto_update);
                            app.status = if app.auto_update {
                                "Automatic updates on. Your work is saved before restart."
                            } else {
                                "Automatic updates off. You can still check manually."
                            }
                            .into();
                        } else {
                            app.auto_update = previous;
                            SendMessageW(
                                GetDlgItem(GetDlgItem(hwnd, SET_VIEW_ID as i32), id as i32),
                                BM_SETCHECK,
                                usize::from(previous),
                                0,
                            );
                        }
                        InvalidateRect(hwnd, std::ptr::null(), 1);
                    }
                    BTN_SCAN => {
                        handle_command(app, BTN_SCAN);
                        InvalidateRect(hwnd, std::ptr::null(), 1);
                    }
                    BTN_HK_BLACK..=BTN_HK_TODO => {
                        let i = (id - BTN_HK_BLACK) as usize;
                        app.set_hotkey(i, bm_checked(hwnd, id));
                        save_cfg(app);
                        sync_cos_checks(hwnd, app);
                        InvalidateRect(hwnd, std::ptr::null(), 1);
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
                    SB_LINEUP => pos -= 30,
                    SB_LINEDOWN => pos += 30,
                    SB_PAGEUP => pos -= SET_VIEW_H - 30,
                    SB_PAGEDOWN => pos += SET_VIEW_H - 30,
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
                app.settings_scroll = (app.settings_scroll - delta as i32 / 120 * 60).clamp(0, max);
                apply_settings_scroll(hwnd, app);
                0
            }
            WM_KEYDOWN => {
                if wparam as u16 == VK_ESCAPE {
                    close_child_panel(hwnd, app);
                }
                0
            }
            WM_CLOSE => {
                close_child_panel(hwnd, app);
                0
            }
            WM_DESTROY => {
                app.settings_hwnd = std::ptr::null_mut();
                0
            }
            WM_ERASEBKGND => 1,
            WM_DRAWITEM => draw_button(lparam),
            WM_CTLCOLORSTATIC | WM_CTLCOLOREDIT | WM_CTLCOLORBTN => {
                control_colors(msg, wparam, lparam)
            }
            WM_PAINT => {
                paint_settings(hwnd, app);
                0
            }
            WM_LBUTTONUP => {
                let x = (lparam as u32 & 0xFFFF) as i32;
                let y = ((lparam as u32 >> 16) & 0xFFFF) as i32;
                if in_close_btn(x, y, SET_W) {
                    close_child_panel(hwnd, app);
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
