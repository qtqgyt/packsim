# Blackwell native explicit block-grid confirmation

## Result

The independent full-item confirmation supports **188 blocks × 256 threads** as the strongest explicit Blackwell grid tested. Across all 16 runtime items and both shops, the canonical paired estimator gives **+16.89%** versus 160 blocks, with 50,000-replicate paired-bootstrap 95% CI **[+16.87%, +16.91%]**. Grid 376 is nearly tied at **+16.80% [+16.77%, +16.82%]**. The independent fresh-seed result confirms the pilot signal, and every item/shop cell improves at both 188 and 376.

This supports using 188 as an **explicit configuration** when that choice is available. Keep the Blackwell default at 160 × 256 pending a deliberate decision about changing default seed-stream behavior: block-grid changes alter thread-seeded RNG stream topology, and this run did not establish cross-grid output equality. The 160-block timing reference is called explicitly; the new run does not compare `blocks=0` default dispatch behavior.

## Method and estimator

The native Blackwell base library was measured at explicit grids 160, 188, and 376, each with 256 threads per block. The full-item phase covered e0–e15 at shops 1 and 340, with 20 rotating-order rounds per item/shop cell. Each grid/item/shop cell received one 20M-pack warmup, followed by timed samples of 200M packs. The fixed confirmation seed was `0xb17c6f048e29a3d5`, independent of the earlier pilot seed `0x42c9b8e7536ad10f`.

The primary estimator is the median across rounds of the **same-round** PPS ratio (grid / 160) for each item, followed by an equal-weight geometric mean across items per shop. The aggregate is the equal-weight geometric mean of the two shop effects. The interval resamples rounds within every shop/item stratum and preserves paired grid observations (50,000 replicates; primary bootstrap seed `20261025`). Ratio of median throughput is a different estimator; it is reported separately below. The 188/376 grids were selected for this confirmation after the seven-grid pilot, so the fresh-seed/all-item result is independent evidence for the selected grids, while the intervals are not adjusted for the prior candidate selection.

## Full-item throughput

Values are median throughput in billions of packs per second (B packs/s). Parenthesized changes use the canonical median of same-round ratios for that item/shop. These are descriptive per-cell rates; the group estimator and CIs follow below.

| Shop | Item | 160 blocks | 188 blocks | 188 change | 376 blocks | 376 change |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | e0 | 17.6914 | 20.6522 | +16.69% | 20.6684 | +16.82% |
| 1 | e1 | 17.6753 | 20.6471 | +16.82% | 20.6551 | +16.90% |
| 1 | e2 | 17.7249 | 20.7194 | +16.92% | 20.7305 | +16.96% |
| 1 | e3 | 17.7195 | 20.7225 | +16.90% | 20.7133 | +16.91% |
| 1 | e4 | 17.7263 | 20.7021 | +16.80% | 20.7138 | +16.83% |
| 1 | e5 | 17.7235 | 20.7194 | +16.92% | 20.7120 | +16.85% |
| 1 | e6 | 17.7002 | 20.6737 | +16.80% | 20.6663 | +16.80% |
| 1 | e7 | 17.6927 | 20.6492 | +16.69% | 20.6626 | +16.79% |
| 1 | e8 | 17.8051 | 20.8041 | +16.86% | 20.7945 | +16.81% |
| 1 | e9 | 17.8248 | 20.8164 | +16.77% | 20.8069 | +16.77% |
| 1 | e10 | 17.7279 | 20.7050 | +16.82% | 20.7157 | +16.87% |
| 1 | e11 | 17.7273 | 20.6941 | +16.79% | 20.7288 | +16.89% |
| 1 | e12 | 17.7202 | 20.7071 | +16.83% | 20.7007 | +16.83% |
| 1 | e13 | 17.7242 | 20.7120 | +16.86% | 20.7000 | +16.81% |
| 1 | e14 | 17.6959 | 20.6740 | +16.75% | 20.6550 | +16.70% |
| 1 | e15 | 17.6920 | 20.6687 | +16.80% | 20.6647 | +16.76% |
| 340 | e0 | 13.4753 | 15.7545 | +16.88% | 15.7099 | +16.58% |
| 340 | e1 | 13.4822 | 15.7573 | +16.87% | 15.7133 | +16.59% |
| 340 | e2 | 10.9692 | 12.8302 | +16.97% | 12.8094 | +16.75% |
| 340 | e3 | 10.9670 | 12.8334 | +17.02% | 12.7957 | +16.68% |
| 340 | e4 | 10.9674 | 12.8277 | +16.95% | 12.8011 | +16.75% |
| 340 | e5 | 10.9617 | 12.8299 | +17.02% | 12.7962 | +16.68% |
| 340 | e6 | 10.9448 | 12.8030 | +16.97% | 12.7974 | +16.98% |
| 340 | e7 | 10.9456 | 12.8023 | +16.95% | 12.7866 | +16.90% |
| 340 | e8 | 13.3201 | 15.5812 | +16.97% | 15.5409 | +16.60% |
| 340 | e9 | 13.3205 | 15.5835 | +16.96% | 15.5350 | +16.61% |
| 340 | e10 | 10.6810 | 12.5012 | +17.04% | 12.4714 | +16.80% |
| 340 | e11 | 10.6823 | 12.4996 | +17.02% | 12.4832 | +16.80% |
| 340 | e12 | 10.6782 | 12.5007 | +17.05% | 12.4807 | +16.95% |
| 340 | e13 | 10.6789 | 12.5088 | +17.12% | 12.4784 | +16.85% |
| 340 | e14 | 10.6803 | 12.4830 | +16.85% | 12.4725 | +16.81% |
| 340 | e15 | 10.6780 | 12.4837 | +16.91% | 12.4758 | +16.83% |

The canonical grouped changes and 95% intervals are:

| Shop/scope | Item group | 188 vs 160 (95% CI) | 376 vs 160 (95% CI) |
| --- | --- | ---: | ---: |
| Shop 1 | all 16 items | +16.814% [+16.788%, +16.840%] | +16.832% [+16.809%, +16.857%] |
| Shop 1 | common e0/e8 | +16.772% [+16.673%, +16.870%] | +16.815% [+16.742%, +16.871%] |
| Shop 1 | fixed e2/e6/e10/e15 | +16.834% [+16.787%, +16.863%] | +16.849% [+16.803%, +16.899%] |
| Shop 340 | all 16 items | +16.972% [+16.943%, +17.002%] | +16.759% [+16.721%, +16.806%] |
| Shop 340 | common e0/e8 | +16.925% [+16.865%, +16.984%] | +16.589% [+16.512%, +16.650%] |
| Shop 340 | fixed e2/e6/e10/e15 | +16.972% [+16.899%, +17.034%] | +16.839% [+16.770%, +16.977%] |
| Equal-weight shop aggregate | all 16 items | +16.893% [+16.874%, +16.913%] | +16.796% [+16.774%, +16.822%] |
| Equal-weight shop aggregate | common e0/e8 | +16.849% [+16.790%, +16.905%] | +16.702% [+16.650%, +16.745%] |
| Equal-weight shop aggregate | fixed e2/e6/e10/e15 | +16.903% [+16.859%, +16.936%] | +16.844% [+16.804%, +16.917%] |

The all-item geometric means of per-item median throughput are 17.7232 → 20.7041 → 20.7055 B packs/s at shop 1, and 11.4130 → 13.3513 → 13.3250 at shop 340 for 160/188/376. Equal-weighting shops gives 14.2223 → 16.6261 → 16.6103 B packs/s. These throughput summaries are distinct from the canonical paired-ratio estimator.

| Shop/scope | 188 ratio of median PPS | 376 ratio of median PPS |
| --- | ---: | ---: |
| Shop 1 | +16.820% | +16.827% |
| Shop 340 | +16.983% | +16.753% |
| Equal-weight shop aggregate | +16.901% | +16.790% |

Because both grids share the same round-level 160 baseline, their effects are correlated. Resampling the same round indices for both candidates gives a direct aggregate difference (188 minus 376) of +0.098 percentage points, paired 95% CI [+0.068, +0.123] points. At shop 1 the point difference is −0.017 points (CI [−0.051, +0.012]); at shop 340 it is +0.212 points (CI [+0.163, +0.255]). This direct paired comparison is more informative than treating the marginal candidate intervals as independent.

Both candidate grids improved all 16 item cells at each shop: 16/16 at shop 1 and 16/16 at shop 340 for both 188 and 376. The 188 gain is slightly higher at shop 340; at shop 1 the candidates are effectively tied. The fixed and common groups point in the same direction, so the full-item effect is not carried by just one item class.

## Small-pack-count checks

The optional check covered e0/e6/e15 at both shops, with ten rotating rounds per item/shop for each pack count. Each grid/item/shop cell received one 20M-pack warmup before its ten timed rounds. There were 60 rounds/180 samples per count and 180 rounds/540 samples total across 1M, 5M, and 15M. These checks have narrower item coverage than the primary phase and are reported separately.

| Packs | Shop | Item | 160 blocks | 188 blocks | 188 change | 376 blocks | 376 change |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000,000 | 1 | e0 | 7.438 | 7.653 | +4.37% | 7.636 | +2.95% |
| 1,000,000 | 1 | e6 | 7.332 | 8.012 | +5.22% | 7.682 | +3.19% |
| 1,000,000 | 1 | e15 | 7.609 | 8.142 | +8.96% | 7.676 | +0.20% |
| 1,000,000 | 340 | e0 | 5.655 | 6.521 | +13.47% | 6.018 | +6.63% |
| 1,000,000 | 340 | e6 | 5.206 | 5.712 | +7.79% | 5.446 | +3.57% |
| 1,000,000 | 340 | e15 | 5.010 | 5.421 | +9.05% | 5.417 | +7.81% |
| 5,000,000 | 1 | e0 | 13.625 | 15.348 | +12.94% | 15.370 | +13.24% |
| 5,000,000 | 1 | e6 | 13.714 | 15.354 | +11.91% | 15.421 | +12.53% |
| 5,000,000 | 1 | e15 | 13.748 | 15.449 | +11.97% | 15.447 | +11.91% |
| 5,000,000 | 340 | e0 | 10.334 | 11.712 | +13.51% | 11.795 | +13.86% |
| 5,000,000 | 340 | e6 | 8.827 | 10.033 | +13.44% | 9.930 | +12.32% |
| 5,000,000 | 340 | e15 | 8.554 | 9.780 | +14.67% | 9.709 | +14.20% |
| 15,000,000 | 1 | e0 | 16.226 | 18.630 | +14.85% | 18.668 | +15.62% |
| 15,000,000 | 1 | e6 | 16.151 | 18.617 | +15.17% | 18.630 | +15.30% |
| 15,000,000 | 1 | e15 | 16.192 | 18.582 | +14.94% | 18.614 | +14.71% |
| 15,000,000 | 340 | e0 | 12.259 | 14.196 | +15.54% | 14.044 | +14.68% |
| 15,000,000 | 340 | e6 | 10.103 | 11.681 | +15.59% | 11.686 | +15.61% |
| 15,000,000 | 340 | e15 | 9.878 | 11.419 | +15.49% | 11.407 | +15.69% |

Grouped small-count paired effects (three items per shop) are shown below. The analyzer uses bootstrap seeds `20261025 + pack_count` for these phases:

| Packs | Scope | 188 vs 160 (95% CI) | 376 vs 160 (95% CI) |
| ---: | --- | ---: | ---: |
| 1,000,000 | Shop 1 | +6.17% [+2.409%, +9.545%] | +2.10% [-0.466%, +6.922%] |
| 1,000,000 | Shop 340 | +10.08% [+8.757%, +12.674%] | +5.99% [+4.725%, +7.811%] |
| 1,000,000 | Equal-weight shop aggregate | +8.10% [+6.124%, +10.332%] | +4.03% [+2.589%, +6.622%] |
| 5,000,000 | Shop 1 | +12.27% [+11.718%, +13.180%] | +12.56% [+11.664%, +14.104%] |
| 5,000,000 | Shop 340 | +13.87% [+13.109%, +14.701%] | +13.46% [+12.410%, +14.300%] |
| 5,000,000 | Equal-weight shop aggregate | +13.07% [+12.603%, +13.671%] | +13.01% [+12.328%, +13.887%] |
| 15,000,000 | Shop 1 | +14.99% [+14.620%, +15.576%] | +15.21% [+14.683%, +15.887%] |
| 15,000,000 | Shop 340 | +15.54% [+15.112%, +15.793%] | +15.33% [+14.983%, +15.652%] |
| 15,000,000 | Equal-weight shop aggregate | +15.26% [+14.985%, +15.575%] | +15.27% [+14.951%, +15.646%] |

At 1M packs, 188 improves both shop groups (+6.17% at shop 1 and +10.08% at shop 340); 376 is weaker at shop 1 (+2.10%, CI crosses zero) and +5.99% at shop 340. At 5M and 15M, both grids improve both shops by roughly 12–15%. The 1M/5M/15M strata therefore support 188 more consistently at the smallest count, but do not establish performance for other pack counts or item mixes.

## Correctness and run audit

- The result JSON and all timing phases report `complete`. The collector returned 0, with 14/14 successful snapshots and a successful final snapshot. Its legacy summary count fields are null for this result schema; counts below are validated directly from the per-phase result arrays and phase counters. Primary counts are 640 rounds, 1,920 timed samples, and 1,280 nonbaseline paired comparisons (640 per candidate grid). Optional counts are 180 rounds and 540 samples across the three pack sizes.
- All 96 same-grid base/candidate cross-build checks passed kept and sold byte equality: 3 grids × 2 shops × 16 items. Each used 1M packs, explicit matching grid, and fixed seed. There are 192 zero-byte sold outputs (base plus candidate per case); empty sold output is valid. This is same-grid cross-build parity, **not cross-grid equality**.
- The run inherited readiness evidence for the already-compiled references: both readiness build return codes were 0; 12/12 saved goldens per build, 32/32 default-geometry parity checks, and 18/18 override checks passed in that prerequisite screen. This confirmation itself performed no new golden phase. `reference_unchanged_after_run` is true.
- Hardware recorded by the runner: NVIDIA RTX PRO 6000 Blackwell Server Edition, compute capability 12.0, driver 580.82.07, CUDA toolkit/compiler 13.0.88; target `sm_120`. The selected Blackwell card has 188 SMs. Candidate grids are 188 and 376 blocks × 256 threads; they launch 48,128 and 96,256 logical threads, respectively, versus 40,960 for 160 × 256.
- Readiness metadata SHA-256: `d82a4f26f2b28f01c60f22456e85ff9a9cf1c41a9bc232e6e2d52dba0160e8a4`. Base library SHA-256: `b7993b583e3575f97aaf93f63acfb810e17bc33277c64672fb75e192fda36e94`; candidate library SHA-256: `b07be11f88431fe0ac5dde5d73e28d8c3e6b9e99c10c64cfe7bcc12a091b59cf`. GPU source hashes matched the readiness metadata; the Blackwell timing phase used the base library only.
- Base source hashes: `kernel.cuh` `ba8b8b4bbc290a905e7e6a7e3d4abd20ad4ced540627cc28082b8d581ac8e45e`; `dispatcher.cuh` `c8beef17371af2a75c0f03604fe8bc3482b63a3a8b117442128d94b240628260`. Candidate source hashes: `kernel.cuh` `48d51484db920b48a5d91a6dfb2d5dc73f18798f7f1df5f76227622bcc577074`; `dispatcher.cuh` `94baaf61a21a0a7390f68fefd1598745e027baabfacad3d017fd2d90de78d44b`.

## Recommendation and limits

Keep 160 × 256 as the default for now, and recommend 188 × 256 as the explicit tuning choice for workloads where the caller selects block count. This full-item, fresh-seed confirmation supports the pilot signal: the seven-grid pilot used three items and ten rounds per item/shop, then selected 188/376 for confirmation; this fresh-seed run across all 16 items measured 188 at +16.89% aggregate versus 160, marginally ahead of 376, while using fewer blocks. Do not silently switch default geometry: changing the grid changes stream topology, so the observed output identity is not established across grids. The inherited correctness checks and the 96 parity cases only establish correctness within their stated fixture or same-grid scope.

The measurements are specific to this RTX PRO 6000, CUDA 13.0.88, the existing library hashes, shops 1 and 340, and tested pack counts. The bootstrap intervals quantify round-to-round variation within this run; they do not capture variation across devices, compiler versions, independent machines, or arbitrary workload distributions.

## Reproduction and raw artifacts

Recompute all validation, full-item tables, small-count summaries, and 50,000 paired-bootstrap intervals from the saved JSON:

```sh
python3 docs/benchmarks/raw/2026-10-24-blackwell-native-grid-confirm-analyze.py
```

The [raw artifacts](raw/) include the result JSON, runner log, collector summary, exact executed wrapper and generic runner, generated analysis JSON, and the earlier grid-sweep pilot result/log/summary/wrapper under separate `*-pilot-*` names. The confirmation result JSON SHA-256 is `066d96c4d1619288ffe297a8d8ffe1dd6c58536a6ed7d6121bfc0f3cdcef865f`. The pilot remains selection context only; the fresh-seed all-item run is the confirmation evidence.
