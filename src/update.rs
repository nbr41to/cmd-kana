//! Installs an already verified update after the GUI process has exited.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

struct InstallPaths {
    target: PathBuf,
    staged: PathBuf,
    backup: PathBuf,
    workspace: PathBuf,
}

fn validate_paths(paths: &InstallPaths, executable: &Path) -> Result<(), String> {
    let parent = paths.target.parent().ok_or("アプリの場所が不正です。")?;
    if !paths.target.is_absolute()
        || paths.target.extension().map_or(true, |ext| ext != "app")
        || paths.staged.parent() != Some(parent)
        || paths.backup.parent() != Some(parent)
        || !paths
            .staged
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".cmd-kana-update-"))
        || !paths
            .backup
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with(".cmd-kana-backup-"))
        || paths.backup.exists()
        || paths.target == paths.staged
        || paths.target == paths.backup
    {
        return Err("更新先のパスを確認できませんでした。".into());
    }
    if executable.parent() != Some(paths.workspace.as_path())
        || !paths
            .workspace
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("cmd-kana-update-"))
    {
        return Err("更新用の一時フォルダを確認できませんでした。".into());
    }
    Ok(())
}

fn wait_for_exit(pid: i32) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if unsafe { kill(pid, 0) } != 0 {
            // ESRCH is 3 on both macOS and the Linux test host.
            if io::Error::last_os_error().raw_os_error() == Some(3) {
                return Ok(());
            }
            return Err("更新前のアプリの終了を確認できませんでした。".into());
        }
        if Instant::now() >= deadline {
            return Err("アプリが終了しなかったため、更新を中止しました。".into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn launch_app(path: &Path, workspace: &Path) -> io::Result<()> {
    let ready = workspace.join("ready");
    let _ = fs::remove_file(&ready);
    let mut child = Command::new(path.join("Contents/MacOS/cmd-kana"))
        .arg("--update-complete")
        .arg(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => break,
            Ok(None) => {
                if fs::read_to_string(&ready).ok().as_deref() == Some(&child.id().to_string()) {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    Err(io::Error::new(
        io::ErrorKind::Other,
        "アプリの起動を確認できませんでした。",
    ))
}

/// Called only once Cocoa has finished launching the replacement application.
pub fn mark_update_ready(args: &[OsString]) {
    if let Some(index) = args.iter().position(|arg| arg == "--update-complete") {
        if let Some(directory) = args.get(index + 1) {
            let path = Path::new(directory);
            if path.is_absolute()
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("cmd-kana-update-"))
            {
                let _ = fs::write(path.join("ready"), std::process::id().to_string());
            }
        }
    }
}

fn install_with_launcher(
    paths: &InstallPaths,
    mut launch: impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), String> {
    if let Err(error) = fs::rename(&paths.target, &paths.backup) {
        let _ = launch(&paths.target);
        return Err(format!("元のアプリを退避できませんでした: {error}"));
    }
    let installed = fs::rename(&paths.staged, &paths.target);
    let result = installed
        .as_ref()
        .map(|_| ())
        .map_err(|error| error.to_string())
        .and_then(|()| launch(&paths.target).map_err(|error| error.to_string()));
    if let Err(error) = result {
        if installed.is_ok() {
            fs::rename(&paths.target, &paths.staged).map_err(|restore_error| {
                format!("更新に失敗しました: {error}\n復旧にも失敗しました: {restore_error}\n元のアプリ: {}", paths.backup.display())
            })?;
        }
        fs::rename(&paths.backup, &paths.target).map_err(|restore_error| {
            format!("更新に失敗しました: {error}\n復旧にも失敗しました: {restore_error}\n元のアプリ: {}", paths.backup.display())
        })?;
        let _ = launch(&paths.target);
        let _ = fs::remove_dir_all(&paths.staged);
        return Err(format!(
            "更新に失敗したため、元のアプリに戻しました。\n{error}"
        ));
    }
    // Cleanup failures do not undo an update that has already launched.
    let _ = fs::remove_dir_all(&paths.backup);
    let _ = fs::remove_dir_all(&paths.workspace);
    Ok(())
}

pub fn run_installer(args: &[OsString]) -> Option<Result<(), String>> {
    if args.get(1).map_or(true, |arg| arg != "--install-update") {
        return None;
    }
    Some((|| {
        if args.len() != 7 {
            return Err("更新用の引数が不正です。".into());
        }
        let pid: i32 = args[2]
            .to_str()
            .and_then(|value| value.parse().ok())
            .filter(|&pid| pid > 1 && pid as u32 != std::process::id())
            .ok_or("更新前のプロセスを確認できませんでした。")?;
        let paths = InstallPaths {
            target: PathBuf::from(&args[3]),
            staged: PathBuf::from(&args[4]),
            backup: PathBuf::from(&args[5]),
            workspace: PathBuf::from(&args[6]),
        };
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        validate_paths(&paths, &executable)?;
        if let Err(error) = wait_for_exit(pid) {
            let _ = fs::remove_dir_all(&paths.staged);
            return Err(error);
        }
        install_with_launcher(&paths, |target| launch_app(target, &paths.workspace))
    })())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};

    extern "C" {
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }

    static NEXT_TEST: AtomicUsize = AtomicUsize::new(0);

    struct Fixture {
        root: PathBuf,
        paths: InstallPaths,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "cmd-kana-installer-test-{}-{}",
                std::process::id(),
                NEXT_TEST.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let paths = InstallPaths {
                target: root.join("cmd kana 日本語.app"),
                staged: root.join(".cmd-kana-update-test.app"),
                backup: root.join(".cmd-kana-backup-test.app"),
                workspace: root.join("cmd-kana-update-test"),
            };
            for (path, version) in [(&paths.target, "old"), (&paths.staged, "new")] {
                fs::create_dir(path).unwrap();
                fs::write(path.join("version"), version).unwrap();
            }
            fs::create_dir(&paths.workspace).unwrap();
            Self { root, paths }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if let Some(pid) = fs::read_to_string(self.paths.workspace.join("ready"))
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .filter(|&pid| pid > 1 && pid as u32 != std::process::id())
            {
                unsafe {
                    kill(pid, 15);
                    waitpid(pid, std::ptr::null_mut(), 0);
                }
            }
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn replaces_and_launches_verified_bundle_then_cleans_up() {
        let fixture = Fixture::new();
        let mut launches = 0;
        install_with_launcher(&fixture.paths, |target| {
            assert_eq!(fs::read_to_string(target.join("version"))?, "new");
            assert!(fixture.paths.backup.exists());
            launches += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(launches, 1);
        assert!(!fixture.paths.backup.exists());
        assert!(!fixture.paths.staged.exists());
        assert!(!fixture.paths.workspace.exists());
    }

    #[test]
    fn restores_old_bundle_when_new_bundle_cannot_be_moved() {
        let fixture = Fixture::new();
        fs::remove_dir_all(&fixture.paths.staged).unwrap();
        let result = install_with_launcher(&fixture.paths, |target| {
            assert_eq!(fs::read_to_string(target.join("version"))?, "old");
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(fixture.paths.target.join("version")).unwrap(),
            "old"
        );
        assert!(!fixture.paths.backup.exists());
    }

    #[test]
    fn restores_and_relaunches_old_bundle_when_launch_fails() {
        let fixture = Fixture::new();
        let mut launches = 0;
        let result = install_with_launcher(&fixture.paths, |target| {
            launches += 1;
            if launches == 1 {
                assert_eq!(fs::read_to_string(target.join("version"))?, "new");
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "simulated launch failure",
                ))
            } else {
                assert_eq!(fs::read_to_string(target.join("version"))?, "old");
                Ok(())
            }
        });
        assert!(result.is_err());
        assert_eq!(launches, 2);
        assert!(!fixture.paths.backup.exists());
        assert!(!fixture.paths.staged.exists());
    }

    #[test]
    fn does_not_modify_application_for_normal_or_malformed_arguments() {
        assert!(run_installer(&["cmd-kana".into()]).is_none());
        assert!(
            run_installer(&["cmd-kana".into(), "--install-update".into()])
                .unwrap()
                .is_err()
        );
    }

    fn fake_executable(app: &Path, script: &str) {
        let directory = app.join("Contents/MacOS");
        fs::create_dir_all(&directory).unwrap();
        let executable = directory.join("cmd-kana");
        fs::write(&executable, script).unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn startup_failure_restores_and_starts_original_process() {
        let fixture = Fixture::new();
        fake_executable(&fixture.paths.target, "#!/bin/sh\nprintf '%s' \"$$\" > \"$2/ready\"\nwhile [ -d \"$2\" ]; do /bin/sleep 1; done\n");
        fake_executable(&fixture.paths.staged, "#!/bin/sh\nexit 12\n");
        let result = install_with_launcher(&fixture.paths, |target| {
            launch_app(target, &fixture.paths.workspace)
        });
        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(fixture.paths.target.join("version")).unwrap(),
            "old"
        );
        assert!(fixture.paths.workspace.join("ready").exists());
        assert!(!fixture.paths.backup.exists());
    }

    #[test]
    fn rejects_install_paths_outside_expected_sibling_directories() {
        let mut fixture = Fixture::new();
        let executable = fixture.paths.workspace.join("helper");
        validate_paths(&fixture.paths, &executable).unwrap();
        fixture.paths.backup = fixture.paths.workspace.join(".cmd-kana-backup-test.app");
        assert!(validate_paths(&fixture.paths, &executable).is_err());
    }

    #[test]
    fn application_signals_finished_launch_to_installer() {
        let fixture = Fixture::new();
        mark_update_ready(&[
            "cmd-kana".into(),
            "--update-complete".into(),
            fixture.paths.workspace.as_os_str().to_owned(),
        ]);
        assert_eq!(
            fs::read_to_string(fixture.paths.workspace.join("ready")).unwrap(),
            std::process::id().to_string()
        );
    }
}
