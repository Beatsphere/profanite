/* eslint-disable */
/**
 * Platform-aware loader for the profanite native module.
 *
 * Resolution order:
 *   1. Published platform subpackages `@profanite/<platform>-<arch>-<abi>`,
 *      installed automatically via `optionalDependencies` in package.json.
 *   2. A local `profanite.<platform>-<arch>(-<abi>).node` file next to this
 *      index.js — what `napi build --platform` produces.
 *   3. A local `cargo build -p profanite-node` artifact anywhere under a
 *      sibling `target/` directory — dev-mode fallback.
 *
 * If nothing resolves, we throw a descriptive error pointing at the
 * install command the user should run.
 */

const { existsSync } = require('fs');
const { join, resolve } = require('path');
const { platform, arch } = process;

function detectLibc() {
  if (platform !== 'linux') return null;
  // Heuristic: /etc/alpine-release exists on Alpine (and Alpine-derived
  // distros), which use musl. Everything else we treat as glibc.
  if (existsSync('/etc/alpine-release')) return 'musl';
  // On other distros, the musl libc may still be present in containers.
  // Fall back to checking ldd output if /etc/alpine-release isn't there.
  try {
    const { execSync } = require('child_process');
    const lddOut = execSync('ldd --version 2>&1 || true', { encoding: 'utf8' });
    if (/musl/i.test(lddOut)) return 'musl';
  } catch {
    // ldd not present — default to gnu.
  }
  return 'gnu';
}

function triple() {
  const parts = [platform, arch];
  if (platform === 'linux') {
    parts.push(detectLibc());
  } else if (platform === 'win32') {
    parts.push('msvc');
  }
  return parts.filter(Boolean).join('-');
}

function tryLoad() {
  const id = triple();

  // 1. Subpackage installed via optionalDependencies.
  try {
    return require(`@beatsphere/profanite-${id}`);
  } catch {
    // fall through
  }

  // 2. Local .node sibling (napi build output, or staged during dev CI).
  const localCandidates = [
    join(__dirname, `profanite.${id}.node`),
    join(__dirname, `profanite.${platform}-${arch}.node`),
    join(__dirname, 'profanite.node'),
  ];
  for (const path of localCandidates) {
    if (existsSync(path)) return require(path);
  }

  // 3. cargo build artifact (dev-mode in the monorepo). cargo emits
  // libprofanite_node.so/.dylib/.dll, but `require()` only loads addons
  // through `.node` — so we go via process.dlopen, which doesn't care
  // about the extension.
  let dir = __dirname;
  for (let i = 0; i < 6; i++) {
    for (const profile of ['release', 'debug']) {
      const libExt = platform === 'win32' ? '.dll' : platform === 'darwin' ? '.dylib' : '.so';
      const prefix = platform === 'win32' ? '' : 'lib';
      const candidate = resolve(dir, 'target', profile, `${prefix}profanite_node${libExt}`);
      if (existsSync(candidate)) {
        const m = { exports: {} };
        process.dlopen(m, candidate);
        return m.exports;
      }
    }
    dir = resolve(dir, '..');
  }

  return null;
}

const nativeBinding = tryLoad();

if (!nativeBinding) {
  throw new Error(
    `profanite: native binding not found for ${triple()}.\n` +
      '  If you installed from npm, try reinstalling: `npm install profanite --force`.\n' +
      '  If you are developing locally, run `cargo build -p profanite-node` from the repo root.'
  );
}

module.exports = {
  Profanite: nativeBinding.Profanite,
};
