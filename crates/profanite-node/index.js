/* eslint-disable */
/**
 * Platform-aware loader for the profanite native module.
 *
 * We don't publish individual per-triple subpackages in v0.1. Instead we
 * ship one source crate and a single .node binary that's built locally
 * via `npm run build` (or downloaded as a prebuild from CI — see
 * package.json#napi.triples for the targets we care about).
 *
 * The loader probes a few well-known locations so the same index.js
 * works whether you:
 *   - just ran `npm run build` in this package (file lands at
 *     `./profanite.<triple>.node`)
 *   - installed a prebuilt binary (same layout)
 *   - built via `cargo build -p profanite-node` for local development
 *     (binary is under target/debug/ or target/release/)
 */

const { existsSync, readFileSync } = require('fs');
const { join, resolve } = require('path');
const { platform, arch } = process;

function loadLocalBuild() {
  const localPath = join(__dirname, `profanite.${platform}-${arch}.node`);
  if (existsSync(localPath)) {
    return require(localPath);
  }
  const genericPath = join(__dirname, 'profanite.node');
  if (existsSync(genericPath)) {
    return require(genericPath);
  }
  return null;
}

function loadCargoBuild() {
  // Walk up from this file to find a target/ directory (monorepo dev mode).
  let dir = __dirname;
  for (let i = 0; i < 6; i++) {
    for (const profile of ['release', 'debug']) {
      const libExt = platform === 'win32' ? '.dll' : platform === 'darwin' ? '.dylib' : '.so';
      const prefix = platform === 'win32' ? '' : 'lib';
      const candidate = resolve(dir, 'target', profile, `${prefix}profanite_node${libExt}`);
      if (existsSync(candidate)) {
        return require(candidate);
      }
    }
    dir = resolve(dir, '..');
  }
  return null;
}

const nativeBinding = loadLocalBuild() || loadCargoBuild();

if (!nativeBinding) {
  throw new Error(
    'profanite native binding not found. Run `npm run build` in this package, or `cargo build -p profanite-node` for local development.'
  );
}

module.exports = {
  Profanite: nativeBinding.Profanite,
};
