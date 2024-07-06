#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod config;

use chrono::{Datelike, Local};
use config::BackupWardenConfig;
use notify::{Config as NotifyConfig, Event, EventKind, PollWatcher, RecursiveMode, Watcher};
use std::fs;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

const CONFIG: &str = include_str!("../backup_warden_config.json");

fn main() {
    let config: BackupWardenConfig = serde_json::from_str(CONFIG).expect("Failed to load config");

    let (tx, rx) = channel();

    let mut watcher = PollWatcher::new(
        tx,
        NotifyConfig::default()
            .with_poll_interval(Duration::from_secs(3600))
            .with_compare_contents(true),
    )
    .expect("Failed to create PollWatcher");

    watcher
        .watch(Path::new(&config.watch_folder), RecursiveMode::Recursive)
        .expect("Failed to watch folder");

    // Check for existing backup folders and create initial backup if none exist
    if !backup_folders_exist(&config) {
        println!("No backup folders found, creating initial backup...");
        backup_folder(&config);
    }

    loop {
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(Ok(event)) => {
                handle_event(&event, &config);
            }
            Ok(Err(e)) => println!("Watch error: {:?}", e),
            Err(_) => (),
        }

        // Check if today is the last day of the month and create a monthly snapshot
        let today = Local::now().date_naive();
        if is_last_day_of_month(today) {
            create_monthly_snapshot(&config, today);
        }
    }
}

fn backup_folders_exist(config: &BackupWardenConfig) -> bool {
    for location in &config.backup_locations {
        let past_30_days_path = Path::new(location).join("Past 30 Days");
        if past_30_days_path.exists() && past_30_days_path.is_dir() {
            return true;
        }
    }
    false
}

fn handle_event(event: &Event, config: &BackupWardenConfig) {
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            backup_folder(config);
        }
        _ => (),
    }
}

fn backup_folder(config: &BackupWardenConfig) {
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let hour = now.format("%I %p").to_string(); // Format hour as "HH AM/PM"

    for location in &config.backup_locations {
        let daily_path = Path::new(location).join("Past 30 Days").join(&date);
        let backup_path = daily_path.join(format!("@{}", hour));
        fs::create_dir_all(&backup_path).expect("Failed to create backup directory");

        copy_dir_all(&config.watch_folder, &backup_path).expect("Failed to copy files");
    }

    cleanup_old_backups(config);
}

fn create_monthly_snapshot(config: &BackupWardenConfig, date: chrono::NaiveDate) {
    let date_str = date.format("%Y-%m-%d").to_string();

    for location in &config.backup_locations {
        let monthly_snapshots_path = Path::new(location)
            .join("Monthly Snapshots")
            .join(&date_str);
        fs::create_dir_all(&monthly_snapshots_path)
            .expect("Failed to create monthly snapshot directory");

        copy_dir_all(&config.watch_folder, &monthly_snapshots_path)
            .expect("Failed to copy files to monthly snapshot");
    }
}

fn copy_dir_all(src: &str, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path.to_string_lossy(), &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

fn cleanup_old_backups(config: &BackupWardenConfig) {
    for location in &config.backup_locations {
        let past_30_days_path = Path::new(location).join("Past 30 Days");
        let mut daily_folders: Vec<_> = fs::read_dir(&past_30_days_path)
            .expect("Failed to read backup directory")
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .collect();

        daily_folders.sort_by_key(|entry| entry.path());

        if daily_folders.len() > config.retention_days {
            let excess = daily_folders.len() - config.retention_days;
            for entry in &daily_folders[..excess] {
                fs::remove_dir_all(entry.path()).expect("Failed to remove old backup");
            }
        }
    }
}

fn is_last_day_of_month(date: chrono::NaiveDate) -> bool {
    let next_day = date + chrono::Duration::days(1);
    next_day.month() != date.month()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, NaiveDate};
    use std::fs::{self};
    use tempfile::tempdir;

    #[test]
    fn test_is_last_day_of_month() {
        assert!(is_last_day_of_month(
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()
        ));
        assert!(!is_last_day_of_month(
            NaiveDate::from_ymd_opt(2024, 1, 30).unwrap()
        ));
        assert!(is_last_day_of_month(
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        ));
    }

    #[test]
    fn test_backup_folder_creation() {
        let temp_dir = tempdir().unwrap();
        let watch_folder = temp_dir.path().join("watch_folder");
        let backup_location = temp_dir.path().join("backup_location");
        let past_30_days = backup_location.join("Past 30 Days");

        fs::create_dir_all(&watch_folder).unwrap();
        fs::create_dir_all(&backup_location).unwrap();

        let config = BackupWardenConfig {
            watch_folder: watch_folder.to_str().unwrap().to_string(),
            backup_locations: vec![backup_location.to_str().unwrap().to_string()],
            retention_days: 30,
        };

        backup_folder(&config);

        let date = Local::now().format("%Y-%m-%d").to_string();
        let daily_path = past_30_days.join(&date);
        let hour = Local::now().format("%I %p").to_string();
        let backup_path = daily_path.join(format!("@{}", hour));

        assert!(backup_path.exists());
    }

    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = tempdir().unwrap();
        let backup_location = temp_dir.path().join("backup_location");
        let past_30_days = backup_location.join("Past 30 Days");

        fs::create_dir_all(&past_30_days).unwrap();

        for i in 0..35 {
            if let Some(date) = NaiveDate::from_ymd_opt(2024, 1, i + 1) {
                let date_str = date.format("%Y-%m-%d").to_string();
                let daily_folder = past_30_days.join(&date_str);
                fs::create_dir_all(&daily_folder).unwrap();
            }
        }

        let config = BackupWardenConfig {
            watch_folder: "dummy".to_string(),
            backup_locations: vec![backup_location.to_str().unwrap().to_string()],
            retention_days: 30,
        };

        cleanup_old_backups(&config);

        let remaining_backups: Vec<_> = fs::read_dir(&past_30_days)
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert_eq!(remaining_backups.len(), 30);
    }

    #[test]
    fn test_create_monthly_snapshot() {
        let temp_dir = tempdir().unwrap();
        let watch_folder = temp_dir.path().join("watch_folder");
        let backup_location = temp_dir.path().join("backup_location");
        let monthly_snapshots = backup_location.join("Monthly Snapshots");

        fs::create_dir_all(&watch_folder).unwrap();
        fs::create_dir_all(&backup_location).unwrap();

        let config = BackupWardenConfig {
            watch_folder: watch_folder.to_str().unwrap().to_string(),
            backup_locations: vec![backup_location.to_str().unwrap().to_string()],
            retention_days: 30,
        };

        let last_day_of_month = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        create_monthly_snapshot(&config, last_day_of_month);

        let snapshot_folders: Vec<_> = fs::read_dir(&monthly_snapshots)
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert_eq!(snapshot_folders.len(), 1);
    }
}
