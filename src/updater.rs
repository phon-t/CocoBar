use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const STARTUP_DELAY: Duration = Duration::from_secs(30);
const MAX_DOWNLOAD: u64 = 200 * 1024 * 1024;

#[derive(Debug)]
struct Release {
    tag: String,
    url: Option<String>,
    size: u64,
    sha256: Option<String>,
}

pub(crate) struct Stage {
    dir: PathBuf,
    version: String,
    helper_owns_files: bool,
}

impl Stage {
    fn new(version: String) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "cocobar-update-{}-{nonce}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).map_err(|e| format!("Could not create the update folder: {e}"))?;
        Ok(Self {
            dir,
            version,
            helper_owns_files: false,
        })
    }
    fn file(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Stage {
    fn drop(&mut self) {
        if !self.helper_owns_files {
            for name in [
                "cocobar.exe",
                "install.ps1",
                "ready",
                "go",
                "cancel",
                "error",
            ] {
                let _ = fs::remove_file(self.file(name));
            }
            let _ = fs::remove_dir(&self.dir);
        }
    }
}

pub(crate) struct PreparedInstall {
    stage: Stage,
    committed: bool,
}

impl PreparedInstall {
    pub(crate) fn commit(&mut self) -> Result<(), String> {
        fs::write(self.stage.file("go"), "install")
            .map_err(|e| format!("Could not authorize the update: {e}"))?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for PreparedInstall {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::write(self.stage.file("cancel"), "cancel");
        }
    }
}

pub(crate) enum Event {
    Progress(String),
    Message(String),
    Downloaded(Stage),
    Ready(PreparedInstall),
}

struct Pending {
    automatic: bool,
    cancel: Arc<AtomicBool>,
    rx: mpsc::Receiver<Result<Event, String>>,
}

pub(crate) struct Updater {
    pending: Option<Pending>,
    next_check: Instant,
}

impl Updater {
    pub(crate) fn new() -> Self {
        Self {
            pending: None,
            next_check: Instant::now() + STARTUP_DELAY,
        }
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.next_check = Instant::now() + STARTUP_DELAY;
        if !enabled && self.pending.as_ref().is_some_and(|p| p.automatic) {
            if let Some(p) = self.pending.take() {
                p.cancel.store(true, Ordering::Relaxed);
            }
        }
    }

    pub(crate) fn automatic_due(&self, enabled: bool, now: Instant) -> bool {
        enabled && self.pending.is_none() && now >= self.next_check
    }

    pub(crate) fn start(&mut self, automatic: bool) -> bool {
        if let Some(pending) = self.pending.as_mut() {
            if !automatic {
                pending.automatic = false;
            }
            return false;
        }
        self.next_check = Instant::now() + CHECK_INTERVAL;
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        self.pending = Some(Pending {
            automatic,
            cancel: cancel.clone(),
            rx,
        });
        std::thread::spawn(move || {
            let result = check_and_download(&tx, &cancel);
            let _ = tx.send(result);
        });
        true
    }

    pub(crate) fn prepare(
        &mut self,
        stage: Stage,
        automatic: bool,
        exe: PathBuf,
        result_path: PathBuf,
    ) {
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        self.pending = Some(Pending {
            automatic,
            cancel: cancel.clone(),
            rx,
        });
        std::thread::spawn(move || {
            let result = prepare_install(stage, &exe, &result_path, std::process::id(), &cancel)
                .map(Event::Ready);
            let _ = tx.send(result);
        });
    }

    pub(crate) fn poll(&mut self) -> Option<(bool, Result<Event, String>)> {
        let pending = self.pending.as_ref()?;
        let automatic = pending.automatic;
        let event = match pending.rx.try_recv() {
            Ok(event) => event,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Update worker stopped. Please try again.".into())
            }
            Err(mpsc::TryRecvError::Empty) => return None,
        };
        if !matches!(&event, Ok(Event::Progress(_))) {
            self.pending = None;
        }
        Some((automatic, event))
    }
}

fn version(s: &str) -> Result<(u64, u64, u64), String> {
    let v = s.strip_prefix('v').unwrap_or(s);
    let parts: Vec<_> = v.split('.').collect();
    if parts.len() != 3
        || parts.iter().any(|p| {
            p.is_empty()
                || !p.bytes().all(|b| b.is_ascii_digit())
                || p.len() > 1 && p.starts_with('0')
        })
    {
        return Err("GitHub returned an invalid stable release version.".into());
    }
    let parse = |p: &str| {
        p.parse::<u64>()
            .map_err(|_| "The release version is too large.".to_string())
    };
    Ok((parse(parts[0])?, parse(parts[1])?, parse(parts[2])?))
}

fn parse_release(line: &str) -> Result<Release, String> {
    let fields: Vec<_> = line.trim().split('|').collect();
    if fields.len() != 4 {
        return Err("Invalid release response from GitHub.".into());
    }
    version(fields[0])?;
    if fields[1].is_empty() {
        if !fields[2].is_empty() || !fields[3].is_empty() {
            return Err("Invalid release asset metadata.".into());
        }
        return Ok(Release {
            tag: fields[0].into(),
            url: None,
            size: 0,
            sha256: None,
        });
    }
    let expected_url = format!(
        "https://github.com/phon-t/CocoBar/releases/download/{}/cocobar.exe",
        fields[0]
    );
    if fields[1] != expected_url {
        return Err("The release download address is invalid.".into());
    }
    let size: u64 = fields[2]
        .parse()
        .map_err(|_| "Invalid release download size.")?;
    if !(1024..=MAX_DOWNLOAD).contains(&size) {
        return Err("The release download size is outside the supported range.".into());
    }
    let sha256 = if fields[3].is_empty() {
        None
    } else {
        let digest = fields[3]
            .strip_prefix("sha256:")
            .ok_or("Unsupported release checksum.")?;
        if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Invalid release checksum.".into());
        }
        Some(digest.to_ascii_lowercase())
    };
    Ok(Release {
        tag: fields[0].into(),
        url: Some(fields[1].into()),
        size,
        sha256,
    })
}

fn run(script: &str) -> Result<String, String> {
    let wrapped = format!("$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; [Console]::OutputEncoding = [Text.UTF8Encoding]::new(); [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; try {{ {script} }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}");
    let out = Command::new("powershell.exe")
        .creation_flags(super::CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped,
        ])
        .output()
        .map_err(|e| format!("Could not start the updater: {e}"))?;
    if !out.status.success() {
        let error = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!(
            "Update failed: {}",
            if error.is_empty() {
                "PowerShell could not complete the request."
            } else {
                &error
            }
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn validate_executable(path: &Path, expected_size: u64) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Could not open the update: {e}"))?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();
    if size != expected_size {
        return Err("The update download is incomplete (size mismatch).".into());
    }
    let invalid = || "The download is not a valid 64-bit Windows executable.".to_string();
    let mut dos = [0; 64];
    file.read_exact(&mut dos).map_err(|_| invalid())?;
    if dos[..2] != *b"MZ" {
        return Err(invalid());
    }
    let pe_offset = u32::from_le_bytes(dos[60..64].try_into().unwrap()) as u64;
    if pe_offset < 64 || pe_offset.saturating_add(26) > size {
        return Err(invalid());
    }
    file.seek(SeekFrom::Start(pe_offset))
        .map_err(|_| invalid())?;
    let mut pe = [0; 26];
    file.read_exact(&mut pe).map_err(|_| invalid())?;
    if pe[..4] != *b"PE\0\0"
        || u16::from_le_bytes([pe[4], pe[5]]) != 0x8664
        || u16::from_le_bytes([pe[24], pe[25]]) != 0x20b
        || pe[22] & 2 == 0
        || pe[23] & 0x20 != 0
    {
        return Err(invalid());
    }
    Ok(())
}

fn check_and_download(
    tx: &mpsc::Sender<Result<Event, String>>,
    cancel: &AtomicBool,
) -> Result<Event, String> {
    let release = parse_release(&run(include_str!("check-release.ps1"))?)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Automatic update canceled.".into());
    }
    if version(&release.tag)? <= version(super::APP_VERSION)? {
        return Ok(Event::Message(format!(
            "Up to date (v{}).",
            super::APP_VERSION
        )));
    }
    let Some(url) = &release.url else {
        return Err(format!("{} has no cocobar.exe release asset.", release.tag));
    };
    tx.send(Ok(Event::Progress(format!(
        "Downloading {}… You can keep using cocoBar.",
        release.tag
    ))))
    .map_err(|_| "Update canceled.")?;
    let stage = Stage::new(release.tag)?;
    let staged = stage.file("cocobar.exe");
    let hash = run(&format!("Invoke-WebRequest -Uri {} -OutFile {} -UseBasicParsing -TimeoutSec 120; (Get-FileHash -LiteralPath {} -Algorithm SHA256).Hash", super::ps_quote(url), super::ps_quote(&staged.to_string_lossy()), super::ps_quote(&staged.to_string_lossy())))?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Automatic update canceled.".into());
    }
    validate_executable(&staged, release.size)?;
    if release
        .sha256
        .is_some_and(|expected| !hash.eq_ignore_ascii_case(&expected))
    {
        return Err("The update checksum does not match GitHub. Please retry.".into());
    }
    Ok(Event::Downloaded(stage))
}

fn prepare_install(
    mut stage: Stage,
    exe: &Path,
    result_path: &Path,
    old_pid: u32,
    cancel: &AtomicBool,
) -> Result<PreparedInstall, String> {
    if !exe.is_absolute() {
        return Err("The application path must be absolute.".into());
    }
    fs::metadata(exe).map_err(|e| format!("Could not locate the application: {e}"))?;
    let exe = exe.to_path_buf();
    let mut candidate_name = exe.as_os_str().to_os_string();
    candidate_name.push(format!(
        ".{}.new",
        stage.dir.file_name().unwrap().to_string_lossy()
    ));
    let candidate = PathBuf::from(candidate_name);
    let mut backup = exe.as_os_str().to_os_string();
    backup.push(".update-backup");
    let mut script = String::new();
    for (name, path) in [
        ("exe", exe.as_path()),
        ("staged", stage.file("cocobar.exe").as_path()),
        ("candidate", candidate.as_path()),
        ("backup", Path::new(&backup)),
        ("stageDir", stage.dir.as_path()),
        ("helperPath", stage.file("install.ps1").as_path()),
        ("ready", stage.file("ready").as_path()),
        ("go", stage.file("go").as_path()),
        ("cancel", stage.file("cancel").as_path()),
        ("errorPath", stage.file("error").as_path()),
        ("resultPath", result_path),
    ] {
        script.push_str(&format!(
            "${name} = {}\n",
            super::ps_quote(&path.to_string_lossy())
        ));
    }
    script.push_str(&format!(
        "$oldPid = {old_pid}\n$version = {}\n",
        super::ps_quote(&stage.version)
    ));
    script.push_str(include_str!("install-update.ps1"));
    // Windows PowerShell 5 needs a BOM for paths containing Unicode.
    fs::write(stage.file("install.ps1"), format!("\u{feff}{script}"))
        .map_err(|e| format!("Could not prepare the update helper: {e}"))?;
    let mut child = Command::new("powershell.exe")
        .creation_flags(super::CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(stage.file("install.ps1"))
        .spawn()
        .map_err(|e| format!("Could not start the update helper: {e}"))?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if cancel.load(Ordering::Relaxed) || Instant::now() > deadline {
            let _ = fs::write(stage.file("cancel"), "cancel");
            stage.helper_owns_files = true;
            return Err(
                "Update preparation canceled or timed out; cocoBar is still running.".into(),
            );
        }
        if stage.file("ready").exists() {
            stage.helper_owns_files = true;
            return Ok(PreparedInstall {
                stage,
                committed: false,
            });
        }
        if let Some(_status) = child.try_wait().map_err(|e| e.to_string())? {
            return Err(fs::read_to_string(stage.file("error"))
                .unwrap_or_else(|_| "The update helper stopped before it was ready.".into()));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_versions_compare_numerically_and_reject_bad_tags() {
        assert!(version("v1.10.0").unwrap() > version("1.9.99").unwrap());
        for bad in [
            "vv1.0.0",
            "1.2",
            "1.2.3.4",
            "1.2.3-beta",
            "1.+2.3",
            "01.2.3",
            "1.2.18446744073709551616",
        ] {
            assert!(version(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn release_metadata_requires_the_exact_asset_and_valid_size_and_checksum() {
        let line = format!("v1.0.0|https://github.com/phon-t/CocoBar/releases/download/v1.0.0/cocobar.exe|2048|sha256:{}", "a".repeat(64));
        let release = parse_release(&line).unwrap();
        assert_eq!(release.size, 2048);
        for bad in [
            line.replace("github.com/", "github.com.evil/"),
            line.replace("v1.0.0/cocobar.exe", "v0.9.0/cocobar.exe"),
            line.replace("2048", "0"),
            line.replace("sha256:", "sha1:"),
            line.replace(&"a".repeat(64), "bad"),
        ] {
            assert!(parse_release(&bad).is_err());
        }
        assert!(parse_release("v1.0.0|||").unwrap().url.is_none());
    }

    #[test]
    fn automatic_checks_require_opt_in_and_respect_the_schedule() {
        let mut updater = Updater::new();
        let later = Instant::now() + STARTUP_DELAY + Duration::from_secs(1);
        assert!(!updater.automatic_due(false, later));
        assert!(!updater.automatic_due(true, Instant::now()));
        assert!(updater.automatic_due(true, later));
        updater.next_check = later + CHECK_INTERVAL;
        assert!(!updater.automatic_due(true, later));
        assert!(updater.automatic_due(true, later + CHECK_INTERVAL));
    }

    #[test]
    fn disabling_auto_updates_cancels_auto_workers_but_preserves_manual_requests() {
        for automatic in [true, false] {
            let mut updater = Updater::new();
            let (_tx, rx) = mpsc::channel();
            let cancel = Arc::new(AtomicBool::new(false));
            updater.pending = Some(Pending {
                automatic,
                cancel: cancel.clone(),
                rx,
            });
            updater.set_enabled(false);
            assert_eq!(cancel.load(Ordering::Relaxed), automatic);
            assert_eq!(updater.pending.is_none(), automatic);
        }
    }

    #[test]
    fn executable_validation_rejects_truncated_wrong_architecture_and_dll_files() {
        let stage = Stage::new("test".into()).unwrap();
        let path = stage.file("cocobar.exe");
        let mut bytes = vec![0; 1024];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[60..64].copy_from_slice(&64u32.to_le_bytes());
        bytes[64..68].copy_from_slice(b"PE\0\0");
        bytes[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        bytes[86] = 2;
        bytes[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        fs::write(&path, &bytes).unwrap();
        assert!(validate_executable(&path, 1024).is_ok());
        assert!(validate_executable(&path, 2048).is_err());
        bytes[87] = 0x20;
        fs::write(&path, &bytes).unwrap();
        assert!(validate_executable(&path, 1024).is_err());
        bytes[87] = 0;
        bytes[68] = 0x4c;
        fs::write(&path, &bytes).unwrap();
        assert!(validate_executable(&path, 1024).is_err());
        fs::write(&path, b"MZ invalid download").unwrap();
        assert!(validate_executable(&path, 19).is_err());
    }
}
