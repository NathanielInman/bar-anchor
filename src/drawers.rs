//! One drawer at a time, across apps.
//!
//! Each app's drawer process listens on a socket named after it in a shared
//! folder (`$XDG_RUNTIME_DIR/bar-anchor/drawers/<name>.sock`). When a drawer
//! opens it calls [`opened`], which writes `hide` to every other socket
//! there, so whatever drawer was open closes, whichever app it belongs to
//! and however this one was opened (a lamp, a key, the command line). There
//! is no list of apps: a new one joins by calling [`listen`] and [`opened`].
//!
//! The protocol is one line, `hide\n`, so an app in another language can
//! join with a few lines of its own (Signal Box does, in Go). A socket whose
//! process is gone refuses the connection and is removed.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long a drawer gets to take the message before it is skipped.
const TIMEOUT: Duration = Duration::from_millis(300);

/// The shared folder, or `None` without `XDG_RUNTIME_DIR`.
pub fn dir() -> Option<PathBuf> {
    let rt = std::env::var_os("XDG_RUNTIME_DIR")?;
    Some(PathBuf::from(rt).join("bar-anchor").join("drawers"))
}

/// Listening for other drawers. Dropping it removes the socket.
#[derive(Debug)]
pub struct Listener {
    path: PathBuf,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Starts listening as `name` (the drawer's layer-shell namespace is a good
/// name). `on_hide` runs on a background thread each time another drawer
/// opens; send it to the UI thread from there. Keep the [`Listener`] for as
/// long as the drawer can open.
pub fn listen(name: &str, on_hide: impl Fn() + Send + 'static) -> std::io::Result<Listener> {
    let dir = dir().ok_or_else(|| std::io::Error::other("XDG_RUNTIME_DIR is not set"))?;
    listen_in(&dir, name, on_hide)
}

/// Tells every other drawer to close. Call it when this drawer opens; it
/// returns at once and does the work on a background thread.
pub fn opened(name: &str) {
    let Some(dir) = dir() else { return };
    let name = name.to_owned();
    std::thread::spawn(move || {
        opened_in(&dir, &name);
    });
}

fn socket(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.sock"))
}

pub(crate) fn listen_in(
    dir: &Path,
    name: &str,
    on_hide: impl Fn() + Send + 'static,
) -> std::io::Result<Listener> {
    std::fs::create_dir_all(dir)?;
    let path = socket(dir, name);
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("another {name} drawer is listening on {}", path.display()),
            ));
        }
        std::fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(conn) = conn else { continue };
            let _ = conn.set_read_timeout(Some(TIMEOUT));
            let mut line = String::new();
            if BufReader::new(conn).read_line(&mut line).is_ok() && line.trim() == "hide" {
                on_hide();
            }
        }
    });
    Ok(Listener { path })
}

/// Sends `hide` to every drawer in `dir` but `name`; returns how many took
/// it. Sockets nobody listens on are removed.
pub(crate) fn opened_in(dir: &Path, name: &str) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let own = socket(dir, name);
    let mut told = 0;
    for e in entries.flatten() {
        let path = e.path();
        if path == own || path.extension().is_none_or(|x| x != "sock") {
            continue;
        }
        match UnixStream::connect(&path) {
            Ok(mut s) => {
                let _ = s.set_write_timeout(Some(TIMEOUT));
                if s.write_all(b"hide\n").is_ok() {
                    told += 1;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                let _ = std::fs::remove_file(&path);
            }
            Err(_) => {}
        }
    }
    told
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn scratch(tag: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("bar-anchor-drawers-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn opening_one_hides_the_others_but_not_itself() {
        let dir = scratch("others");
        let (tx, rx) = mpsc::channel();
        let mut keep = vec![];
        for name in ["tannoy", "roundhouse", "logbook"] {
            let tx = tx.clone();
            keep.push(listen_in(&dir, name, move || tx.send(name).unwrap()).unwrap());
        }
        assert_eq!(opened_in(&dir, "tannoy"), 2);
        let mut got: Vec<_> = (0..2)
            .map(|_| rx.recv_timeout(TIMEOUT * 3).unwrap())
            .collect();
        got.sort();
        assert_eq!(got, ["logbook", "roundhouse"]);
        assert!(
            rx.recv_timeout(Duration::from_millis(100)).is_err(),
            "tannoy hid itself"
        );
        drop(keep);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_dead_drawer_is_cleaned_up() {
        let dir = scratch("dead");
        std::fs::create_dir_all(&dir).unwrap();
        // A socket file left by a process that is gone.
        drop(UnixListener::bind(socket(&dir, "tablet")).unwrap());
        assert!(socket(&dir, "tablet").exists());
        assert_eq!(opened_in(&dir, "waybill"), 0);
        assert!(!socket(&dir, "tablet").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_second_listener_with_the_same_name_is_refused() {
        let dir = scratch("twice");
        let _first = listen_in(&dir, "layover", || {}).unwrap();
        let err = listen_in(&dir, "layover", || {}).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::AddrInUse);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dropping_the_listener_removes_its_socket() {
        let dir = scratch("drop");
        let l = listen_in(&dir, "timetable", || {}).unwrap();
        assert!(socket(&dir, "timetable").exists());
        drop(l);
        assert!(!socket(&dir, "timetable").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_folder_means_nobody_to_tell() {
        assert_eq!(opened_in(&scratch("none"), "tannoy"), 0);
    }
}
