//! Out-of-scope detection for rr's recording domain.
//!
//! rr records Linux userspace processes via ptrace. Code that runs in
//! a hardware enclave, a guest kernel, or a unikernel/no-std OS is
//! never observable by rr regardless of how the RDRAND/RDSEED is
//! gated. This module flags such crates so they can be reported in a
//! separate `out_of_scope` category and not inflate the rr-relevant
//! population counts.
//!
//! Three sibling modules contribute detectors:
//!   - `sgx`: SGX enclaves and SGX-adjacent confidential-compute code
//!     (Enarx framework, Fortanix SGX, MobileCoin Attestation).
//!   - `kernel`: unikernels and no-std OS kernels (HermitOS, Asterinas).
//!   - `tee`: TEE-targeted libraries that aren't SGX-specific.
//!
//! Detector precedence is SGX > kernel > TEE on the rare crate that
//! could match more than one. Only the first match's reason and
//! signals are recorded.

use crate::{kernel, sgx, tee};

#[derive(Debug, Default, Clone)]
pub struct Detection {
    pub is_out_of_scope: bool,
    /// `sgx`, `kernel`, `tee`, or empty.
    pub reason: String,
    pub signals: Vec<String>,
}

pub fn detect(crate_name: &str, cargo_tomls: &[(&str, &str)]) -> Detection {
    if let Some(d) = sgx::detect(crate_name, cargo_tomls) {
        return Detection {
            is_out_of_scope: true,
            reason: "sgx".into(),
            signals: d,
        };
    }
    if let Some(d) = kernel::detect(crate_name, cargo_tomls) {
        return Detection {
            is_out_of_scope: true,
            reason: "kernel".into(),
            signals: d,
        };
    }
    if let Some(d) = tee::detect(crate_name, cargo_tomls) {
        return Detection {
            is_out_of_scope: true,
            reason: "tee".into(),
            signals: d,
        };
    }
    Detection::default()
}
