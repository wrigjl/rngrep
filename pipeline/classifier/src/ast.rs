use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::visit::Visit;
use syn::{Attribute, File, ItemFn, ItemImpl, ItemMod, Macro};

/// Instruction-set family a finding belongs to. The pipeline is
/// ISA-agnostic; x86 (RDRAND/RDSEED) and AArch64 (RNDR/RNDRRS via MRS) are
/// detected in one corpus walk and split into separate output files so the
/// x86 dataset stays byte-identical while ARM is reported alongside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Isa {
    X86,
    Arm,
}

/// A single source-level finding inside one `.rs` file.
#[derive(Debug, Clone)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub isa: Isa,
    pub kind: Kind,
    /// Specific intrinsic name (`_rdrand64_step`, `__rndr`) or asm mnemonic
    /// (`rdrand`, `rndr`).
    pub detail: String,
    /// Stack of `#[cfg(...)]` predicates from enclosing items (outermost first).
    pub cfg_gates: Vec<String>,
    /// True if the enclosing function or any parent item is `#[cfg(test)]`,
    /// inside `mod tests`, or marked `#[test]`/`#[bench]`.
    pub in_test_or_bench: bool,
    /// True if the enclosing function contains an
    /// `is_x86_feature_detected!("rdrand"|"rdseed")` call.
    pub runtime_gated: bool,
    /// True if the enclosing function is decorated with
    /// `#[target_feature(enable = "rdrand"|"rdseed")]`. This is a
    /// compile-time contract: the function unconditionally emits the
    /// intrinsic and requires its caller to ensure the feature is
    /// available, which in practice means the binary is compiled with
    /// `+rdrand` (no runtime CPUID check needed at this call site).
    pub target_feature_attr: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// `_rdrand{16,32,64}_step` or `_rdseed{16,32,64}_step` intrinsic call.
    Intrinsic,
    /// `asm!`, `llvm_asm!`, or `global_asm!` block emitting RDRAND/RDSEED.
    InlineAsm,
}

/// Parse a Rust source file and return all findings. Returns `None` if the
/// file is not valid Rust (we silently drop these — many `.rs` files in
/// crates are `tests/data/*.rs` fixtures that don't parse standalone).
pub fn analyze(file_path: &str, src: &str) -> Option<Vec<Finding>> {
    let parsed: File = syn::parse_file(src).ok()?;
    let aliases = collect_intrinsic_aliases(&parsed);
    let mut v = Visitor {
        file: file_path,
        findings: Vec::new(),
        cfg_stack: Vec::new(),
        test_stack: Vec::new(),
        fn_runtime_gated_stack: Vec::new(),
        fn_target_feature_stack: Vec::new(),
        intrinsic_aliases: aliases,
    };
    v.visit_file(&parsed);
    // Dedupe findings at the same (isa, line, kind, detail): the AST visitor
    // and the in-macro token scanner can both fire for one syntactic site.
    let mut seen = HashSet::new();
    v.findings.retain(|f| {
        let key = (f.isa, f.line, f.kind, f.detail.clone());
        seen.insert(key)
    });
    Some(v.findings)
}

/// Collect `use ... as alias` aliases that resolve to an RDRAND/RDSEED
/// intrinsic. Looks at both top-level/module-level `use` items and renames
/// embedded inside macro bodies (e.g. `cfg_if! { use ... as foo }`), which
/// is the common idiom for platform-specific intrinsic plumbing.
fn collect_intrinsic_aliases(file: &File) -> HashMap<String, Isa> {
    // Scan the whole file as a token stream so `<intrinsic> as <alias>`
    // renames are caught wherever they appear: top-level, module-level,
    // grouped (`{a as b, c as d}`, e.g. mbedtls), inside macro bodies, and --
    // crucially -- function-local `use ...::_rd.._step as name;` statements,
    // which item-level walking skips (e.g. nanorand).
    let mut out = HashMap::new();
    collect_token_renames(&file.to_token_stream(), &mut out);
    out
}

/// Find `<intrinsic> as <ident>` patterns anywhere in a token stream,
/// remembering the ISA the aliased intrinsic belongs to.
fn collect_token_renames(toks: &TokenStream, out: &mut HashMap<String, Isa>) {
    let flat: Vec<TokenTree> = flatten(toks);
    let mut i = 0;
    while i + 2 < flat.len() {
        if let (TokenTree::Ident(src), TokenTree::Ident(kw), TokenTree::Ident(dst)) =
            (&flat[i], &flat[i + 1], &flat[i + 2])
            && let Some(isa) = intrinsic_isa(&src.to_string())
            && *kw == "as"
        {
            out.insert(dst.to_string(), isa);
        }
        i += 1;
    }
}

fn flatten(toks: &TokenStream) -> Vec<TokenTree> {
    let mut out = Vec::new();
    for tt in toks.clone() {
        match tt {
            TokenTree::Group(g) => {
                out.extend(flatten(&g.stream()));
            }
            other => out.push(other),
        }
    }
    out
}

struct Visitor<'a> {
    file: &'a str,
    findings: Vec<Finding>,
    cfg_stack: Vec<String>,
    test_stack: Vec<bool>,
    // Per-function runtime-detect and target_feature flags, held per ISA as
    // `(x86, arm)` so a finding is annotated with the gate for *its* ISA
    // (an aarch64 call site should not inherit an x86 `is_x86_feature_detected`
    // in the same function, and vice versa).
    fn_runtime_gated_stack: Vec<(bool, bool)>,
    fn_target_feature_stack: Vec<(bool, bool)>,
    intrinsic_aliases: HashMap<String, Isa>,
}

impl<'a> Visitor<'a> {
    fn push_attrs(&mut self, attrs: &[Attribute]) -> (usize, bool) {
        let mut pushed_cfgs = 0;
        let mut test_marker = false;
        for a in attrs {
            if a.path().is_ident("cfg") {
                // Real conditional-compilation gate.
                self.cfg_stack.push(attr_to_string(a));
                pushed_cfgs += 1;
                if cfg_predicate_mentions_test(a) {
                    test_marker = true;
                }
            } else if a.path().is_ident("cfg_attr") {
                // `#[cfg_attr(pred, inner)]` conditionally *applies* `inner`; it
                // does not gate whether the item is compiled. Reading its
                // predicate as a compile gate misfires on
                // `doc(cfg(target_feature = "rdrand"))` (docs-only, e.g.
                // safe_arch); reading it as test membership misfires on
                // `cfg_attr(test, assert_instr(...))` (a stdarch bench helper on
                // a normal public fn, e.g. vektor). So `cfg_attr` contributes to
                // neither the cfg gate stack nor the test marker.
            } else if attr_path_is(a, "test") || attr_path_is(a, "bench") {
                test_marker = true;
            }
        }
        self.test_stack.push(test_marker);
        (pushed_cfgs, test_marker)
    }

    fn pop_attrs(&mut self, pushed_cfgs: usize) {
        for _ in 0..pushed_cfgs {
            self.cfg_stack.pop();
        }
        self.test_stack.pop();
    }

    fn in_test(&self) -> bool {
        self.test_stack.iter().any(|b| *b)
    }

    fn runtime_gated(&self, isa: Isa) -> bool {
        let (x86, arm) = self.fn_runtime_gated_stack.last().copied().unwrap_or((false, false));
        match isa {
            Isa::X86 => x86,
            Isa::Arm => arm,
        }
    }

    fn target_feature_attr(&self, isa: Isa) -> bool {
        let (x86, arm) = self.fn_target_feature_stack.last().copied().unwrap_or((false, false));
        match isa {
            Isa::X86 => x86,
            Isa::Arm => arm,
        }
    }

    fn record(&mut self, isa: Isa, line: usize, kind: Kind, detail: String) {
        self.findings.push(Finding {
            file: self.file.to_string(),
            line,
            isa,
            kind,
            detail,
            cfg_gates: self.cfg_stack.clone(),
            in_test_or_bench: self.in_test(),
            runtime_gated: self.runtime_gated(isa),
            target_feature_attr: self.target_feature_attr(isa),
        });
    }
}

impl<'ast, 'a> Visit<'ast> for Visitor<'a> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let (pushed, _) = self.push_attrs(&node.attrs);
        let gated = body_has_runtime_detect(&node.block.to_token_stream());
        let tfa = attrs_enable_target_feature(&node.attrs);
        self.fn_runtime_gated_stack.push(gated);
        self.fn_target_feature_stack.push(tfa);
        syn::visit::visit_item_fn(self, node);
        self.fn_target_feature_stack.pop();
        self.fn_runtime_gated_stack.pop();
        self.pop_attrs(pushed);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let (pushed, _) = self.push_attrs(&node.attrs);
        let gated = body_has_runtime_detect(&node.block.to_token_stream());
        let tfa = attrs_enable_target_feature(&node.attrs);
        self.fn_runtime_gated_stack.push(gated);
        self.fn_target_feature_stack.push(tfa);
        syn::visit::visit_impl_item_fn(self, node);
        self.fn_target_feature_stack.pop();
        self.fn_runtime_gated_stack.pop();
        self.pop_attrs(pushed);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let (pushed, _) = self.push_attrs(&node.attrs);
        let gated = node
            .default
            .as_ref()
            .map(|b| body_has_runtime_detect(&b.to_token_stream()))
            .unwrap_or((false, false));
        let tfa = attrs_enable_target_feature(&node.attrs);
        self.fn_runtime_gated_stack.push(gated);
        self.fn_target_feature_stack.push(tfa);
        syn::visit::visit_trait_item_fn(self, node);
        self.fn_target_feature_stack.pop();
        self.fn_runtime_gated_stack.pop();
        self.pop_attrs(pushed);
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        let mut test_marker = node.ident == "tests" || node.ident == "test";
        for a in &node.attrs {
            if a.path().is_ident("cfg") && cfg_predicate_mentions_test(a) {
                test_marker = true;
            }
        }
        let (pushed, _) = self.push_attrs(&node.attrs);
        if test_marker {
            *self.test_stack.last_mut().unwrap() = true;
        }
        syn::visit::visit_item_mod(self, node);
        self.pop_attrs(pushed);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let (pushed, _) = self.push_attrs(&node.attrs);
        syn::visit::visit_item_impl(self, node);
        self.pop_attrs(pushed);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        // Lowest-level macro hook: fires for macros in expression, item, AND
        // statement position. A statement-position `asm!("rdrand ...");`
        // (Stmt::Macro) routes here but never through visit_expr_macro, which
        // is why inline-asm-in-statement crates (bitcoin-random, lcpfs,
        // virtfw-libhw, cocoon-tpm-crypto) were missed.
        self.visit_mac(node);
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        // Intrinsic (or alias) referenced as a *value*, e.g. passed as a
        // function pointer: `call_cpu_rng(n, _rdrand_step, cast)` (mbedtls).
        // A direct call also visits its callee path here; the (line, kind,
        // detail) dedupe in `analyze` collapses the duplicate.
        if let Some(last) = node.path.segments.last() {
            let name = last.ident.to_string();
            let line = last.ident.span().start().line;
            if let Some(isa) = intrinsic_isa(&name) {
                self.record(isa, line, Kind::Intrinsic, name);
            } else if let Some(&isa) = self.intrinsic_aliases.get(&name) {
                self.record(isa, line, Kind::Intrinsic, format!("alias:{name}"));
            }
        }
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*node.func
            && let Some(last) = p.path.segments.last()
        {
            let name = last.ident.to_string();
            let line = last.ident.span().start().line;
            if let Some(isa) = intrinsic_isa(&name) {
                self.record(isa, line, Kind::Intrinsic, name);
            } else if let Some(&isa) = self.intrinsic_aliases.get(&name) {
                self.record(isa, line, Kind::Intrinsic, format!("alias:{name}"));
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

impl<'a> Visitor<'a> {
    fn visit_mac(&mut self, mac: &Macro) {
        let Some(last) = mac.path.segments.last() else {
            return;
        };
        let name = last.ident.to_string();
        let line = last.ident.span().start().line;
        if name == "asm" || name == "llvm_asm" || name == "global_asm" {
            for (mnem, isa) in scan_asm_mnemonics(&mac.tokens) {
                self.record(isa, line, Kind::InlineAsm, mnem);
            }
            return;
        }
        // Generic macro body: token-stream scan for intrinsic identifiers
        // (literal names or known local aliases). This catches uses inside
        // `cfg_if!`, `paste!`, declarative macros, etc., which `syn`'s
        // structural walk does not descend into.
        let mut hits: Vec<(String, usize)> = Vec::new();
        scan_tokens_for_intrinsics(&mac.tokens, &self.intrinsic_aliases, &mut |id, l| {
            hits.push((id, l));
        });
        for (ident, l) in hits {
            let (isa, detail) = if let Some(isa) = intrinsic_isa(&ident) {
                (isa, ident)
            } else {
                // Must be a known alias (the scanner only emits intrinsics or
                // aliases); default to X86 if somehow absent.
                let isa = self.intrinsic_aliases.get(&ident).copied().unwrap_or(Isa::X86);
                (isa, format!("alias:{ident}"))
            };
            // Use macro invocation line as a fallback when the span has no
            // location info.
            let l = if l > 0 { l } else { line };
            self.record(isa, l, Kind::Intrinsic, detail);
        }
    }
}

fn scan_tokens_for_intrinsics(
    toks: &TokenStream,
    aliases: &HashMap<String, Isa>,
    sink: &mut dyn FnMut(String, usize),
) {
    let tts: Vec<TokenTree> = toks.clone().into_iter().collect();
    for i in 0..tts.len() {
        match &tts[i] {
            TokenTree::Ident(id) => {
                let name = id.to_string();
                if !(intrinsic_isa(&name).is_some() || aliases.contains_key(&name)) {
                    continue;
                }
                // A following `(...)` is a direct call. Otherwise still record
                // the intrinsic when used as a *value* inside a macro -- e.g.
                // passed as a function pointer in
                // `generate!(u16, ...::_rdrand16_step)` (hyper-thread-random).
                // But skip both sides of a `use X as Y` rename, which is an
                // alias *definition*, not a use: `_rd.._step as name` (next
                // token `as`) and `... as rdrand_step` (previous token `as`,
                // e.g. getrandom's cfg_if! plumbing). Aliases are collected
                // separately.
                let next = tts.get(i + 1);
                let is_call = matches!(next, Some(TokenTree::Group(g))
                    if g.delimiter() == proc_macro2::Delimiter::Parenthesis);
                let next_is_as = matches!(next, Some(TokenTree::Ident(kw)) if *kw == "as");
                let prev_is_as =
                    i > 0 && matches!(&tts[i - 1], TokenTree::Ident(kw) if *kw == "as");
                if is_call || (!next_is_as && !prev_is_as) {
                    sink(name, id.span().start().line);
                }
            }
            TokenTree::Group(g) => {
                scan_tokens_for_intrinsics(&g.stream(), aliases, sink);
            }
            _ => {}
        }
    }
}

fn is_intrinsic(name: &str) -> bool {
    matches!(
        name,
        "_rdrand16_step" | "_rdrand32_step" | "_rdrand64_step" |
        "_rdseed16_step" | "_rdseed32_step" | "_rdseed64_step"
    )
}

/// AArch64 FEAT_RNG intrinsics (`core::arch::aarch64`). Both read a hardware
/// entropy source via an `MRS` of the RNDR / RNDRRS system registers. As of
/// stable 1.95 these are still behind an unstable `stdarch` gate, so most
/// real emitters use inline `asm!` instead; the names are still worth
/// detecting for crates built on nightly.
fn is_arm_intrinsic(name: &str) -> bool {
    matches!(name, "__rndr" | "__rndrrs")
}

/// ISA an intrinsic name belongs to, or `None` if it is not an entropy
/// intrinsic.
fn intrinsic_isa(name: &str) -> Option<Isa> {
    if is_intrinsic(name) {
        Some(Isa::X86)
    } else if is_arm_intrinsic(name) {
        Some(Isa::Arm)
    } else {
        None
    }
}

fn body_has_runtime_detect(toks: &TokenStream) -> (bool, bool) {
    // Cheap: stringify and look for the canonical runtime feature-detection
    // macros. Returns `(x86, arm)`. On x86 the guard is
    // `is_x86_feature_detected!("rdrand"|"rdseed")`; on AArch64 it is
    // `is_aarch64_feature_detected!("rand")` (FEAT_RNG). Raw `__cpuid` /
    // system-register probes are rarer and accepted as a first-cut
    // false-negative on both.
    let s = toks.to_string();
    let x86 = s.contains("is_x86_feature_detected ! (\"rdrand\"")
        || s.contains("is_x86_feature_detected ! (\"rdseed\"");
    let arm = s.contains("is_aarch64_feature_detected ! (\"rand\"");
    (x86, arm)
}

fn scan_asm_mnemonics(toks: &TokenStream) -> Vec<(String, Isa)> {
    // Inline-asm bodies are string literals inside the macro. Stringify the
    // token stream and look for the mnemonics. Case-insensitive.
    let s = toks.to_string().to_ascii_lowercase();
    let mut out: Vec<(String, Isa)> = Vec::new();
    if contains_word(&s, "rdrand") {
        out.push(("rdrand".into(), Isa::X86));
    }
    if contains_word(&s, "rdseed") {
        out.push(("rdseed".into(), Isa::X86));
    }
    // AArch64 RNDR / RNDRRS. There is no standalone instruction: both are
    // read with `MRS <Xt>, RNDR|RNDRRS`, either by the named system register
    // (`mrs x0, rndr`) or by its raw encoding S3_3_C2_C4_0 / S3_3_C2_C4_1
    // (`mrs x0, s3_3_c2_c4_1`). Both assemble to the same MRS and disassemble
    // back to the RNDR/RNDRRS names (verified: `d5 3b 24 0N` / `d5 3b 24 2N`).
    // `contains_word` boundaries keep `rndr` from matching inside `rndrrs`.
    let packed: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if contains_word(&s, "rndrrs") || packed.contains("s3_3_c2_c4_1") {
        out.push(("rndrrs".into(), Isa::Arm));
    }
    if contains_word(&s, "rndr") || packed.contains("s3_3_c2_c4_0") {
        out.push(("rndr".into(), Isa::Arm));
    }
    // Opcode-byte-encoded forms: some crates emit `.byte 0x0f, 0xc7, 0xf0`
    // (optionally with a REX prefix) instead of the mnemonic, so the
    // assembler needn't know the instruction (e.g. bitcoin-random). RDRAND is
    // `0F C7 /6` (ModRM F0-F7); RDSEED is `0F C7 /7` (F8-FF). Detect the byte
    // sequence in the whitespace-stripped asm text.
    let mut from = 0;
    while let Some(rel) = packed[from..].find("0x0f,0xc7,0xf") {
        let idx = from + rel + "0x0f,0xc7,0xf".len();
        match packed[idx..].chars().next() {
            Some('0'..='7') if !out.iter().any(|(m, _)| m == "rdrand") => {
                out.push(("rdrand".into(), Isa::X86))
            }
            Some('8'..='9' | 'a'..='f') if !out.iter().any(|(m, _)| m == "rdseed") => {
                out.push(("rdseed".into(), Isa::X86))
            }
            _ => {}
        }
        from = idx;
    }
    out
}

fn contains_word(hay: &str, needle: &str) -> bool {
    let bytes = hay.as_bytes();
    let n = needle.as_bytes();
    if bytes.len() < n.len() {
        return false;
    }
    let mut i = 0;
    while i + n.len() <= bytes.len() {
        if &bytes[i..i + n.len()] == n {
            let before = if i == 0 { b' ' } else { bytes[i - 1] };
            let after = bytes.get(i + n.len()).copied().unwrap_or(b' ');
            let ok = !is_ident_byte(before) && !is_ident_byte(after);
            if ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn attr_path_is(a: &Attribute, name: &str) -> bool {
    a.path().is_ident(name)
}

fn attr_to_string(a: &Attribute) -> String {
    a.to_token_stream().to_string()
}

/// True if a real `#[cfg(...)]` predicate references the built-in `test`
/// cfg as a bare identifier. String-literal contents (e.g. a Cargo feature
/// named `integration-test`) are stripped first so they cannot
/// false-positive, and `fastest`/`latest`-style identifiers do not match
/// because the split is on identifier boundaries.
fn cfg_predicate_mentions_test(a: &Attribute) -> bool {
    let s = a.to_token_stream().to_string();
    let mut stripped = String::with_capacity(s.len());
    let mut in_str = false;
    for c in s.chars() {
        if c == '"' {
            in_str = !in_str;
            continue;
        }
        if !in_str {
            stripped.push(c);
        }
    }
    stripped
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|t| t == "test")
}

/// `#[target_feature(enable = "rdrand"|"rdseed")]` on a function declares
/// that the function's body may emit those intrinsics and shifts the
/// CPUID-check obligation to the caller. From the dissertation's
/// perspective, this is a compile-time contract — the function emits
/// RDRAND unconditionally.
fn attrs_enable_target_feature(attrs: &[Attribute]) -> (bool, bool) {
    let mut x86 = false;
    let mut arm = false;
    for a in attrs {
        if !a.path().is_ident("target_feature") {
            continue;
        }
        let s = a.to_token_stream().to_string();
        if s.contains("\"rdrand\"") || s.contains("\"rdseed\"") {
            x86 = true;
        }
        // AArch64 FEAT_RNG is enabled by `target_feature(enable = "rand")`.
        // The quotes delimit the token so this does not match inside
        // `"rdrand"`.
        if s.contains("\"rand\"") {
            arm = true;
        }
    }
    (x86, arm)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings(src: &str) -> Vec<Finding> {
        analyze("t.rs", src).expect("parses")
    }

    #[test]
    fn arm_intrinsic_runtime_gated() {
        let f = findings(
            r#"
            pub fn get() -> Option<u64> {
                if std::arch::is_aarch64_feature_detected!("rand") {
                    let mut x = 0u64;
                    unsafe { core::arch::aarch64::__rndr(&mut x) };
                    Some(x)
                } else { None }
            }
            "#,
        );
        let a: Vec<_> = f.iter().filter(|f| f.isa == Isa::Arm).collect();
        assert_eq!(a.len(), 1, "one ARM intrinsic finding");
        assert!(a[0].runtime_gated, "gated by is_aarch64_feature_detected");
        assert!(!a[0].target_feature_attr);
        assert_eq!(a[0].detail, "__rndr");
    }

    #[test]
    fn arm_asm_named_and_target_feature() {
        let f = findings(
            r#"
            #[target_feature(enable = "rand")]
            pub unsafe fn r() -> u64 {
                let v: u64;
                core::arch::asm!("mrs {0}, rndr", out(reg) v);
                v
            }
            "#,
        );
        let a: Vec<_> = f.iter().filter(|f| f.isa == Isa::Arm).collect();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].kind, Kind::InlineAsm);
        assert_eq!(a[0].detail, "rndr");
        assert!(a[0].target_feature_attr, "target_feature(enable=rand)");
    }

    #[test]
    fn arm_asm_encoded_sysreg_is_rndrrs() {
        let f = findings(
            r#"
            pub unsafe fn r() -> u64 {
                let v: u64;
                core::arch::asm!("mrs {0}, s3_3_c2_c4_1", out(reg) v);
                v
            }
            "#,
        );
        let a: Vec<_> = f.iter().filter(|f| f.isa == Isa::Arm).collect();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].detail, "rndrrs", "s3_3_c2_c4_1 is RNDRRS");
    }

    #[test]
    fn rndr_word_boundary_not_matched_inside_rndrrs() {
        // An asm block naming only rndrrs must not also record a bare rndr.
        let f = findings(
            r#"pub unsafe fn r() { core::arch::asm!("mrs x0, rndrrs"); }"#,
        );
        let mnems: Vec<_> = f.iter().map(|f| f.detail.as_str()).collect();
        assert!(mnems.contains(&"rndrrs"));
        assert!(!mnems.contains(&"rndr"));
    }

    #[test]
    fn x86_still_detected_and_isa_tagged() {
        let f = findings(
            r#"
            #[target_feature(enable = "rdrand")]
            pub unsafe fn r() -> u32 {
                let mut x = 0u32;
                core::arch::x86_64::_rdrand32_step(&mut x);
                x
            }
            "#,
        );
        let x: Vec<_> = f.iter().filter(|f| f.isa == Isa::X86).collect();
        assert_eq!(x.len(), 1);
        assert!(x[0].target_feature_attr);
        assert!(f.iter().all(|f| f.isa == Isa::X86), "no spurious ARM");
    }
}
