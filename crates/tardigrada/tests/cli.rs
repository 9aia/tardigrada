//! E2E — spawn the real `tardigrada` binary. Each test gets its
//! own temp HOME/XDG so the default-path resolution is exercised
//! end-to-end, not just the flag path. `TARDIGRADA_IDENTITY_BACKEND`
//! defaults to `file` here so the keyfile assertions stay
//! deterministic on hosts with (or without) a session keyring;
//! backend-selection tests override it explicitly.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A bogus session bus — makes `KeyringIdentity::platform()` fail
/// with `Unavailable` deterministically, on any host.
const NO_BUS: &str = "unix:path=/nonexistent-tardigrada-test-bus";

fn tardigrada(home: &Path, args: &[&str]) -> Output {
    tardigrada_env(home, args, &[("TARDIGRADA_IDENTITY_BACKEND", "file")])
}

fn tardigrada_env(home: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tardigrada"));
    cmd.args(args)
        .env("HOME", home)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("TARDIGRADA_IDENTITY_FILE")
        .env_remove("CUORA_IDENTITY_FILE")
        .env_remove("ARCA_IDENTITY_FILE")
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        // Isolate from the developer's real config even if a
        // parent env leaks through.
        .env("TARDIGRADA_IDENTITY_FILE", "");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).trim().to_string()
}

fn keyfile(home: &Path) -> PathBuf {
    home.join(".config/tardigrada/identity.txt")
}

#[test]
fn init_show_export_import_roundtrip() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();

    // show before init fails with the actionable error.
    let o = tardigrada(h, &["show"]);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("tardigrada init"), "{}", stderr(&o));

    // init creates the keyfile under the XDG default.
    let o = tardigrada(h, &["init"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let recipient = stdout(&o);
    assert!(recipient.starts_with("age1"), "{recipient}");
    assert!(keyfile(h).exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(keyfile(h)).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // init again never clobbers.
    let text = std::fs::read_to_string(keyfile(h)).unwrap();
    let o = tardigrada(h, &["init"]);
    assert!(o.status.success());
    assert_eq!(stdout(&o), recipient);
    assert_eq!(std::fs::read_to_string(keyfile(h)).unwrap(), text);

    // show prints the same recipient, context on stderr.
    let o = tardigrada(h, &["show"]);
    assert!(o.status.success());
    assert_eq!(stdout(&o), recipient);
    assert!(stderr(&o).contains("identity"), "{}", stderr(&o));

    // --json show.
    let o = tardigrada(h, &["--json", "show"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["recipient"], recipient);
    assert_eq!(v["backend"], "file");

    // backend inspects storage.
    let o = tardigrada(h, &["backend"]);
    assert!(o.status.success());
    assert_eq!(stdout(&o), "file");
    let o = tardigrada(h, &["--json", "backend"]);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["source"], keyfile(h).display().to_string());
    assert_eq!(v["mode"], "file");
    assert_eq!(v["pinned"], false);

    // export --out writes a second keyfile (0600).
    let out_path = h.join("moved/identity.txt");
    let o = tardigrada(h, &["export", "--out", out_path.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(out_path.exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&out_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // bare export prints the AGE-SECRET-KEY line. Piped stdout is
    // not a terminal, so no scrollback warning fires.
    let o = tardigrada(h, &["export"]);
    assert!(o.status.success());
    assert!(stdout(&o).starts_with("AGE-SECRET-KEY-"), "{}", stdout(&o));
    assert!(
        !stderr(&o).contains("terminal"),
        "piped export must not warn: {}",
        stderr(&o)
    );

    // import into a fresh HOME adopts the exported keyfile —
    // same recipient, proving the roundtrip moved real key
    // material, not just a path.
    let home2 = tempfile::tempdir().unwrap();
    let o = tardigrada(home2.path(), &["import", out_path.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), recipient);
    let o = tardigrada(home2.path(), &["show"]);
    assert_eq!(stdout(&o), recipient);
}

#[test]
fn import_missing_file_fails() {
    let home = tempfile::tempdir().unwrap();
    let o = tardigrada(home.path(), &["import", "/nonexistent/keyfile"]);
    assert!(!o.status.success());
}

#[test]
fn export_without_identity_fails() {
    let home = tempfile::tempdir().unwrap();
    let o = tardigrada(home.path(), &["export", "--out", "/tmp/x-id.txt"]);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("tardigrada init"));
}

#[test]
fn identity_file_flag_overrides() {
    let home = tempfile::tempdir().unwrap();
    let custom = home.path().join("custom/key.txt");
    let o = tardigrada(
        home.path(),
        &["--identity-file", custom.to_str().unwrap(), "init"],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(custom.exists());
    // The default path was NOT created — the flag won.
    assert!(!keyfile(home.path()).exists());
}

#[test]
fn json_error_shape() {
    let home = tempfile::tempdir().unwrap();
    let o = tardigrada(home.path(), &["--json", "show"]);
    assert!(!o.status.success());
    let v: serde_json::Value = serde_json::from_str(&stderr(&o)).unwrap();
    assert!(v["error"].as_str().unwrap().contains("tardigrada init"));
}

#[test]
fn manifest_is_json() {
    let home = tempfile::tempdir().unwrap();
    let o = tardigrada(home.path(), &["manifest"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["name"], "tardigrada");
}

#[test]
fn backend_reports_mode_source_and_pin() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();

    // file mode, unpinned.
    let o = tardigrada(h, &["--json", "backend"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["backend"], "file");
    assert_eq!(v["mode"], "file");
    assert_eq!(v["pinned"], false);
    assert_eq!(v["source"], keyfile(h).display().to_string());

    // an explicit pin reports pinned=true and still resolves file
    // — even under backend=keyring, with no session bus at all.
    let custom = h.join("pin/key.txt");
    let o = tardigrada_env(
        h,
        &[
            "--json",
            "--identity-file",
            custom.to_str().unwrap(),
            "backend",
        ],
        &[
            ("TARDIGRADA_IDENTITY_BACKEND", "keyring"),
            ("DBUS_SESSION_BUS_ADDRESS", NO_BUS),
        ],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["backend"], "file");
    assert_eq!(v["mode"], "keyring");
    assert_eq!(v["pinned"], true);
    assert_eq!(v["source"], custom.display().to_string());
}

#[test]
fn keyring_backend_fails_closed_without_a_session_store() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    let envs = [
        ("TARDIGRADA_IDENTITY_BACKEND", "keyring"),
        ("DBUS_SESSION_BUS_ADDRESS", NO_BUS),
    ];

    // init can't store into a session store that isn't there.
    let o = tardigrada_env(h, &["init"], &envs);
    assert!(!o.status.success());
    assert!(
        stderr(&o).contains("TARDIGRADA_IDENTITY_BACKEND"),
        "{}",
        stderr(&o)
    );
    // the keyfile is NOT silently created — fail closed means the
    // file is never the fallback.
    assert!(!keyfile(h).exists());

    let o = tardigrada_env(h, &["show"], &envs);
    assert!(!o.status.success());
    assert!(stderr(&o).contains("backend unavailable"), "{}", stderr(&o));
}

#[test]
fn auto_backend_falls_back_to_the_keyfile_headless() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    let envs = [
        ("TARDIGRADA_IDENTITY_BACKEND", "auto"),
        ("DBUS_SESSION_BUS_ADDRESS", NO_BUS),
    ];

    // init → keyfile with a fallback note on stderr.
    let o = tardigrada_env(h, &["init"], &envs);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stderr(&o).contains("no session keyring"), "{}", stderr(&o));
    assert!(keyfile(h).exists());
    let recipient = stdout(&o);
    assert!(recipient.starts_with("age1"), "{recipient}");

    // show/export read the same keyfile.
    let o = tardigrada_env(h, &["show"], &envs);
    assert!(o.status.success());
    assert_eq!(stdout(&o), recipient);
    let o = tardigrada_env(h, &["backend"], &envs);
    assert_eq!(stdout(&o), "file");

    // --json backend reports the resolved file + configured auto.
    let o = tardigrada_env(h, &["--json", "backend"], &envs);
    let v: serde_json::Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["backend"], "file");
    assert_eq!(v["mode"], "auto");
}

#[test]
fn unrecognized_backend_warns_and_defaults_to_keyring() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    let envs = [
        ("TARDIGRADA_IDENTITY_BACKEND", "bogus"),
        ("DBUS_SESSION_BUS_ADDRESS", NO_BUS),
    ];
    let o = tardigrada_env(h, &["show"], &envs);
    assert!(!o.status.success());
    // the typo is named on stderr, and the default (keyring,
    // fail closed) applied — no silent file read.
    assert!(
        stderr(&o).contains("unrecognized TARDIGRADA_IDENTITY_BACKEND"),
        "{}",
        stderr(&o)
    );
    assert!(stderr(&o).contains("backend unavailable"), "{}", stderr(&o));
}

#[test]
fn env_backend_file_roundtrip() {
    let home = tempfile::tempdir().unwrap();
    let h = home.path();
    let envs = [("TARDIGRADA_IDENTITY_BACKEND", "file")];
    let o = tardigrada_env(h, &["init"], &envs);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(keyfile(h).exists());
    let o = tardigrada_env(h, &["show"], &envs);
    assert!(o.status.success());
    assert!(stdout(&o).starts_with("age1"));
}
