# tauri-ui-inspector

Agent-friendly CLI for references created by [`tauri-plugin-ui-inspector`](https://github.com/mathematic-inc/tauri-plugin-ui-inspector).

The installed binary is `ui-inspector`. It can start a picker in a running Tauri application and retrieve, resolve, list, or delete durable `@ui_<ULID>` references. Run `ui-inspector --help` for commands.

## Prebuilt installation

Release archives contain the executable and install without a Rust compiler.
Install with cargo-binstall, with source compilation disabled:

```sh
cargo binstall --disable-strategies compile tauri-ui-inspector
```

Or declare the GitHub release directly in `mise.toml`:

```toml
[tools]
"github:mathematic-inc/tauri-plugin-ui-inspector" = "latest"
```

Run `mise install` to download and activate the executable. No custom mise plugin
is required. The Cargo backend (`cargo:tauri-ui-inspector`) also supports these releases;
set `cargo.binstall_only = true` to reject source compilation.

| Platform | Architectures | Archive |
| --- | --- | --- |
| macOS | x64, ARM64 | `.tar.gz` |
| Linux GNU (glibc 2.35 or newer) | x64, ARM64 | `.tar.gz` |
| Linux musl | x64, ARM64 | `.tar.gz` |
| Windows MSVC | x64, ARM64 | `.zip` |

Every archive includes SHA-256 checksums and GitHub build provenance. CI builds all
eight targets and runs the extracted executables on the matching architecture.
After publication, the release workflow installs through cargo-binstall and mise
and runs both installations. A missing prebuilt binary fails the release checks.

The crate is named `tauri-ui-inspector`; the installed executable is
`ui-inspector`. The CLI connects to an application using the Tauri plugin.
