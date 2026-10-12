# Dependency decisions

## 2026-10-10: fix the build and pin the toolchain

### Context

The crates no longer resolved. Fresh dependency resolution failed because
`zip 2.6.1` was yanked on crates.io. The standalone `adsorption-search-util`
crate referenced path dependencies that no longer exist in the local
`chemrust` repository (`chemrust-core` was renamed to `chemrust-geometry`,
and `chemrust-parser` / `chemrust-scanner` were pruned in commit `a27f7f2`).

### Decisions

| Decision | Reason |
|---|---|
| Pin `zip` to `2.4.2` | All `zip` releases in the `^2.6.1` range are yanked. `2.4.2` is the highest non-yanked version that keeps the 2.x builder API (`ZipWriter`, `SimpleFileOptions`, `deflate-flate2`) |
| Supersede `chemrust-scanner` with the `chemrust-nasl` crate | The `MountingChecker` flow was replaced by `SearchConfig` / `SearchReports` in chemrust-nasl. The util crate now runs searches through `rhino_lib::run`, the same flow the server uses |
| Replace `chemrust-parser` with `castep-cell-io` | `.cell` parsing moved to the `castep-cell-io` / `castep-cell-fmt` crates. `CellParser` is obsolete |
| Update `castep-cell-io` to `0.7.0` | The 0.2.x line is frozen. 0.7.0 is the current API (`CellDocument` + builder + `castep-cell-fmt::parse`) |
| Add `adsorption-search-util` to the workspace `members` | The crate sits inside the workspace directory. Without membership, cargo refuses to build it in place |
| Create `flake.nix` with the fenix stable toolchain plus the `wasm32-unknown-unknown` target | The wasm crate builds through `wasm-pack`. The Nix toolchain needed the target and the tool |

### Migration notes

- `castep-cell-io` 0.7.0 no longer depends on `chemrust-core`.
  `CellDocument` therefore no longer implements `CrystalModel`.
  The server and util crates each carry a small `CellModelAdapter`
  (`adsorption-search-server/src/form_handler/cell_model.rs`,
  `adsorption-search-util/src/cell_model.rs`) that extracts the lattice and
  atom data and implements the trait.
- `LatticeCart` in 0.7.0 stores the lattice vectors as rows.
  `chemrust-core` expects columns. The adapter transposes the matrix.
  `LatticeABC` angles are degrees in the file and radians in
  `CellConstants`. The adapter converts them.
- The `rhino` git crate still pins `castep-cell-io` 0.2.x internally.
  The two versions coexist in the lockfile. The server no longer calls
  `rhino_lib::seeding::get_all_potentials`. It collects potential file
  names from the 0.7.0 documents and reads the files itself.
- `adsorption-search-util` public API: `init_checker` and `filtered_atoms`
  were removed. `load_model` returns a 0.7.0 `CellDocument`. `TaskTable`
  implements `rhino_lib::SearchJob`.
- `flake.nix` merges the wasm target standard library with
  `fenix.packages.<system>.combine [ hostToolchain,
  targets."wasm32-unknown-unknown".stable.rust-std ]`.
  Fenix does not accept target triples inside `withComponents`.

### Verification performed

- `cargo build --workspace`: exit 0.
- `cargo update` and `cargo generate-lockfile` (fresh, no lock): succeed.
  `zip` resolves to 2.4.2.
- `nix build .#devShells.x86_64-linux.default`: exit 0.
- `nix develop .#default`: the toolchain carries
  `lib/rustlib/wasm32-unknown-unknown` and `wasm-pack`.
- `cargo build -p adsorption-search-ui-wasm --target
  wasm32-unknown-unknown` inside the dev shell: exit 0.

### Known remaining issues

- 7 pre-existing warnings in `adsorption-search-server` (unused imports,
  dead code).
- `proc-macro-error2 2.0.1` future-incompatibility note (transitive, from
  the `crystal-cif-io` git dependency).
- Stale-but-present dependencies left untouched: `gloo` 0.11,
  `wasm-bindgen` 0.2.100, `web-sys` 0.3.77, `html` 0.6.3, `css-style`
  0.14.1.
