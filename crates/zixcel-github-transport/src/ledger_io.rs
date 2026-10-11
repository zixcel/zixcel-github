use std::fs::{File, OpenOptions, create_dir_all, rename};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::TransportError;

// Keep the sidecar inode stable across atomic ledger replacement. Do not unlink
// it: dropping the handle (including process termination) releases the OS lock.
pub(crate) fn lock(path: &Path) -> Result<File, TransportError> {
    let parent = path.parent().ok_or(TransportError("ledger-path"))?;
    create_dir_all(parent).map_err(|_| TransportError("ledger-lock"))?;
    let leaf = path.file_name().ok_or(TransportError("ledger-path"))?;
    let mut lock_name = leaf.to_os_string();
    lock_name.push(".lock");
    let lock_path = parent.join(lock_name);
    let file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&lock_path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let before = lock_path
                .symlink_metadata()
                .map_err(|_| TransportError("ledger-lock"))?;
            if !before.is_file() || before.permissions().mode() & 0o077 != 0 {
                return Err(TransportError("ledger-lock"));
            }
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&lock_path)
                .map_err(|_| TransportError("ledger-lock"))?;
            let opened = file.metadata().map_err(|_| TransportError("ledger-lock"))?;
            if !opened.is_file() || before.dev() != opened.dev() || before.ino() != opened.ino() {
                return Err(TransportError("ledger-lock"));
            }
            file
        }
        Err(_) => return Err(TransportError("ledger-lock")),
    };
    file.lock().map_err(|_| TransportError("ledger-lock"))?;
    Ok(file)
}

pub(crate) fn replace(path: &Path, bytes: &[u8]) -> Result<(), TransportError> {
    let parent = path.parent().ok_or(TransportError("ledger-path"))?;
    create_dir_all(parent).map_err(|_| TransportError("ledger-write"))?;
    let mut nonce = [0_u8; 16];
    getrandom::fill(&mut nonce).map_err(|_| TransportError("entropy"))?;
    let leaf = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(TransportError("ledger-path"))?;
    let temporary = parent.join(format!(".{leaf}.{}.tmp", hex::encode(nonce)));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| TransportError("ledger-write"))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| TransportError("ledger-write"))?;
    rename(&temporary, path).map_err(|_| TransportError("ledger-write"))?;
    std::fs::File::open(parent)
        .and_then(|value| value.sync_all())
        .map_err(|_| TransportError("ledger-write"))
}
