# Stat Data Viewer

Read-only desktop viewer for SAS, SPSS and Stata data files. Open a dataset, inspect values and metadata, query with DuckDB SQL, export CSV / Parquet / Excel. Nothing is written back to the statistical file.

| Source | Formats |
|---|---|
| SAS | `.sas7bdat`, `.xpt` (v5/v8), `.sas7bcat` (catalog only, not a dataset) |
| SPSS | `.sav`, `.zsav`, `.por` |
| Stata | `.dta` (v104–v119) |

## Develop

System dependencies:

- Rust 1.88+ (`rustup`; `rust-toolchain.toml` pins 1.88.0)
- Node.js 20+
- LLVM / Clang (`bindgen` needs `libclang`)
- **macOS**: Xcode Command Line Tools (system `iconv`)
- **Linux**: `clang`, `llvm-dev`, `libclang-dev`, WebKitGTK 4.1 (for Tauri), and glibc `iconv`. If linking fails on musl/Alpine, install `libiconv` and set `READSTAT_LINK_ICONV=1`.
- **Windows**: LLVM (Chocolatey: `choco install llvm`), plus Visual Studio Build Tools. `iconv` is vendored (`win-iconv`). zlib is compiled via `libz-sys`.

```bash
npm install
npm run tauri dev
```

Frontend only (no native commands):

```bash
npm run dev
```

Rust crates:

```bash
cargo test -p readstat
cargo check --workspace
```

## Build

```bash
# current architecture
npm run tauri build

# macOS universal (requires both targets)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin

# Windows x64 (run on Windows; unsigned)
npm run tauri build
```

Linux packages are not a release target yet. CI compiles on Ubuntu to catch ReadStat / DuckDB build breaks.

GitHub Actions builds installers on a `v*` tag (or the **Release** workflow):

- Windows: NSIS `.exe`
- macOS: `.dmg` for Apple Silicon and Intel

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Re-import (encoding / format)

Encoding is **not** guessed. Older Chinese SAS/SPSS files are often GBK or GB18030; pre-14 Stata may be Windows-1252.

If strings or labels are garbled, or the file has no reliable extension:

1. Open the dataset.
2. **Re-import…**
3. Pick encoding (`GBK`, `GB18030`, `Latin1`, `Windows-1252`, `UTF-8`) and/or format.
4. For SAS value labels, attach a `.sas7bcat` (same-stem / `formats.sas7bcat` is auto-detected when present).

## Architecture

`ReadStat C (vendored)` → `readstat-sys` (bindgen) → `readstat` (Arrow batches) → DuckDB session file → Tauri commands (JSON pages) → Vue 3 grid.

Locked crate pair: **duckdb 1.2.2** + **arrow 54**. Do not mix with duckdb 1.105xx (Arrow 58, higher MSRV).

Data tables store **raw values**. Labels live in `meta_variables` / `meta_value_labels`. Dates are formatted in Rust using SAS (1960-01-01), Stata and SPSS epochs.

## Golden tests

Put samples in `tests/fixtures/`:

- `sample.sav` / `sample.dta` / `sample.sas7bdat` / `sample.xpt`
- `xpt_v5.xpt`, `xpt_v8.xpt`, `dta_v113.dta`, `dta_v118.dta`

Tests skip missing files. Official ReadStat / pyreadstat samples are preferred.

## License notes

ReadStat C is vendored from [WizardMac/ReadStat](https://github.com/WizardMac/ReadStat) v1.1.9 (MIT). Windows builds also vendor [win-iconv](https://github.com/win-iconv/win-iconv) (public domain).
