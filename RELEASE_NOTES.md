# cocoBar v0.7.0

Saved notes now appear as separate cards with previews and saved dates. Notes autosave reliably, keep drafts open after a failed save, and support editing, deletion, undo, and older-note pages. Existing notes and tasks are preserved.

- Added Ctrl+Alt+N for Notes and Ctrl+Alt+T for To Do, with individual shortcut settings.
- Unified checkboxes, aligned settings and action buttons, and kept the close button inside the top-right corner with a thin L-shaped gap.
- Fixed tab selection: the active tab is blue and the other is white.
- Page controls now appear only when needed and clearly say "Page 1 of 2". Task rows and navigation are aligned.
- Improved tail physics and prevented the tip from being clipped while dragging.
- Fixed animation transitions, drag/resize handling, and accessory rendering in both poses.
- Added optional automatic GitHub updates, off by default. Enabled checks run at startup and every six hours.
- Hardened update downloads and installation with size/checksum/PE checks, readiness before restart, atomic replacement, backups, cancellation, and rollback after a failed startup.
- Reduced repeated rendering allocations and kept native UI resources stable.

Download **cocobar.exe** from the release assets and run it directly. Windows 64-bit; no installer needed. Data remains in `%APPDATA%\cocoBar`.
