# Backup Warden

Backup Warden is a small "no-frills," set-and-forget, disposable, redundancy file-watcher written in Rust. 

** Started as a learning project for Rust years ago and is probably not practical in most cases where better solutions exist **

Each build of Backup Warden is a standalone watcher program with its embedded configuration, a "Warden." It monitors a specified folder for changes and, if so, creates backups every hour. It keeps a configurable rolling history (e.g. past 30 Days) and stores monthly snapshots. 
Best used on temporary projects or can easily be included in other applications or environments as a plug and play backup system. 

### Why? 

- Dead-simple.
- Tiny executable size.
- Only does what you tell it to.

## Features

- Monitors a specified folder for changes.
- Creates hourly backups, stored in daily folders.
- Keeps a rolling history of the specified retention period.
- Stores monthly snapshots (last change in each month).
- Supports multiple backup locations.

## Configuration

The configuration is embedded into the binary at compile time. Modify the `backup_warden_config.json` file in the project directory before building the binary. 

Below is an example configuration file:

```json
{
    "watch_folder": "path/to/watch",
    "backup_locations": ["path/to/backup1", "path/to/backup2"],
    "retention_days": 30
}
```
- watch_folder:         The folder to monitor for changes.
- backup_locations:     A list of locations where backups will be stored.
- retention_days:       The number of days to retain daily backups.

## Setup

1. Modify the backup_warden_config.json file with the necessary configuration settings.
2. Ensure the directories specified in the watch_folder and backup_locations exist and have the appropriate permissions.
3. Move the binary to your PC's startup folder (or wherever).

## Build

To build the project, you need to have Rust installed. You can install Rust from rustup.rs.

   1. Clone the repository:
   ```sh
   git clone https://github.com/yourusername/backup-warden.git
   cd backup-warden
   ```
   2. Modify the backup_warden_config.json file with your configuration.
   3. Build the project:
   ```sh
   cargo build --release
   ```
   4. The binary will be located in the target/release directory.

## License

Backup Warden is licensed under the GNU General Public License v3.0. See the LICENSE file for details.

## Contributing

This project is really simple so I likely won't update it much beyond quality of life improvements or the odd extra feature however contributions are welcome! 

