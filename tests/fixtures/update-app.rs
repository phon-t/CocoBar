fn main() {
    #[cfg(fail_startup)]
    std::process::exit(2);

    #[cfg(not(fail_startup))]
    {
        let name = if cfg!(new_version) { "started-new.txt" } else { "started-old.txt" };
        std::fs::write(name, std::process::id().to_string()).unwrap();
        for _ in 0..300 {
            if !cfg!(new_version) && std::path::Path::new("old-stop").exists() { return; }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
