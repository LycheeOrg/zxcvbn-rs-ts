// Runtime smoke test for the wasm-pack `--target web` output in `rust/pkg`.
//
// Run after `wasm-pack build --target web --out-dir pkg`:
//   node test/smoke.mjs
//
// This is a plain Node script rather than a `cargo test`/`wasm-bindgen-test`
// because it exists to catch integration bugs in the *published* JS/wasm glue
// (module loading, JSON round-tripping across the wasm boundary), not to
// re-test the zxcvbn scoring logic itself, which is covered by `cargo test`.

import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

import init, { zxcvbn, setPanicHook } from '../pkg/zxcvbn_wasm.js';

const wasmBytes = await readFile(new URL('../pkg/zxcvbn_wasm_bg.wasm', import.meta.url));
await init({ module_or_path: wasmBytes });
setPanicHook();

// Weak, common password.
{
	const result = zxcvbn('password', []);
	assert.equal(result.score, 0);
	assert.equal(typeof result.guesses, 'number');
	assert.ok(result.feedback.warning);
	assert.ok(result.sequence.length > 0);
}

// User inputs make an otherwise-plausible password weaker.
{
	const withoutInputs = zxcvbn('bruce1979', []);
	const withInputs = zxcvbn('bruce1979', ['bruce', '1979']);
	assert.ok(withInputs.guesses <= withoutInputs.guesses);
}

// `user_inputs` is optional and accepts null/undefined.
{
	assert.doesNotThrow(() => zxcvbn('correcthorsebatterystaple', null));
	assert.doesNotThrow(() => zxcvbn('correcthorsebatterystaple', undefined));
	assert.doesNotThrow(() => zxcvbn('correcthorsebatterystaple'));
}

// Very long/entropic passwords must not throw even though some internal
// per-match guess counts saturate at u64::MAX (see lib.rs doc comment).
{
	const result = zxcvbn('Tr0ub4dour&3zebraCanyonPlateau!92xQ7#mLwZk', []);
	assert.equal(result.score, 4);
	assert.equal(typeof result.guesses, 'number');
}

// Empty password.
{
	const result = zxcvbn('', []);
	assert.equal(result.score, 0);
	assert.equal(result.guesses, 0);
}

console.log('smoke test passed');
