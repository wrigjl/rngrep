# Reverse-dependency closure (lower bound)

Default-linked closure (`kind=normal`, `optional=false`, dependent
resolved to each crate's default version). Seeds excluded from their
own closure. See `pipeline/revdep` and PLAN.md section 5.

Corpus: 295947 crates, 356393391514 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (10): concrete-csprng, foundation_compact, getrandom, hardware-rust-crypto, hyper-thread-random, lib-q-random, rmath, tfhe-csprng, typed_arch, vektor

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 39075 |
| Crate-count share of corpus | 13.203% |
| Direct dependents (depth 1) | 2633 |
| Downloads-weighted share | 1.80% |
| Closure downloads | 6419245369 |

## Wrapper-library closure (reported separately)

Seeds (6): rdrand, x86, x86_64, x64, lokacore, mc-rand

| Metric | Value |
|---|---:|
| Crates transitively pulling a wrapper | 287 |
| Crate-count share of corpus | 0.097% |
| Direct dependents (depth 1) | 110 |
| Downloads-weighted share | 0.01% |
| Closure downloads | 20603888 |
| Crates in both closures | 71 |

## Special-case direct dependents

Direct (depth-1) dependent counts for the crates PLAN section 5 tracks.

| Crate | In corpus | Direct dependents |
|---|---|---:|
| getrandom | yes | 2622 |
| ring | yes | 1103 |
| rand | yes | 21303 |
| rand_core | yes | 1713 |
| aws-lc-rs | yes | 143 |

## Top compile_time-closure crates by downloads

| Crate | Downloads | Depth |
|---|---:|---:|
| ring | 607343115 | 1 |
| jobserver | 437548701 | 1 |
| sct | 300347217 | 2 |
| dirs-sys | 272817717 | 2 |
| dirs | 264184809 | 3 |
| quinn-proto | 224616817 | 1 |
| quinn | 219191826 | 2 |
| redox_users | 180275712 | 1 |
| dirs-sys-next | 169354034 | 2 |
| dirs-next | 151856341 | 3 |
| jsonwebtoken | 148986694 | 1 |
| const-random | 145969217 | 2 |
| const-random-macro | 144925028 | 1 |
| webpki | 127226036 | 2 |
| backoff | 91059276 | 1 |

