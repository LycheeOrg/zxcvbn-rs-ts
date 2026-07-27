//! WebAssembly bindings for [`zxcvbn`](https://docs.rs/zxcvbn), the Rust port of
//! Dropbox's zxcvbn password strength estimator.
//!
//! The crate exposes a single [`zxcvbn`] function to JavaScript/TypeScript that mirrors
//! the upstream Rust API, serialized as a plain JS object.

use serde::Serialize;
use wasm_bindgen::prelude::*;
use zxcvbn::time_estimates::CrackTimeSeconds;

/// Sets up better panic messages in the browser/Node console.
///
/// Optional: call this once at startup. Safe to call more than once.
#[wasm_bindgen(js_name = setPanicHook)]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[derive(Serialize)]
struct CrackTimesSeconds {
    online_throttling_100_per_hour: f64,
    online_no_throttling_10_per_second: f64,
    offline_slow_hashing_1e4_per_second: f64,
    offline_fast_hashing_1e10_per_second: f64,
}

#[derive(Serialize)]
struct CrackTimesDisplay {
    online_throttling_100_per_hour: String,
    online_no_throttling_10_per_second: String,
    offline_slow_hashing_1e4_per_second: String,
    offline_fast_hashing_1e10_per_second: String,
}

#[derive(Serialize)]
struct FeedbackResult {
    warning: Option<String>,
    suggestions: Vec<String>,
}

#[derive(Serialize)]
struct ZxcvbnResult {
    score: u8,
    guesses: f64,
    guesses_log10: f64,
    calc_time_ms: f64,
    crack_times_seconds: CrackTimesSeconds,
    crack_times_display: CrackTimesDisplay,
    feedback: FeedbackResult,
    sequence: Vec<zxcvbn::Match>,
}

fn seconds_as_f64(seconds: CrackTimeSeconds) -> f64 {
    match seconds {
        CrackTimeSeconds::Integer(i) => i as f64,
        CrackTimeSeconds::Float(f) => f,
    }
}

fn build_result(entropy: &zxcvbn::Entropy) -> ZxcvbnResult {
    let crack_times = entropy.crack_times();

    ZxcvbnResult {
        score: entropy.score().into(),
        guesses: entropy.guesses() as f64,
        guesses_log10: entropy.guesses_log10(),
        calc_time_ms: entropy.calculation_time().as_secs_f64() * 1000.0,
        crack_times_seconds: CrackTimesSeconds {
            online_throttling_100_per_hour: seconds_as_f64(
                crack_times.online_throttling_100_per_hour(),
            ),
            online_no_throttling_10_per_second: seconds_as_f64(
                crack_times.online_no_throttling_10_per_second(),
            ),
            offline_slow_hashing_1e4_per_second: seconds_as_f64(
                crack_times.offline_slow_hashing_1e4_per_second(),
            ),
            offline_fast_hashing_1e10_per_second: seconds_as_f64(
                crack_times.offline_fast_hashing_1e10_per_second(),
            ),
        },
        crack_times_display: CrackTimesDisplay {
            online_throttling_100_per_hour: crack_times
                .online_throttling_100_per_hour()
                .to_string(),
            online_no_throttling_10_per_second: crack_times
                .online_no_throttling_10_per_second()
                .to_string(),
            offline_slow_hashing_1e4_per_second: crack_times
                .offline_slow_hashing_1e4_per_second()
                .to_string(),
            offline_fast_hashing_1e10_per_second: crack_times
                .offline_fast_hashing_1e10_per_second()
                .to_string(),
        },
        feedback: entropy
            .feedback()
            .map(|feedback| FeedbackResult {
                warning: feedback.warning().map(|w| w.to_string()),
                suggestions: feedback
                    .suggestions()
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            })
            .unwrap_or(FeedbackResult {
                warning: None,
                suggestions: Vec::new(),
            }),
        sequence: entropy.sequence().to_vec(),
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface CrackTimesSeconds {
    online_throttling_100_per_hour: number;
    online_no_throttling_10_per_second: number;
    offline_slow_hashing_1e4_per_second: number;
    offline_fast_hashing_1e10_per_second: number;
}

export interface CrackTimesDisplay {
    online_throttling_100_per_hour: string;
    online_no_throttling_10_per_second: string;
    offline_slow_hashing_1e4_per_second: string;
    offline_fast_hashing_1e10_per_second: string;
}

export interface Feedback {
    warning: string | null;
    suggestions: string[];
}

/**
 * A single matched pattern that contributed to the password's guess count.
 *
 * The shape of the extra fields depends on `pattern`: `"dictionary"`, `"spatial"`,
 * `"repeat"`, `"sequence"`, `"regex"`, `"date"`, or `"bruteforce"`. See the zxcvbn-rs
 * docs for `zxcvbn::matching::patterns::MatchPattern` for the full field list per variant.
 */
export interface MatchSequenceItem {
    i: number;
    j: number;
    token: string;
    pattern: "dictionary" | "spatial" | "repeat" | "sequence" | "regex" | "date" | "bruteforce";
    guesses: number | null;
    [key: string]: unknown;
}

export interface ZxcvbnResult {
    /** Overall strength score from 0 (weakest) to 4 (strongest). Anything below 3 is considered too weak. */
    score: number;
    /** Estimated number of guesses needed to crack the password. */
    guesses: number;
    /** Order of magnitude (log10) of `guesses`. */
    guesses_log10: number;
    /** How long the estimate itself took to compute, in milliseconds. */
    calc_time_ms: number;
    crack_times_seconds: CrackTimesSeconds;
    crack_times_display: CrackTimesDisplay;
    feedback: Feedback;
    sequence: MatchSequenceItem[];
}
"#;

/// Estimate the strength of `password`, optionally penalizing patterns derived from
/// `user_inputs` (e.g. username, email, first name) since those are easy for an
/// attacker to guess.
///
/// Only the first 100 characters of `password` are considered.
///
/// The result is round-tripped through JSON rather than handed to the JS engine
/// field-by-field: some per-match guess counts saturate at `u64::MAX` for very long
/// or highly entropic passwords, which a direct conversion to a JS number would
/// reject outright. Serializing to JSON text first and letting `JSON.parse` handle
/// it mirrors ordinary `JSON.stringify`/`JSON.parse` precision loss for such values
/// instead of throwing.
#[wasm_bindgen(js_name = zxcvbn, unchecked_return_type = "ZxcvbnResult")]
pub fn zxcvbn(password: &str, user_inputs: Option<Vec<String>>) -> Result<JsValue, JsValue> {
    let user_inputs = user_inputs.unwrap_or_default();
    let user_inputs: Vec<&str> = user_inputs.iter().map(String::as_str).collect();

    let entropy = zxcvbn::zxcvbn(password, &user_inputs);
    let result = build_result(&entropy);

    let json = serde_json::to_string(&result).map_err(|err| JsValue::from_str(&err.to_string()))?;
    js_sys::JSON::parse(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_password_scores_zero() {
        let entropy = zxcvbn::zxcvbn("password", &[]);
        let result = build_result(&entropy);
        assert_eq!(result.score, 0);
        assert!(result.feedback.warning.is_some() || !result.feedback.suggestions.is_empty());
    }

    #[test]
    fn strong_password_scores_high() {
        let entropy = zxcvbn::zxcvbn("Tr0ub4dour&3zebraCanyonPlateau!92", &[]);
        let result = build_result(&entropy);
        assert!(result.score >= 3);
    }

    #[test]
    fn user_inputs_reduce_score() {
        let entropy_without = zxcvbn::zxcvbn("bruce1979", &[]);
        let entropy_with = zxcvbn::zxcvbn("bruce1979", &["bruce", "1979"]);
        let without = build_result(&entropy_without);
        let with = build_result(&entropy_with);
        assert!(with.guesses <= without.guesses);
    }

    #[test]
    fn crack_times_display_is_human_readable() {
        let entropy = zxcvbn::zxcvbn("password", &[]);
        let result = build_result(&entropy);
        assert!(!result
            .crack_times_display
            .online_throttling_100_per_hour
            .is_empty());
    }
}
