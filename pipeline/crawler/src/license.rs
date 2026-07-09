//! Decides whether a crate-version's declared SPDX `license` string (from
//! the pinned db-dump's `versions.csv`) permits unmodified redistribution,
//! so `fetch` can skip crates the corpus isn't allowed to mirror before
//! spending any bandwidth on them.
//!
//! A license is ACCEPTED if `spdx::Expression::parse` (after normalizing a
//! few known non-strict Rust-ecosystem conventions) succeeds and every term
//! resolves to a real SPDX license id. That covers permissive licenses,
//! copyleft licenses (GPL/LGPL/AGPL family -- redistributable, just with
//! reciprocal conditions), and OR-expressions mixing the two. Everything
//! else (blank, crates.io's "non-standard" placeholder, bare `LicenseRef-*`
//! terms with no resolvable open fallback, and any other unparseable
//! string) is REJECTED with a reason string for `license_exclusions.csv`.

use regex::Regex;
use std::sync::LazyLock;

static SLASH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*/\s*").unwrap());
static LOWER_CONNECTIVE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(or|and)\b").unwrap());
static PLUS_SUFFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([A-Za-z0-9][A-Za-z0-9.\-]*)\+").unwrap());

/// Normalizes known Rust-ecosystem non-strict-SPDX conventions into strict
/// SPDX expression syntax:
///   - "MIT/Apache-2.0", "MIT / Apache-2.0"  -> "MIT OR Apache-2.0"
///   - "MIT or Apache-2.0"                   -> "MIT OR Apache-2.0"
///   - "GPL-3.0+", "GPL-2.0+ WITH X"         -> "GPL-3.0-or-later", "GPL-2.0-or-later WITH X"
fn normalize(s: &str) -> String {
    let s = SLASH_RE.replace_all(s, " OR ");
    let s = LOWER_CONNECTIVE_RE.replace_all(&s, |caps: &regex::Captures| caps[1].to_uppercase());
    let s = PLUS_SUFFIX_RE.replace_all(&s, "$1-or-later");
    s.into_owned()
}

/// Returns `Ok(())` if redistributable, `Err(reason)` otherwise.
pub fn check(license_str: &str) -> Result<(), &'static str> {
    let mut parsed = spdx::Expression::parse(license_str);
    if parsed.is_err() {
        let normalized = normalize(license_str);
        if normalized != license_str {
            if let Ok(expr) = spdx::Expression::parse(&normalized) {
                parsed = Ok(expr);
            }
        }
    }

    match parsed {
        Ok(expr) => {
            let mut any_non_spdx = false;
            for node in expr.iter() {
                if let spdx::expression::ExprNode::Req(req) = node {
                    if let spdx::LicenseItem::Other { .. } = &req.req.license {
                        any_non_spdx = true;
                    }
                }
            }
            if any_non_spdx {
                Err("unresolvable-license-ref")
            } else {
                Ok(())
            }
        }
        Err(_) => {
            if license_str == "non-standard" {
                Err("non-standard-placeholder")
            } else if license_str.is_empty() {
                Err("blank")
            } else {
                Err("unparseable-spdx")
            }
        }
    }
}
