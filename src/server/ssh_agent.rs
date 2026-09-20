//! Stable SSH agent socket for panes on a long-lived remote server.
//!
//! The first SSH bridge that starts a remote server inherits an ephemeral
//! `SSH_AUTH_SOCK`. That socket disappears when the originating session ends.
//! Later bridges attach to the already-running server and would otherwise leave
//! panes pointing at a dead path.
//!
//! Each bridge reports its live socket. The server publishes a stable symlink
//! next to the session sockets and retargets it on every report so new and
//! already-running panes keep using the same `SSH_AUTH_SOCK`.

use std::io;
use std::path::{Path, PathBuf};

const LINK_NAME: &str = "ssh-agent.sock";

fn link_path() -> PathBuf {
    crate::session::data_dir().join(LINK_NAME)
}

pub(crate) fn refresh(socket_path: &Path) -> io::Result<()> {
    if socket_path.as_os_str().is_empty() || !socket_path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SSH_AUTH_SOCK must be an absolute path",
        ));
    }

    let link = link_path();
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::platform::replace_symlink(socket_path, &link)
}

pub(crate) fn pane_socket_path() -> Option<PathBuf> {
    let path = link_path();
    match path.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_symlink() => Some(path),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("herdr-{name}-{}-{nanos}", std::process::id()))
    }

    fn isolate_session(name: &str) -> PathBuf {
        let config_home = unique_test_path(name);
        std::env::set_var("XDG_CONFIG_HOME", &config_home);
        std::env::remove_var(crate::session::SESSION_ENV_VAR);
        crate::session::clear_explicit_session_for_test();
        config_home
    }

    fn restore_session(config_home: &Path) {
        let _ = std::fs::remove_dir_all(config_home);
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn pane_socket_path_is_inactive_until_refresh() {
        let _guard = env_lock();
        let config_home = isolate_session("ssh-agent-inactive");
        assert_eq!(pane_socket_path(), None);
        restore_session(&config_home);
    }

    #[test]
    fn refresh_rejects_relative_socket_path() {
        let _guard = env_lock();
        let config_home = isolate_session("ssh-agent-relative");
        let err = refresh(Path::new("agent.sock")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(pane_socket_path(), None);
        restore_session(&config_home);
    }

    #[cfg(unix)]
    #[test]
    fn refresh_creates_stable_symlink() {
        let _guard = env_lock();
        let config_home = isolate_session("ssh-agent-create");
        let target = config_home.join("forwarded-agent");
        refresh(&target).unwrap();

        let published = pane_socket_path().expect("stable agent socket");
        assert_eq!(published, crate::session::data_dir().join(LINK_NAME));
        assert_eq!(std::fs::read_link(&published).unwrap(), target);
        restore_session(&config_home);
    }

    #[cfg(unix)]
    #[test]
    fn refresh_replaces_existing_symlink_target() {
        let _guard = env_lock();
        let config_home = isolate_session("ssh-agent-replace");
        let first = config_home.join("agent-one");
        let second = config_home.join("agent-two");
        refresh(&first).unwrap();
        refresh(&second).unwrap();

        let published = pane_socket_path().expect("stable agent socket");
        assert_eq!(std::fs::read_link(&published).unwrap(), second);
        restore_session(&config_home);
    }
}
