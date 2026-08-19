use std::path::{Path, PathBuf};
use std::sync::Mutex;

use readstat::FileFormat;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct PendingOpenPaths {
    paths: Mutex<Vec<String>>,
}

impl PendingOpenPaths {
    pub fn new() -> Self {
        Self {
            paths: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, paths: impl IntoIterator<Item = String>) {
        let Ok(mut guard) = self.paths.lock() else {
            return;
        };
        for path in paths {
            if !guard.iter().any(|existing| existing == &path) {
                guard.push(path);
            }
        }
    }

    pub fn take(&self) -> Vec<String> {
        self.paths
            .lock()
            .map(|mut guard| std::mem::take(&mut *guard))
            .unwrap_or_default()
    }
}

pub fn associated_path(path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    FileFormat::from_path(path).map(|_| path.to_path_buf())
}

pub fn arg_to_path(arg: &str) -> Option<PathBuf> {
    if arg.is_empty() || arg.starts_with('-') {
        return None;
    }
    if arg.to_ascii_lowercase().starts_with("file:") {
        return url::Url::parse(arg).ok()?.to_file_path().ok();
    }
    if arg.contains("://") {
        return None;
    }
    Some(PathBuf::from(arg))
}

pub fn paths_from_args<I, S>(args: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .filter_map(|arg| arg_to_path(arg.as_ref()))
        .filter_map(associated_path)
        .collect()
}

pub fn enqueue(app: &AppHandle, files: Vec<PathBuf>) {
    let paths: Vec<String> = files
        .into_iter()
        .filter_map(associated_path)
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    if paths.is_empty() {
        return;
    }
    app.state::<PendingOpenPaths>().push(paths);
    let _ = app.emit("open-files", ());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn take_pending_open_paths(pending: State<PendingOpenPaths>) -> Vec<String> {
    pending.take()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(paths: &[PathBuf]) -> Vec<String> {
        paths
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn launch_args_keep_associated_datasets_and_skip_flags() {
        let paths = paths_from_args([
            "--enable-features=foo",
            "/tmp/adsl.sas7bdat",
            "/tmp/notes.txt",
            "/tmp/adae.XPT",
            "/tmp/formats.sas7bcat",
            "/tmp/demo.sav",
        ]);
        assert_eq!(
            names(&paths),
            ["adsl.sas7bdat", "adae.XPT", "formats.sas7bcat", "demo.sav"]
        );
    }

    #[test]
    fn launch_args_accept_every_supported_extension() {
        let files = [
            "a.sas7bdat",
            "a.xpt",
            "a.xport",
            "a.sav",
            "a.zsav",
            "a.por",
            "a.dta",
            "a.sas7bcat",
        ];
        let args: Vec<String> = files.iter().map(|name| format!("/tmp/{name}")).collect();
        assert_eq!(names(&paths_from_args(args)), files);
    }

    #[test]
    fn launch_args_ignore_non_file_urls() {
        let paths = paths_from_args(["https://example.com/adsl.sas7bdat", "custom://open"]);
        assert!(paths.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn file_url_becomes_dataset_path() {
        let paths = paths_from_args(["file:///tmp/adsl.xpt", "file:///tmp/my%20ads.sas7bdat"]);
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/tmp/adsl.xpt"),
                PathBuf::from("/tmp/my ads.sas7bdat"),
            ]
        );
    }

    #[test]
    fn take_drains_pending_paths_without_duplicates() {
        let pending = PendingOpenPaths::new();
        pending.push(["/tmp/a.xpt".into(), "/tmp/b.sas7bdat".into()]);
        pending.push(["/tmp/a.xpt".into(), "/tmp/c.sav".into()]);
        assert_eq!(
            pending.take(),
            ["/tmp/a.xpt", "/tmp/b.sas7bdat", "/tmp/c.sav"]
        );
        assert!(pending.take().is_empty());
    }
}
