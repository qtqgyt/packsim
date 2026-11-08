# Fresh A100 native 512-thread geometry confirmation

## Result and status

This fresh native confirmation compares 80 × 512 against 160 × 256 on an NVIDIA A100-SXM4-40GB (`sm_80`), preserving 40,960 logical threads. It recorded **640 main paired comparisons / 1,280 finite timed samples**: 20 alternating pairs for each e0–e15 item at shops 1 and 340. All 32 initial default kept/sold parity checks passed.

The main result file's overall status is **`failed`**, with the exact error `AttributeError("'list' object has no attribute 'setdefault'")` during the later small-workload/override coverage bookkeeping. The wrapper summary confirms that the JSON was parseable, the main stage had 640 pairs and 1,280 timed samples, but `remote_json_status=failed`. This report preserves that status and error unchanged. It analyzes the fully saved main payload after independently checking every pair, sample, finite rate, alternating order, and initial parity entry. It does not claim the main runner completed all planned phases.

The 200M main measurements used 20M warmup packs, `blocks=0`, and cleared `PACKSIM_BLOCKS`. Timing seed: `0xa100c0f1d5e7b923`; parity seed: `0xa1007a11d5e7b923`. The device query recorded A100-SXM4-40GB, driver 580.82.07, CUDA Toolkit 13.0.88.

## Main confirmation timing

The effect is the equal-weight geometric mean of per-item median paired candidate/base throughput ratios. The 95% CIs use 50,000 stratified paired-bootstrap replicates: independently resample the 20 rounds within each item, calculate each item's median ratio, then take the geometric mean across e0–e15.

| Shop | Change (95% CI) |
| ---: | ---: |
| 1 | +4.409% [+4.398%, +4.417%] |
| 340 | +29.639% [+29.619%, +29.662%] |

Every item median is positive at both shops. Median throughput is in billions of packs per second.

| Item | Shop 1 base | Shop 1 candidate | Change | Shop 340 base | Shop 340 candidate | Change |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| e0 | 5.1858 | 5.4151 | +4.434% | 4.1281 | 4.7513 | +15.080% |
| e1 | 5.1855 | 5.4153 | +4.414% | 4.1305 | 4.7513 | +15.033% |
| e2 | 5.2094 | 5.4451 | +4.525% | 3.5354 | 4.7689 | +34.870% |
| e3 | 5.2097 | 5.4457 | +4.524% | 3.5350 | 4.7682 | +34.886% |
| e4 | 5.2101 | 5.4443 | +4.497% | 3.5328 | 4.7683 | +34.956% |
| e5 | 5.2101 | 5.4471 | +4.555% | 3.5331 | 4.7678 | +34.940% |
| e6 | 5.2220 | 5.4143 | +3.684% | 3.5231 | 4.7536 | +34.924% |
| e7 | 5.2214 | 5.4149 | +3.695% | 3.5229 | 4.7529 | +34.928% |
| e8 | 5.1137 | 5.3874 | +5.367% | 3.9117 | 4.5408 | +16.044% |
| e9 | 5.1127 | 5.3862 | +5.373% | 3.9135 | 4.5401 | +16.000% |
| e10 | 5.2112 | 5.4461 | +4.524% | 3.4204 | 4.5981 | +34.426% |
| e11 | 5.2106 | 5.4469 | +4.529% | 3.4199 | 4.5975 | +34.401% |
| e12 | 5.2105 | 5.4459 | +4.527% | 3.4213 | 4.5987 | +34.411% |
| e13 | 5.2105 | 5.4460 | +4.511% | 3.4215 | 4.5981 | +34.423% |
| e14 | 5.2220 | 5.4137 | +3.701% | 3.4096 | 4.5928 | +34.687% |
| e15 | 5.2221 | 5.4148 | +3.704% | 3.4094 | 4.5918 | +34.697% |

This is a fresh seed and a separate confirmation from the earlier A100 native screen, which measured +4.354% at shop 1 (95% CI [+4.339%, +4.366%]) and +29.639% at shop 340 ([+29.619%, +29.680%]). Do not pool the two runs. Their shop-level effects are consistent.

## Recovered small-workload and explicit-geometry coverage

The failed main runner reached coverage bookkeeping after its full main timing stage, then raised the recorded `list.setdefault` error while saving a coverage sample. A separate recovery run used seed `0xf50ca100927e63b4` and the same frozen native libraries. It completed **240 paired comparisons / 480 timed samples** across 24 kept/sold parity cells, all passing. Coverage used e0/e6/e15 at both shops, ten alternating pairs per cell, 1M warmup for 1M/5M/15M default samples, and 20M warmup for the 200M explicit-160 control.

The recovery result uses a flat `item/shop/packs/mode` sample schema. Its results and summary both report complete status and 240 pairs. The analyzer checks all 24 cells and all 480 finite timed samples separately from the main payload.

| Packs / mode | Shop | Change (95% CI) |
| --- | ---: | ---: |
| 1M default | 1 | +1.191% [−0.400%, +3.206%] |
| 1M default | 340 | +13.498% [+11.502%, +14.938%] |
| 5M default | 1 | +2.689% [+2.009%, +3.270%] |
| 5M default | 340 | +23.042% [+22.543%, +23.465%] |
| 15M default | 1 | +3.486% [+3.331%, +3.748%] |
| 15M default | 340 | +25.968% [+25.631%, +26.339%] |
| 200M explicit 160 | 1 | +0.023% [−0.021%, +0.036%] |
| 200M explicit 160 | 340 | +0.009% [−0.014%, +0.055%] |

The 5M and 15M default groups are positive at both shops. At 1M, shop 340 is positive while shop 1's interval includes zero. One 1M shop-1 item median (e15) is −1.306%; the other two are positive. The explicit-160 control is neutral at both shops, confirming that the large default-geometry improvement does not appear when 160 × 256 is explicitly selected. Coverage is a targeted three-item sample, not all-item evidence at each pack count.

Per-item median throughput for the recovery is in billions of packs per second:

| Packs / mode | Shop | Item | Base | Candidate | Change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1M default | 1 | e0 | 2.7271 | 2.7894 | +1.812% |
| 1M default | 1 | e6 | 2.7198 | 2.8339 | +3.119% |
| 1M default | 1 | e15 | 2.7988 | 2.7950 | −1.306% |
| 1M default | 340 | e0 | 2.0035 | 2.1103 | +5.390% |
| 1M default | 340 | e6 | 1.8465 | 2.1587 | +17.324% |
| 1M default | 340 | e15 | 1.7625 | 2.0847 | +18.246% |
| 5M default | 1 | e0 | 4.4460 | 4.5861 | +3.325% |
| 5M default | 1 | e6 | 4.4833 | 4.5818 | +2.252% |
| 5M default | 1 | e15 | 4.4801 | 4.5827 | +2.494% |
| 5M default | 340 | e0 | 3.3908 | 3.8101 | +12.914% |
| 5M default | 340 | e6 | 2.9710 | 3.8138 | +28.734% |
| 5M default | 340 | e15 | 2.8607 | 3.6857 | +28.150% |
| 15M default | 1 | e0 | 4.9236 | 5.1204 | +3.828% |
| 15M default | 1 | e6 | 4.9591 | 5.1244 | +3.358% |
| 15M default | 1 | e15 | 4.9474 | 5.1227 | +3.273% |
| 15M default | 340 | e0 | 3.8374 | 4.3949 | +14.487% |
| 15M default | 340 | e6 | 3.3154 | 4.3942 | +32.447% |
| 15M default | 340 | e15 | 3.2052 | 4.2238 | +31.820% |
| 200M explicit 160 | 1 | e0 | 5.1845 | 5.1842 | +0.006% |
| 200M explicit 160 | 1 | e6 | 5.2221 | 5.2227 | +0.033% |
| 200M explicit 160 | 1 | e15 | 5.2221 | 5.2217 | +0.030% |
| 200M explicit 160 | 340 | e0 | 4.1288 | 4.1324 | +0.050% |
| 200M explicit 160 | 340 | e15 | 3.4121 | 3.4117 | −0.027% |
| 200M explicit 160 | 340 | e6 | 3.5299 | 3.5298 | +0.004% |

## Build, library, and source provenance

The frozen native reference preparation reports `reference_ready`: both sm_80 builds succeeded, each variant passed 12/12 goldens, initial default parity passed 32/32, and 18/18 override checks passed. Both timing phases reused these frozen libraries; the analyzer hashes the copied binaries and checks them against the reference metadata.

- Baseline commit: `17c42f6fa84abad25642f9cca605581688fe7a0a`.
- Base kernel/dispatcher SHA-256: `ba8b8b4bbc290a905e7a6e7e3d4abd20ad4ced540627cc28082b8d581ac8e45e` / `c8beef17371af2a75c0f03604fe8bc3482b63a3a8b117442128d94b240628260`.
- Candidate kernel/dispatcher SHA-256: `48d51484db920b48a5d91a6dfb2d5dc73f18798f7f1df5f76227622bcc577074` / `94baaf61a21a0a7390f68fefd1598745e027baabfacad3d017fd2d90de78d44b`.
- Reused base/candidate library SHA-256: `38d9d5e91bd7ddc5f1122693c5af1e0261d96e39864062594dd155262a88326b` / `ba72006fe7a47892cb7fd887e4f80f6e88b529bb85dc049b1be61879ddee37bc`.

## Recommendation and scope

The saved evidence supports evaluating an **A100-SXM4-40GB-specific** 80 × 512 default at the measured workloads. The fresh all-item 200M confirmation is positive at both shops with no item regressions; 5M and 15M targeted groups are positive at both shops; the 1M shop-1 group is inconclusive and includes one small e15 regression; explicit 160 × 256 is neutral. The 40,960-thread mapping is unchanged.

This does not support a blanket `sm_80` or all-A100 policy. The measured device is one A100-SXM4-40GB with driver 580.82.07; other A100 variants and other SM80 products were not timed. The current source at `44cf2e748df641e66f03f28b6103688ba4a3a706` scopes 80 × 512 to T4 and keeps 160 × 256 on other GPUs. An A100-specific policy would need an explicit model guard and its own compiled launch-bounds path, followed by source-level correctness and override validation before integration. The main run's final status failure remains part of the evidence record despite the complete saved main dataset.

## Reproduction and raw artifacts

From the repository root, run the analyzer locally; it does not access a GPU:

```sh
python3 docs/benchmarks/raw/2026-10-24-a100-geometry-512-native-confirm-analyze.py
```

It validates original result/summary/log SHA-256 values, the preserved main failure and complete main sample matrix, recovered coverage's 24-cell flat schema, reference build/golden/parity/override results, and both reused library hashes before recomputing all tables and 50,000-replicate intervals.

The copied main and recovery results, summaries, runner logs, executed scripts, reference metadata and build logs, and the two verified `.so` files are stored under `docs/benchmarks/raw/2026-10-24-a100-geometry-512-native-confirm-*`. Main results SHA-256: `dd1426a54ac146ede4da1222c236386f8653c3e7f8c960b783731bb9b5822ecf`; recovered coverage results SHA-256: `caf0a373a05262982907904ea0b475f91de6352efde01f8a3fa70884cfeaf9c5`.
