# Releasing profanite

Tagging `vX.Y.Z` kicks off three independent workflows that ship the same version to npm, PyPI, and crates.io. Each is idempotent (`--skip-existing` / `dry-run` guards), so partial failures can be re-run safely.

## One-time setup — repo secrets

Add these under **Settings → Secrets and variables → Actions**:

| Secret | Purpose | How to get it |
|---|---|---|
| `NPM_TOKEN` | Publish to npm (main `@beatsphere/profanite` package + 6 platform subpackages `@beatsphere/profanite-<triple>`) | `npm login` then copy from `~/.npmrc`, or create an automation token at npmjs.com |
| `CARGO_REGISTRY_TOKEN` | Publish `profanite-core` to crates.io | `cargo login` after creating a token at crates.io/me |
| `PYPI_API_TOKEN` (optional) | Publish wheels to PyPI | Only needed if NOT using PyPI Trusted Publishers (see below) |

### Preferred: PyPI Trusted Publishers (no long-lived token)

1. Create the `profanite` project on PyPI (publish a first sdist manually if needed).
2. PyPI → project → **Manage → Publishing → Add a new pending publisher**.
   - Owner: `Beatsphere`
   - Repo: `profanite`
   - Workflow: `release-python.yml`
   - Environment: `pypi`
3. Under repo **Settings → Environments → New environment → `pypi`** (no secrets needed, just the name). Add optional "Required reviewers" if you want release gating.

If Trusted Publishers is too fiddly, skip all of that and set `PYPI_API_TOKEN`; the workflow auto-falls-back when the OIDC flow has no issuer.

### npm: scope setup

The main package is `@beatsphere/profanite`. Platform subpackages are
`@beatsphere/profanite-linux-x64-gnu`, `@beatsphere/profanite-darwin-arm64`, etc.

The `@beatsphere` scope needs to exist on your npm account before publishing.
`npm login` then either create an organization at npmjs.com/settings/orgs, or
publish under your personal scope if the account already owns it.

The first `npm publish` creates each subpackage automatically. Scope setup
is a one-time thing.

## Cutting a release

1. **Bump versions in lockstep**:
   ```bash
   # Workspace Cargo version
   sed -i 's/^version = ".*"/version = "0.1.1"/' Cargo.toml
   # Node main package
   sed -i 's/"version": ".*"/"version": "0.1.1"/' crates/profanite-node/package.json
   # Node optional dependency versions point at the same triple
   # (package.json's optionalDependencies block — bump each to 0.1.1)
   # Python wheel version
   sed -i 's/^version = ".*"/version = "0.1.1"/' crates/profanite-py/pyproject.toml
   ```
   (A `scripts/bump-version.py` wrapper is on the follow-up list.)

2. **Regenerate docs** so the version stamp in README matches the new tag:
   ```bash
   python3 scripts/sync-readme.py
   ```

3. **Verify green locally**:
   ```bash
   cargo test --workspace --exclude profanite-py --all-features
   cargo run -p profanite-bench -- full      # all gates must be green
   cargo publish --dry-run -p profanite-core --features all-langs --allow-dirty
   ```

4. **Commit + tag + push**:
   ```bash
   git commit -am "Release v0.1.1"
   git tag v0.1.1
   git push origin main v0.1.1
   ```

5. **Watch the Actions tab**. Three workflows fire on the tag:
   - `Release Node` — builds 6 `.node` binaries, uploads platform subpackages + main, ~15 min total
   - `Release Python` — builds ~8 wheels (Linux x2 × manylinux+musllinux, macOS x2, Windows) + sdist, uploads to PyPI, ~20 min
   - `Release crates.io` — uploads the core crate, ~5 min

## Post-release verification

From a clean environment (none of these should require compilation):

```bash
# Rust
cargo new --bin /tmp/profanite-verify && cd /tmp/profanite-verify
echo 'profanite-core = "0.1.1"' >> Cargo.toml
cargo build && cargo run

# Node
cd $(mktemp -d) && npm init -y && npm install @beatsphere/profanite
node -e 'const {Profanite} = require("@beatsphere/profanite"); console.log(new Profanite().containsProfanity("hello"))'

# Python
python3 -m venv /tmp/pv && source /tmp/pv/bin/activate && pip install profanite
python3 -c 'from profanite import Profanite; print(Profanite().contains_profanity("hello"))'
```

## Rolling back

- **crates.io**: cannot un-publish. Yank only: `cargo yank -p profanite-core --version 0.1.1`. New users can't depend on it; existing lockfiles keep working.
- **npm**: `npm unpublish profanite@0.1.1` is possible within 72 hours. After that, publish a patch with the fix.
- **PyPI**: same as crates.io. No un-publish. Yank via PyPI UI.

In all three registries, the correct response to a bad release is to publish a patched version, not to try to take the broken one down.
