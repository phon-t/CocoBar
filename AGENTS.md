# Cat Companion — Desktop Pet & Utility App

A Windows desktop companion cat built in Rust (raw Win32, no rendering framework). The cat sits on
the desktop as a transparent always-on-top layered window, its pupils follow the cursor, it blinks
naturally, and it doubles as a tray/hotkey app.

## Build & Run

- Toolchain: rustup stable-x86_64-pc-windows-gnu (1.97.1) + MSYS2 ucrt64 GCC at `C:\msys64`.
- Build: `$env:PATH = "C:\msys64\ucrt64\bin;$env:PATH;$env:USERPROFILE\.cargo\bin"; cargo build --release` (run in this folder).
- Run: `E:\huh\cat-companion\target\release\cat-companion.exe`
- IMPORTANT: stop the running exe before rebuilding (file lock → `Access is denied (os error 5)`):
  `Get-Process -Name cat-companion | Stop-Process -Force`
- Verify window: `enumwin.ps1` scripts in `C:\Users\hp\AppData\Local\Temp\opencode\` list HWNDs,
  rects and PIDs; screenshots taken via `System.Drawing` `CopyFromScreen` scripts in the same folder.

## Architecture (src/main.rs, ~815 lines, single file)

- Per-frame compositing loop: `SetTimer` 16 ms (60 FPS) → `WM_TIMER` → rebuild 32-bit DIB
  (premultiplied RGBA) → `UpdateLayeredWindow` with `BLENDFUNCTION(AC_SRC_OVER, 0, 255, AC_SRC_ALPHA)`.
- Window: `WS_POPUP | WS_VISIBLE`, ex-styles `WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE |
  WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT` (TRANSPARENT removed in move mode).
- HWND stored in `GWLP_USERDATA`; `GetMessageW` loop; window class "CatCompanionWnd",
  title "Cat Companion", `SetProcessDPIAware` at startup.
- Layers: base (cat with eye sockets), closed (cat with closed eyes), eyes (pupils), hl (shine) —
  all embedded via `include_bytes!` from `assets/` (2000×2000 source PNGs, decoded with `image`
  crate png feature). Layer buffers are `Vec<u8>` RGBA, scaled per-frame with
  `imageops::resize`/`crop_imm`+`resize` when window size changes.
- Composite per frame: base scaled, then eyes + highlight drawn at offset `(off_x, off_y)` from
  socket center (scaled), blink crossfade between open/closed using per-pixel alpha blend
  (`t` = fade progress), final premultiply + alpha = 255.
- Eye tracking: cursor pos → normalized offset relative to window center, clamped ±1, ×
  `MAX_OFF_X/MAX_OFF_Y` × scale; smoothed by lerp (0.25 fast, 0.06 slow). Wander: if cursor still
  >1.5 s, offsets drift ±14/±10 px scaled with smooth sine.
- Blink: state machine — `blink_next` random 3.0–7.5 s (via `Instant::now().subsec_nanos() % 4500`
  + 3000 ms), blink total 0.27 s: fade 0.11 s, hold closed 0.05 s, fade back 0.11 s.
- Move mode (Ctrl+Alt+D or tray): window becomes click-through=false, drag via `WM_LBUTTONDOWN` →
  `WM_NCLBUTTONDOWN` with `HTCAPTION`.
- Tray icon: `NOTIFYICONDATAW` NIM_ADD + NIM_SETVERSION, `WM_APP+2` callback → `TrackPopupMenu`
  (Black Cat / White Cat / sizes / Move / Exit).
- Hotkeys (RegisterHotKey, Ctrl+Alt): B=black, W=white, 1=Small, 2=Medium, 3=Large, D=move mode.
- Config: `%APPDATA%\CatCompanion\config.txt` = `color size_idx pos_x pos_y`; saved on
  WM_DESTROY; loaded at startup (fallback: bottom-right `(sw-w-24, sh-h-48)`, Medium).

## Key constants (main.rs:33–68)

- `SIZES = [320, 520, 760]` (width); Large on a 768px screen clamps to fit (see below).
- Content bbox in 2000px source: X 364–1556 (`CONTENT_X/W`), Y 148–1896 (`CONTENT_Y/H`).
- Socket centers (source px): left (730,730), right (1270,722). Pupils in `eyes.png`:
  left X590–892/Y510–956, right X1134–1448/Y490–944. Highlights at pupil centers.
- `MAX_OFF_X = 77.0`, `MAX_OFF_Y = 87.0` (pupil travel in source px, scaled).
- Menu IDs 101–107, hotkey IDs 1–6, `TRAY_CB = WM_APP+2`, `TIMER_ID = 2`.

## Recent changes (last session)

- `set_size` now clamps to screen: `max_w = sw-40`, `max_h = sh-60`; if height overflows,
  width is recomputed proportionally (main.rs:327). Verified: on 1366×768 screen Large = 483×708.
- Verified working: rendering (body 48,48,48 / 239,239,239; cheeks 255,197,227; eyes follow
  cursor; blink cycles every 3–7.5 s), white/black switch, size hotkeys, move mode, config save.

## Roadmap (user-requested, build in NEW session)

The user wants the cat to become a real utility tool. Add a **double-click menu** on the cat
(currently double-click does nothing; `WM_LBUTTONDOWN` is used for move-mode dragging):

1. Double-click (click count 2 → e.g. `WM_LBUTTONDBLCLK`, needs `CS_DBLCLKS` in WNDCLASSEX) opens
   a menu with:
   - **Resize**: current 3 presets are too coarse ("Large" fills the whole small screen) — add
     finer control, e.g. a slider (custom window or TrackPopupMenu with granular sub-items),
     maybe mouse-wheel resize over the cat.
   - **Black/White cat** switch (already exists via hotkey/tray — mirror into the menu).
2. **Reminders** (e.g. set a message + time, tray balloon notification when due).
3. **Quick notes** (open small editor window, persist to `%APPDATA%\CatCompanion\notes.txt`).
4. **To-do list** (add/check/remove items, persist to `%APPDATA%\CatCompanion\todos.txt`).
   Menu-driven, with small helper windows (WM_COMMAND-based dialogs, no winit/egui dependencies —
   keep the raw Win32 single-exe approach, add `windows-sys` dialogs or simple custom windows).

## Testing helpers (C:\Users\hp\AppData\Local\Temp\opencode\)

- `screenshot.ps1 -Path <out> -Name <name>`: full-screen PNG capture.
- `enumwin.ps1` / `enumwin2.ps1`: enumerate top-level windows (title/class/PID/rect).
- `sendkey.ps1 -Key "57"`: sends Ctrl+Alt+Key (hex vk) via keybd_event (simulates hotkeys).
- `burst.ps1 -Frames N -DelayMs D`: rapid screenshot burst (used to catch blinks).
- `catcolor2.ps1`, `eyes.ps1`: pixel analysis of captured screenshots.
