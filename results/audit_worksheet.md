# RDRAND classifier hand-audit worksheet

Snapshot 2026-05-13. One block per crate; verify each flagged site against the source context, then fill the VERDICT.

Verdict legend: OK (category correct) | WRONG->X (should be X) | NOTE (correct but caveat).


======================================================================
## concrete-csprng 0.4.1  --  classifier category: **compile_time**

--- src/seeders/implem/rdseed.rs:27  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         21 unsafe fn rdseed_random_m128() -> u128 {
         22     let mut rand1: u64 = 0;
         23     let mut rand2: u64 = 0;
         24     let mut output_bytes = [0u8; 16];
         25     unsafe {
         26         loop {
    >>   27             if core::arch::x86_64::_rdseed64_step(&mut rand1) == 1 {
         28                 break;
         29             }
         30         }
         31         loop {
         32             if core::arch::x86_64::_rdseed64_step(&mut rand2) == 1 {
         33                 break;

--- src/seeders/implem/rdseed.rs:32  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         26         loop {
         27             if core::arch::x86_64::_rdseed64_step(&mut rand1) == 1 {
         28                 break;
         29             }
         30         }
         31         loop {
    >>   32             if core::arch::x86_64::_rdseed64_step(&mut rand2) == 1 {
         33                 break;
         34             }
         35         }
         36     }
         37     output_bytes[0..8].copy_from_slice(&rand1.to_ne_bytes());
         38     output_bytes[8..16].copy_from_slice(&rand2.to_ne_bytes());

VERDICT: ____

======================================================================
## foundation_compact 0.2.0  --  classifier category: **compile_time**

--- src/entropy/backends/rdrand.rs:39  kind=intrinsic detail=alias:rdrand_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         33         // in via https://github.com/rust-lang/rust/commit/f2eb88b which is
         34         // expected to be included in 1.93.0. Since our MSRV is 1.85, we need to
         35         // use unsafe here and suppress the lint.
         36         //
         37         // TODO(MSRV 1.93): remove allow(unused_unsafe) and the unsafe block.
         38         #[allow(unused_unsafe)]
    >>   39         if unsafe { rdrand_step(&mut val) } == 1 {
         40             return Some(val);
         41         }
         42     }
         43     None
         44 }
         45 

VERDICT: ____

======================================================================
## getrandom 0.4.3  --  classifier category: **compile_time**

--- src/backends/rdrand.rs:39  kind=intrinsic detail=alias:rdrand_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         33         // in via https://github.com/rust-lang/rust/commit/f2eb88b which is
         34         // expected to be included in 1.93.0. Since our MSRV is 1.85, we need to
         35         // use unsafe here and suppress the lint.
         36         //
         37         // TODO(MSRV 1.93): remove allow(unused_unsafe) and the unsafe block.
         38         #[allow(unused_unsafe)]
    >>   39         if unsafe { rdrand_step(&mut val) } == 1 {
         40             return Some(val);
         41         }
         42     }
         43     None
         44 }
         45 

VERDICT: ____

======================================================================
## hardware-rust-crypto 1.0.1  --  classifier category: **compile_time**

--- src/random/entropy.rs:169  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "cpu-rng-reseed" , target_arch = "x86_64"))]
        163         for chunk in out.chunks_mut(8) {
        164             let mut value = 0_u64;
        165             let mut ready = false;
        166             for _ in 0..super::RETRIES {
        167                 // The rdseed target feature is enabled on this fn, so calling
        168                 // the intrinsic is in-context; `_rdseed64_step` takes `&mut`.
    >>  169                 if _rdseed64_step(&mut value) == 1 {
        170                     ready = true;
        171                     break;
        172                 }
        173                 core::hint::spin_loop();
        174             }
        175             if !ready {

VERDICT: ____

======================================================================
## hyper-thread-random 0.4.2  --  classifier category: **compile_time**

--- src/generate_hyper_thread_safe_random_u16.rs:10  kind=intrinsic detail=_rdrand16_step site_cat=compile_time in_test=false rt_gated=false cg_present=true tf_attr=false
    cfg_gates: # [cfg (all (target_arch = "x86" , target_feature = "rdrand"))]
          4 
          5 /// Generates a random u16 for the current hyper thread.
          6 #[cfg(all(target_arch = "x86", target_feature = "rdrand"))]
          7 #[inline(always)]
          8 pub fn generate_hyper_thread_safe_random_u16() -> u16
          9 {
    >>   10 	generate!(u16, ::std::arch::x86::_rdrand16_step)
         11 }
         12 
         13 /// Generates a random u16 for the current hyper thread.
         14 #[cfg(all(target_arch = "x86_64", target_feature = "rdrand"))]
         15 #[inline(always)]
         16 pub fn generate_hyper_thread_safe_random_u16() -> u16

--- src/generate_hyper_thread_safe_random_u16.rs:18  kind=intrinsic detail=_rdrand16_step site_cat=compile_time in_test=false rt_gated=false cg_present=true tf_attr=false
    cfg_gates: # [cfg (all (target_arch = "x86_64" , target_feature = "rdrand"))]
         12 
         13 /// Generates a random u16 for the current hyper thread.
         14 #[cfg(all(target_arch = "x86_64", target_feature = "rdrand"))]
         15 #[inline(always)]
         16 pub fn generate_hyper_thread_safe_random_u16() -> u16
         17 {
    >>   18 	generate!(u16, ::std::arch::x86_64::_rdrand16_step)
         19 }
         20 
         21 /// Generates a random u16 for the current hyper thread.
         22 #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "rdrand")))]
         23 pub fn generate_hyper_thread_safe_random_u16() -> u16
         24 {

--- src/generate_hyper_thread_safe_random_u32.rs:10  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=true tf_attr=false
    cfg_gates: # [cfg (all (target_arch = "x86" , target_feature = "rdrand"))]
          4 
          5 /// Generates a random u32 for the current hyper thread.
          6 #[cfg(all(target_arch = "x86", target_feature = "rdrand"))]
          7 #[inline(always)]
          8 pub fn generate_hyper_thread_safe_random_u32() -> u32
          9 {
    >>   10 	generate!(u32, ::std::arch::x86::_rdrand32_step)
         11 }
         12 
         13 /// Generates a random u32 for the current hyper thread.
         14 #[cfg(all(target_arch = "x86_64", target_feature = "rdrand"))]
         15 #[inline(always)]
         16 pub fn generate_hyper_thread_safe_random_u32() -> u32

--- src/generate_hyper_thread_safe_random_u32.rs:18  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=true tf_attr=false
    cfg_gates: # [cfg (all (target_arch = "x86_64" , target_feature = "rdrand"))]
         12 
         13 /// Generates a random u32 for the current hyper thread.
         14 #[cfg(all(target_arch = "x86_64", target_feature = "rdrand"))]
         15 #[inline(always)]
         16 pub fn generate_hyper_thread_safe_random_u32() -> u32
         17 {
    >>   18 	generate!(u32, ::std::arch::x86_64::_rdrand32_step)
         19 }
         20 
         21 /// Generates a random u32 for the current hyper thread.
         22 #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "rdrand")))]
         23 pub fn generate_hyper_thread_safe_random_u32() -> u32
         24 {

--- src/generate_hyper_thread_safe_random_u64.rs:10  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=true tf_attr=false
    cfg_gates: # [cfg (all (target_arch = "x86" , target_feature = "rdrand"))]
          4 
          5 /// Generates a random u64 for the current hyper thread.
          6 #[cfg(all(target_arch = "x86", target_feature = "rdrand"))]
          7 #[inline(always)]
          8 pub fn generate_hyper_thread_safe_random_u64() -> u64
          9 {
    >>   10 	generate!(u64, ::std::arch::x86::_rdrand64_step)
         11 }
         12 
         13 /// Generates a random u64 for the current hyper thread.
         14 #[cfg(all(target_arch = "x86_64", target_feature = "rdrand"))]
         15 #[inline(always)]
         16 pub fn generate_hyper_thread_safe_random_u64() -> u64

--- src/generate_hyper_thread_safe_random_u64.rs:18  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=true tf_attr=false
    cfg_gates: # [cfg (all (target_arch = "x86_64" , target_feature = "rdrand"))]
         12 
         13 /// Generates a random u64 for the current hyper thread.
         14 #[cfg(all(target_arch = "x86_64", target_feature = "rdrand"))]
         15 #[inline(always)]
         16 pub fn generate_hyper_thread_safe_random_u64() -> u64
         17 {
    >>   18 	generate!(u64, ::std::arch::x86_64::_rdrand64_step)
         19 }
         20 
         21 /// Generates a random u64 for the current hyper thread.
         22 #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "rdrand")))]
         23 pub fn generate_hyper_thread_safe_random_u64() -> u64
         24 {

VERDICT: ____

======================================================================
## lib-q-random 0.0.8  --  classifier category: **compile_time**

--- src/hardware_rng.rs:101  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "std" , target_arch = "x86_64"))]
         95         let remain = dest.len() - offset;
         96         if remain >= 8 {
         97             let mut v = 0u64;
         98             let mut ok = false;
         99             for _ in 0..RDRAND_RETRIES {
        100                 // SAFETY: `_rdrand64_step` is only used inside this `rdrand` target feature.
    >>  101                 if _rdrand64_step(&mut v) != 0 {
        102                     ok = true;
        103                     break;
        104                 }
        105             }
        106             if !ok {
        107                 return Err(());

--- src/hardware_rng.rs:115  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "std" , target_arch = "x86_64"))]
        109             dest[offset..offset + 8].copy_from_slice(&v.to_le_bytes());
        110             offset += 8;
        111         } else if remain >= 4 {
        112             let mut v = 0u32;
        113             let mut ok = false;
        114             for _ in 0..RDRAND_RETRIES {
    >>  115                 if _rdrand32_step(&mut v) != 0 {
        116                     ok = true;
        117                     break;
        118                 }
        119             }
        120             if !ok {
        121                 return Err(());

--- src/hardware_rng.rs:129  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "std" , target_arch = "x86_64"))]
        123             dest[offset..offset + 4].copy_from_slice(&v.to_le_bytes());
        124             offset += 4;
        125         } else {
        126             let mut v = 0u32;
        127             let mut ok = false;
        128             for _ in 0..RDRAND_RETRIES {
    >>  129                 if _rdrand32_step(&mut v) != 0 {
        130                     ok = true;
        131                     break;
        132                 }
        133             }
        134             if !ok {
        135                 return Err(());

--- src/hardware_rng.rs:156  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "std" , target_arch = "x86"))]
        150     let mut offset = 0usize;
        151     while offset < dest.len() {
        152         let remain = dest.len() - offset;
        153         let mut v = 0u32;
        154         let mut ok = false;
        155         for _ in 0..RDRAND_RETRIES {
    >>  156             if _rdrand32_step(&mut v) != 0 {
        157                 ok = true;
        158                 break;
        159             }
        160         }
        161         if !ok {
        162             return Err(());

--- src/hardware_rng.rs:178  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "std" , target_arch = "x86_64"))]
        172 #[target_feature(enable = "rdrand")]
        173 unsafe fn rdrand_fill_any_word(scratch: &mut [u8; 8]) -> RdrandFillResult {
        174     use core::arch::x86_64::_rdrand64_step;
        175 
        176     let mut v = 0u64;
        177     for _ in 0..RDRAND_RETRIES {
    >>  178         if _rdrand64_step(&mut v) != 0 {
        179             scratch.copy_from_slice(&v.to_le_bytes());
        180             return Ok(());
        181         }
        182     }
        183     Err(())
        184 }

--- src/hardware_rng.rs:193  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (all (feature = "std" , target_arch = "x86"))]
        187 #[target_feature(enable = "rdrand")]
        188 unsafe fn rdrand_fill_any_word(scratch: &mut [u8; 8]) -> RdrandFillResult {
        189     use core::arch::x86::_rdrand32_step;
        190 
        191     let mut v = 0u32;
        192     for _ in 0..RDRAND_RETRIES {
    >>  193         if _rdrand32_step(&mut v) != 0 {
        194             scratch[..4].copy_from_slice(&v.to_le_bytes());
        195             return Ok(());
        196         }
        197     }
        198     Err(())
        199 }

VERDICT: ____

======================================================================
## rmath 0.1.5  --  classifier category: **compile_time**

--- src/rand/default_rand_amd64.rs:23  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         17         Ok(DefaultRand {ph: PhantomData})
         18     }
         19     
         20     #[target_feature(enable = "rdrand")]
         21     unsafe fn gen_u32(&mut self) -> Result<u32> {
         22         let mut out = 0;
    >>   23         if march::_rdrand32_step(&mut out) > 0 {
         24             Ok(out)
         25         } else {
         26             Err(RandError::new(RandErrKind::NoNewRandNumberGen, ""))
         27         }
         28     }
         29 

--- src/rand/default_rand_amd64.rs:45  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (target_arch = "x86_64")]
         39     }
         40 
         41     #[cfg(target_arch = "x86_64")]
         42     #[target_feature(enable = "rdrand")]
         43     unsafe fn gen_u64(&mut self) -> Result<u64> {
         44         let mut out = 0;
    >>   45         if march::_rdrand64_step(&mut out) > 0 {
         46             Ok(out)
         47         } else {
         48             Err(RandError::new(RandErrKind::NoNewRandNumberGen, ""))
         49         }
         50     }
         51     

--- src/rand/default_seed_amd64.rs:25  kind=intrinsic detail=_rdseed32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         19         Ok(DefaultSeed {ph: PhantomData})
         20     }
         21 
         22     #[target_feature(enable = "rdseed")]
         23     unsafe fn rdseed_u32(&self) -> Result<u32> {
         24         let mut out = 0;
    >>   25         if march::_rdseed32_step(&mut out) > 0 {
         26             Ok(out)
         27         } else {
         28             Err(RandError::new(RandErrKind::NoNewRandSeedGen, ""))
         29         }
         30     }
         31     

--- src/rand/default_seed_amd64.rs:49  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
    cfg_gates: # [cfg (target_arch = "x86_64")]
         43     }
         44 
         45     #[cfg(target_arch = "x86_64")]
         46     #[target_feature(enable = "rdseed")]
         47     unsafe fn rdseed_u64(&self) -> Result<u64> {
         48         let mut out = 0;
    >>   49         if march::_rdseed64_step(&mut out) > 0 {
         50             Ok(out)
         51         } else {
         52             Err(RandError::new(RandErrKind::NoNewRandSeedGen, ""))
         53         }
         54     }
         55     

VERDICT: ____

======================================================================
## tfhe-csprng 0.9.1  --  classifier category: **compile_time**

--- src/seeders/implem/rdseed.rs:43  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         37 unsafe fn rdseed_random_m128() -> u128 {
         38     let mut rand1: u64 = 0;
         39     let mut rand2: u64 = 0;
         40     let mut output_bytes = [0u8; 16];
         41 
         42     loop {
    >>   43         if core::arch::x86_64::_rdseed64_step(&mut rand1) == 1 {
         44             break;
         45         }
         46     }
         47     loop {
         48         if core::arch::x86_64::_rdseed64_step(&mut rand2) == 1 {
         49             break;

--- src/seeders/implem/rdseed.rs:48  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         42     loop {
         43         if core::arch::x86_64::_rdseed64_step(&mut rand1) == 1 {
         44             break;
         45         }
         46     }
         47     loop {
    >>   48         if core::arch::x86_64::_rdseed64_step(&mut rand2) == 1 {
         49             break;
         50         }
         51     }
         52 
         53     output_bytes[0..8].copy_from_slice(&rand1.to_ne_bytes());
         54     output_bytes[8..16].copy_from_slice(&rand2.to_ne_bytes());

VERDICT: ____

======================================================================
## typed_arch 0.1.1  --  classifier category: **compile_time**

--- src/x86/rdrand.rs:8  kind=intrinsic detail=_rdrand16_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
          2 
          3 /// Returns a hardware generated 16-bit random value.
          4 #[inline]
          5 #[target_feature(enable = "rdrand")]
          6 pub unsafe fn _rdrand16_step() -> Option<u16> {
          7     let mut v: u16 = 0;
    >>    8     if ::arch::_rdrand16_step(&mut v) == 1 {
          9         Some(v)
         10     } else {
         11         None
         12     }
         13 }
         14 

--- src/x86/rdrand.rs:20  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         14 
         15 /// Read a hardware generated 32-bit random value.
         16 #[inline]
         17 #[target_feature(enable = "rdrand")]
         18 pub unsafe fn _rdrand32_step() -> Option<u32> {
         19     let mut v: u32 = 0;
    >>   20     if ::arch::_rdrand32_step(&mut v) == 1 {
         21         Some(v)
         22     } else {
         23         None
         24     }
         25 }
         26 

--- src/x86/rdrand.rs:32  kind=intrinsic detail=_rdseed16_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         26 
         27 /// Returns a 16-bit NIST SP800-90B and SP800-90C compliant random value.
         28 #[inline]
         29 #[target_feature(enable = "rdseed")]
         30 pub unsafe fn _rdseed16_step() -> Option<u16> {
         31     let mut v: u16 = 0;
    >>   32     if ::arch::_rdseed16_step(&mut v) == 1 {
         33         Some(v)
         34     } else {
         35         None
         36     }
         37 }
         38 

--- src/x86/rdrand.rs:44  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         38 
         39 /// Returns a 32-bit NIST SP800-90B and SP800-90C compliant random value.
         40 #[inline]
         41 #[target_feature(enable = "rdseed")]
         42 pub unsafe fn _rdseed32_step() -> Option<u32> {
         43     let mut v: u32 = 0;
    >>   44     if ::arch::_rdrand32_step(&mut v) == 1 {
         45         Some(v)
         46     } else {
         47         None
         48     }
         49 }

--- src/x86_64/rdrand.rs:8  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
          2 
          3 /// Returns a hardware generated 64-bit random value
          4 #[inline]
          5 #[target_feature(enable = "rdrand")]
          6 pub unsafe fn _rdrand64_step() -> Option<u64> {
          7     let mut v: u64 = 0;
    >>    8     if ::arch::_rdrand64_step(&mut v) == 1 {
          9         Some(v)
         10     } else {
         11         None
         12     }
         13 }
         14 

--- src/x86_64/rdrand.rs:20  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         14 
         15 /// Returns a 64-bit NIST SP800-90B and SP800-90C compliant random value.
         16 #[inline]
         17 #[target_feature(enable = "rdseed")]
         18 pub unsafe fn _rdseed64_step() -> Option<u64> {
         19     let mut v: u64 = 0;
    >>   20     if ::arch::_rdseed64_step(&mut v) == 1 {
         21         Some(v)
         22     } else {
         23         None
         24     }
         25 }

VERDICT: ____

======================================================================
## vektor 0.2.2  --  classifier category: **compile_time**

--- src/x86/rdrand.rs:15  kind=intrinsic detail=_rdrand16_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
          9 /// Return 1 if a random value was generated, and 0 otherwise.
         10 #[inline]
         11 #[target_feature(enable = "rdrand")]
         12 #[cfg_attr(test, assert_instr(rdrand))]
         13 #[cfg_attr(feature = "cargo-clippy", allow(stutter))]
         14 pub unsafe fn _rdrand16_step(val: &mut u16) -> i32 {
    >>   15     crate::mem::transmute(crate::myarch::_rdrand16_step(crate::mem::transmute(val)))
         16 }
         17 
         18 /// Read a hardware generated 32-bit random value and store the result in val.
         19 /// Return 1 if a random value was generated, and 0 otherwise.
         20 #[inline]
         21 #[target_feature(enable = "rdrand")]

--- src/x86/rdrand.rs:25  kind=intrinsic detail=_rdrand32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         19 /// Return 1 if a random value was generated, and 0 otherwise.
         20 #[inline]
         21 #[target_feature(enable = "rdrand")]
         22 #[cfg_attr(test, assert_instr(rdrand))]
         23 #[cfg_attr(feature = "cargo-clippy", allow(stutter))]
         24 pub unsafe fn _rdrand32_step(val: &mut u32) -> i32 {
    >>   25     crate::mem::transmute(crate::myarch::_rdrand32_step(crate::mem::transmute(val)))
         26 }
         27 
         28 /// Read a 16-bit NIST SP800-90B and SP800-90C compliant random value and store
         29 /// in val. Return 1 if a random value was generated, and 0 otherwise.
         30 #[inline]
         31 #[target_feature(enable = "rdseed")]

--- src/x86/rdrand.rs:34  kind=intrinsic detail=_rdseed16_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         28 /// Read a 16-bit NIST SP800-90B and SP800-90C compliant random value and store
         29 /// in val. Return 1 if a random value was generated, and 0 otherwise.
         30 #[inline]
         31 #[target_feature(enable = "rdseed")]
         32 #[cfg_attr(test, assert_instr(rdseed))]
         33 pub unsafe fn _rdseed16_step(val: &mut u16) -> i32 {
    >>   34     crate::mem::transmute(crate::myarch::_rdseed16_step(crate::mem::transmute(val)))
         35 }
         36 
         37 /// Read a 32-bit NIST SP800-90B and SP800-90C compliant random value and store
         38 /// in val. Return 1 if a random value was generated, and 0 otherwise.
         39 #[inline]
         40 #[target_feature(enable = "rdseed")]

--- src/x86/rdrand.rs:43  kind=intrinsic detail=_rdseed32_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         37 /// Read a 32-bit NIST SP800-90B and SP800-90C compliant random value and store
         38 /// in val. Return 1 if a random value was generated, and 0 otherwise.
         39 #[inline]
         40 #[target_feature(enable = "rdseed")]
         41 #[cfg_attr(test, assert_instr(rdseed))]
         42 pub unsafe fn _rdseed32_step(val: &mut u32) -> i32 {
    >>   43     crate::mem::transmute(crate::myarch::_rdseed32_step(crate::mem::transmute(val)))
         44 }
         45 

--- src/x86_64/rdrand.rs:15  kind=intrinsic detail=_rdrand64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
          9 /// Return 1 if a random value was generated, and 0 otherwise.
         10 #[inline]
         11 #[target_feature(enable = "rdrand")]
         12 #[cfg_attr(test, assert_instr(rdrand))]
         13 #[cfg_attr(feature = "cargo-clippy", allow(stutter))]
         14 pub unsafe fn _rdrand64_step(val: &mut u64) -> i32 {
    >>   15     crate::mem::transmute(crate::myarch::_rdrand64_step(crate::mem::transmute(val)))
         16 }
         17 
         18 /// Read a 64-bit NIST SP800-90B and SP800-90C compliant random value and store
         19 /// in val. Return 1 if a random value was generated, and 0 otherwise.
         20 #[inline]
         21 #[target_feature(enable = "rdseed")]

--- src/x86_64/rdrand.rs:24  kind=intrinsic detail=_rdseed64_step site_cat=compile_time in_test=false rt_gated=false cg_present=false tf_attr=true
         18 /// Read a 64-bit NIST SP800-90B and SP800-90C compliant random value and store
         19 /// in val. Return 1 if a random value was generated, and 0 otherwise.
         20 #[inline]
         21 #[target_feature(enable = "rdseed")]
         22 #[cfg_attr(test, assert_instr(rdseed))]
         23 pub unsafe fn _rdseed64_step(val: &mut u64) -> i32 {
    >>   24     crate::mem::transmute(crate::myarch::_rdseed64_step(crate::mem::transmute(val)))
         25 }
         26 

VERDICT: ____

======================================================================
## enarx-shim-kvm 0.6.3  --  classifier category: **out_of_scope**  (oos_reason=sgx)
oos_signals: name:enarx-prefix;dependencies:sallyport:dep

--- src/random.rs:10  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
          4 
          5 /// Get a random number
          6 pub fn random() -> u64 {
          7     let mut r: u64 = 0;
          8 
          9     for _ in 0..1024 {
    >>   10         if unsafe { core::arch::x86_64::_rdrand64_step(&mut r) } == 1 {
         11             return r;
         12         }
         13     }
         14 
         15     panic!("Could not get random!")
         16 }

VERDICT: ____

======================================================================
## enarx-shim-sgx 0.6.3  --  classifier category: **out_of_scope**  (oos_reason=sgx)
oos_signals: name:enarx-prefix;dependencies:sallyport:dep;dependencies:sgx:dep

--- src/entry.rs:27  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
         21 }
         22 
         23 fn random() -> u64 {
         24     let mut r: u64 = 0;
         25 
         26     for _ in 0..1024 {
    >>   27         if unsafe { core::arch::x86_64::_rdrand64_step(&mut r) } == 1 {
         28             return r;
         29         }
         30     }
         31 
         32     exit(1)
         33 }

VERDICT: ____

======================================================================
## enclave 0.1.4  --  classifier category: **out_of_scope**  (oos_reason=sgx)
oos_signals: dependencies:sgx-isa:dep

--- src/rand.rs:15  kind=inline_asm detail=rdrand site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
          9  * option) any later version.
         10  */
         11 
         12 pub fn rand() -> u64 {
         13 	let ret;
         14 	let mut retry=10;
    >>   15 	unsafe{asm!("
         16 1:
         17 		rdrand $0
         18 		jc 2f
         19 		dec $1
         20 		jnz 1b
         21 2:

VERDICT: ____

======================================================================
## freertos-std 0.0.1  --  classifier category: **out_of_scope**  (oos_reason=sgx)
oos_signals: metadata:fortanix-sgx

--- src/sys/sgx/mod.rs:152  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
        146 
        147 pub mod rand {
        148     pub fn rdrand64() -> u64 {
        149         unsafe {
        150             let mut ret: u64 = 0;
        151             for _ in 0..10 {
    >>  152                 if crate::arch::x86_64::_rdrand64_step(&mut ret) == 1 {
        153                     return ret;
        154                 }
        155             }
        156             rtabort!("Failed to obtain random data");
        157         }
        158     }

VERDICT: ____

======================================================================
## libhermit-rs 0.6.3  --  classifier category: **out_of_scope**  (oos_reason=kernel)
oos_signals: name:libhermit;description:library-os;keyword:unikernel;keyword:libos;dependencies:hermit-entry:dep;dependencies:hermit-sync:dep

--- src/arch/x86_64/kernel/processor.rs:911  kind=intrinsic detail=_rdseed64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
        905 			let mut value = 0;
        906 
        907 			// Some RDRAND implementations on AMD CPUs have had bugs where the carry
        908 			// flag was incorrectly set without there actually being a random value
        909 			// available. Even though no bugs are known for RDSEED, we should not
        910 			// consider the default values random for extra security.
    >>  911 			while unsafe { _rdseed64_step(&mut value) != 1 } || value == 0 || value == !0 {
        912 				// Spin as per the recommendation in the
        913 				// Intel® Digital Random Number Generator (DRNG) implementation guide
        914 				spin_loop();
        915 			}
        916 
        917 			word.copy_from_slice(&value.to_ne_bytes());

VERDICT: ____

======================================================================
## ostd 0.18.0  --  classifier category: **out_of_scope**  (oos_reason=kernel)
oos_signals: name;dependencies:ostd-macros:dep:ostd;dependencies:ostd-pod:dep:ostd;dependencies:ostd-test:dep:ostd;target.dependencies:tdx-guest:dep

--- src/arch/x86/mod.rs:162  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
        156     // Implementation Guide" - Section 5.2.1 and "Intel(R) 64 and IA-32 Architectures
        157     // Software Developer's Manual" - Volume 1 - Section 7.3.17.1.
        158     const RETRY_LIMIT: usize = 10;
        159 
        160     for _ in 0..RETRY_LIMIT {
        161         let mut val = 0;
    >>  162         let generated = unsafe { _rdrand64_step(&mut val) };
        163         if generated == 1 {
        164             return Some(val);
        165         }
        166     }
        167     None
        168 }

VERDICT: ____

======================================================================
## rialo-tee-secret-sharing 0.12.2  --  classifier category: **out_of_scope**  (oos_reason=tee)
oos_signals: name:tee-segment;description:tee;keyword:tee

--- src/hw_rng.rs:49  kind=intrinsic detail=_rdseed64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         43 #[allow(unsafe_code)]
         44 #[allow(dead_code)]
         45 #[inline]
         46 fn rdseed64_once() -> Option<u64> {
         47     let mut out = 0u64;
         48     // SAFETY: guarded by is_x86_feature_detected! below
    >>   49     let ok = unsafe { core::arch::x86_64::_rdseed64_step(&mut out) };
         50     (ok == 1).then_some(out)
         51 }
         52 
         53 #[cfg(target_arch = "x86_64")]
         54 #[allow(unsafe_code)]
         55 #[allow(dead_code)]

--- src/hw_rng.rs:60  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         54 #[allow(unsafe_code)]
         55 #[allow(dead_code)]
         56 #[inline]
         57 fn rdrand64_once() -> Option<u64> {
         58     let mut out = 0u64;
         59     // SAFETY: guarded by is_x86_feature_detected! below
    >>   60     let ok = unsafe { core::arch::x86_64::_rdrand64_step(&mut out) };
         61     (ok == 1).then_some(out)
         62 }
         63 
         64 #[cfg(target_arch = "x86_64")]
         65 #[allow(unsafe_code)]
         66 #[allow(dead_code)]

VERDICT: ____

======================================================================
## rusty-hermit 0.3.54  --  classifier category: **out_of_scope**  (oos_reason=kernel)
oos_signals: name:rusty-hermit;keyword:unikernel;keyword:libos

--- src/arch/x86_64/kernel/processor.rs:883  kind=intrinsic detail=_rdrand32_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
        877 pub fn generate_random_number32() -> Option<u32> {
        878 	unsafe {
        879 		if SUPPORTS_RDRAND {
        880 			let mut value: u32 = 0;
        881 
        882 			for _ in 0..RDRAND_RETRY_LIMIT {
    >>  883 				if _rdrand32_step(&mut value) == 1 {
        884 					return Some(value);
        885 				}
        886 			}
        887 		}
        888 		None
        889 	}

--- src/arch/x86_64/kernel/processor.rs:898  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
        892 pub fn generate_random_number64() -> Option<u64> {
        893 	unsafe {
        894 		if SUPPORTS_RDRAND {
        895 			let mut value: u64 = 0;
        896 
        897 			for _ in 0..RDRAND_RETRY_LIMIT {
    >>  898 				if _rdrand64_step(&mut value) == 1 {
        899 					return Some(value);
        900 				}
        901 			}
        902 		}
        903 		None
        904 	}

VERDICT: ____

======================================================================
## sallyport 0.6.3  --  classifier category: **out_of_scope**  (oos_reason=sgx)
oos_signals: name

--- src/guest/call/syscall/stub.rs:137  kind=intrinsic detail=_rdrand64_step site_cat=out_of_scope in_test=false rt_gated=false cg_present=false tf_attr=false
        131             return Err(EINVAL);
        132         }
        133 
        134         for (i, chunk) in self.buf.chunks_mut(8).enumerate() {
        135             let mut el = 0u64;
        136             loop {
    >>  137                 if unsafe { core::arch::x86_64::_rdrand64_step(&mut el) } == 1 {
        138                     chunk.copy_from_slice(&el.to_ne_bytes()[..chunk.len()]);
        139                     break;
        140                 } else {
        141                     if (self.flags & GRND_NONBLOCK) != 0 {
        142                         return Err(EAGAIN);
        143                     }

VERDICT: ____

======================================================================
## hoba 0.5.1  --  classifier category: **runtime**

--- src/lib.rs:260  kind=intrinsic detail=_rdrand64_step site_cat=runtime in_test=false rt_gated=true cg_present=false tf_attr=false
    cfg_gates: # [cfg (all (feature = "whiten" , target_arch = "x86_64"))]
        254     use std::arch::x86_64::_rdrand64_step;
        255     if !is_x86_feature_detected!("rdrand") {
        256         return None;
        257     }
        258     let mut v: u64 = 0;
        259     // SAFETY: rdrand availability checked above; the intrinsic is safe to call when supported.
    >>  260     let ok = unsafe { _rdrand64_step(&mut v) };
        261     if ok == 1 {
        262         Some(v)
        263     } else {
        264         None
        265     }
        266 }

VERDICT: ____

======================================================================
## nist-rand 0.1.0  --  classifier category: **runtime**

--- src/entropy.rs:795  kind=inline_asm detail=rdseed site_cat=runtime in_test=false rt_gated=true cg_present=false tf_attr=false
    cfg_gates: # [cfg (all (feature = "advance" , any (target_arch = "x86" , target_arch = "x86_64")))]
        789         for _ in 0..ITERS {
        790             let mut val: usize = 0;
        791             let mut ok: u8;
        792             // Retry loop: RDSEED can fail if the entropy source is exhausted
        793             for _ in 0..100 {
        794                 unsafe {
    >>  795                     core::arch::asm!(
        796                         "rdseed {val}",
        797                         "setc {ok}",
        798                         val = out(reg) val,
        799                         ok = out(reg_byte) ok,
        800                     );
        801                 }

--- src/entropy.rs:819  kind=inline_asm detail=rdrand site_cat=runtime in_test=false rt_gated=true cg_present=false tf_attr=false
    cfg_gates: # [cfg (all (feature = "advance" , any (target_arch = "x86" , target_arch = "x86_64")))]
        813         for _ in 0..ITERS {
        814             let mut val: usize = 0;
        815             let mut ok: u8;
        816             // RDRAND is faster but can also theoretically fail
        817             for _ in 0..10 {
        818                 unsafe {
    >>  819                     core::arch::asm!(
        820                         "rdrand {val}",
        821                         "setc {ok}",
        822                         val = out(reg) val,
        823                         ok = out(reg_byte) ok,
        824                     );
        825                 }

VERDICT: ____

======================================================================
## Nuclide 0.3.0  --  classifier category: **runtime**

--- src/rng.rs:26  kind=intrinsic detail=_rdrand64_step site_cat=runtime in_test=false rt_gated=true cg_present=false tf_attr=false
         20 pub(crate) fn rand() -> u64 {
         21 
         22     #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
         23     {
         24         if is_x86_feature_detected!("rdrand"){// USE RDRAND if possible
         25             let mut x: u64 = 0;
    >>   26             unsafe { core::arch::x86_64::_rdrand64_step(&mut x) };
         27             return x;
         28         }// If processor is x86 and does not support RDRAND use xor shift
         29        return default_xor()
         30     }
         31 
         32     {// All other architectures use xor shift

VERDICT: ____

======================================================================
## number-theory 0.0.26  --  classifier category: **runtime**

--- src/primitive/eightbytes.rs:21  kind=intrinsic detail=_rdrand64_step site_cat=runtime in_test=false rt_gated=true cg_present=false tf_attr=false
         15 
         16     fn rng() -> Self {
         17         #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
         18     {
         19         if is_x86_feature_detected!("rdrand"){// USE RDRAND if possible
         20             let mut x: u64 = 0;
    >>   21             unsafe { core::arch::x86_64::_rdrand64_step(&mut x) };
         22             return x;
         23         }// If processor is x86 and does not support RDRAND use xor shift
         24        return drbg(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64)
         25     }
         26         #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
         27     {// All other architectures use xor shift

VERDICT: ____

======================================================================
## bitcoin-random 0.1.21  --  classifier category: **unconditional**

--- src/rd_rand.rs:32  kind=inline_asm detail=rdrand site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (have_getcpuid)]
         26         let mut r1: u32 = 0;
         27         let mut r2: u32 = 0;
         28 
         29         for _ in 0..10 {
         30             unsafe {
         31                 // rdrand %eax
    >>   32                 core::arch::asm!(".byte 0x0f, 0xc7, 0xf0; setc {ok}",
         33                                   out("eax") r1, ok = out(reg_byte) ok,
         34                                   options(nostack, preserves_flags));
         35             }
         36             if ok != 0 { break; }
         37         }
         38         for _ in 0..10 {

--- src/rd_rand.rs:41  kind=inline_asm detail=rdrand site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (have_getcpuid)]
         35             }
         36             if ok != 0 { break; }
         37         }
         38         for _ in 0..10 {
         39             unsafe {
         40                 // rdrand %eax
    >>   41                 core::arch::asm!(".byte 0x0f, 0xc7, 0xf0; setc {ok}",
         42                                   out("eax") r2, ok = out(reg_byte) ok,
         43                                   options(nostack, preserves_flags));
         44             }
         45             if ok != 0 { break; }
         46         }
         47         return ((r2 as u64) << 32) | (r1 as u64);

--- src/rd_rand.rs:63  kind=inline_asm detail=rdrand site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (have_getcpuid)]
         57 
         58         for _ in 0..10 {
         59 
         60             unsafe {
         61 
         62                 // rdrand %rax
    >>   63                 core::arch::asm!(".byte 0x48, 0x0f, 0xc7, 0xf0; setc {ok}",
         64                                   out("rax") r, ok = out(reg_byte) ok,
         65                                   options(nostack, preserves_flags));
         66             }
         67 
         68             if ok != 0 { break; }
         69         }

--- src/rd_seed.rs:26  kind=inline_asm detail=rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (have_getcpuid)]
         20         let mut r1: u32 = 0;
         21         let mut r2: u32 = 0;
         22 
         23         loop {
         24             unsafe {
         25                 // rdseed %eax
    >>   26                 core::arch::asm!(".byte 0x0f, 0xc7, 0xf8; setc {ok}",
         27                                   out("eax") r1, ok = out(reg_byte) ok,
         28                                   options(nostack, preserves_flags));
         29             }
         30             if ok != 0 { break; }
         31             unsafe { core::arch::asm!("pause", options(nomem, nostack, preserves_flags)); }
         32         }

--- src/rd_seed.rs:36  kind=inline_asm detail=rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (have_getcpuid)]
         30             if ok != 0 { break; }
         31             unsafe { core::arch::asm!("pause", options(nomem, nostack, preserves_flags)); }
         32         }
         33         loop {
         34             unsafe {
         35                 // rdseed %eax
    >>   36                 core::arch::asm!(".byte 0x0f, 0xc7, 0xf8; setc {ok}",
         37                                   out("eax") r2, ok = out(reg_byte) ok,
         38                                   options(nostack, preserves_flags));
         39             }
         40             if ok != 0 { break; }
         41             unsafe { core::arch::asm!("pause", options(nomem, nostack, preserves_flags)); }
         42         }

--- src/rd_seed.rs:53  kind=inline_asm detail=rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (have_getcpuid)]
         47     {
         48         let mut ok: u8 = 0;
         49         let mut r: u64 = 0;
         50         loop {
         51             unsafe {
         52                 // rdseed %rax
    >>   53                 core::arch::asm!(".byte 0x48, 0x0f, 0xc7, 0xf8; setc {ok}",
         54                                   out("rax") r, ok = out(reg_byte) ok,
         55                                   options(nostack, preserves_flags));
         56             }
         57             if ok != 0 { break; }
         58             unsafe { core::arch::asm!("pause", options(nomem, nostack, preserves_flags)); }
         59         }

VERDICT: ____

======================================================================
## ciphern 0.2.1  --  classifier category: **unconditional**

--- src/random/hardware.rs:124  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (any (target_arch = "x86" , target_arch = "x86_64"))]
        118         // or have any side effects beyond the output register.
        119         // It's supported on all x86 processors since Ivy Bridge (2013).
        120         // The _rdrand64_step function returns 1 on success, 0 on failure,
        121         // and the result is only valid when it returns 1.
        122         let result = unsafe {
        123             let mut val: u64 = core::mem::zeroed();
    >>  124             let status = core::arch::x86_64::_rdrand64_step(&mut val);
        125             if status == 1 {
        126                 val
        127             } else {
        128                 continue;
        129             }
        130         };

--- src/random/hardware.rs:163  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (any (target_arch = "x86" , target_arch = "x86_64"))]
        157 #[inline]
        158 fn read_rdseed_u64() -> Result<u64> {
        159     for _ in 0..RDRAND_MAX_RETRIES {
        160         // SAFETY: RDSEED is a read-only instruction similar to RDRAND
        161         let result = unsafe {
        162             let mut val: u64 = core::mem::zeroed();
    >>  163             let status = core::arch::x86_64::_rdseed64_step(&mut val);
        164             if status == 1 {
        165                 val
        166             } else {
        167                 continue;
        168             }
        169         };

VERDICT: ____

======================================================================
## cocoon-tpm-crypto 0.1.4  --  classifier category: **unconditional**

--- src/pure_rust/rng/x86_64_rdseed.rs:40  kind=inline_asm detail=rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         34 fn rdseed() -> Result<u64, RdSeedError> {
         35     let mut retries = 0;
         36     let result = loop {
         37         let result: u64;
         38         let success: u8;
         39         unsafe {
    >>   40             asm!(
         41                 "rdseed {result:r};\n\
         42                  setc {success};\n\
         43                  ",
         44                 result = out(reg) result,
         45                 success = out(reg_byte) success,
         46             );

VERDICT: ____

======================================================================
## irox-arch-x86-64 0.1.2  --  classifier category: **unconditional**

--- src/rand.rs:12  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          6 use core::arch::x86_64::{_rdseed32_step, _rdseed64_step};
          7 
          8 ///
          9 /// Wrapper around the `rdrand64` instruction
         10 pub fn rdrand64() -> Option<u64> {
         11     let mut out: u64 = 0;
    >>   12     (unsafe { _rdrand64_step(&mut out) } == 1).then_some(out)
         13 }
         14 
         15 ///
         16 /// Wrapper around the `rdrand32` instruction
         17 pub fn rdrand32() -> Option<u32> {
         18     let mut out = 0u32;

--- src/rand.rs:19  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         13 }
         14 
         15 ///
         16 /// Wrapper around the `rdrand32` instruction
         17 pub fn rdrand32() -> Option<u32> {
         18     let mut out = 0u32;
    >>   19     (unsafe { _rdrand32_step(&mut out) } == 1).then_some(out)
         20 }
         21 ///
         22 /// Wrapper around the `rdseed` instruction
         23 pub fn rdseed32() -> Option<u32> {
         24     let mut out: u32 = 0;
         25     (unsafe { _rdseed32_step(&mut out) } == 1).then_some(out)

--- src/rand.rs:25  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         19     (unsafe { _rdrand32_step(&mut out) } == 1).then_some(out)
         20 }
         21 ///
         22 /// Wrapper around the `rdseed` instruction
         23 pub fn rdseed32() -> Option<u32> {
         24     let mut out: u32 = 0;
    >>   25     (unsafe { _rdseed32_step(&mut out) } == 1).then_some(out)
         26 }
         27 ///
         28 /// Wrapper around the `rdseed64` instruction
         29 pub fn rdseed64() -> Option<u64> {
         30     let mut out: u64 = 0;
         31     (unsafe { _rdseed64_step(&mut out) } == 1).then_some(out)

--- src/rand.rs:31  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         25     (unsafe { _rdseed32_step(&mut out) } == 1).then_some(out)
         26 }
         27 ///
         28 /// Wrapper around the `rdseed64` instruction
         29 pub fn rdseed64() -> Option<u64> {
         30     let mut out: u64 = 0;
    >>   31     (unsafe { _rdseed64_step(&mut out) } == 1).then_some(out)
         32 }
         33 ///
         34 /// Wrapper around the `rdseed64` instruction
         35 pub fn seed() -> Option<u64> {
         36     rdseed64()
         37 }

VERDICT: ____

======================================================================
## lcpfs 2026.1.102  --  classifier category: **unconditional**

--- src/arch/x86_64.rs:38  kind=inline_asm detail=rdrand site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         32             //
         33             // VERIFICATION: RDRAND available on Intel Ivy Bridge+ (2012), AMD Excavator+ (2015)
         34             //
         35             // JUSTIFICATION:
         36             // Hardware RNG provides high-quality entropy for ChaCha20 seeding.
         37             unsafe {
    >>   38                 core::arch::asm!(
         39                     "rdrand {0}",
         40                     "setc {1}",
         41                     out(reg) value,
         42                     out(reg_byte) success,
         43                     options(nomem, nostack),
         44                 );

--- src/arch/x86_64.rs:91  kind=inline_asm detail=rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         85             //
         86             // VERIFICATION: RDSEED available on Intel Broadwell+ (2014), AMD Zen+ (2018)
         87             //
         88             // JUSTIFICATION:
         89             // RDSEED provides true entropy (not DRBG output) for seeding PRNGs.
         90             unsafe {
    >>   91                 core::arch::asm!(
         92                     "rdseed {0}",
         93                     "setc {1}",
         94                     out(reg) value,
         95                     out(reg_byte) success,
         96                     options(nomem, nostack),
         97                 );

VERDICT: ____

======================================================================
## lib-q-fn-dsa 0.0.8  --  classifier category: **unconditional**

--- examples/hardware_rng_example.rs:265  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
        259         // 1. Use the RDRAND instruction
        260         // 2. Check the carry flag for success
        261         // 3. Retry if necessary
        262         // SAFETY: This is a placeholder for demonstration
        263         unsafe {
        264             let mut out = 0u32;
    >>  265             core::arch::x86_64::_rdrand32_step(&mut out);
        266             out
        267         }
        268     }
        269 }
        270 
        271 #[cfg(target_arch = "x86_64")]

VERDICT: ____

======================================================================
## lokacore 0.3.0  --  classifier category: **unconditional**

--- src/arch/rdrand.rs:18  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         12 /// well use that one.
         13 ///
         14 /// See
         15 /// [`_rdrand16_step`](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_rdrand16_step)
         16 pub fn rdrand16_step() -> Option<u16> {
         17   let mut x = 0_u16;
    >>   18   if 1 == unsafe { _rdrand16_step(&mut x) } {
         19     Some(x)
         20   } else {
         21     None
         22   }
         23 }
         24 

--- src/arch/rdrand.rs:37  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         31 /// well use that one if you're on a `x86_64`.
         32 ///
         33 /// See
         34 /// [`_rdrand32_step`](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_rdrand32_step)
         35 pub fn rdrand32_step() -> Option<u32> {
         36   let mut x = 0_u32;
    >>   37   if 1 == unsafe { _rdrand32_step(&mut x) } {
         38     Some(x)
         39   } else {
         40     None
         41   }
         42 }
         43 

--- src/arch/rdrand.rs:56  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         50 ///
         51 /// See
         52 /// [`_rdrand64_step`](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_rdrand64_step)
         53 #[cfg(target_arch = "x86_64")]
         54 pub fn rdrand64_step() -> Option<u64> {
         55   let mut x = 0_u64;
    >>   56   if 1 == unsafe { _rdrand64_step(&mut x) } {
         57     Some(x)
         58   } else {
         59     None
         60   }
         61 }

VERDICT: ____

======================================================================
## mbedtls 0.13.5  --  classifier category: **unconditional**

--- src/rng/rdrand.rs:47  kind=intrinsic detail=alias:_rdrand_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         41         }
         42     }
         43     None
         44 }
         45 
         46 fn rdrand() -> Option<usize> {
    >>   47     call_cpu_rng(RDRAND_READ_ATTEMPTS, _rdrand_step, |x| x as usize)
         48 }
         49 
         50 fn rdseed() -> Option<usize> {
         51     call_cpu_rng(RDSEED_READ_ATTEMPTS, _rdseed_step, |x| x as usize)
         52 }
         53 

--- src/rng/rdrand.rs:51  kind=intrinsic detail=alias:_rdseed_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         45 
         46 fn rdrand() -> Option<usize> {
         47     call_cpu_rng(RDRAND_READ_ATTEMPTS, _rdrand_step, |x| x as usize)
         48 }
         49 
         50 fn rdseed() -> Option<usize> {
    >>   51     call_cpu_rng(RDSEED_READ_ATTEMPTS, _rdseed_step, |x| x as usize)
         52 }
         53 
         54 fn write_rng_to_slice(outbuf: &mut [u8], rng: fn() -> Option<usize>) -> c_int {
         55     let stepsize = core::mem::size_of::<usize>();
         56 
         57     for chunk in outbuf.chunks_mut(stepsize) {

VERDICT: ____

======================================================================
## mbedtls-smx 0.13.11  --  classifier category: **unconditional**

--- src/rng/rdrand.rs:47  kind=intrinsic detail=alias:_rdrand_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         41         }
         42     }
         43     None
         44 }
         45 
         46 fn rdrand() -> Option<usize> {
    >>   47     call_cpu_rng(RDRAND_READ_ATTEMPTS, _rdrand_step, |x| x as usize)
         48 }
         49 
         50 fn rdseed() -> Option<usize> {
         51     call_cpu_rng(RDSEED_READ_ATTEMPTS, _rdseed_step, |x| x as usize)
         52 }
         53 

--- src/rng/rdrand.rs:51  kind=intrinsic detail=alias:_rdseed_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         45 
         46 fn rdrand() -> Option<usize> {
         47     call_cpu_rng(RDRAND_READ_ATTEMPTS, _rdrand_step, |x| x as usize)
         48 }
         49 
         50 fn rdseed() -> Option<usize> {
    >>   51     call_cpu_rng(RDSEED_READ_ATTEMPTS, _rdseed_step, |x| x as usize)
         52 }
         53 
         54 fn write_rng_to_slice(outbuf: &mut [u8], rng: fn() -> Option<usize>) -> c_int {
         55     let stepsize = core::mem::size_of::<usize>();
         56 
         57     for chunk in outbuf.chunks_mut(stepsize) {

VERDICT: ____

======================================================================
## mc-rand 1.1.0  --  classifier category: **unconditional**

--- src/rdrandrng/retry.rs:21  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         15     #[cfg(target_arch = "x86_64")]
         16     use core::arch::x86_64::_rdrand32_step;
         17 
         18     let mut result: u32 = 0;
         19     let mut retries = RETRIES;
         20     while retries > 0 {
    >>   21         if 1 == unsafe { _rdrand32_step(&mut result) } && result != 0 && result != 0xFFFF_FFFFu32 {
         22             return result;
         23         }
         24         retries -= 1;
         25     }
         26     panic!("_rdrand32_step unexpectedly failed {} times", RETRIES);
         27 }

--- src/rdrandrng/retry.rs:38  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         32     // Get rdrand compiler intrinsic for 64 bits
         33     use core::arch::x86_64::_rdrand64_step;
         34 
         35     let mut result: u64 = 0;
         36     let mut retries = RETRIES;
         37     while retries > 0 {
    >>   38         if 1 == unsafe { _rdrand64_step(&mut result) }
         39             && result != 0
         40             && result != 0xFFFF_FFFF_FFFF_FFFFu64
         41         {
         42             return result;
         43         }
         44         retries -= 1;

VERDICT: ____

======================================================================
## nanorand 0.8.0  --  classifier category: **unconditional**

--- src/entropy.rs:77  kind=intrinsic detail=alias:rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (feature = "rdseed")];# [cfg (any (target_arch = "x86" , target_arch = "x86_64"))]
         71 	#[cfg(target_arch = "x86")]
         72 	use core::arch::x86::_rdseed64_step as rdseed;
         73 	#[cfg(target_arch = "x86_64")]
         74 	use core::arch::x86_64::_rdseed64_step as rdseed;
         75 	let mut x = 0;
         76 	for _ in 0..10 {
    >>   77 		if 0 != unsafe { rdseed(&mut x) } {
         78 			return Some(x);
         79 		}
         80 	}
         81 	None
         82 }
         83 

--- src/entropy.rs:130  kind=intrinsic detail=alias:rdseed site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (feature = "std")]
        124 /// using system time-based entropy generation.
        125 ///
        126 /// # Panics
        127 ///
        128 /// This function panics if sufficient entropy could not be obtained.
        129 pub fn backup(out: &mut [u8]) {
    >>  130 	if let Some(amt) = rdseed(out) {
        131 		if amt >= out.len() {
        132 			return;
        133 		}
        134 	};
        135 
        136 	panic!("Failed to source sufficient entropy!")

VERDICT: ____

======================================================================
## ort-openrouter-cli 0.4.7  --  classifier category: **unconditional**

--- src/syscall.rs:166  kind=inline_asm detail=rdrand site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        160         "getrandom buffer len must be multiple of 8"
        161     );
        162     let mut r: u64;
        163     let mut i = 0;
        164     while i < buf.len() {
        165         unsafe {
    >>  166             asm!("RDRAND rax", out("rax") r);
        167             buf[i..i + 8].copy_from_slice(&r.to_be_bytes());
        168         }
        169         i += 8;
        170     }
        171 }
        172 

VERDICT: ____

======================================================================
## rdrand 0.9.0  --  classifier category: **unconditional**

--- src/lib.rs:110  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86")]
        104     pub use core::arch::x86_64::*;
        105 
        106     #[cfg(target_arch = "x86")]
        107     pub(crate) unsafe fn _rdrand64_step(dest: &mut u64) -> i32 {
        108         let mut ret1: u32 = 0;
        109         let mut ret2: u32 = 0;
    >>  110         let ok = _rdrand32_step(&mut ret1) & _rdrand32_step(&mut ret2);
        111         *dest = (ret1 as u64) << 32 | (ret2 as u64);
        112         ok
        113     }
        114 
        115     #[cfg(target_arch = "x86")]
        116     pub(crate) unsafe fn _rdseed64_step(dest: &mut u64) -> i32 {

--- src/lib.rs:119  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86")]
        113     }
        114 
        115     #[cfg(target_arch = "x86")]
        116     pub(crate) unsafe fn _rdseed64_step(dest: &mut u64) -> i32 {
        117         let mut ret1: u32 = 0;
        118         let mut ret2: u32 = 0;
    >>  119         let ok = _rdseed32_step(&mut ret1) & _rdseed32_step(&mut ret2);
        120         *dest = (ret1 as u64) << 32 | (ret2 as u64);
        121         ok
        122     }
        123 
        124     #[cfg(target_arch = "aarch64")]
        125     pub(crate) unsafe fn rand(out: &mut u64) -> i32 {

--- src/lib.rs:559  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        553 
        554 #[cfg(target_arch = "x86_64")]
        555 impl_rand!(
        556     RdRand,
        557     "rdrand",
        558     "rdrand",
    >>  559     arch::_rdrand32_step,
        560     arch::_rdrand64_step,
        561     maxstep = arch::_rdrand64_step,
        562     maxty = u64
        563 );
        564 #[cfg(target_arch = "x86_64")]
        565 impl_rand!(

--- src/lib.rs:560  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        554 #[cfg(target_arch = "x86_64")]
        555 impl_rand!(
        556     RdRand,
        557     "rdrand",
        558     "rdrand",
        559     arch::_rdrand32_step,
    >>  560     arch::_rdrand64_step,
        561     maxstep = arch::_rdrand64_step,
        562     maxty = u64
        563 );
        564 #[cfg(target_arch = "x86_64")]
        565 impl_rand!(
        566     RdSeed,

--- src/lib.rs:561  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        555 impl_rand!(
        556     RdRand,
        557     "rdrand",
        558     "rdrand",
        559     arch::_rdrand32_step,
        560     arch::_rdrand64_step,
    >>  561     maxstep = arch::_rdrand64_step,
        562     maxty = u64
        563 );
        564 #[cfg(target_arch = "x86_64")]
        565 impl_rand!(
        566     RdSeed,
        567     "rdseed",

--- src/lib.rs:569  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        563 );
        564 #[cfg(target_arch = "x86_64")]
        565 impl_rand!(
        566     RdSeed,
        567     "rdseed",
        568     "rdseed",
    >>  569     arch::_rdseed32_step,
        570     arch::_rdseed64_step,
        571     maxstep = arch::_rdseed64_step,
        572     maxty = u64
        573 );
        574 #[cfg(target_arch = "x86")]
        575 impl_rand!(

--- src/lib.rs:570  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        564 #[cfg(target_arch = "x86_64")]
        565 impl_rand!(
        566     RdSeed,
        567     "rdseed",
        568     "rdseed",
        569     arch::_rdseed32_step,
    >>  570     arch::_rdseed64_step,
        571     maxstep = arch::_rdseed64_step,
        572     maxty = u64
        573 );
        574 #[cfg(target_arch = "x86")]
        575 impl_rand!(
        576     RdRand,

--- src/lib.rs:571  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        565 impl_rand!(
        566     RdSeed,
        567     "rdseed",
        568     "rdseed",
        569     arch::_rdseed32_step,
        570     arch::_rdseed64_step,
    >>  571     maxstep = arch::_rdseed64_step,
        572     maxty = u64
        573 );
        574 #[cfg(target_arch = "x86")]
        575 impl_rand!(
        576     RdRand,
        577     "rdrand",

--- src/lib.rs:579  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        573 );
        574 #[cfg(target_arch = "x86")]
        575 impl_rand!(
        576     RdRand,
        577     "rdrand",
        578     "rdrand",
    >>  579     arch::_rdrand32_step,
        580     arch::_rdrand64_step,
        581     maxstep = arch::_rdrand32_step,
        582     maxty = u32
        583 );
        584 #[cfg(target_arch = "x86")]
        585 impl_rand!(

--- src/lib.rs:580  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        574 #[cfg(target_arch = "x86")]
        575 impl_rand!(
        576     RdRand,
        577     "rdrand",
        578     "rdrand",
        579     arch::_rdrand32_step,
    >>  580     arch::_rdrand64_step,
        581     maxstep = arch::_rdrand32_step,
        582     maxty = u32
        583 );
        584 #[cfg(target_arch = "x86")]
        585 impl_rand!(
        586     RdSeed,

--- src/lib.rs:581  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        575 impl_rand!(
        576     RdRand,
        577     "rdrand",
        578     "rdrand",
        579     arch::_rdrand32_step,
        580     arch::_rdrand64_step,
    >>  581     maxstep = arch::_rdrand32_step,
        582     maxty = u32
        583 );
        584 #[cfg(target_arch = "x86")]
        585 impl_rand!(
        586     RdSeed,
        587     "rdseed",

--- src/lib.rs:589  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        583 );
        584 #[cfg(target_arch = "x86")]
        585 impl_rand!(
        586     RdSeed,
        587     "rdseed",
        588     "rdseed",
    >>  589     arch::_rdseed32_step,
        590     arch::_rdseed64_step,
        591     maxstep = arch::_rdseed32_step,
        592     maxty = u32
        593 );
        594 #[cfg(target_arch = "aarch64")]
        595 impl_rand!(

--- src/lib.rs:590  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        584 #[cfg(target_arch = "x86")]
        585 impl_rand!(
        586     RdSeed,
        587     "rdseed",
        588     "rdseed",
        589     arch::_rdseed32_step,
    >>  590     arch::_rdseed64_step,
        591     maxstep = arch::_rdseed32_step,
        592     maxty = u32
        593 );
        594 #[cfg(target_arch = "aarch64")]
        595 impl_rand!(
        596     RdRand,

--- src/lib.rs:591  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        585 impl_rand!(
        586     RdSeed,
        587     "rdseed",
        588     "rdseed",
        589     arch::_rdseed32_step,
        590     arch::_rdseed64_step,
    >>  591     maxstep = arch::_rdseed32_step,
        592     maxty = u32
        593 );
        594 #[cfg(target_arch = "aarch64")]
        595 impl_rand!(
        596     RdRand,
        597     "rand",

VERDICT: ____

======================================================================
## safe_arch 1.0.0  --  classifier category: **unconditional**

--- src/x86_x64/rdrand.rs:13  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          7 /// * **Intrinsic:** [`_rdrand16_step`]
          8 /// * **Assembly:** `rdrand r16`
          9 #[must_use]
         10 #[inline(always)]
         11 #[cfg_attr(docsrs, doc(cfg(target_feature = "rdrand")))]
         12 pub fn rdrand_u16(out: &mut u16) -> i32 {
    >>   13   unsafe { _rdrand16_step(out) }
         14 }
         15 
         16 /// Try to obtain a random `u32` from the hardware RNG.
         17 ///
         18 /// * **Intrinsic:** [`_rdrand32_step`]
         19 /// * **Assembly:** `rdrand r32`

--- src/x86_x64/rdrand.rs:24  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         18 /// * **Intrinsic:** [`_rdrand32_step`]
         19 /// * **Assembly:** `rdrand r32`
         20 #[must_use]
         21 #[inline(always)]
         22 #[cfg_attr(docsrs, doc(cfg(target_feature = "rdrand")))]
         23 pub fn rdrand_u32(out: &mut u32) -> i32 {
    >>   24   unsafe { _rdrand32_step(out) }
         25 }
         26 
         27 /// Try to obtain a random `u64` from the hardware RNG.
         28 ///
         29 /// * **Intrinsic:** [`_rdrand64_step`]
         30 /// * **Assembly:** `rdrand r64`

--- src/x86_x64/rdrand.rs:36  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         30 /// * **Assembly:** `rdrand r64`
         31 #[must_use]
         32 #[inline(always)]
         33 #[cfg(target_arch = "x86_64")]
         34 #[cfg_attr(docsrs, doc(cfg(target_feature = "rdrand")))]
         35 pub fn rdrand_u64(out: &mut u64) -> i32 {
    >>   36   unsafe { _rdrand64_step(out) }
         37 }
         38 

--- src/x86_x64/rdseed.rs:13  kind=intrinsic detail=_rdseed16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          7 /// * **Intrinsic:** [`_rdseed16_step`]
          8 /// * **Assembly:** `rdseed r16`
          9 #[must_use]
         10 #[inline(always)]
         11 #[cfg_attr(docsrs, doc(cfg(target_feature = "rdseed")))]
         12 pub fn rdseed_u16(out: &mut u16) -> i32 {
    >>   13   unsafe { _rdseed16_step(out) }
         14 }
         15 
         16 /// Try to obtain a random `u32` from the hardware RNG.
         17 ///
         18 /// * **Intrinsic:** [`_rdseed32_step`]
         19 /// * **Assembly:** `rdseed r32`

--- src/x86_x64/rdseed.rs:24  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         18 /// * **Intrinsic:** [`_rdseed32_step`]
         19 /// * **Assembly:** `rdseed r32`
         20 #[must_use]
         21 #[inline(always)]
         22 #[cfg_attr(docsrs, doc(cfg(target_feature = "rdseed")))]
         23 pub fn rdseed_u32(out: &mut u32) -> i32 {
    >>   24   unsafe { _rdseed32_step(out) }
         25 }
         26 
         27 /// Try to obtain a random `u64` from the hardware RNG.
         28 ///
         29 /// * **Intrinsic:** [`_rdseed64_step`]
         30 /// * **Assembly:** `rdseed r64`

--- src/x86_x64/rdseed.rs:36  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         30 /// * **Assembly:** `rdseed r64`
         31 #[must_use]
         32 #[inline(always)]
         33 #[cfg(target_arch = "x86_64")]
         34 #[cfg_attr(docsrs, doc(cfg(target_feature = "rdseed")))]
         35 pub fn rdseed_u64(out: &mut u64) -> i32 {
    >>   36   unsafe { _rdseed64_step(out) }
         37 }
         38 

VERDICT: ____

======================================================================
## sec-mem 0.1.1  --  classifier category: **unconditional**

--- src/secret_box.rs:28  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         22         }
         23 
         24         #[cfg(target_arch = "x86_64")]
         25         {
         26             let mut canary = 0u64;
         27             unsafe {
    >>   28                 if core::arch::x86_64::_rdrand64_step(&mut canary) == 1 {
         29                     return canary as usize;
         30                 }
         31             }
         32         }
         33 
         34         #[cfg(target_arch = "x86")]

--- src/secret_box.rs:38  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         32         }
         33 
         34         #[cfg(target_arch = "x86")]
         35         {
         36             let mut canary = 0u32;
         37             unsafe {
    >>   38                 if core::arch::x86::_rdrand32_step(&mut canary) == 1 {
         39                     return canary as usize;
         40                 }
         41             }
         42         }
         43 
         44         // Fallback: ASLR pointer mixed with timestamp

VERDICT: ____

======================================================================
## simple_random_numbers 0.1.1  --  classifier category: **unconditional**

--- src/lib.rs:22  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         16 
         17 // tries until a pseudo-random number is returned
         18 pub fn pseudo_rand() -> u64 {
         19     use std::arch::x86_64::_rdrand64_step;
         20     let mut rand_num: u64 = 0;
         21 
    >>   22     if unsafe { _rdrand64_step(&mut rand_num) } == 1 {
         23         rand_num
         24     } else {
         25         pseudo_rand()
         26     }
         27 }
         28 

--- src/lib.rs:34  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         28 
         29 // tries once to get a pseudo-random number and returns None if failed
         30 pub fn try_pseudo_rand() -> Option<u64> {
         31     use std::arch::x86_64::_rdrand64_step;
         32     let mut rand_num: u64 = 0;
         33 
    >>   34     if unsafe { _rdrand64_step(&mut rand_num) } == 1 {
         35         Some(rand_num)
         36     } else {
         37         None
         38     }
         39 }
         40 

--- src/lib.rs:46  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         40 
         41 // tries until a entropy-random number is returned
         42 pub fn entropy_rand() -> u64 {
         43     use std::arch::x86_64::_rdseed64_step;
         44     let mut rand_seed: u64 = 0;
         45 
    >>   46     if unsafe { _rdseed64_step(&mut rand_seed) } == 1 {
         47         rand_seed
         48     } else {
         49         entropy_rand()
         50     }
         51 }
         52 

--- src/lib.rs:58  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         52 
         53 // tries once to get an entropy-random number and returns None if failed
         54 pub fn try_entropy_rand() -> Option<u64> {
         55     use std::arch::x86_64::_rdseed64_step;
         56     let mut rand_seed: u64 = 0;
         57 
    >>   58     if unsafe { _rdseed64_step(&mut rand_seed) } == 1 {
         59         Some(rand_seed)
         60     } else {
         61         None
         62     }
         63 }

VERDICT: ____

======================================================================
## swiss-army-knife 0.0.37  --  classifier category: **unconditional**

--- src/random/fast_slightly_insecure_random_u16.rs:14  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          8 pub fn fast_slightly_insecure_random_u16() -> u16
          9 {
         10 	let mut random_value = unsafe_uninitialized();
         11 	
         12 	loop
         13 	{
    >>   14 		match unsafe { _rdrand16_step(&mut random_value) }
         15 		{
         16 			0 => break,
         17 			
         18 			1 => continue,
         19 			
         20 			unexpected @ _ => unreachable_code(format_args!("Intel _rdrand16_step() intrinsic returned a result other than 0 or 1: {}", unexpected)),

--- src/random/fast_slightly_insecure_random_u32.rs:14  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          8 pub fn fast_slightly_insecure_random_u32() -> u32
          9 {
         10 	let mut random_value= unsafe_uninitialized();
         11 	
         12 	loop
         13 	{
    >>   14 		match unsafe { _rdrand32_step(&mut random_value) }
         15 		{
         16 			0 => break,
         17 			
         18 			1 => continue,
         19 			
         20 			unexpected @ _ => unreachable_code(format_args!("Intel _rdrand32_step() intrinsic returned a result other than 0 or 1: {}", unexpected)),

--- src/random/fast_slightly_insecure_random_u64.rs:14  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          8 pub fn fast_slightly_insecure_random_u64() -> u64
          9 {
         10 	let mut random_value= unsafe_uninitialized();
         11 	
         12 	loop
         13 	{
    >>   14 		match unsafe { _rdrand64_step(&mut random_value) }
         15 		{
         16 			0 => break,
         17 			
         18 			1 => continue,
         19 			
         20 			unexpected @ _ => unreachable_code(format_args!("Intel _rdrand64_step() intrinsic returned a result other than 0 or 1: {}", unexpected)),

VERDICT: ____

======================================================================
## urng 0.8.0  --  classifier category: **unconditional**

--- src/seedgen.rs:238  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (any (target_arch = "x86" , target_arch = "x86_64"))]
        232 // ── x86 / x86_64 intrinsics ───────────────────────────────
        233 
        234 #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        235 unsafe fn rdseed32_once() -> Option<u32> {
        236     let mut value = 0u32;
        237     for _ in 0..4 {
    >>  238         if unsafe { _rdseed32_step(&mut value) } == 1 {
        239             return Some(value);
        240         }
        241     }
        242     None
        243 }
        244 

--- src/seedgen.rs:249  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (any (target_arch = "x86" , target_arch = "x86_64"))]
        243 }
        244 
        245 #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        246 unsafe fn rdrand32_once() -> Option<u32> {
        247     let mut value = 0u32;
        248     for _ in 0..4 {
    >>  249         if unsafe { _rdrand32_step(&mut value) } == 1 {
        250             return Some(value);
        251         }
        252     }
        253     None
        254 }
        255 

--- src/seedgen.rs:260  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
        254 }
        255 
        256 #[cfg(target_arch = "x86_64")]
        257 unsafe fn rdseed64_once() -> Option<u64> {
        258     let mut value = 0u64;
        259     for _ in 0..4 {
    >>  260         if unsafe { _rdseed64_step(&mut value) } == 1 {
        261             return Some(value);
        262         }
        263     }
        264     None
        265 }
        266 

--- src/seedgen.rs:271  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
        265 }
        266 
        267 #[cfg(target_arch = "x86_64")]
        268 unsafe fn rdrand64_once() -> Option<u64> {
        269     let mut value = 0u64;
        270     for _ in 0..4 {
    >>  271         if unsafe { _rdrand64_step(&mut value) } == 1 {
        272             return Some(value);
        273         }
        274     }
        275     None
        276 }

VERDICT: ____

======================================================================
## virtfw-libhw 0.5.0  --  classifier category: **unconditional**

--- src/x86cpuid.rs:69  kind=inline_asm detail=rdrand site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         63 }
         64 
         65 // not really a good fit, but hey, you need the cpuid check anyway ...
         66 pub fn rdrand() -> u64 {
         67     let rax: u64;
         68     unsafe {
    >>   69         asm!("rdrand %rax",
         70              out("rax") rax,
         71              options(att_syntax));
         72     }
         73     rax
         74 }

VERDICT: ____

======================================================================
## winit-gfx-hal 0.0.0  --  classifier category: **unconditional**

--- src/arch/rdrand.rs:14  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
          8 /// well use that one.
          9 ///
         10 /// See
         11 /// [`_rdrand16_step`](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_rdrand16_step)
         12 pub fn rdrand16_step() -> Option<u16> {
         13   let mut x = 0_u16;
    >>   14   if 0 < unsafe { _rdrand16_step(&mut x) } {
         15     Some(x)
         16   } else {
         17     None
         18   }
         19 }
         20 

--- src/arch/rdrand.rs:30  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         24 /// well use that one if you're on a `x86_64`.
         25 ///
         26 /// See
         27 /// [`_rdrand32_step`](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_rdrand32_step)
         28 pub fn rdrand32_step() -> Option<u32> {
         29   let mut x = 0_u32;
    >>   30   if 0 < unsafe { _rdrand32_step(&mut x) } {
         31     Some(x)
         32   } else {
         33     None
         34   }
         35 }
         36 

--- src/arch/rdrand.rs:46  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         40 ///
         41 /// See
         42 /// [`_rdrand64_step`](https://software.intel.com/sites/landingpage/IntrinsicsGuide/#text=_rdrand64_step)
         43 #[cfg(target_arch = "x86_64")]
         44 pub fn rdrand64_step() -> Option<u64> {
         45   let mut x = 0_u64;
    >>   46   if 0 < unsafe { _rdrand64_step(&mut x) } {
         47     Some(x)
         48   } else {
         49     None
         50   }
         51 }

VERDICT: ____

======================================================================
## x64 0.16.2  --  classifier category: **unconditional**

--- src/instructions/random.rs:28  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         22     /// Uniformly sampled u64.
         23     /// May fail in rare circumstances or heavy load.
         24     #[inline]
         25     pub fn get_u64(self) -> Option<u64> {
         26         let mut res: u64 = 0;
         27         unsafe {
    >>   28             match core::arch::x86_64::_rdrand64_step(&mut res) {
         29                 1 => Some(res),
         30                 x => {
         31                     debug_assert_eq!(x, 0, "rdrand64 returned non-binary value");
         32                     None
         33                 }
         34             }

--- src/instructions/random.rs:43  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         37     /// Uniformly sampled u32.
         38     /// May fail in rare circumstances or heavy load.
         39     #[inline]
         40     pub fn get_u32(self) -> Option<u32> {
         41         let mut res: u32 = 0;
         42         unsafe {
    >>   43             match core::arch::x86_64::_rdrand32_step(&mut res) {
         44                 1 => Some(res),
         45                 x => {
         46                     debug_assert_eq!(x, 0, "rdrand32 returned non-binary value");
         47                     None
         48                 }
         49             }

--- src/instructions/random.rs:58  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         52     /// Uniformly sampled u16.
         53     /// May fail in rare circumstances or heavy load.
         54     #[inline]
         55     pub fn get_u16(self) -> Option<u16> {
         56         let mut res: u16 = 0;
         57         unsafe {
    >>   58             match core::arch::x86_64::_rdrand16_step(&mut res) {
         59                 1 => Some(res),
         60                 x => {
         61                     debug_assert_eq!(x, 0, "rdrand16 returned non-binary value");
         62                     None
         63                 }
         64             }

VERDICT: ____

======================================================================
## x86 0.52.0  --  classifier category: **unconditional**

--- src/random.rs:25  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         19 /// Generates a 16-bit random value and stores it in `e`.
         20 ///
         21 /// # Safety
         22 /// Will crash if RDRAND instructions are not supported.
         23 #[inline(always)]
         24 pub unsafe fn rdrand16(e: &mut u16) -> bool {
    >>   25     _rdrand16_step(e) == 1
         26 }
         27 
         28 /// Generates a 32-bit random value and stores it in `e`.
         29 ///
         30 /// # Safety
         31 /// Will crash if RDRAND instructions are not supported.

--- src/random.rs:34  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         28 /// Generates a 32-bit random value and stores it in `e`.
         29 ///
         30 /// # Safety
         31 /// Will crash if RDRAND instructions are not supported.
         32 #[inline(always)]
         33 pub unsafe fn rdrand32(e: &mut u32) -> bool {
    >>   34     _rdrand32_step(e) == 1
         35 }
         36 
         37 /// Generates a 64-bit random value and stores it in `e`.
         38 ///
         39 /// # Safety
         40 /// Will crash if RDRAND instructions are not supported.

--- src/random.rs:44  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
         38 ///
         39 /// # Safety
         40 /// Will crash if RDRAND instructions are not supported.
         41 #[cfg(target_arch = "x86_64")]
         42 #[inline(always)]
         43 pub unsafe fn rdrand64(e: &mut u64) -> bool {
    >>   44     _rdrand64_step(e) == 1
         45 }
         46 
         47 /// RdRand trait to implement the generic rdrand_slice function.
         48 pub trait RdRand {
         49     /// Fills `self` with random bits. Returns true on success or false otherwise
         50     ///

--- src/random.rs:120  kind=intrinsic detail=_rdseed16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        114 /// Generates a 16-bit random value and stores it in `e`.
        115 ///
        116 /// # Safety
        117 /// Will crash if RDSEED instructions are not supported.
        118 #[inline(always)]
        119 pub unsafe fn rdseed16(e: &mut u16) -> bool {
    >>  120     _rdseed16_step(e) == 1
        121 }
        122 
        123 /// Generates a 32-bit random value and stores it in `e`.
        124 ///
        125 /// # Safety
        126 /// Will crash if RDSEED instructions are not supported.

--- src/random.rs:129  kind=intrinsic detail=_rdseed32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
        123 /// Generates a 32-bit random value and stores it in `e`.
        124 ///
        125 /// # Safety
        126 /// Will crash if RDSEED instructions are not supported.
        127 #[inline(always)]
        128 pub unsafe fn rdseed32(e: &mut u32) -> bool {
    >>  129     _rdseed32_step(e) == 1
        130 }
        131 
        132 /// Generates a 64-bit random value and stores it in `e`.
        133 ///
        134 /// # Safety
        135 /// Will crash if RDSEED instructions are not supported.

--- src/random.rs:139  kind=intrinsic detail=_rdseed64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
    cfg_gates: # [cfg (target_arch = "x86_64")]
        133 ///
        134 /// # Safety
        135 /// Will crash if RDSEED instructions are not supported.
        136 #[cfg(target_arch = "x86_64")]
        137 #[inline(always)]
        138 pub unsafe fn rdseed64(e: &mut u64) -> bool {
    >>  139     _rdseed64_step(e) == 1
        140 }
        141 
        142 /// RdSeed trait to implement the generic rdseed_slice function.
        143 pub trait RdSeed {
        144     /// Fills `self` with random bits. Returns true on success or false otherwise
        145     ///

VERDICT: ____

======================================================================
## x86_64 0.15.4  --  classifier category: **unconditional**

--- src/instructions/random.rs:27  kind=intrinsic detail=_rdrand64_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         21     /// Uniformly sampled u64.
         22     /// May fail in rare circumstances or heavy load.
         23     #[inline]
         24     pub fn get_u64(self) -> Option<u64> {
         25         let mut res: u64 = 0;
         26         unsafe {
    >>   27             match core::arch::x86_64::_rdrand64_step(&mut res) {
         28                 1 => Some(res),
         29                 x => {
         30                     debug_assert_eq!(x, 0, "rdrand64 returned non-binary value");
         31                     None
         32                 }
         33             }

--- src/instructions/random.rs:42  kind=intrinsic detail=_rdrand32_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         36     /// Uniformly sampled u32.
         37     /// May fail in rare circumstances or heavy load.
         38     #[inline]
         39     pub fn get_u32(self) -> Option<u32> {
         40         let mut res: u32 = 0;
         41         unsafe {
    >>   42             match core::arch::x86_64::_rdrand32_step(&mut res) {
         43                 1 => Some(res),
         44                 x => {
         45                     debug_assert_eq!(x, 0, "rdrand32 returned non-binary value");
         46                     None
         47                 }
         48             }

--- src/instructions/random.rs:57  kind=intrinsic detail=_rdrand16_step site_cat=unconditional in_test=false rt_gated=false cg_present=false tf_attr=false
         51     /// Uniformly sampled u16.
         52     /// May fail in rare circumstances or heavy load.
         53     #[inline]
         54     pub fn get_u16(self) -> Option<u16> {
         55         let mut res: u16 = 0;
         56         unsafe {
    >>   57             match core::arch::x86_64::_rdrand16_step(&mut res) {
         58                 1 => Some(res),
         59                 x => {
         60                     debug_assert_eq!(x, 0, "rdrand16 returned non-binary value");
         61                     None
         62                 }
         63             }

VERDICT: ____


======================================================================

Missing tarballs: 0