# Ada L4 native block-grid confirmation — 2026-10-25

## Result

This independent-seed confirmation supports explicit block counts **232** and **464** over the 160-block reference for the tested L4 library, 256-thread call geometry, shops, and pack sizes. The two candidates were selected from the preceding pilot; this run evaluates those preselected candidates rather than searching the full grid again.

For the primary 200M-pack phase, the paired-ratio estimand favors **232** for shop 1 and **464** for shop 340 by a small margin. The equal-weight aggregate point estimate slightly favors 232. Both candidates beat 160 in every item/shop cell by the median paired ratio. For all three small-count phases (1M, 5M, 15M), 232 has the higher shop and aggregate point estimate. At 1M, its shop 340 confidence interval includes parity with 160; the 464 intervals include parity for both shops and aggregate.

### Primary paired-round ratios

Within each item/shop cell, each round's candidate throughput is divided by that same round's 160 throughput. The cell value is the median of those 20 paired ratios. Scores are geometric means of the 16 item-cell medians per shop; aggregate is the geometric mean of the two shop scores. Intervals are 95% percentile intervals from 50,000 paired stratified bootstrap replicates: resample rounds with replacement within each item/shop cell, retaining the same sampled rounds across grids.

| Scope | 232 / 160 | 95% interval | 464 / 160 | 95% interval |
|---|---:|---:|---:|---:|
| Shop 1 | 1.10035 (+10.04%) | 1.09653–1.10504 | 1.09683 (+9.68%) | 1.09279–1.09981 |
| Shop 340 | 1.09283 (+9.28%) | 1.08863–1.09571 | 1.09495 (+9.50%) | 1.09021–1.09979 |
| Equal-weight aggregate | 1.09658 (+9.66%) | 1.09380–1.09930 | 1.09589 (+9.59%) | 1.09279–1.09867 |

The intervals above compare each grid with 160. A separate paired bootstrap directly comparing 232 with 464 gives a geometric mean ratio of 1.00285 for shop 1 (95% interval 0.99996–1.00657), 0.99972 for shop 340 (0.99526–1.00250), and 1.00128 aggregate (0.99871–1.00361). These intervals include parity, so the confirmation does not show a reliable performance difference between the two candidates.

### Median throughput by item

Each entry is the median packs/s across 20 rounds, in **billions of packs/s**. Column groups are shop 1 and shop 340, each showing 160 / 232 / 464 blocks.

| Item | Shop 1: 160 | 232 | 464 | Shop 340: 160 | 232 | 464 |
|---:|---:|---:|---:|---:|---:|---:|
| e0 | 4.584 | 5.029 | 5.020 | 3.646 | 3.998 | 4.040 |
| e1 | 4.618 | 5.036 | 5.012 | 3.645 | 3.978 | 4.018 |
| e2 | 4.582 | 5.073 | 5.022 | 3.084 | 3.351 | 3.376 |
| e3 | 4.577 | 5.009 | 4.995 | 3.066 | 3.393 | 3.357 |
| e4 | 4.576 | 5.023 | 4.984 | 3.103 | 3.345 | 3.340 |
| e5 | 4.587 | 5.057 | 5.017 | 3.086 | 3.372 | 3.380 |
| e6 | 4.559 | 5.072 | 5.026 | 3.075 | 3.360 | 3.348 |
| e7 | 4.534 | 4.989 | 5.002 | 3.069 | 3.365 | 3.338 |
| e8 | 4.412 | 4.844 | 4.872 | 3.495 | 3.816 | 3.842 |
| e9 | 4.390 | 4.884 | 4.879 | 3.479 | 3.822 | 3.869 |
| e10 | 4.527 | 5.078 | 4.970 | 3.014 | 3.298 | 3.268 |
| e11 | 4.512 | 5.037 | 4.972 | 3.014 | 3.292 | 3.259 |
| e12 | 4.505 | 4.952 | 4.948 | 2.995 | 3.275 | 3.263 |
| e13 | 4.508 | 4.955 | 4.948 | 2.979 | 3.259 | 3.273 |
| e14 | 4.486 | 4.975 | 4.937 | 2.979 | 3.258 | 3.253 |
| e15 | 4.601 | 4.928 | 4.930 | 2.986 | 3.277 | 3.267 |

No item/shop cell had a median paired ratio below 1 for either candidate. Across all 640 paired rounds per candidate, 232 was slower than its same-round 160 baseline in 6 rounds; 464 was slower in 13 rounds.

### Separate geometric mean of median throughputs

The preceding pilot also summarized geometric means of each cell's median throughput. That is a different estimand from the median of paired round ratios above; the two summaries should not be conflated. Here the throughput-score ratios to 160 are 1.10176 / 1.09617 for shop 1 (232 / 464), 1.09349 / 1.09362 for shop 340, and 1.09762 / 1.09489 for the equal-weight aggregate. Absolute geometric mean median throughputs were:

| Scope | 160 (B packs/s) | 232 (B packs/s) | 464 (B packs/s) |
|---|---:|---:|---:|
| Shop 1 | 4.53449 | 4.99593 | 4.97056 |
| Shop 340 | 3.16146 | 3.45702 | 3.45745 |
| Equal-weight aggregate | 3.78624 | 4.15585 | 4.14553 |

This throughput score also favors 232 in aggregate, but the paired-ratio estimand remains the primary confirmation measure.

## Small-count phases

Each pack size used 10 rounds per item/shop cell for e0, e6, and e15. Counts are 60 rounds and 180 samples per size, or 180 rounds and 540 samples total across all three sizes. The paired-ratio point estimates and bootstrap intervals use the same procedure as above, with three item strata per shop and 10 resampled rounds per cell.

| Packs/sample | Scope | 232 / 160 (95% interval) | 464 / 160 (95% interval) |
|---:|---|---:|---:|
| 1M | Shop 1 | 1.0413 (1.0244–1.0573) | 1.0222 (0.9826–1.0487) |
| 1M | Shop 340 | 1.0167 (0.9975–1.0337) | 1.0003 (0.9875–1.0107) |
| 1M | Aggregate | 1.0289 (1.0159–1.0399) | 1.0112 (0.9907–1.0248) |
| 5M | Shop 1 | 1.0559 (1.0493–1.0628) | 1.0494 (1.0459–1.0572) |
| 5M | Shop 340 | 1.0571 (1.0497–1.0613) | 1.0459 (1.0387–1.0494) |
| 5M | Aggregate | 1.0565 (1.0518–1.0605) | 1.0476 (1.0438–1.0517) |
| 15M | Shop 1 | 1.0642 (1.0613–1.0672) | 1.0615 (1.0600–1.0644) |
| 15M | Shop 340 | 1.0735 (1.0708–1.0763) | 1.0697 (1.0679–1.0727) |
| 15M | Aggregate | 1.0688 (1.0669–1.0709) | 1.0656 (1.0645–1.0677) |

Geometric means of the three per-item median throughputs, in billions of packs/s:

| Packs/sample | Shop | 160 | 232 | 464 |
|---:|---:|---:|---:|---:|
| 1M | 1 | 2.734 | 2.831 | 2.779 |
| 1M | 340 | 1.879 | 1.908 | 1.890 |
| 5M | 1 | 4.206 | 4.456 | 4.423 |
| 5M | 340 | 2.973 | 3.140 | 3.105 |
| 15M | 1 | 4.613 | 4.912 | 4.902 |
| 15M | 340 | 3.303 | 3.545 | 3.536 |

At 1M, the only small-phase item/shop median paired ratio below baseline is 464 at shop 340/e6 (0.9819); 232 has no item/shop median regression. At 5M and 15M neither candidate has a median regression in any of the six item/shop cells.

## Correctness and run audit

- The retry completed on an NVIDIA L4 (compute capability 8.9), native target `sm_89`, driver 580.82.07, CUDA 13.0, NVCC 13.0.88. The recorded NVCC executable SHA-256 is `b8e2347d9d7fcadf8812a27324d515de99d033d3e48611811ece7e66d7895f8a`.
- The readiness gate passed: both native builds returned 0; actual library hashes matched readiness metadata; both readiness-tracked Turing GPU files (`kernel.cuh`, `dispatcher.cuh`) matched for base and candidate; inherited golden checks were 12/12 true per variant, inherited parity 32/32 true, and override checks 18/18 true. Digests were base library `6a38682cb450615ccb2303bc136672fb13e9da0c3f2c0ae51714444683f6f93a`, candidate library `f8479782a5aa55c43a17ad3327f9565efe22af5f4a0e67ee1831ae7e22a4c39a`; base kernel/dispatcher `ba8b8b4bbc290a905e7a6e7e3d4abd20ad4ced540627cc28082b8d581ac8e45e` / `c8beef17371af2a75c0f03604fe8bc3482b63a3a8b117442128d94b240628260`; candidate kernel/dispatcher `48d51484db920b48a5d91a6dfb2d5dc73f18798f7f1df5f76227622bcc577074` / `94baaf61a21a0a7390f68fefd1598745e027baabfacad3d017fd2d90de78d44b`.
- Same-grid output comparison passed for all 96 item/shop/grid cases: e0–e15 × shops 1/340 × grids 160/232/464. Kept and sold bytes matched between base and candidate at each identical grid, seed, item, and shop. All 192 sold outputs were empty; empty sold files are valid. This is a base/candidate comparison **within a grid**.
- Primary phase completed all 32 cells ×20 rounds: 640 rounds, 1,920 samples, and 1,280 non-baseline comparisons. Each grid/cell had a 20M-pack warmup and each timed sample processed 200M packs. The seed was `0x8b41d7e29c6035af`, distinct from the pilot seed `0x5d8ae4c21b79603f`. The sample order rotated by round; 160 was measured once per round and shared as the paired reference.
- Optional small phases completed 60 rounds/180 samples per size (1M, 5M, 15M); 180 rounds/540 samples total. These cover e0/e6/e15 for both shops.
- The retry saved 20/20 snapshots, final snapshot succeeded, result status was complete, campaign child exited 0, and the reference libraries/source hashes remained unchanged. The summary wrapper's generic sample/round fields are null; counts above are independently tallied from the complete raw result records.

The first attempt is preserved separately. It stopped before any parity or timing calls because its source-path check expected the two Turing files directly under `src/`, while they are under `src/gpu/arch/turing/`. Its snapshot failed with “download returned 1 or produced no data,” leaving no results JSON or timing samples. The corrected retry wrapper used the actual path; the recovery dataset above is complete and valid. The failed initialization is documented here and its original log/summary/script are retained in the raw artifacts.

## Interpretation and recommendation

The confirmation seed is independent of the pilot seed, but each grid shares a seed by design. Confidence intervals are descriptive, with no multiplicity adjustment across the several pack sizes, shops, and item cells. Changing `PACKSIM_BLOCKS` or the explicit positive block-count argument changes the grid and therefore the thread-seeded RNG stream topology. Same-seed outputs across different grids are not expected to match; this report makes no cross-grid parity claim. The runner removes inherited `PACKSIM_BLOCKS`/`PACKSIM_THREADS` environment overrides and passes the requested grid explicitly. The correctness check compares base and candidate only at the same grid.

For the tested L4 call geometry, use **232 as the leading opt-in candidate across pack sizes**, with a possible shop-specific 464 choice at shop 340 for 200M packs if a follow-up confirms its small estimated edge. At 1M packs, shop 340's 232 interval includes parity, so the measured advantage there is uncertain. These results support an opt-in L4 setting only; they do not establish a portable default or behavior on other GPUs/builds. Any policy/default change remains subject to separate authorization and compatibility review.

## Reproduction artifacts

Raw recovery results, execution log, summary, and corrected retry wrapper are retained in [`raw/`](raw/), along with the separate failed-attempt log, summary, and runner. [`2026-10-25-ada-l4-independent-grid-confirm-analyze.py`](raw/2026-10-25-ada-l4-independent-grid-confirm-analyze.py) regenerates the 50k stratified bootstrap and summaries; the generated JSON is [`analysis JSON`](raw/2026-10-25-ada-l4-independent-grid-confirm-analysis.json).
