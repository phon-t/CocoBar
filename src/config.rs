use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) struct ConfigData {
    pub color: usize,
    pub size_idx: usize,
    pub size_px: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub always_on_top: bool,
    pub cosmetic_bell: Option<usize>,
    pub cosmetic_scarf: Option<usize>,
    pub cosmetic_tie: Option<usize>,
    pub hotkeys: u16,
    pub desktop_shortcut: bool,
    pub auto_update: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct UserData {
    pub note: String,
    pub todos: Vec<(String, bool)>,
    pub notes: Vec<super::notes::Note>,
}

pub(crate) fn app_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("COCOBAR_DATA_DIR") {
        return PathBuf::from(dir);
    }
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(appdata).join("cocoBar")
}

pub(crate) fn ensure_app_dir() {
    let dir = app_dir();
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

// Flush a sibling file before replacing the previous snapshot. A failed write
// leaves the last saved file intact; callers can report the error and retry.
fn atomic_write(path: &Path, content: &str) -> io::Result<()> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let temp = path.with_file_name(name);
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub(crate) fn load_config(path: &Path) -> ConfigData {
    let defaults = ConfigData {
        color: 0,
        size_idx: 1,
        size_px: -1,
        pos_x: -1,
        pos_y: -1,
        always_on_top: true,
        cosmetic_bell: None,
        cosmetic_scarf: None,
        cosmetic_tie: None,
        hotkeys: 0b1_1111_1111,
        desktop_shortcut: true,
        auto_update: false,
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return defaults,
    };

    let mut color = defaults.color;
    let mut size_idx = defaults.size_idx;
    let mut size_px = defaults.size_px;
    let mut pos_x = defaults.pos_x;
    let mut pos_y = defaults.pos_y;
    let mut always_on_top = defaults.always_on_top;
    let mut cosmetic_bell = defaults.cosmetic_bell;
    let mut cosmetic_scarf = defaults.cosmetic_scarf;
    let mut cosmetic_tie = defaults.cosmetic_tie;
    let mut hotkeys = defaults.hotkeys;
    let mut desktop_shortcut = defaults.desktop_shortcut;
    let mut auto_update = defaults.auto_update;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(val) = line.strip_prefix("{color: ") {
            if let Some(v) = val.strip_suffix('}') {
                color = v.parse().unwrap_or(defaults.color);
            }
        } else if let Some(val) = line.strip_prefix("{size_idx: ") {
            if let Some(v) = val.strip_suffix('}') {
                size_idx = v.parse().unwrap_or(defaults.size_idx);
            }
        } else if let Some(val) = line.strip_prefix("{size_px: ") {
            if let Some(v) = val.strip_suffix('}') {
                size_px = v.parse().unwrap_or(defaults.size_px);
            }
        } else if let Some(val) = line.strip_prefix("{pos_x: ") {
            if let Some(v) = val.strip_suffix('}') {
                pos_x = v.parse().unwrap_or(defaults.pos_x);
            }
        } else if let Some(val) = line.strip_prefix("{pos_y: ") {
            if let Some(v) = val.strip_suffix('}') {
                pos_y = v.parse().unwrap_or(defaults.pos_y);
            }
        } else if let Some(val) = line.strip_prefix("{always_on_top: ") {
            if let Some(v) = val.strip_suffix('}') {
                let n: i32 = v.parse().unwrap_or(1);
                always_on_top = n != 0;
            }
        } else if let Some(val) = line.strip_prefix("{cosmetic_bell: ") {
            if let Some(v) = val.strip_suffix('}') {
                let n: i32 = v.parse().unwrap_or(-1);
                cosmetic_bell = if n < 0 { None } else { Some(n as usize) };
            }
        } else if let Some(val) = line.strip_prefix("{cosmetic_scarf: ") {
            if let Some(v) = val.strip_suffix('}') {
                let n: i32 = v.parse().unwrap_or(-1);
                cosmetic_scarf = if n < 0 { None } else { Some(n as usize) };
            }
        } else if let Some(val) = line.strip_prefix("{cosmetic_tie: ") {
            if let Some(v) = val.strip_suffix('}') {
                let n: i32 = v.parse().unwrap_or(-1);
                cosmetic_tie = if n < 0 { None } else { Some(n as usize) };
            }
        } else if let Some(val) = line.strip_prefix("{desktop_shortcut: ") {
            if let Some(v) = val.strip_suffix('}') {
                desktop_shortcut = v != "0";
            }
        } else if let Some(val) = line.strip_prefix("{auto_update: ") {
            if let Some(v) = val.strip_suffix('}') {
                auto_update = v == "1";
            }
        } else if let Some(val) = line.strip_prefix("{hotkeys: ") {
            if let Some(v) = val.strip_suffix('}') {
                let n: i32 = v.parse().unwrap_or(defaults.hotkeys as i32);
                hotkeys = (hotkeys & !0x7f) | n.clamp(0, 127) as u16;
            }
        } else if let Some(val) = line.strip_prefix("{notes_hotkey: ") {
            if let Some(v) = val.strip_suffix('}') {
                hotkeys = (hotkeys & !(1 << 7)) | (u16::from(v != "0") << 7);
            }
        } else if let Some(val) = line.strip_prefix("{todo_hotkey: ") {
            if let Some(v) = val.strip_suffix('}') {
                hotkeys = (hotkeys & !(1 << 8)) | (u16::from(v != "0") << 8);
            }
        }
    }

    ConfigData {
        color: color.min(2),
        size_idx: size_idx.min(2),
        size_px,
        pos_x,
        pos_y,
        always_on_top,
        cosmetic_bell: cosmetic_bell.filter(|&i| i < 2),
        cosmetic_scarf: cosmetic_scarf.filter(|&i| i < 5),
        cosmetic_tie: if cosmetic_bell.is_some_and(|i| i < 2) {
            None
        } else {
            cosmetic_tie.filter(|&i| i < 3)
        },
        hotkeys,
        desktop_shortcut,
        auto_update,
    }
}

pub(crate) fn save_config(path: &Path, data: &ConfigData) -> io::Result<()> {
    let bell = match data.cosmetic_bell {
        Some(i) => i.to_string(),
        None => "-1".to_string(),
    };
    let scarf = match data.cosmetic_scarf {
        Some(i) => i.to_string(),
        None => "-1".to_string(),
    };
    let tie = match data.cosmetic_tie {
        Some(i) => i.to_string(),
        None => "-1".to_string(),
    };
    let aot = if data.always_on_top { "1" } else { "0" };

    let content = format!(
        "{{color: {}}}\n\
         {{size_idx: {}}}\n\
         {{size_px: {}}}\n\
         {{pos_x: {}}}\n\
         {{pos_y: {}}}\n\
         {{always_on_top: {}}}\n\
         {{cosmetic_bell: {}}}\n\
         {{cosmetic_scarf: {}}}\n\
         {{cosmetic_tie: {}}}\n\
         {{hotkeys: {}}}\n\
         {{notes_hotkey: {}}}\n\
         {{todo_hotkey: {}}}\n\
         {{desktop_shortcut: {}}}\n\
         {{auto_update: {}}}\n",
        data.color,
        data.size_idx,
        data.size_px,
        data.pos_x,
        data.pos_y,
        aot,
        bell,
        scarf,
        tie,
        data.hotkeys & 0x7f,
        u8::from(data.hotkeys & (1 << 7) != 0),
        u8::from(data.hotkeys & (1 << 8) != 0),
        u8::from(data.desktop_shortcut),
        u8::from(data.auto_update),
    );

    atomic_write(path, &content)
}

pub(crate) fn load_user_data(path: &Path) -> io::Result<UserData> {
    let defaults = UserData::default();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(defaults),
        Err(e) => return Err(e),
    };

    let mut note = String::new();
    let mut todos: Vec<(String, bool)> = Vec::new();
    let mut notes = Vec::new();
    let mut collection_format = false;

    for line in content.lines() {
        if line == "V\t2" {
            collection_format = true;
        } else if let Some(raw) = line.strip_prefix("S\t") {
            let mut fields = raw.splitn(4, '\t');
            let invalid =
                || io::Error::new(io::ErrorKind::InvalidData, "Invalid saved note record");
            let id: u64 = fields
                .next()
                .ok_or_else(invalid)?
                .parse()
                .map_err(|_| invalid())?;
            let created_at = fields
                .next()
                .ok_or_else(invalid)?
                .parse()
                .map_err(|_| invalid())?;
            let updated_at = fields
                .next()
                .ok_or_else(invalid)?
                .parse()
                .map_err(|_| invalid())?;
            let text = unesc(fields.next().ok_or_else(invalid)?);
            if id == 0 || notes.iter().any(|note: &super::notes::Note| note.id == id) {
                return Err(invalid());
            }
            notes.push(super::notes::Note {
                id,
                text,
                created_at,
                updated_at,
            });
        } else if line.starts_with("N\t") {
            let raw = &line[2..];
            note = unesc(raw);
        } else if line.starts_with("T\t") {
            let rest = &line[2..];
            let mut parts = rest.splitn(2, '\t');
            let done_str = parts.next().unwrap_or("0");
            let text = parts.next().unwrap_or("");
            let done = done_str == "1";
            todos.push((unesc(text), done));
        }
    }

    if !collection_format && notes.is_empty() && !note.is_empty() {
        // Older versions did not record a note timestamp. Preserve the text
        // without claiming a creation/saved date that was never stored.
        notes.push(super::notes::Note {
            id: 1,
            text: note.clone(),
            created_at: 0,
            updated_at: 0,
        });
    }
    notes.sort_by_key(|note| std::cmp::Reverse((note.updated_at, note.id)));
    Ok(UserData { note, todos, notes })
}

pub(crate) fn save_user_data(path: &Path, data: &UserData) -> io::Result<()> {
    let mut lines: Vec<String> = Vec::new();
    if !data.note.is_empty() || !data.todos.is_empty() || !data.notes.is_empty() {
        lines.push("V\t2".into());
    }

    if !data.note.is_empty() {
        lines.push(format!("N\t{}", esc(&data.note)));
    }

    for (text, done) in &data.todos {
        let d = if *done { "1" } else { "0" };
        lines.push(format!("T\t{}\t{}", d, esc(text)));
    }
    for note in &data.notes {
        lines.push(format!(
            "S\t{}\t{}\t{}\t{}",
            note.id,
            note.created_at,
            note.updated_at,
            esc(&note.text)
        ));
    }

    let content = lines.join("\n");
    atomic_write(path, &content)
}

pub(crate) fn migrate_legacy(appdata: &Path) {
    let new_config = appdata.join("cocoBar").join("config.txt");
    let new_data = appdata.join("cocoBar").join("mydata.txt");
    let old_config = appdata.join("CatCompanion").join("config.txt");
    let old_data = appdata.join("CatCompanion").join("mydata.txt");

    if old_config.exists() && !new_config.exists() {
        ensure_app_dir();
        let _ = fs::copy(&old_config, &new_config);
    }
    if old_data.exists() && !new_data.exists() {
        let _ = fs::create_dir_all(new_data.parent().unwrap());
        let _ = fs::copy(&old_data, &new_data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "cocobar-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn escaping_preserves_literal_sequences_and_windows_newlines() {
        for text in [
            "",
            "first\r\nsecond\nthird",
            r"C:\notes\new\readme",
            r"literal \n and \\n",
            "tabs\t猫 🐈 café",
            "trailing\\",
        ] {
            assert_eq!(unesc(&esc(text)), text);
        }
    }

    #[test]
    fn notes_and_tasks_round_trip_and_replace_previous_save() {
        let dir = test_dir();
        let path = dir.join("nested/mydata.txt");
        let data = UserData {
            note: "猫 🐈\r\nC:\\notes\\new\nlast\\".repeat(8000),
            todos: vec![("one\ttwo\nthree".into(), false), ("Done ✓".into(), true)],
            notes: vec![],
        };
        save_user_data(&path, &data).unwrap();
        assert_eq!(load_user_data(&path).unwrap(), data);
        save_user_data(&path, &UserData::default()).unwrap();
        assert_eq!(load_user_data(&path).unwrap(), UserData::default());
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_save_keeps_existing_destination_and_cleans_temp_file() {
        let dir = test_dir();
        let destination = dir.join("mydata.txt");
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("keep"), "previous data").unwrap();
        assert!(save_user_data(&destination, &UserData::default()).is_err());
        assert_eq!(
            fs::read_to_string(destination.join("keep")).unwrap(),
            "previous data"
        );
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        assert!(load_user_data(&destination).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn legacy_data_still_loads() {
        let dir = test_dir();
        let path = dir.join("mydata.txt");
        fs::write(&path, "N\tfirst\\nsecond\nT\t1\told task").unwrap();
        assert_eq!(
            load_user_data(&path).unwrap(),
            UserData {
                note: "first\nsecond".into(),
                todos: vec![("old task".into(), true)],
                notes: vec![super::super::notes::Note {
                    id: 1,
                    text: "first\nsecond".into(),
                    created_at: 0,
                    updated_at: 0
                }],
            }
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn config_validates_selections_and_remembers_disabled_shortcut() {
        let dir = test_dir();
        let path = dir.join("config.txt");
        fs::write(&path, "{color: 99}\n{size_idx: 99}\n{cosmetic_bell: 99}\n{cosmetic_scarf: 99}\n{cosmetic_tie: 2}\n{desktop_shortcut: 0}\n").unwrap();
        let cfg = load_config(&path);
        assert_eq!(
            (
                cfg.color,
                cfg.size_idx,
                cfg.cosmetic_bell,
                cfg.cosmetic_scarf,
                cfg.cosmetic_tie
            ),
            (2, 2, None, None, Some(2))
        );
        assert!(!cfg.desktop_shortcut);
        save_config(&path, &cfg).unwrap();
        assert!(!load_config(&path).desktop_shortcut);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn direct_note_and_todo_shortcuts_default_on_and_remember_individual_toggles() {
        let dir = test_dir();
        let path = dir.join("config.txt");
        fs::write(&path, "{hotkeys: 5}\n").unwrap();
        let mut cfg = load_config(&path);
        assert_eq!(cfg.hotkeys, 5 | (1 << 7) | (1 << 8));
        cfg.hotkeys &= !(1 << 7);
        save_config(&path, &cfg).unwrap();
        assert_eq!(load_config(&path).hotkeys, 5 | (1 << 8));
        cfg.hotkeys = 0;
        save_config(&path, &cfg).unwrap();
        assert_eq!(load_config(&path).hotkeys, 0);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn automatic_updates_default_off_and_only_enable_with_explicit_opt_in() {
        let dir = test_dir();
        let path = dir.join("config.txt");
        assert!(!load_config(&path).auto_update);
        for old in [
            "{color: 1}\n",
            "{auto_update: bad}\n",
            "{auto_update: 2}\n",
            "{auto_update: 0}\n",
        ] {
            fs::write(&path, old).unwrap();
            assert!(!load_config(&path).auto_update);
        }
        let mut cfg = load_config(&path);
        cfg.auto_update = true;
        save_config(&path, &cfg).unwrap();
        assert!(load_config(&path).auto_update);
        cfg.auto_update = false;
        save_config(&path, &cfg).unwrap();
        assert!(!load_config(&path).auto_update);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn legacy_notes_migrate_even_when_config_already_exists() {
        let dir = test_dir();
        fs::create_dir(dir.join("CatCompanion")).unwrap();
        fs::create_dir(dir.join("cocoBar")).unwrap();
        fs::write(dir.join("CatCompanion/mydata.txt"), "N\tlegacy notes").unwrap();
        fs::write(dir.join("cocoBar/config.txt"), "current config").unwrap();
        migrate_legacy(&dir);
        assert_eq!(
            load_user_data(&dir.join("cocoBar/mydata.txt"))
                .unwrap()
                .note,
            "legacy notes"
        );
        assert_eq!(
            fs::read_to_string(dir.join("cocoBar/config.txt")).unwrap(),
            "current config"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn read_only_file_save_fails_without_truncating_previous_notes() {
        let dir = test_dir();
        let path = dir.join("mydata.txt");
        fs::write(&path, "N\tprevious notes").unwrap();
        let original = fs::metadata(&path).unwrap().permissions();
        let mut protected = original.clone();
        protected.set_readonly(true);
        fs::set_permissions(&path, protected).unwrap();
        let result = save_user_data(
            &path,
            &UserData {
                note: "new notes".into(),
                todos: vec![],
                notes: vec![],
            },
        );
        let saved = fs::read_to_string(&path).unwrap();
        fs::set_permissions(&path, original).unwrap();
        assert!(result.is_err());
        assert_eq!(saved, "N\tprevious notes");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn card_collection_round_trips_timestamps_and_unicode_text() {
        let dir = test_dir();
        let path = dir.join("mydata.txt");
        let (notes, _) = super::super::notes::edited_notes(
            &[],
            None,
            "Work\r\n猫\tC:\\notes\\new",
            1_800_000_000,
        );
        let (notes, _) = super::super::notes::edited_notes(&notes, None, "Personal", 1_800_000_010);
        let data = UserData {
            note: notes[0].text.clone(),
            todos: vec![],
            notes,
        };
        save_user_data(&path, &data).unwrap();
        assert_eq!(load_user_data(&path).unwrap(), data);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn malformed_collection_is_reported_instead_of_silently_dropping_notes() {
        let dir = test_dir();
        let path = dir.join("mydata.txt");
        fs::write(&path, "V\t2\nS\t1\tbad date\t100\tnote").unwrap();
        assert!(load_user_data(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
