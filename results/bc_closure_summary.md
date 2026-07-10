# Reverse-dependency closure (lower bound)

Default-linked closure (`kind=normal`, `optional=false`, dependent
resolved to each crate's default version). Seeds excluded from their
own closure. See `pipeline/revdep` and PLAN.md section 5.

Corpus: 295947 crates, 356393391514 total downloads.

## Compile-time RDRAND closure (headline)

Seeds (382): aether-art, aggrs, akaze, ambers, aminograph, andlock, annatar, apexbase, aprender-db, aprender-graph, aprender-rag, aprender-viz, aprender-zram, arctic-map, ark-feanor, armory_wallet_checksum_searcher, arq, asm_tl, asmcrypto, asmjson, audio_samples_qoe, autodiff_rs, autoeq, bammap2, barbell, bebelm, bempp, binseq, bintext, bio_utils_rs, bit_ops, bitarena, bitarray, bitcode, bitnuc, bitslice, board-game, bonbon, bonsai-pot, boxmux, bqtools, bramble-rs, bred, bridgitte, bropages, cabac, cabin-language, cargo-exam, cargo-updater, caustic, caydence, cd-pls, cf1-rs, cf_speedtest, cfg-cli, chalamet_pir, chela, chess_perft, claude-token-monitor, cobs-simd, codebase-to-markdown, codebase-to-xml, coinlive, coll-birth, collectivity, color-ssh, colorkit, colorutils-rs, compactrs, compensated-summation, compressed_vec, configman, coopr, copc_converter, copia, copp, cortex_rust, cpu6502, crypto2, csv_lib, ctry, cube3d, darling-binary, darling-installer, darling_packages, dcat, deacon, demine, depi, diffusionx, dissect, dokkoo, dpf, dsi-bitstream, dtact, dtoa-benchmark, dxgi-capture-rs, dylex, dynamic_learned_index, edgevec, ef_rs, eiche, ember-graph, engram, enigma-cipher, enigma-cracker, enigma-simulator, ensure_simd, envcheck, erydanos, essence, exact-clustering, extract-frames, fabbula, fahtsex, falcon512_rs, fast-grep, fast_fp, fastcsv, fastdu, fasterp, fastmaths, fastwebsockets, fastwebsockets-monoio, fath, fcwt, fdf, ferromark, ffsvm, fiat-lux, filerefine, flacenc, fluor, fmtsize, frawk, funcperm, gbwt, gbz, gbz-base, gencrypt, ggblas, gl-env, go-brrr, gprofile, green-kernels, grepox, gxhash, hackatime-heatmap, hamming-dict, hamming-lsh, hdbscan-rs, helioxide, hibernia, hist-rs, hitomi, hive-gpu, hnsw, hoox, hot-loop, hp2, hpx-fastwebsockets, htscodecs-wrapper, i0, iapws95, icefast, ikill, image-dwt, image-hdr, inari, inferno-ai, infotheory, initium-cli, interatomic, ipset_lookup, irox-arch-x86-64, itoa-benchmark, jdb_pgm, jedit, jmap-backup-rs, js-json-query, json-eval-rs, kansei, kapacitor-udf, keepass-diff, keeprompt, kentro, kiddo, kira-autolys, kira-energetics, kira-mitoqc, kira-mmcif, kira-nuclearqc, kira-organelle, kira-proteoqc, kira-riboqc, kira-secretion, kira-spliceqc, kowito-json, ksw2rs, ktstr, lake, langr, larz, laye, lccc-siphash, legato, libbladerf-rs, libbruteforce, libgrammstein, liblevenshtein, libm-ext, libnoise, libote-codes, lling-llang, lmutils, lockpick, lokacore, lowestbins, major-pickems-sim, makigami, map2fig, marsupial, mdkb, mdlite, merkurio, metallic, mini-film, mini_myers, minivault, mmr, modlin, monoio-tungstenite, monolithium, moobi, motedb, motif-rs, multhreadown, mv-norm, mysql_inserter, nano_clock, nanocount, natsuki, ncbi-vdb-sys, ncount, neighborlist-rs, neonwhite_seed_finder, neuralib, neuralrs, node2text, nox-mixnet, nucgen, obcore, ode_integrate, ordag, ordent, ore-private-pool-cli, ore-private-pool-srv, ort-openrouter-cli, owens-t, oxide-rs, oxits, oxiwhisper, packed-seq, paraseq, patterns, pdg-rs, pfv-rs, pggname, pgm_index, pgv_rs, pic-scale, pic-scale-safe, piscem, pixeldike, pkboost, polyfit-residuals, portablegl, pq-mayo, primerpincer, probemap, quadrank, quickheap, qvnt, qvnt-i, rabitq-rs, rawgrep, raycrypt, rconvolve, reader-sync, redstone-ml, regit-blackscholes, repa, risinglight, rlnc, rmsafe, rpn-c, rssn, rssn-advanced, rt-graph, runegate, runfile, rush-build, rustedbytes-counterlines, rustvani, rustytime-server, rwkv-tts-rs, saltine-gromark, sassy, scanr, scx_cake, seq-hash, seqpls, seqwish, shin-hentai, simd-json-derive, simd-lookup, simd-minimizers, simd-rs63, simd-sketch, simd_rand, simdphrase, simple-sds, simple-sds-sbwt, simul, sizr, skope, smooth-numbers, snakedown, sntrup761, sockudo-ws, sonic-rs, spatial_alg, splik, stabilizer-stream, stalkr, stardetect, statsrelay, stealth, strix, sunny, superline, sux, swarm-bot, syncable-cli, t1ha, tachyon_json, tantivy-remote, taylorator, tenebris, ticky, tinyget, tjdistler-iqa, tmi, tokio-websockets, treed_gpu, tremor-kv, trypema, turboquant-rs, underthesea_core, utpm, valar-spiral-rs, valar-ypir, varint-simd, vbinseq, vector-rust-library, vein-recall, walky, wd2sql, wham, whisper-macos-cli, wideword, winit-gfx-hal, wreck, wty, x3, xn, yadf, yatch, yuv, yuvutils-rs, zackstrap, zawk

| Metric | Value |
|---|---:|
| Crates transitively pulling a compile_time source | 1408 |
| Crate-count share of corpus | 0.476% |
| Direct dependents (depth 1) | 515 |
| Downloads-weighted share | 0.01% |
| Closure downloads | 29821099 |

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

