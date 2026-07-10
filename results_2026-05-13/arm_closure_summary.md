# Reverse-dependency closure (lower bound)

Default-linked closure (`kind=normal`, `optional=false`, dependent
resolved to each crate's default version). Seeds excluded from their
own closure. See `pipeline/revdep` and PLAN.md section 5.

Corpus: 266128 crates, 301453258010 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (1): getrandom

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 34000 |
| Crate-count share of corpus | 12.776% |
| Direct dependents (depth 1) | 2117 |
| Downloads-weighted share | 1.84% |
| Closure downloads | 5538582293 |

## Wrapper-library closure (reported separately)

Seeds (3): aarch64-cpu, cortex-a, armv8

| Metric | Value |
|---|---:|
| Crates transitively pulling a wrapper | 142 |
| Crate-count share of corpus | 0.053% |
| Direct dependents (depth 1) | 36 |
| Downloads-weighted share | 0.00% |
| Closure downloads | 1478637 |
| Crates in both closures | 0 |

## Special-case direct dependents

Direct (depth-1) dependent counts for the crates PLAN section 5 tracks.

| Crate | In corpus | Direct dependents |
|---|---|---:|
| getrandom | yes | 2117 |
| ring | yes | 1052 |
| rand | yes | 19716 |
| rand_core | yes | 1509 |
| aws-lc-rs | yes | 116 |

## Top compile_time-closure crates by downloads

| Crate | Downloads | Depth |
|---|---:|---:|
| ring | 518332225 | 1 |
| jobserver | 372242970 | 1 |
| sct | 271876079 | 2 |
| dirs-sys | 229347900 | 2 |
| dirs | 223346200 | 3 |
| quinn-proto | 174816637 | 1 |
| quinn | 170407279 | 2 |
| dirs-sys-next | 151193667 | 2 |
| redox_users | 150041559 | 1 |
| dirs-next | 136475506 | 3 |
| const-random | 125879444 | 2 |
| jsonwebtoken | 125679255 | 1 |
| const-random-macro | 124940590 | 1 |
| webpki | 120466994 | 2 |
| backoff | 82470393 | 1 |

