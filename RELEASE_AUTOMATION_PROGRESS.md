# Release Automation — Progress & Resume Notes

> Working session snapshot. Resume from **"Next steps"** below.
> Full design rationale lives in the session plan (`plan.md` in the Copilot
> session-state folder); this file is the self-contained, repo-local version so
> you can pick up without the session.

---

## Goal (recap)

Extend the GitHub Actions pipeline so a release automatically builds `hants`
binaries for the common OS/arch matrix, packages them into installers, creates a
GitHub Release with assets+checksums, and publishes to package managers.

User-facing install paths to deliver:
- Source archives (zip / tar.gz)
- Windows **MSI** installer
- **`.deb`** for manual Ubuntu/Debian install
- Package managers: **WinGet** (Win), **Homebrew** (macOS), **APT repo** (Linux)
- Bonus: `cargo install hants` via **crates.io**

---

## Decisions (CONFIRMED with maintainer)

1. **Release trigger model:** *Tag-driven (Pattern A)*. Push a `v*` git tag →
   pipeline builds, packages, creates a **draft** GitHub Release, uploads all
   assets, then publishes to package managers, then promotes the release. The
   pipeline OWNS release creation (not manual). Keeps a single source of truth and
   avoids half-published releases.
2. **Pipeline backbone:** **`dist` (cargo-dist)** for build matrix, archives,
   checksums, MSI, shell/powershell installers, Homebrew tap, and the GitHub
   Release. Custom jobs bolt on `.deb`, the APT repo, WinGet, and crates.io.
3. **Build matrix (full set chosen):**
   - `x86_64-pc-windows-msvc` (Win x64)
   - `aarch64-pc-windows-msvc` (Win ARM64)
   - `x86_64-unknown-linux-gnu` (Linux x64)
   - `aarch64-unknown-linux-gnu` (Linux ARM64)
   - `x86_64-unknown-linux-musl` (Linux x64 static)
   - `x86_64-apple-darwin` (macOS Intel)
   - `aarch64-apple-darwin` (macOS Apple Silicon)
4. **Signing for v1:** **Linux GPG only** (free; signs the APT repo / `.deb`).
   **Skip** Windows Authenticode and macOS notarization for now — document the
   "unknown publisher" / Gatekeeper warnings and how users bypass them. Leave
   signing wiring as documented placeholders for later.
5. **Linux channel:** **self-hosted APT repo on GitHub Pages**, GPG-signed
   (not Launchpad PPA).
6. **crates.io:** **Yes** — pipeline publishes so `cargo install hants` works.

### ⚠️ Project-specific gotcha
`hants` depends on **`arboard`** for clipboard. On Linux this needs **runtime**
X11/XCB libs (`libxcb1`, `libxcb-render0`, `libxcb-shape0`, `libxcb-xfixes0`).
- → `.deb` must declare these in `Depends:`.
- → The musl "static" build still won't have clipboard in a headless box; treat
  musl as a convenience portable artifact, not the primary Linux one.

### Secrets to provision (v1)
- `GH_PAT_PUBLISH` — fine-grained PAT with `contents:write` on the **homebrew tap**
  repo + **winget-pkgs fork** + ability to push to `gh-pages` (APT repo). Default
  `GITHUB_TOKEN` cannot push to other repos.
- `APT_GPG_PRIVATE_KEY` (ASCII-armored) + `APT_GPG_PASSPHRASE` — sign APT repo/.deb.
- `CARGO_REGISTRY_TOKEN` — crates.io publish.
- (Deferred) Apple + Windows signing secrets — NOT needed for v1.

### Accounts / one-time setup still owed by maintainer
- Create tap repo **`ivan-nosar/homebrew-hants`** (empty is fine; dist pushes the
  formula). Users will run `brew install ivan-nosar/hants/hants`.
- Fork **`microsoft/winget-pkgs`** under the publishing account for WinGet PRs.
- Generate a **GPG key** for the APT repo; export private key → `APT_GPG_PRIVATE_KEY`.
- Get a **crates.io API token** → `CARGO_REGISTRY_TOKEN`.
- Enable **GitHub Pages** (branch `gh-pages` or Pages action) for the APT repo.

---

## What's been DONE so far

1. ✅ Installed `dist` (cargo-dist) **0.32.0** locally at
   `%USERPROFILE%\.cargo\bin\dist.exe` (not on PATH in fresh shells — invoke with
   full path or add to PATH).
2. ✅ Enriched **`Cargo.toml`** `[package]` with `description`, `authors`,
   `license = "MIT"`, `readme`, `repository`, `homepage`, `keywords`,
   `categories` (required by crates.io / dist / WinGet / Homebrew metadata).
3. ✅ Ran `dist init` → it:
   - added **`[profile.dist]`** to `Cargo.toml` (inherits release).
   - created **`dist-workspace.toml`** with `[dist]` config (ci=github,
     installers = shell/powershell/homebrew/msi, all 7 targets, hosting=github,
     install-path=CARGO_HOME, install-updater=false).

### Current git status
```
 M Cargo.toml          (metadata + [profile.dist])
?? TODO.md             (was already present/untracked)
?? dist-workspace.toml (new, from dist init)
```
Only `.github/workflows/ci.yml` exists. **`release.yml` was NOT generated yet.**

### ⚠️ Known issue to resolve first on resume
- The expected **`.github/workflows/release.yml`** is missing. `dist init` reported
  "running 'dist generate'" but no workflow file appeared. The last command
  (`dist generate` / inspecting output) was interrupted by shutdown.
  **First action on resume:** re-run generation and confirm the workflow exists:
  ```powershell
  cd 'd:\Projects\hants.worktrees\agents-ci-pipeline-release-automation'
  & "$env:USERPROFILE\.cargo\bin\dist.exe" generate --allow-dirty -v info
  Get-ChildItem .github\workflows
  ```
  If it errors, read the error — likely a config nit in `dist-workspace.toml`.

---

## NEXT STEPS (resume checklist, in order)

### Phase 1 — Get the dist core working
- [ ] Re-run `dist generate --allow-dirty` and confirm
      `.github/workflows/release.yml` is created.
- [ ] Run `dist plan` (a.k.a. `dist build --help`) / `dist plan` to preview the
      artifact matrix and sanity-check all 7 targets resolve.
- [ ] Review `release.yml`: trigger is tag `v*`; ensure draft-release behavior.

### Phase 2 — Linux runtime deps for `arboard`
- [ ] Add a CI dependency-install step for Linux gnu builds (XCB dev libs), e.g.
      `sudo apt-get install -y libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`.
      In dist this is the `[dist] dependencies.apt` table in `dist-workspace.toml`.
- [ ] For musl: confirm it builds (clipboard may be a no-op); keep as portable.

### Phase 3 — `.deb` packaging (cargo-deb)
- [ ] Add `[package.metadata.deb]` to `Cargo.toml`:
      - `depends = "libxcb1, libxcb-render0, libxcb-shape0, libxcb-xfixes0"`
      - maintainer, `assets` mapping `target/release/hants` → `/usr/bin/hants`,
        plus README/LICENSE → `/usr/share/doc/hants/`.
      - (optional) shell completions + man page.
- [ ] Add a CI job (amd64 + arm64) that runs `cargo deb` and uploads the `.deb`
      into the GitHub Release. cross/arm64 via `cross` or arm64 runner.

### Phase 4 — APT repo on GitHub Pages (GPG-signed)
- [ ] CI job (after release assets exist): import `APT_GPG_PRIVATE_KEY`, build the
      repo with `reprepro` (or `aptly`), sign `Release`, publish `.deb`s + `key.gpg`
      to `gh-pages`.
- [ ] User flow ends up as (see README install section draft below).

### Phase 5 — WinGet
- [ ] Add job using `vedantmgoyal2009/winget-releaser` (or `wingetcreate update`)
      that opens a PR to `microsoft/winget-pkgs` for package id `ivan-nosar.hants`,
      pointing at the released **MSI** + its SHA256. Uses `GH_PAT_PUBLISH`.
- [ ] First submission may need manual approval; later updates auto.

### Phase 6 — Homebrew tap
- [ ] In `dist-workspace.toml` set the tap repo (e.g.
      `tap = "ivan-nosar/homebrew-hants"`) and `publish-jobs`/token so dist pushes
      `Formula/hants.rb` on release. Provide `GH_PAT_PUBLISH`.
- [ ] (Later) consider homebrew-core once project hits notability thresholds.

### Phase 7 — crates.io
- [ ] Add a publish job (or `dist`'s `publish-jobs = ["...]`) running
      `cargo publish` with `CARGO_REGISTRY_TOKEN`, gated on tag builds.
- [ ] Ensure `Cargo.toml` has all required fields (done) and `cargo publish
      --dry-run` passes.

### Phase 8 — Keep existing CI for commits/PRs
- [ ] Leave `ci.yml` (debug build + tests + coverage on push/PR) as-is. Per
      `TODO.md`: debug profile for regular commits, release/dist for tags.

### Phase 9 — Docs
- [ ] Add **Installation** section to `README.md` (draft below).
- [ ] Document maintainer release process: `git tag vX.Y.Z && git push --tags`.
- [ ] Tick off `TODO.md`: "Pack into installer", "Publish to package managers".

### Phase 10 — Validation (dry run)
- [ ] Push a pre-release tag `v0.2.0-rc.1`; verify every asset builds, checksums
      match, MSI installs, `.deb` installs + clipboard works on Ubuntu desktop,
      brew formula installs, WinGet manifest validates (`winget validate` /
      Windows Sandbox), `cargo install` works.

---

## README Installation section (draft to paste later)

```markdown
## Installation

### Windows
- WinGet: `winget install ivan-nosar.hants`
- MSI: download `hants-<ver>-x86_64.msi` from Releases and run it.
- Manual: download the `*-pc-windows-msvc.zip`, extract, add to PATH.

### macOS
- Homebrew: `brew install ivan-nosar/hants/hants`
- Manual: download `*-apple-darwin.tar.gz`, extract, move `hants` to /usr/local/bin.

### Linux (Ubuntu/Debian)
- APT repo:
  ```sh
  curl -fsSL https://ivan-nosar.github.io/hants/key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/hants.gpg
  echo "deb [signed-by=/usr/share/keyrings/hants.gpg] https://ivan-nosar.github.io/hants stable main" | sudo tee /etc/apt/sources.list.d/hants.list
  sudo apt update && sudo apt install hants
  ```
- Manual .deb: `sudo apt install ./hants_<ver>_amd64.deb`
- Manual tarball: download `*-linux-gnu.tar.gz`, extract, copy to /usr/local/bin.

### Rust users
- `cargo install hants`

### From source
- `git clone … && cargo build --release` → `target/release/hants`
```

---

## Handy commands

```powershell
# dist is here (not always on PATH):
& "$env:USERPROFILE\.cargo\bin\dist.exe" --version   # 0.32.0

# Regenerate CI from dist config:
& "$env:USERPROFILE\.cargo\bin\dist.exe" generate --allow-dirty -v info

# Preview the release artifact matrix:
& "$env:USERPROFILE\.cargo\bin\dist.exe" plan

# Repo: ivan-nosar/hants ; no release tags exist yet.
```
