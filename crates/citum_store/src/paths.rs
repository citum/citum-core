use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

#[cfg(unix)]
pub fn data_dir() -> Option<PathBuf> {
    xdg_dir("XDG_DATA_HOME", ".local/share")
}

#[cfg(unix)]
pub fn config_dir() -> Option<PathBuf> {
    xdg_dir("XDG_CONFIG_HOME", ".config")
}

#[cfg(unix)]
pub fn cache_dir() -> Option<PathBuf> {
    xdg_dir("XDG_CACHE_HOME", ".cache")
}

#[cfg(unix)]
fn xdg_dir(var: &str, fallback: &str) -> Option<PathBuf> {
    xdg_resolve(env::var_os(var), fallback, dirs::home_dir())
}

#[cfg(unix)]
fn xdg_resolve(
    env_val: Option<OsString>,
    fallback: &str,
    home: Option<PathBuf>,
) -> Option<PathBuf> {
    env_val
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home.map(|h| h.join(fallback)))
}

#[cfg(not(unix))]
pub fn data_dir() -> Option<PathBuf> {
    dirs::data_dir()
}

#[cfg(not(unix))]
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir()
}

#[cfg(not(unix))]
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_resolve_absolute() {
        let env_val = Some(OsString::from("/custom/path"));
        let home = Some(PathBuf::from("/home/user"));
        let res = xdg_resolve(env_val, ".config", home);
        assert_eq!(res, Some(PathBuf::from("/custom/path")));
    }

    #[test]
    fn test_xdg_resolve_relative_ignored() {
        let env_val = Some(OsString::from("relative/path"));
        let home = Some(PathBuf::from("/home/user"));
        let res = xdg_resolve(env_val, ".config", home);
        assert_eq!(res, Some(PathBuf::from("/home/user/.config")));
    }

    #[test]
    fn test_xdg_resolve_fallback() {
        let env_val = None;
        let home = Some(PathBuf::from("/home/user"));
        let res = xdg_resolve(env_val, ".cache", home);
        assert_eq!(res, Some(PathBuf::from("/home/user/.cache")));
    }

    #[test]
    fn test_xdg_resolve_no_home() {
        let env_val = None;
        let home = None;
        let res = xdg_resolve(env_val, ".local/share", home);
        assert_eq!(res, None);
    }
}
