/// Cheap byte-level test for tokens that warrant AST parsing. Avoids syn's
/// parsing cost on the ~99% of files that obviously can't contain any
/// RDRAND/RDSEED reference.
pub fn looks_interesting(src: &[u8]) -> bool {
    // x86 RDRAND/RDSEED, plus AArch64 RNDR/RNDRRS. The `rndr` substring
    // covers the `__rndr`/`__rndrrs` intrinsics and the `rndr`/`rndrrs`
    // named system registers; `s3_3_c2_c4` covers the raw MRS encoding used
    // when the assembler does not know the sysreg name.
    has_ci(src, b"rdrand")
        || has_ci(src, b"rdseed")
        || has_ci(src, b"rndr")
        || has_ci(src, b"s3_3_c2_c4")
}

/// Case-insensitive subsequence check using memchr's fast scan for the
/// first byte. Pure ASCII; we don't case-fold any non-ASCII bytes.
fn has_ci(hay: &[u8], needle_lower: &[u8]) -> bool {
    if needle_lower.is_empty() || hay.len() < needle_lower.len() {
        return false;
    }
    let first = needle_lower[0];
    let first_upper = first.to_ascii_uppercase();
    let mut i = 0;
    while i + needle_lower.len() <= hay.len() {
        // Fast-skip to the next candidate start.
        let rest = &hay[i..];
        let lower_pos = memchr::memchr(first, rest);
        let upper_pos = memchr::memchr(first_upper, rest);
        let next = match (lower_pos, upper_pos) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => return false,
        };
        i += next;
        if i + needle_lower.len() > hay.len() {
            return false;
        }
        let slice = &hay[i..i + needle_lower.len()];
        if slice
            .iter()
            .zip(needle_lower.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == *b)
        {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_rdrand_lowercase() {
        assert!(looks_interesting(b"unsafe { _rdrand64_step(&mut x) }"));
    }

    #[test]
    fn finds_rdrand_uppercase() {
        assert!(looks_interesting(b"asm!(\"RDRAND %eax\")"));
    }

    #[test]
    fn rejects_unrelated() {
        assert!(!looks_interesting(b"fn main() { println!(\"hello\"); }"));
    }
}
