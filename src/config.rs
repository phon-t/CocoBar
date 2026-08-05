use std::fs;
use std::path::{Path, PathBuf};

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
}

pub(crate) struct UserData {
    pub note: String,
    pub todos: Vec<(String, bool)>,
}

pub(crate) fn app_dir() -> PathBuf {
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
    s.replace("\\", "\\\\").replace("\n", "\\n")
}

fn unesc(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\\\", "\\")
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
        }
    }

    ConfigData {
        color,
        size_idx,
        size_px,
        pos_x,
        pos_y,
        always_on_top,
        cosmetic_bell,
        cosmetic_scarf,
        cosmetic_tie,
    }
}

pub(crate) fn save_config(path: &Path, data: &ConfigData) {
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
         {{cosmetic_tie: {}}}\n",
        data.color,
        data.size_idx,
        data.size_px,
        data.pos_x,
        data.pos_y,
        aot,
        bell,
        scarf,
        tie,
    );

    let _ = fs::write(path, content);
}

pub(crate) fn load_user_data(path: &Path) -> UserData {
    let defaults = UserData {
        note: String::new(),
        todos: Vec::new(),
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return defaults,
    };

    let mut note = String::new();
    let mut todos: Vec<(String, bool)> = Vec::new();

    for line in content.lines() {
        if line.starts_with("N\t") {
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

    UserData { note, todos }
}

pub(crate) fn save_user_data(path: &Path, data: &UserData) {
    let mut lines: Vec<String> = Vec::new();

    if !data.note.is_empty() {
        lines.push(format!("N\t{}", esc(&data.note)));
    }

    for (text, done) in &data.todos {
        let d = if *done { "1" } else { "0" };
        lines.push(format!("T\t{}\t{}", d, esc(text)));
    }

    let content = lines.join("\n");
    let _ = fs::write(path, content);
}

pub(crate) fn migrate_legacy(appdata: &Path) {
    let new_config = appdata.join("cocoBar").join("config.txt");
    let new_data = appdata.join("cocoBar").join("mydata.txt");
    let old_config = appdata.join("CatCompanion").join("config.txt");
    let old_data = appdata.join("CatCompanion").join("mydata.txt");

    if old_config.exists() && !new_config.exists() {
        ensure_app_dir();
        let _ = fs::copy(&old_config, &new_config);
        if old_data.exists() {
            let _ = fs::copy(&old_data, &new_data);
        }
    }
}
