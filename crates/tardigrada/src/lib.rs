//! tardigrada — the age-identity lifecycle tool, as a library.
//! Everything that used to live in a binary lives here: [`Cli`]/
//! [`Cmd`], identity-path resolution, verb dispatch. `run()` is the
//! compat CLI body; drivers split it into [`parse`]/[`parse_from`] +
//! [`dispatch`] — argv in, exit code out.
//!
//! tardigrada — dormant identity that persists anywhere. Named for
//! the tardigrade: it survives anything via cryptobiosis, and so
//! does this — a per-machine, per-human age identity that outlives
//! any single tool. It is the key behind encrypted stores (cuora
//! vaults, tamias sealed packs, rage files); tardigrada owns the
//! key's *lifecycle* — bootstrap, inspect, move — never the
//! ciphertext.
//!
//! v1 storage is FILE only: an `identity.txt`-style keyfile
//! (`AGE-SECRET-KEY-…`, mode `0600`) at
//! `$TARDIGRADA_IDENTITY_FILE` → `$XDG_CONFIG_HOME/tardigrada/
//! identity.txt` → `~/.config/tardigrada/identity.txt`, or the
//! `--identity-file` global flag. The port/impl come from cuora
//! (`cuora-identity`, `cuora-identity-file`) — consumed, not
//! copied. A keyring backend is future work (cuora-identity-keyring
//! hardcodes the `"cuora"` service name today).

use std::path::PathBuf;

use age::x25519;
use clap::{Parser, Subcommand};
use cuora_identity::{ensure, IdentityError, IdentitySource};
use cuora_identity_file::FileIdentity;
use serde_json::json;

/// tardigrada — the per-machine, per-human age identity behind
/// encrypted stores.
///
/// `init` bootstraps the identity (generate + store, never
/// clobbers); `show` prints the `age1…` recipient to share with
/// tools that seal to it; `import`/`export` move key material
/// between files and machines; `backend` inspects storage.
/// `--json` is honored on every verb; the keyfile is the only
/// secret tardigrade touches.
#[derive(Parser)]
#[command(name = "tardigrada", version, about, long_about)]
pub struct Cli {
    /// Emit JSON — stable machine shapes on every verb; errors are
    /// `{"error": "…"}` on stderr with a non-zero exit.
    #[arg(long, global = true)]
    json: bool,

    /// Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE`
    /// and the XDG default
    /// (`$XDG_CONFIG_HOME/tardigrada/identity.txt`).
    #[arg(long, global = true, value_name = "PATH")]
    identity_file: Option<String>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bootstrap the age identity — generate + store it. Never
    /// clobbers an existing identity: a second `init` reports the
    /// current recipient and exits clean.
    Init,
    /// Print the `age1…` recipient of the stored identity — the
    /// value tools seal to (cuora vaults, tamias packs, rage
    /// recipients files). Goes to stdout bare (scriptable);
    /// backend/source context on stderr. Fails when no identity
    /// exists — run `tardigrada init` first.
    Show,
    /// Adopt an existing `identity.txt`-style keyfile as this
    /// machine's identity — parses the file, stores the identity
    /// at the configured path (overwrites whatever is there; the
    /// old key is gone for good).
    Import {
        /// Path to the identity.txt keyfile to adopt.
        file: String,
    },
    /// Write the identity keyfile out — mode `0600`, the same
    /// format `import` reads. `--out PATH` writes a file for
    /// moving between tools/machines; bare `export` prints the
    /// `AGE-SECRET-KEY-…` line to stdout (pipe it into another
    /// tool's keyfile, or `rage -e -i /dev/stdin`). This is the
    /// explicit escape hatch — the value is secret.
    Export {
        /// Destination keyfile path (mode `0600`, parents created).
        #[arg(long, value_name = "PATH")]
        out: Option<String>,
    },
    /// Inspect the configured identity backend — always `file` in
    /// v1, plus the resolved keyfile path.
    Backend,
    /// Capabilities, JSON shapes, env vars — agent self-discovery
    /// (always JSON).
    Manifest,
}

/// Process argv → a parsed [`Cli`]. Split from [`dispatch`] so
/// drivers can parse *their* argv without owning process args.
/// Clap errors/help still exit the process, same as argv parsing.
pub fn parse() -> Cli {
    Cli::parse()
}

/// Same, from an argv iterator — `parse_from(["tardigrada",
/// "show"])`.
pub fn parse_from<I, T>(itr: I) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Cli::parse_from(itr)
}

/// The clap verb tree as a [`clap::Command`] — self-discovery and
/// doc generation derive from this.
pub fn command() -> clap::Command {
    <Cli as clap::CommandFactory>::command()
}

/// One parsed command → exit code. The dispatch every driver
/// shares; argv parsing is a caller concern.
pub fn dispatch(cli: &Cli) -> i32 {
    match execute(cli) {
        Ok(code) => code,
        Err(e) => fail(cli, &e.to_string()),
    }
}

/// The compat CLI body — parse process argv, dispatch, hand back
/// the exit code. Owns sigpipe reset before parse.
pub fn run() -> i32 {
    run_from(std::env::args_os())
}

/// Same, from an argv iterator — the passthrough entry drivers
/// re-exec.
pub fn run_from<I, T>(argv: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    sigpipe::reset();
    let cli = parse_from(argv);
    dispatch(&cli)
}

/// tardigrada errors — identity-port errors plus CLI-level
/// failures (missing identity, bad export target).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The identity port failed (io/parse/store).
    #[error("{0}")]
    Identity(#[from] IdentityError),
    /// No identity is stored — verbs that need one say so plainly.
    #[error("no identity at {path} — run `tardigrada init` first")]
    Missing { path: String },
    /// The import source held nothing parseable.
    #[error("no identity at {path} — nothing to import")]
    Empty { path: String },
    /// `export` to stdout under `--json` is refused — a secret
    /// inside a JSON envelope invites accidental logging; use a
    /// file `--out` or bare `export` for the pipeable form.
    #[error("`export --json` needs `--out PATH` — bare stdout export stays unwrapped")]
    JsonStdoutExport,
}

fn execute(cli: &Cli) -> Result<i32, Error> {
    match &cli.command {
        Cmd::Manifest => {
            out(&serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into()));
            Ok(0)
        }
        cmd => {
            let path = identity_path(cli);
            let src = FileIdentity::new(path.clone());
            match cmd {
                Cmd::Init => {
                    let (id, created) = ensure(&src)?;
                    note(
                        cli,
                        &format!(
                            "{} identity at {}",
                            if created { "created" } else { "found" },
                            src.describe()
                        ),
                    );
                    emit_identity_ex(cli, &id, &src, None, Some(created));
                }
                Cmd::Show => {
                    let id = load(&src)?;
                    note(
                        cli,
                        &format!("identity: {} (backend {})", src.describe(), src.kind()),
                    );
                    emit_identity_ex(cli, &id, &src, None, None);
                }
                Cmd::Import { file } => {
                    let from = FileIdentity::new(file);
                    let id = match from.load()? {
                        Some(id) => id,
                        None => {
                            return Err(Error::Empty {
                                path: from.describe(),
                            })
                        }
                    };
                    src.store(&id)?;
                    note(
                        cli,
                        &format!("imported {} → {}", from.describe(), src.describe()),
                    );
                    emit_identity(cli, &id, &src, Some(from.describe()));
                }
                Cmd::Export { out: dest } => {
                    let id = load(&src)?;
                    match dest {
                        Some(p) => {
                            let dst = FileIdentity::new(p);
                            dst.store(&id)?;
                            note(cli, &format!("exported identity to {}", dst.describe()));
                            emit_identity(cli, &id, &src, None);
                        }
                        None => {
                            use age::secrecy::ExposeSecret;
                            if cli.json {
                                return Err(Error::JsonStdoutExport);
                            }
                            // Bare stdout export — the pipeable
                            // form: `tardigrada export > keyfile` or
                            // into another tool's stdin.
                            println!("{}", id.to_string().expose_secret());
                        }
                    }
                }
                Cmd::Backend => {
                    if cli.json {
                        out(&json!({
                            "backend": src.kind(),
                            "source": src.describe(),
                        })
                        .to_string());
                    } else {
                        println!("{}", src.kind());
                        note(cli, &format!("source: {}", src.describe()));
                    }
                }
                // Handled before the inner match — exhaustiveness only.
                Cmd::Manifest => unreachable!(),
            }
            Ok(0)
        }
    }
}

/// The resolved keyfile path: `--identity-file` flag →
/// `$TARDIGRADA_IDENTITY_FILE` → `$XDG_CONFIG_HOME/tardigrada/
/// identity.txt` → `~/.config/tardigrada/identity.txt`.
pub fn identity_path(cli: &Cli) -> PathBuf {
    if let Some(p) = &cli.identity_file {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(p) = std::env::var("TARDIGRADA_IDENTITY_FILE") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    let xdg = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home().join(".config"));
    xdg.join("tardigrada").join("identity.txt")
}

/// The default path with no flag — exported for docs/tests.
pub fn default_identity_path() -> PathBuf {
    identity_path(&Cli {
        json: false,
        identity_file: None,
        command: Cmd::Backend,
    })
}

/// `~` for path defaults.
fn home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Load the stored identity or fail with the actionable error.
fn load(src: &FileIdentity) -> Result<x25519::Identity, Error> {
    match src.load()? {
        Some(id) => Ok(id),
        None => Err(Error::Missing {
            path: src.describe(),
        }),
    }
}

/// Emit the identity: recipient bare on stdout, `--json` wraps
/// backend/source (and `created`/`imported_from` when present) in
/// one object. Context notes go to stderr via [`note`].
fn emit_identity(cli: &Cli, id: &x25519::Identity, src: &FileIdentity, from: Option<String>) {
    emit_identity_ex(cli, id, src, from, None)
}

fn emit_identity_ex(
    cli: &Cli,
    id: &x25519::Identity,
    src: &FileIdentity,
    from: Option<String>,
    created: Option<bool>,
) {
    let recipient = id.to_public().to_string();
    if cli.json {
        let mut v = json!({
            "recipient": recipient,
            "backend": src.kind(),
            "source": src.describe(),
        });
        if let Some(c) = created {
            v["created"] = json!(c);
        }
        if let Some(f) = from {
            v["imported_from"] = json!(f);
        }
        out(&v.to_string());
    } else {
        println!("{recipient}");
    }
}

/// `tardigrada manifest` — the self-discovery surface, always JSON.
fn manifest() -> serde_json::Value {
    json!({
        "name": "tardigrada",
        "version": env!("CARGO_PKG_VERSION"),
        "about": "the per-machine, per-human age identity behind encrypted stores",
        "verbs": ["init", "show", "import", "export", "backend", "manifest"],
        "globals": {
            "--json": "stable machine shapes; errors are {\"error\": \"…\"} on stderr",
            "--identity-file": "keyfile path override"
        },
        "env": {
            "TARDIGRADA_IDENTITY_FILE": "keyfile path override",
            "XDG_CONFIG_HOME": "config root for the default keyfile"
        },
        "backend": {
            "file": "identity.txt keyfile, mode 0600 — the only v1 backend"
        },
        "interop": {
            "cuora": "point cuora at the same file (CUORA_IDENTITY_FILE) or `tardigrada export --out ~/.config/cuora/identity.txt`"
        }
    })
}

/// stdout line.
fn out(s: &str) {
    use std::io::Write;
    let mut h = std::io::stdout().lock();
    let _ = writeln!(h, "{s}");
}

/// stderr context note — never on stdout, never when `--json`
/// (stdout stays machine-clean; stderr is free either way but a
/// quiet pipe is friendlier).
fn note(cli: &Cli, msg: &str) {
    if !cli.json {
        eprintln!("tardigrada: {msg}");
    }
}

/// Error emit — `{"error": "…"}` on stderr under `--json`, plain
/// text otherwise; exit 1 either way.
fn fail(cli: &Cli, msg: &str) -> i32 {
    if cli.json {
        eprintln!("{}", json!({ "error": msg }));
    } else {
        eprintln!("tardigrada: error: {msg}");
    }
    1
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn identity_path_flag_beats_env_and_xdg() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", Some("/tmp/env-id")),
                ("XDG_CONFIG_HOME", Some("/tmp/xdg")),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "--identity-file", "/tmp/flag-id", "show"]);
                assert_eq!(identity_path(&cli), PathBuf::from("/tmp/flag-id"));
            },
        );
    }

    #[test]
    fn identity_path_env_beats_xdg() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", Some("/tmp/env-id")),
                ("XDG_CONFIG_HOME", Some("/tmp/xdg")),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "show"]);
                assert_eq!(identity_path(&cli), PathBuf::from("/tmp/env-id"));
            },
        );
    }

    #[test]
    fn identity_path_defaults_to_xdg_then_home() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
                ("XDG_CONFIG_HOME", Some("/tmp/xdg")),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "show"]);
                assert_eq!(
                    identity_path(&cli),
                    PathBuf::from("/tmp/xdg/tardigrada/identity.txt")
                );
            },
        );
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
                ("XDG_CONFIG_HOME", None::<&str>),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "show"]);
                assert_eq!(
                    identity_path(&cli),
                    PathBuf::from("/tmp/home/.config/tardigrada/identity.txt")
                );
            },
        );
    }

    #[test]
    fn empty_flag_and_env_fall_through() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", Some("")),
                ("XDG_CONFIG_HOME", Some("/tmp/xdg")),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "--identity-file", "", "show"]);
                assert_eq!(
                    identity_path(&cli),
                    PathBuf::from("/tmp/xdg/tardigrada/identity.txt")
                );
            },
        );
    }

    #[test]
    fn manifest_is_json_and_lists_verbs() {
        let v = manifest();
        assert_eq!(v["name"], "tardigrada");
        let verbs = v["verbs"].as_array().unwrap();
        for v in ["init", "show", "import", "export", "backend"] {
            assert!(verbs.iter().any(|x| x == v), "missing {v}");
        }
    }

    #[test]
    fn missing_identity_is_an_actionable_error() {
        let dir = tempfile::tempdir().unwrap();
        let cli = parse_from([
            "tardigrada",
            "--identity-file",
            dir.path().join("nope.txt").to_str().unwrap(),
            "show",
        ]);
        assert_eq!(dispatch(&cli), 1);
    }

    #[test]
    fn ensure_never_clobbers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id.txt");
        let cli = parse_from([
            "tardigrada",
            "--identity-file",
            path.to_str().unwrap(),
            "init",
        ]);
        assert_eq!(dispatch(&cli), 0);
        let first = std::fs::read_to_string(&path).unwrap();
        assert_eq!(dispatch(&cli), 0);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), first);
    }
}
