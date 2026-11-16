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
//! Storage is backend-selected, mirroring cuora's model:
//! `TARDIGRADA_IDENTITY_BACKEND` picks `keyring` (default — the
//! platform session secret store, fail closed when unreachable),
//! `auto` (keyring with keyfile fallback — the headless/CI mode),
//! or `file` (keyfile only, never probes the store). An explicit
//! `--identity-file`/`TARDIGRADA_IDENTITY_FILE` pin always means
//! file and bypasses the keyring probe entirely. The port and
//! impls come from cuora (`cuora-identity`, `cuora-identity-file`,
//! `cuora-identity-keyring`, git-pinned) — consumed, not copied.
//! The keyring entry is the canonical `tardigrada/age-identity`;
//! legacy `cuora`/`arca` entries read as fallbacks.
//!
//! Keyfile path resolution: `--identity-file` →
//! `TARDIGRADA_IDENTITY_FILE` → the `cuora-identity-file` default
//! (`$XDG_CONFIG_HOME/tardigrada/identity.txt` →
//! `~/.config/tardigrada/identity.txt`, with legacy `cuora`/`arca`
//! keyfiles resolving first when they exist).

use std::io::IsTerminal;
use std::path::PathBuf;

use age::x25519;
use clap::{Parser, Subcommand};
use cuora_identity::{ensure, IdentityError, IdentitySource};
use cuora_identity_file::FileIdentity;
use cuora_identity_keyring::KeyringIdentity;
use serde_json::json;

/// tardigrada — the per-machine, per-human age identity behind
/// encrypted stores.
///
/// `init` bootstraps the identity (generate + store, never
/// clobbers); `show` prints the `age1…` recipient to share with
/// tools that seal to it; `import`/`export` move key material
/// between files and machines; `backend` inspects storage.
/// `--json` is honored on every verb; the identity key is the
/// only secret tardigrade touches.
#[derive(Parser)]
#[command(name = "tardigrada", version, about, long_about)]
pub struct Cli {
    /// Emit JSON — stable machine shapes on every verb; errors are
    /// `{"error": "…"}` on stderr with a non-zero exit.
    #[arg(long, global = true)]
    json: bool,

    /// Identity keyfile path, overriding `$TARDIGRADA_IDENTITY_FILE`
    /// and the XDG default
    /// (`$XDG_CONFIG_HOME/tardigrada/identity.txt`). An explicit
    /// pin always selects the file backend — the keyring is never
    /// probed.
    #[arg(long, global = true, value_name = "PATH")]
    identity_file: Option<String>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Bootstrap the age identity — generate + store it in the
    /// configured backend. Never clobbers an existing identity: a
    /// second `init` reports the current recipient and exits
    /// clean. Under `keyring`/`auto` an existing keyfile identity
    /// is adopted into the store (migration), never orphaned.
    Init,
    /// Print the `age1…` recipient of the stored identity — the
    /// value tools seal to (cuora vaults, tamias packs, rage
    /// recipients files). Goes to stdout bare (scriptable);
    /// backend/source context on stderr. Fails when no identity
    /// exists — run `tardigrada init` first.
    Show,
    /// Adopt an existing `identity.txt`-style keyfile as this
    /// machine's identity — parses the file, stores the identity
    /// in the configured backend (overwrites whatever is there;
    /// the old key is gone for good).
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
    /// Inspect the configured identity backend — the resolved
    /// source (`keyring` or `file`), the configured mode
    /// (`keyring`/`auto`/`file`), and whether an explicit file pin
    /// is in effect.
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
    /// The identity port failed (io/parse/store/unavailable).
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

/// Which identity backend `TARDIGRADA_IDENTITY_BACKEND` selects —
/// cuora's model: `keyring` (default, fail closed) | `auto`
/// (keyring → keyfile fallback) | `file` (keyfile only).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Backend {
    Keyring,
    Auto,
    File,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Backend::Keyring => "keyring",
            Backend::Auto => "auto",
            Backend::File => "file",
        }
    }
}

/// The configured backend — `TARDIGRADA_IDENTITY_BACKEND`, absent
/// or unrecognized → `keyring` (fail closed; a typo'd opt-out gets
/// a stderr note, never a silent surprise).
fn configured_backend(cli: &Cli) -> Backend {
    match std::env::var("TARDIGRADA_IDENTITY_BACKEND") {
        Ok(v) if !v.is_empty() => match v.to_ascii_lowercase().as_str() {
            "keyring" => Backend::Keyring,
            "auto" => Backend::Auto,
            "file" => Backend::File,
            _ => {
                note(
                    cli,
                    &format!("unrecognized TARDIGRADA_IDENTITY_BACKEND {v:?} — ignoring"),
                );
                Backend::Keyring
            }
        },
        _ => Backend::Keyring,
    }
}

/// An explicit keyfile pin — `--identity-file` or
/// `TARDIGRADA_IDENTITY_FILE` — always means the file backend and
/// bypasses the keyring probe entirely (no dbus round-trip).
fn explicit_pin(cli: &Cli) -> bool {
    cli.identity_file.as_deref().is_some_and(|p| !p.is_empty())
        || std::env::var("TARDIGRADA_IDENTITY_FILE")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
}

/// The resolved identity source plus how it was chosen — verbs
/// dispatch on [`Inner`], reports read `mode`/`pinned`.
struct Source {
    inner: Inner,
    /// The configured `TARDIGRADA_IDENTITY_BACKEND` selection.
    mode: Backend,
    /// An explicit `--identity-file`/`TARDIGRADA_IDENTITY_FILE`
    /// pin forced the file backend.
    pinned: bool,
}

enum Inner {
    File(FileIdentity),
    Keyring(KeyringIdentity),
}

impl Source {
    /// The file backend at the resolved keyfile path.
    fn file(cli: &Cli, mode: Backend, pinned: bool) -> Self {
        Source {
            inner: Inner::File(FileIdentity::new(identity_path(cli))),
            mode,
            pinned,
        }
    }

    fn keyring(k: KeyringIdentity, mode: Backend) -> Self {
        Source {
            inner: Inner::Keyring(k),
            mode,
            pinned: false,
        }
    }

    fn describe(&self) -> String {
        match &self.inner {
            Inner::File(f) => f.describe(),
            Inner::Keyring(k) => k.describe(),
        }
    }

    fn kind(&self) -> &'static str {
        match &self.inner {
            Inner::File(f) => f.kind(),
            Inner::Keyring(k) => k.kind(),
        }
    }

    fn store(&self, id: &x25519::Identity) -> Result<(), IdentityError> {
        match &self.inner {
            Inner::File(f) => f.store(id),
            Inner::Keyring(k) => k.store(id),
        }
    }
}

/// Pick the identity source for this run — an explicit pin always
/// means file; else `TARDIGRADA_IDENTITY_BACKEND` decides:
/// `file` never probes the store, `keyring` fails closed when no
/// session store is reachable, `auto` probes and falls back to
/// the keyfile with a stderr note.
fn resolve(cli: &Cli) -> Result<Source, Error> {
    let mode = configured_backend(cli);
    if explicit_pin(cli) {
        if mode == Backend::Keyring {
            note(cli, "explicit identity file pin overrides backend=keyring");
        }
        return Ok(Source::file(cli, mode, true));
    }
    match mode {
        Backend::File => Ok(Source::file(cli, mode, false)),
        Backend::Keyring => Ok(Source::keyring(strict_keyring()?, mode)),
        Backend::Auto => match KeyringIdentity::platform() {
            Ok(k) => Ok(Source::keyring(k, mode)),
            Err(e) => {
                // Headless is the expected case — say why the
                // file was chosen.
                note(
                    cli,
                    &format!("no session keyring reachable ({e}) — using the identity file"),
                );
                Ok(Source::file(cli, mode, false))
            }
        },
    }
}

/// `backend=keyring` probe — reachable store or a typed
/// `Unavailable` error, never a silent file fallback.
fn strict_keyring() -> Result<KeyringIdentity, Error> {
    KeyringIdentity::platform().map_err(|e| {
        Error::Identity(IdentityError::Unavailable(format!(
            "backend=keyring: {e} — \
             set TARDIGRADA_IDENTITY_BACKEND=auto|file for headless/CI"
        )))
    })
}

/// The keyfile the file backend (and `auto` fallback) resolves.
fn file_fallback(cli: &Cli) -> FileIdentity {
    FileIdentity::new(identity_path(cli))
}

/// `load` with `None` → the actionable "run `tardigrada init`"
/// error every caller that *requires* an identity shares.
fn required(src: &dyn IdentitySource) -> Result<x25519::Identity, Error> {
    match src.load()? {
        Some(id) => Ok(id),
        None => Err(Error::Missing {
            path: src.describe(),
        }),
    }
}

/// Read-path load — `show`/`export`. `auto` falls back to the
/// keyfile when the session store is reachable-but-empty or its
/// read fails (a degraded session, not a crash); `keyring` fails
/// closed and `file` never consults the store.
fn load_main(cli: &Cli, src: &Source) -> Result<x25519::Identity, Error> {
    match &src.inner {
        Inner::File(f) => required(f),
        Inner::Keyring(k) => match k.load() {
            Ok(Some(id)) => Ok(id),
            // `auto`: an empty store falls through to the keyfile.
            Ok(None) if src.mode == Backend::Auto => required(&file_fallback(cli)),
            Ok(None) => Err(Error::Missing { path: k.describe() }),
            Err(e) if src.mode == Backend::Auto => {
                note(
                    cli,
                    &format!("keyring identity unreadable ({e}) — using the identity file"),
                );
                required(&file_fallback(cli))
            }
            Err(e) => Err(e.into()),
        },
    }
}

/// Best-effort read of the *current* identity — `import`'s
/// replace-warning and nothing else. Same backend resolution as
/// [`load_main`] but `Ok(None)` where `load_main` would raise
/// `Missing`; read errors still propagate — a blind overwrite is
/// worse than a failed import.
fn peek(cli: &Cli, src: &Source) -> Result<Option<x25519::Identity>, Error> {
    match &src.inner {
        Inner::File(f) => Ok(f.load()?),
        Inner::Keyring(k) => match k.load() {
            Ok(Some(id)) => Ok(Some(id)),
            // `auto`: an empty store falls through to the keyfile.
            Ok(None) if src.mode == Backend::Auto => Ok(file_fallback(cli).load()?),
            Ok(None) => Ok(None),
            Err(e) if src.mode == Backend::Auto => {
                note(
                    cli,
                    &format!("keyring identity unreadable ({e}) — checking the identity file"),
                );
                Ok(file_fallback(cli).load()?)
            }
            Err(e) => Err(e.into()),
        },
    }
}

fn execute(cli: &Cli) -> Result<i32, Error> {
    match &cli.command {
        Cmd::Manifest => {
            out(&serde_json::to_string_pretty(&manifest()).unwrap_or_else(|_| "{}".into()));
            Ok(0)
        }
        cmd => {
            let src = resolve(cli)?;
            match cmd {
                Cmd::Init => init(cli, &src)?,
                Cmd::Show => {
                    let id = load_main(cli, &src)?;
                    note(
                        cli,
                        &format!("identity: {} (backend {})", src.describe(), src.kind()),
                    );
                    emit_identity(cli, &id, &src, None, None, None);
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
                    // `import` overwrites by design — but replacing a
                    // *different* identity must be visible: name both
                    // recipients so a wrong-source import is caught
                    // before the old key is gone for good.
                    let replaced = peek(cli, &src)?
                        .filter(|old| old.to_public() != id.to_public())
                        .map(|old| old.to_public().to_string());
                    if let Some(old) = &replaced {
                        note(cli, &format!("replacing existing identity {old}"));
                    }
                    src.store(&id)?;
                    note(
                        cli,
                        &format!("imported {} → {}", from.describe(), src.describe()),
                    );
                    emit_identity(cli, &id, &src, Some(from.describe()), None, replaced);
                }
                Cmd::Export { out: dest } => {
                    let id = load_main(cli, &src)?;
                    match dest {
                        Some(p) => {
                            let dst = FileIdentity::new(p);
                            // Same guard as `import`: clobbering a
                            // *different* identity at the destination
                            // is deliberate-or-mistake — say which.
                            // Read failures don't block: `store`
                            // surfaces the real write error.
                            if let Ok(Some(old)) = dst.load() {
                                if old.to_public() != id.to_public() {
                                    note(
                                        cli,
                                        &format!(
                                            "overwriting a different identity at {}",
                                            dst.describe()
                                        ),
                                    );
                                }
                            }
                            dst.store(&id)?;
                            note(cli, &format!("exported identity to {}", dst.describe()));
                            emit_identity(cli, &id, &src, None, None, None);
                        }
                        None => {
                            use age::secrecy::ExposeSecret;
                            if cli.json {
                                return Err(Error::JsonStdoutExport);
                            }
                            // Bare stdout export — the pipeable
                            // form: `tardigrada export > keyfile` or
                            // into another tool's stdin. On a TTY the
                            // secret lands in scrollback — warn so
                            // the escape hatch stays deliberate.
                            if std::io::stdout().is_terminal() {
                                eprintln!(
                                    "tardigrada: warning: printing the secret key to a \
                                     terminal — prefer `export --out PATH`"
                                );
                            }
                            println!("{}", id.to_string().expose_secret());
                        }
                    }
                }
                Cmd::Backend => {
                    if cli.json {
                        out(&json!({
                            "backend": src.kind(),
                            "mode": src.mode.name(),
                            "pinned": src.pinned,
                            "source": src.describe(),
                        })
                        .to_string());
                    } else {
                        println!("{}", src.kind());
                        if src.pinned {
                            note(cli, "pinned by --identity-file/TARDIGRADA_IDENTITY_FILE");
                        }
                        note(
                            cli,
                            &format!("mode: {} (TARDIGRADA_IDENTITY_BACKEND)", src.mode.name()),
                        );
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

/// `init` — ensure against the resolved source. A keyring backend
/// adopts an existing keyfile identity into the store (migration,
/// the file stays as a backup); `auto` treats an unreadable store
/// read as a degraded session and bootstraps the keyfile instead;
/// `keyring` fails closed on a store error.
fn init(cli: &Cli, src: &Source) -> Result<(), Error> {
    match &src.inner {
        Inner::File(f) => {
            let (id, created) = ensure(f)?;
            note(
                cli,
                &format!(
                    "{} identity at {}",
                    if created { "created" } else { "found" },
                    f.describe()
                ),
            );
            emit_identity(cli, &id, src, None, Some(created), None);
        }
        Inner::Keyring(k) => match k.load() {
            Ok(Some(id)) => {
                note(cli, &format!("found identity at {}", k.describe()));
                emit_identity(cli, &id, src, None, Some(false), None);
            }
            Ok(None) => {
                // Migration: an existing keyfile identity is
                // imported into the store, never orphaned — its
                // ciphertext must keep decrypting. The file stays
                // as a backup.
                let f = file_fallback(cli);
                match f.load()? {
                    Some(id) => {
                        k.store(&id)?;
                        note(cli, &format!("moved keyfile identity → {}", k.describe()));
                        emit_identity_migrated(cli, &id, src, &f);
                    }
                    None => {
                        let (id, created) = ensure(k)?;
                        note(
                            cli,
                            &format!(
                                "{} identity at {}",
                                if created { "created" } else { "found" },
                                k.describe()
                            ),
                        );
                        emit_identity(cli, &id, src, None, Some(created), None);
                    }
                }
            }
            Err(e) => {
                if src.mode == Backend::Keyring {
                    // Fail closed — falling back could mint a
                    // divergent file identity the keyring can't
                    // decrypt.
                    return Err(e.into());
                }
                note(
                    cli,
                    &format!("keyring identity unreadable ({e}) — using the identity file"),
                );
                let f = file_fallback(cli);
                let (id, created) = ensure(&f)?;
                note(
                    cli,
                    &format!(
                        "{} identity at {}",
                        if created { "created" } else { "found" },
                        f.describe()
                    ),
                );
                // Report the source that actually served — the
                // file, not the unreachable keyring.
                let file_src = Source::file(cli, src.mode, src.pinned);
                emit_identity(cli, &id, &file_src, None, Some(created), None);
            }
        },
    }
    Ok(())
}

/// `init`'s migration emit — the identity now lives in the
/// keyring, adopted from `f`'s keyfile (`migrated_from` in JSON).
fn emit_identity_migrated(cli: &Cli, id: &x25519::Identity, src: &Source, f: &FileIdentity) {
    if cli.json {
        out(&json!({
            "recipient": id.to_public().to_string(),
            "backend": src.kind(),
            "source": src.describe(),
            "created": false,
            "migrated_from": f.describe(),
        })
        .to_string());
    } else {
        println!("{}", id.to_public());
    }
}

/// The resolved keyfile path: `--identity-file` flag →
/// `$TARDIGRADA_IDENTITY_FILE` → the `cuora-identity-file`
/// default (`$CUORA_IDENTITY_FILE`/`$ARCA_IDENTITY_FILE` legacy
/// overrides, then the first existing of
/// `<config>/tardigrada|cuora|arca/identity.txt`, creation at the
/// canonical tardigrada path).
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
    FileIdentity::default_path()
}

/// The default path with no flag — exported for docs/tests.
pub fn default_identity_path() -> PathBuf {
    identity_path(&Cli {
        json: false,
        identity_file: None,
        command: Cmd::Backend,
    })
}

/// Emit the identity: recipient bare on stdout, `--json` wraps
/// backend/source (and `created`/`imported_from`/`replaced_recipient`
/// when present) in one object. Context notes go to stderr via
/// [`note`].
fn emit_identity(
    cli: &Cli,
    id: &x25519::Identity,
    src: &Source,
    from: Option<String>,
    created: Option<bool>,
    replaced: Option<String>,
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
        if let Some(r) = replaced {
            v["replaced_recipient"] = json!(r);
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
            "--identity-file": "keyfile path override — pins the file backend"
        },
        "env": {
            "TARDIGRADA_IDENTITY_BACKEND": "keyring (default, fail closed — no file fallback) | auto (keyring → keyfile fallback, headless) | file (never probe the keyring)",
            "TARDIGRADA_IDENTITY_FILE": "keyfile path override — pins the file backend",
            "XDG_CONFIG_HOME": "config root for the default keyfile",
            "CUORA_KEYRING_TIMEOUT_MS": "bounds one keyring op (cuora-identity-keyring); 0 waits forever"
        },
        "backends": {
            "keyring": "platform session secret store — entry tardigrada/age-identity (legacy cuora/arca entries read as fallbacks); default, fail closed",
            "auto": "TARDIGRADA_IDENTITY_BACKEND=auto — keyring when reachable, keyfile fallback",
            "file": "identity.txt keyfile, mode 0600 — opt-out via TARDIGRADA_IDENTITY_BACKEND=file"
        },
        "interop": {
            "cuora": "shares this identity — the same keyring entry (tardigrada/age-identity) or the same keyfile (CUORA_IDENTITY_FILE)"
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
    fn identity_path_env_beats_default() {
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
    fn identity_path_defaults_to_tardigrada_canonical() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
                ("CUORA_IDENTITY_FILE", None::<&str>),
                ("ARCA_IDENTITY_FILE", None::<&str>),
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
                ("CUORA_IDENTITY_FILE", None::<&str>),
                ("ARCA_IDENTITY_FILE", None::<&str>),
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
                ("CUORA_IDENTITY_FILE", None::<&str>),
                ("ARCA_IDENTITY_FILE", None::<&str>),
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
    fn backend_env_selects_the_mode() {
        let cli = || parse_from(["tardigrada", "backend"]);
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_BACKEND", None::<&str>),
                ("TARDIGRADA_IDENTITY_FILE", Some("/tmp/pin")),
            ],
            || assert_eq!(configured_backend(&cli()), Backend::Keyring),
        );
        for (v, want) in [
            ("keyring", Backend::Keyring),
            ("auto", Backend::Auto),
            ("file", Backend::File),
            ("AUTO", Backend::Auto),
            // unrecognized → ignored, default keyring applies
            ("bogus", Backend::Keyring),
        ] {
            temp_env::with_var("TARDIGRADA_IDENTITY_BACKEND", Some(v), || {
                assert_eq!(configured_backend(&cli()), want, "{v}");
            });
        }
    }

    #[test]
    fn explicit_pin_detects_flag_and_env() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
                ("TARDIGRADA_IDENTITY_BACKEND", Some("keyring")),
            ],
            || {
                let cli = parse_from(["tardigrada", "--identity-file", "/tmp/x", "show"]);
                assert!(explicit_pin(&cli));
                let cli = parse_from(["tardigrada", "show"]);
                assert!(!explicit_pin(&cli));
            },
        );
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_FILE", Some("/tmp/env-id")),
                ("TARDIGRADA_IDENTITY_BACKEND", Some("keyring")),
            ],
            || {
                let cli = parse_from(["tardigrada", "show"]);
                assert!(explicit_pin(&cli));
            },
        );
    }

    #[test]
    fn explicit_pin_resolves_file_without_probing_keyring() {
        // backend=keyring + a pin → the file source, no store probe.
        // Deterministic on every host: the pin short-circuits before
        // KeyringIdentity::platform() is ever called.
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_BACKEND", Some("keyring")),
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
                ("CUORA_IDENTITY_FILE", None::<&str>),
                ("ARCA_IDENTITY_FILE", None::<&str>),
                ("XDG_CONFIG_HOME", Some("/tmp/xdg")),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "--identity-file", "/tmp/pin-id", "backend"]);
                let src = resolve(&cli).unwrap();
                assert_eq!(src.kind(), "file");
                assert_eq!(src.describe(), "/tmp/pin-id");
                assert!(src.pinned);
                assert_eq!(src.mode, Backend::Keyring);
            },
        );
    }

    #[test]
    fn file_backend_never_probes_the_store() {
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_BACKEND", Some("file")),
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
                ("CUORA_IDENTITY_FILE", None::<&str>),
                ("ARCA_IDENTITY_FILE", None::<&str>),
                ("XDG_CONFIG_HOME", Some("/tmp/xdg")),
                ("HOME", Some("/tmp/home")),
            ],
            || {
                let cli = parse_from(["tardigrada", "backend"]);
                let src = resolve(&cli).unwrap();
                assert_eq!(src.kind(), "file");
                assert_eq!(
                    src.describe(),
                    "/tmp/xdg/tardigrada/identity.txt".to_string()
                );
                assert!(!src.pinned);
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
        assert!(v["env"]["TARDIGRADA_IDENTITY_BACKEND"].is_string());
    }

    #[test]
    fn missing_identity_is_an_actionable_error() {
        let dir = tempfile::tempdir().unwrap();
        temp_env::with_var("TARDIGRADA_IDENTITY_BACKEND", Some("file"), || {
            let cli = parse_from([
                "tardigrada",
                "--identity-file",
                dir.path().join("nope.txt").to_str().unwrap(),
                "show",
            ]);
            assert_eq!(dispatch(&cli), 1);
        });
    }

    #[test]
    fn ensure_never_clobbers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id.txt");
        temp_env::with_var("TARDIGRADA_IDENTITY_BACKEND", Some("file"), || {
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
        });
    }

    #[test]
    fn file_backend_init_show_export_import_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("id.txt");
        let p = path.to_str().unwrap();
        temp_env::with_vars(
            [
                ("TARDIGRADA_IDENTITY_BACKEND", Some("file")),
                ("TARDIGRADA_IDENTITY_FILE", None::<&str>),
            ],
            || {
                let cli = |args: &[&str]| {
                    let mut v = vec!["tardigrada", "--identity-file", p];
                    v.extend_from_slice(args);
                    parse_from(v)
                };
                assert_eq!(dispatch(&cli(&["init"])), 0);
                assert!(path.exists());
                assert_eq!(dispatch(&cli(&["show"])), 0);
                let out = dir.path().join("moved.txt");
                let cli_export = parse_from([
                    "tardigrada",
                    "--identity-file",
                    p,
                    "export",
                    "--out",
                    out.to_str().unwrap(),
                ]);
                assert_eq!(dispatch(&cli_export), 0);
                // import adopts the moved keyfile into the file backend
                let other = dir.path().join("other.txt");
                let cli_import = parse_from([
                    "tardigrada",
                    "--identity-file",
                    other.to_str().unwrap(),
                    "import",
                    out.to_str().unwrap(),
                ]);
                assert_eq!(dispatch(&cli_import), 0);
                assert!(other.exists());
            },
        );
    }
}
