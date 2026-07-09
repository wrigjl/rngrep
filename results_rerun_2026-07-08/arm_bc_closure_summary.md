# Reverse-dependency closure (lower bound)

Default-linked closure (`kind=normal`, `optional=false`, dependent
resolved to each crate's default version). Seeds excluded from their
own closure. See `pipeline/revdep` and PLAN.md section 5.

Corpus: 295947 crates, 356393391514 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (381): merkurio, pixeldike, repa, ember-graph, pkboost, metallic, cargo-exam, sassy, obcore, cargo-updater, engram, enigma-cipher, enigma-cracker, enigma-simulator, ensure_simd, scanr, caustic, i0, caydence, iapws95, envcheck, ode_integrate, erydanos, cd-pls, aether-art, polyfit-residuals, essence, portablegl, cf1-rs, cf_speedtest, cfg-cli, icefast, aggrs, mini-film, mini_myers, scx_cake, pq-mayo, risinglight, chalamet_pir, minivault, rlnc, akaze, chela, chess_perft, primerpincer, rmsafe, exact-clustering, probemap, seq-hash, seqpls, seqwish, extract-frames, ambers, fabbula, aminograph, fahtsex, falcon512_rs, andlock, annatar, fast-grep, fast_fp, claude-token-monitor, fastdu, fastcsv, fasterp, fastmaths, fastwebsockets, fastwebsockets-monoio, fath, ikill, image-dwt, image-hdr, fcwt, ordag, ordent, fdf, ore-private-pool-cli, ore-private-pool-srv, ort-openrouter-cli, inari, apexbase, shin-hentai, ferromark, inferno-ai, rpn-c, infotheory, initium-cli, owens-t, bammap2, mmr, barbell, ffsvm, fiat-lux, aprender-db, modlin, interatomic, aprender-graph, oxide-rs, quadrank, bebelm, simd-json-derive, simd-lookup, simd-minimizers, simd-rs63, simd-sketch, simd_rand, simdphrase, monoio-tungstenite, monolithium, simple-sds-sbwt, simple-sds, moobi, cobs-simd, quickheap, simul, filerefine, qvnt-i, bempp, motif-rs, motedb, qvnt, sizr, ipset_lookup, flacenc, aprender-rag, rabitq-rs, oxits, aprender-viz, oxiwhisper, aprender-zram, multhreadown, packed-seq, fluor, mv-norm, codebase-to-markdown, codebase-to-xml, fmtsize, mysql_inserter, rawgrep, raycrypt, arctic-map, nano_clock, paraseq, rconvolve, nanocount, ark-feanor, natsuki, rssn, rssn-advanced, skope, frawk, armory_wallet_checksum_searcher, ncbi-vdb-sys, ncount, reader-sync, arq, patterns, neighborlist-rs, neonwhite_seed_finder, asm_tl, asmjson, redstone-ml, funcperm, asmcrypto, pdg-rs, regit-blackscholes, tinyget, neuralib, neuralrs, tjdistler-iqa, gbz, gbwt, tmi, gbz-base, pfv-rs, smooth-numbers, pggname, pgm_index, snakedown, tokio-websockets, pgv_rs, sntrup761, gencrypt, sockudo-ws, ggblas, sonic-rs, audio_samples_qoe, rt-graph, pic-scale-safe, pic-scale, itoa-benchmark, autodiff_rs, autoeq, spatial_alg, jdb_pgm, jedit, larz, gl-env, piscem, binseq, bit_ops, bitarena, bitarray, bitcode, laye, lccc-siphash, bitnuc, jmap-backup-rs, bitslice, treed_gpu, tremor-kv, go-brrr, legato, js-json-query, json-eval-rs, trypema, bintext, board-game, bio_utils_rs, kansei, kapacitor-udf, bonbon, bonsai-pot, kiddo, kira-autolys, kira-energetics, kira-mitoqc, kira-mmcif, kira-nuclearqc, kira-organelle, kira-proteoqc, kira-secretion, kira-riboqc, kira-spliceqc, node2text, keeprompt, keepass-diff, boxmux, kentro, libbladerf-rs, libbruteforce, bqtools, lokacore, turboquant-rs, bramble-rs, splik, bred, coinlive, coll-birth, collectivity, bridgitte, nox-mixnet, color-ssh, colorkit, libgrammstein, bropages, colorutils-rs, libm-ext, liblevenshtein, kowito-json, lowestbins, nucgen, libnoise, libote-codes, crypto2, compactrs, compensated-summation, compressed_vec, ksw2rs, ktstr, gprofile, configman, cabac, cabin-language, coopr, copc_converter, copia, copp, green-kernels, stabilizer-stream, lake, langr, csv_lib, stalkr, grepox, stardetect, underthesea_core, ctry, major-pickems-sim, runegate, runfile, cortex_rust, cube3d, rush-build, makigami, statsrelay, stealth, hibernia, rustedbytes-counterlines, cpu6502, hist-rs, gxhash, hitomi, hive-gpu, marsupial, hnsw, map2fig, darling-binary, darling-installer, darling_packages, rustvani, hot-loop, hackatime-heatmap, hamming-dict, hamming-lsh, dcat, rustytime-server, deacon, mdkb, mdlite, rwkv-tts-rs, hpx-fastwebsockets, htscodecs-wrapper, demine, walky, utpm, hp2, lling-llang, depi, strix, saltine-gromark, valar-spiral-rs, valar-ypir, vbinseq, vector-rust-library, wd2sql, varint-simd, wreck, wideword, hoox, wty, sunny, superline, vein-recall, wham, x3, lmutils, whisper-macos-cli, hdbscan-rs, lockpick, sux, tenebris, winit-gfx-hal, helioxide, swarm-bot, diffusionx, syncable-cli, t1ha, tachyon_json, xn, ticky, dissect, edgevec, ef_rs, yadf, taylorator, dxgi-capture-rs, dylex, dokkoo, dynamic_learned_index, yatch, dpf, yuv, yuvutils-rs, eiche, zackstrap, zawk, dsi-bitstream, dtoa-benchmark, dtact, tantivy-remote

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 1406 |
| Crate-count share of corpus | 0.475% |
| Direct dependents (depth 1) | 514 |
| Downloads-weighted share | 0.01% |
| Closure downloads | 29820951 |

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
| getrandom | yes | 2622 |
| ring | yes | 1103 |
| rand | yes | 21303 |
| rand_core | yes | 1713 |
| aws-lc-rs | yes | 143 |

## Top compile_time-closure crates by downloads

| Crate | Downloads | Depth |
|---|---:|---:|
| typst | 1618205 | 4 |
| video-levels | 1557628 | 1 |
| typst-svg | 1439577 | 2 |
| typst-realize | 1397690 | 4 |
| typst-html | 1397616 | 3 |
| iroh | 1075506 | 2 |
| hayro | 1044736 | 1 |
| iroh-relay | 929130 | 1 |
| deno_websocket | 673534 | 1 |
| deno | 629595 | 1 |
| deno_http | 616886 | 2 |
| typst-as-lib | 578418 | 5 |
| deno_runtime | 568848 | 1 |
| fast_websocket_client | 479677 | 1 |
| deno_node | 460679 | 2 |

