# Reverse-dependency closure (upper bound)

Include-optional closure (`kind=normal`, optional deps kept,
dependent resolved to each crate's default version). Counts every
crate that *could* inherit the source if the relevant feature is
enabled. Seeds excluded from their own closure. See
`pipeline/revdep` and PLAN.md section 5.

Corpus: 295947 crates, 356393391514 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (10): concrete-csprng, foundation_compact, getrandom, hardware-rust-crypto, hyper-thread-random, lib-q-random, rmath, tfhe-csprng, typed_arch, vektor

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 210073 |
| Crate-count share of corpus | 70.983% |
| Direct dependents (depth 1) | 3251 |
| Downloads-weighted share | 46.47% |
| Closure downloads | 165599076757 |

## Wrapper-library closure (reported separately)

Seeds (6): rdrand, x86, x86_64, x64, lokacore, mc-rand

| Metric | Value |
|---|---:|
| Crates transitively pulling a wrapper | 209723 |
| Crate-count share of corpus | 70.865% |
| Direct dependents (depth 1) | 3190 |
| Downloads-weighted share | 45.28% |
| Closure downloads | 161364990876 |
| Crates in both closures | 209684 |

## Special-case direct dependents

Direct (depth-1) dependent counts for the crates PLAN section 5 tracks.

| Crate | In corpus | Direct dependents |
|---|---|---:|
| getrandom | yes | 3150 |
| ring | yes | 1356 |
| rand | yes | 23112 |
| rand_core | yes | 1953 |
| aws-lc-rs | yes | 267 |

## Top compile_time-closure crates by downloads

| Crate | Downloads | Depth |
|---|---:|---:|
| hashbrown | 1903808015 | 8 |
| rand | 1351861078 | 1 |
| indexmap | 1249048896 | 3 |
| memchr | 1120026204 | 8 |
| serde_json | 1042964632 | 4 |
| regex-automata | 1030741455 | 8 |
| once_cell | 1026468409 | 5 |
| log | 1018587730 | 7 |
| cc | 1011187278 | 2 |
| regex | 953827935 | 9 |
| clap | 948258312 | 5 |
| aho-corasick | 939316595 | 8 |
| parking_lot_core | 925668471 | 4 |
| parking_lot | 900178249 | 5 |
| digest | 837441934 | 2 |

