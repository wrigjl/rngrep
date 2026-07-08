# Reverse-dependency closure (upper bound)

Include-optional closure (`kind=normal`, optional deps kept,
dependent resolved to each crate's default version). Counts every
crate that *could* inherit the source if the relevant feature is
enabled. Seeds excluded from their own closure. See
`pipeline/revdep` and PLAN.md section 5.

Corpus: 266128 crates, 301453258010 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (7): typed_arch, getrandom, vektor, rmath, concrete-csprng, tfhe-csprng, hyper-thread-random

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 186906 |
| Crate-count share of corpus | 70.232% |
| Direct dependents (depth 1) | 2666 |
| Downloads-weighted share | 46.62% |
| Closure downloads | 140535344145 |

## Wrapper-library closure (reported separately)

Seeds (6): rdrand, x86, x86_64, x64, lokacore, mc-rand

| Metric | Value |
|---|---:|
| Crates transitively pulling a wrapper | 186586 |
| Crate-count share of corpus | 70.111% |
| Direct dependents (depth 1) | 2615 |
| Downloads-weighted share | 45.27% |
| Closure downloads | 136467818432 |
| Crates in both closures | 186545 |

## Special-case direct dependents

Direct (depth-1) dependent counts for the crates PLAN section 5 tracks.

| Crate | In corpus | Direct dependents |
|---|---|---:|
| getrandom | yes | 2581 |
| ring | yes | 1286 |
| rand | yes | 21414 |
| rand_core | yes | 1708 |
| aws-lc-rs | yes | 209 |

## Top compile_time-closure crates by downloads

| Crate | Downloads | Depth |
|---|---:|---:|
| hashbrown | 1552373419 | 8 |
| rand | 1135228462 | 1 |
| indexmap | 1060637885 | 3 |
| memchr | 947653009 | 8 |
| serde_json | 891697204 | 4 |
| regex-automata | 891130385 | 8 |
| once_cell | 878066156 | 5 |
| log | 878027484 | 7 |
| cc | 865149647 | 2 |
| regex | 825646695 | 9 |
| clap | 819472689 | 5 |
| aho-corasick | 811897812 | 8 |
| parking_lot_core | 802760782 | 4 |
| parking_lot | 779726819 | 5 |
| mio | 706345198 | 8 |

