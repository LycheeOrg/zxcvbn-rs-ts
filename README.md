# zxcvbn-wasm

[![CI](https://github.com/LycheeOrg/zxcvbn-rs-ts/actions/workflows/ci.yml/badge.svg)](https://github.com/LycheeOrg/zxcvbn-rs-ts/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/%40lycheeorg%2Fzxcvbn-wasm)](https://www.npmjs.com/package/@lycheeorg/zxcvbn-wasm)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/LycheeOrg/zxcvbn-rs-ts/badge)](https://scorecard.dev/viewer/?uri=github.com/LycheeOrg/zxcvbn-rs-ts)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A WebAssembly build of [`zxcvbn-rs`](https://github.com/shssoichiro/zxcvbn-rs) — the Rust
port of Dropbox's [zxcvbn](https://github.com/dropbox/zxcvbn) password strength
estimator — packaged for use from TypeScript/JavaScript.

Instead of re-implementing password scoring in JS, this compiles the real Rust crate to
Wasm and ships thin, typed bindings around it, so you get the same scoring behavior,
dictionaries and pattern matching as the Rust ecosystem, in the browser or in Node.

## Why

- **Same estimator as the Rust backend.** If your server already uses `zxcvbn-rs` (or
  you'd like to), the client-side check now scores passwords identically.
- **Small, sandboxed core.** The scoring logic runs in Wasm; the JS surface is a single
  typed function.
- **No reimplementation drift.** Dictionaries, spatial graphs, and scoring live upstream
  in `zxcvbn-rs`; this repo only adds the Wasm glue.

## Installation

```sh
npm install @lycheeorg/zxcvbn-wasm
```

## Usage

The package is built with `wasm-pack --target web`, i.e. it's an ES module that loads
its `.wasm` file itself — no bundler plugin required, though bundlers that understand
`new URL(..., import.meta.url)` (Vite, Webpack 5, Rollup) will bundle the `.wasm` file
for you automatically.

### Browser / bundler

```ts
import init, { zxcvbn } from "@lycheeorg/zxcvbn-wasm";

await init(); // fetches and instantiates the .wasm file once

const result = zxcvbn("correct horse battery staple", ["alice", "alice@example.com"]);

console.log(result.score); // 0 (weakest) – 4 (strongest)
console.log(result.crack_times_display.offline_slow_hashing_1e4_per_second);
console.log(result.feedback.warning, result.feedback.suggestions);
```

The second argument is a list of user-supplied inputs (username, email, name, etc.) that
are penalized if they show up in the password, since they're easy for an attacker to
guess. It's optional — pass `[]`, `null`, or omit it entirely.

### Node.js

`--target web` output expects to `fetch()` its own `.wasm` file, which doesn't exist in
Node. Read the file yourself and hand the bytes to `init()`:

```ts
import { readFile } from "node:fs/promises";
import init, { zxcvbn } from "@lycheeorg/zxcvbn-wasm";

const wasmUrl = import.meta.resolve("@lycheeorg/zxcvbn-wasm/zxcvbn_wasm_bg.wasm");
const wasmBytes = await readFile(new URL(wasmUrl));
await init({ module_or_path: wasmBytes });

const result = zxcvbn("hunter2", []);
```

## API

```ts
function zxcvbn(password: string, userInputs?: string[] | null): ZxcvbnResult;
```

Only the first 100 characters of `password` are evaluated (matching upstream
`zxcvbn-rs` behavior, and guarding against pathologically long input).

`ZxcvbnResult` (full definitions are shipped in the package's `.d.ts`):

| Field                  | Type                 | Description                                                     |
| ---------------------- | -------------------- | ---------------------------------------------------------------- |
| `score`                | `number`             | Overall strength, `0` (weakest) – `4` (strongest). `< 3` is weak. |
| `guesses`              | `number`             | Estimated number of guesses needed to crack the password.        |
| `guesses_log10`        | `number`             | `log10` of `guesses`.                                             |
| `calc_time_ms`         | `number`             | How long the estimate itself took to compute.                    |
| `crack_times_seconds`  | `CrackTimesSeconds`  | Estimated seconds to crack, per attack scenario.                 |
| `crack_times_display`  | `CrackTimesDisplay`  | Same estimates as human-readable strings, e.g. `"3 hours"`.      |
| `feedback`             | `Feedback`           | `{ warning: string \| null, suggestions: string[] }`.             |
| `sequence`              | `MatchSequenceItem[]` | The patterns (dictionary words, dates, keyboard runs, …) that were matched and used to compute the guess count. |

`crack_times_seconds` / `crack_times_display` both have the same four keys, covering
four attack scenarios: `online_throttling_100_per_hour`,
`online_no_throttling_10_per_second`, `offline_slow_hashing_1e4_per_second`, and
`offline_fast_hashing_1e10_per_second`.

## Repository layout

```
rust/               the wasm-bindgen wrapper crate around the `zxcvbn` crate
  src/lib.rs        the entire public API surface (one function, plus TS type declarations)
  test/             a Node smoke test run against the built package in CI
.github/
  workflows/
    ci.yml                  tests, lints, wasm build, smoke test on every push/PR
    publish.yml             builds and publishes to npm on a GitHub Release
    dependency-review.yml   flags newly-introduced vulnerable dependencies on PRs
    scorecard.yml           OpenSSF Scorecard supply-chain analysis
  dependabot.yml    keeps GitHub Actions and Cargo dependencies up to date
  CODEOWNERS        default reviewers for pull requests
  FUNDING.yml       sponsor button configuration
```

The published npm package **is** the `wasm-pack` build output (`rust/pkg`); there's no
separate hand-written TS package to keep in sync.

## Development

Requires a Rust toolchain, the `wasm32-unknown-unknown` target, and
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

cd rust
cargo test                                                    # unit tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check

wasm-pack build --target web --out-dir pkg --release --scope lycheeorg
node test/smoke.mjs                                           # exercises the built package
```

## Security

See [SECURITY.md](SECURITY.md) for how to report a vulnerability. Every PR is checked
by [Dependency Review](.github/workflows/dependency-review.yml) and the repo is
continuously assessed by [OpenSSF Scorecard](.github/workflows/scorecard.yml).

## Credits & license

This project is a thin Wasm wrapper; all password-scoring logic comes from
[`zxcvbn-rs`](https://github.com/shssoichiro/zxcvbn-rs) by Josh Holmer, itself a port of
Dropbox's original [zxcvbn](https://github.com/dropbox/zxcvbn) by Dan Wheeler.

MIT licensed — see [LICENSE](LICENSE) for the wrapper and upstream crate's license text.
