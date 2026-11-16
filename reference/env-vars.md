# Environment variables

| Var | Read by | Effect |
| --- | --- | --- |
| `TARDIGRADA_IDENTITY_FILE` | path resolution | Keyfile path override (empty = ignored) |
| `XDG_CONFIG_HOME` | path resolution | Config root — default keyfile is `$XDG_CONFIG_HOME/tardigrada/identity.txt` |
| `HOME` | path resolution | Fallback config root `~/.config/…` when `XDG_CONFIG_HOME` is unset |
| `CUORA_IDENTITY_FILE` | **cuora**, not tardigrada | Point cuora at tardigrada's keyfile (or vice versa via `TARDIGRADA_IDENTITY_FILE`) |

## Resolution order

`--identity-file` flag → `TARDIGRADA_IDENTITY_FILE` →
`$XDG_CONFIG_HOME/tardigrada/identity.txt` →
`~/.config/tardigrada/identity.txt`
