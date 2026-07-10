# Reverse-dependency closure (lower bound)

Default-linked closure (`kind=normal`, `optional=false`, dependent
resolved to each crate's default version). Seeds excluded from their
own closure. See `pipeline/revdep` and PLAN.md section 5.

Corpus: 266128 crates, 301453258010 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (389): mdkb, mdlite, redstone-ml, pic-scale, pic-scale-safe, rs_poker, nox-mixnet, rs_tfhe, neighborlist-rs, saltine-gromark, neonwhite_seed_finder, regit-blackscholes, RustedSciThe, tinyget, reindeer2, tjdistler-iqa, multhreadown, piscem, repa, tmi, nucgen, pixeldike, merkurio, pkboost, numeris, sassy, metallic, mv-norm, tokio-websockets, neuralib, mysql_inserter, obcore, scanr, pmcp, aegis-cli, aegisvault, aether-art, nano_clock, nanocount, ode_integrate, natsuki, ncbi-vdb-sys, polyfit-residuals, ncount, omendb, rust_client, akaze, portablegl, rustedbytes-counterlines, oohid, risinglight, pq-mayo, treed_gpu, mini_myers, ambers, tremor-kv, seekstorm, rlnc, aminograph, andlock, minivault, seqpls, rmsafe, rssn, rssn-advanced, seqwish, primerpincer, annatar, probemap, procx, rusty_neat, rustytime-server, trypema, apexbase, rwkv-tts-rs, rpn-c, ordag, ordent, ore-private-pool-cli, ore-private-pool-srv, shin-hentai, turboquant-rs, ort-openrouter-cli, owens-t, runegate, runfile, aprender-db, aprender-graph, node2text, rush-build, noest, oxide-rs, qft, spel-right, patterns, quadrank, mmr, aprender-rag, quickheap, oxits, oxiwhisper, qvnt-i, qvnt, simd-json-derive, simd-minimizers, simd-lookup, simd-sketch, simd_rand, simdphrase, monoio-tungstenite, monolithium, moobi, simple-sds-sbwt, simple-sds, packed-seq, rt-graph, simul, motedb, pfv-rs, motif-rs, pggname, pgm_index, sizr, pgv_rs, underthesea_core, rabitq-rs, aprender-viz, aprender-zram, paraseq, utpm, ark-feanor, ark-spartan-golden, smooth-numbers, snakedown, rawgrep, splik, raycrypt, sntrup761, wreck, valar-spiral-rs, walky, valar-ypir, sockudo-ws, wty, armory_wallet_checksum_searcher, arq, varint-simd, rconvolve, x3, vbinseq, vector-rust-library, sonic-rs, asmcrypto, asmjson, wave_func_collapse, wd2sql, xn, spartan, spatial_alg, skope, wham, stabilizer-stream, yadf, wideword, stalkr, stardetect, reader-sync, yatch, statsrelay, stealth, yuv, yuvutils-rs, zackstrap, winit-gfx-hal, zawk, vertx-rust, yom, htscodecs-wrapper, autodiff_rs, sunny, superbloom, ember-graph, sux, swarm-bot, syncable-cli, strix, gbwt, gbz, gbz-base, t1ha, engram, fdf, tachyon_json, enigma-cipher, enigma-cracker, enigma-simulator, enk, ensure_simd, envcheck, gencrypt, eqr, ferromark, cargo-exam, erydanos, esed, cargo-rclean, ffsvm, cargo-updater, ggblas, crypto2, caustic, essence, caydence, fiat-lux, gprofile, exact-clustering, cd-pls, gl-env, fabbula, filerefine, icefast, fahtsex, green-kernels, csv_lib, grepox, cf1-rs, cf_speedtest, cfg-cli, fast-grep, ctry, fast_fp, cube3d, fastdu, fastcsv, fasterp, go-brrr, fastmaths, flacenc, fastwebsockets, fastwebsockets-monoio, fath, fcwt, chalamet_pir, binseq, bintext, bio_utils_rs, bit_ops, bitarena, bitarray, bitcode, gxhash, chela, bitnuc, chess_perft, bitslice, fmtcnv, fmtsize, hackatime-heatmap, darling-installer, darling-binary, darling_packages, dat, claude-token-monitor, dcat, ikill, bammap2, barbell, image-dwt, deacon, image-hdr, image-find, board-game, frawk, bonbon, bonsai-pot, batlimit, inari, inferno-ai, infotheory, boxmux, bqtools, demine, interatomic, funcperm, bempp, bred, bridgitte, bropages, depi, ipset_lookup, taylorator, hamming-dict, hamming-lsh, tantivy-remote, irox-arch-x86-64, cobs-simd, tenebris, burn_depth, hibernia, ticky, cabac, cabin-language, hist-rs, hitomi, hive-gpu, hdbscan-rs, configman, helioxide, coolfindpattern, diffusionx, coopr, copc_converter, copp, cortex_rust, dissect, dither-qr, compactrs, compensated-summation, compressed_vec, ksw2rs, coinlive, dokkoo, codebase-to-markdown, collectivity, ktstr, domaincheck, color-ssh, colorkit, colorutils-rs, dpf, dqr, lake, major-pickems-sim, dsi-bitstream, hoox, dtact, makigami, dtoa-benchmark, hnsw, hot-loop, map2fig, dxgi-capture-rs, dylex, larz, lmutils, localsnd, marsupial, lockpick, libbladerf-rs, libbruteforce, lccc-siphash, lokacore, hp2, libdeflate, hpx-fastwebsockets, lowestbins, legato, edgevec, ef_rs, liblevenshtein, libm-ext, libnoise, itoa-benchmark, libote-codes, kansei, kapacitor-udf, jdb_pgm, jedit, js-json-query, json-eval-rs, jsontoml, keeprompt, keepass-diff, kentro, eiche, kk-crypto, kowito-json, kira-autolys, kira-energetics, kira-mitoqc, kira-mmcif, kira-organelle, kira-nuclearqc, kira-proteoqc, kira-riboqc, kira-secretion, kira-spliceqc, listsend, kiddo

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 1056 |
| Crate-count share of corpus | 0.397% |
| Direct dependents (depth 1) | 455 |
| Downloads-weighted share | 0.01% |
| Closure downloads | 16849834 |

## Wrapper-library closure (reported separately)

Seeds (0): 

| Metric | Value |
|---|---:|
| Crates transitively pulling a wrapper | 0 |
| Crate-count share of corpus | 0.000% |
| Direct dependents (depth 1) | 0 |
| Downloads-weighted share | 0.00% |
| Closure downloads | 0 |
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
| video-levels | 1348858 | 1 |
| iroh | 665929 | 2 |
| deno_websocket | 645799 | 1 |
| deno | 627310 | 2 |
| deno_http | 591106 | 2 |
| deno_runtime | 548009 | 1 |
| iroh-relay | 537518 | 1 |
| deno_node | 439170 | 2 |
| fast_websocket_client | 394554 | 1 |
| twilight-gateway | 373967 | 1 |
| volo-thrift | 303881 | 1 |
| hive_metastore | 282439 | 2 |
| evtx | 186996 | 1 |
| iroh-blobs | 159937 | 3 |
| oculante | 149182 | 2 |

