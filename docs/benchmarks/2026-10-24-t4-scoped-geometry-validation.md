# T4-scoped 512-thread geometry: correctness and SASS validation

## Scope and candidate policy

This report evaluates a T4-only default geometry policy against the existing 160 × 256 default. On a Tesla T4, with no positive `blocks` argument or `PACKSIM_BLOCKS` setting, the dispatcher selects 80 × 512 and retains 40,960 logical threads. It selects this path only after successfully querying a CUDA device with compute capability 7.5, 40 SMs, and a name containing `T4`. A positive argument or environment block count keeps the explicit 256-thread geometry. Other devices keep 160 × 256.

The kernel uses `__launch_bounds__(512,2)` when compiled for `__CUDA_ARCH__ == 750`; other targets retain `__launch_bounds__(256,4)`. That compile-time condition includes every `sm_75` board, even though the runtime 80 × 512 dispatch guard is specific to a 40-SM T4 name. The dispatcher and kernel source hashes for this candidate are `b1e079dcc149ac521e172c398efd29201722cbc9325aab251ff1e30ca27ccc30` and `7126278a9b974ca5176c7c4d85235d15d9ccf7c6963052db1496f387f7401e47`. The candidate was developed on isolated branch `codex/t4-geometry-scoped`; the validated dispatcher/kernel change is integrated with this report.

## Four-architecture validation

Validation built the scoped candidate for the native architecture on each GPU. It passed the same correctness counts on all four devices: 12/12 golden fixtures per build, 32/32 e0–e15 default kept/sold comparisons at shops 1 and 340, and 18/18 kept/sold override checks for e0/e6/e15 at both shops. All result files report `correctness_passed_sass_equal`; no timing was performed in these validation runs.

| Device | Target | Source-reviewed default | SASS comparison reference | Goldens / parity / overrides | Raw and normalized SASS SHA-256 |
| --- | ---: | ---: | --- | ---: | --- |
| Tesla T4 | sm_75 | 80 × 512 | Measured, unscoped 80 × 512 T4 library | 12/12 · 32/32 · 18/18 | `30ecd37d0a7c00e8aea8bb803755d4e3a99de4d1e5aaeb9eac797f747c3b5b32` |
| NVIDIA L4 | sm_89 | 160 × 256 | Native 160 × 256 base library | 12/12 · 32/32 · 18/18 | `466f56f5a76c4c548f4bb1ebf3f2c2292894ff18cf2823cb91a0332a778a437b` |
| NVIDIA A100-SXM4-40GB | sm_80 | 160 × 256 | Native 160 × 256 base library | 12/12 · 32/32 · 18/18 | `b3129834ba0e5d296bd3e7d420fedf31f34ffc2ccd0081281c6732beebff4a7a` |
| NVIDIA RTX PRO 6000 Blackwell Server Edition | sm_120 | 160 × 256 | Native 160 × 256 base library | 12/12 · 32/32 · 18/18 | `c9ab6c84cbc64ba2ed2d3d4e19fe2c51a091cd9133e7d108faa2b66b91394880` |

For each device, the raw candidate and reference SASS strings are byte-for-byte equal. Their normalized strings are also byte-for-byte equal. Normalization follows `run_t4_scoped_geometry_validate.py`: replace the library path with `<LIBRARY>`, then replace only path-bearing `File:` headers. Each embedded raw SASS output matches its separately saved sidecar. The compressed validation JSON retains both complete SASS outputs; separate 50-plus-MB SASS files were not duplicated into this report's raw-artifact directory.

This is compile-time SASS evidence and source-reviewed dispatch behavior. The validator explicitly records `runtime_geometry_observation.observed=false`; it did not profile or interpose the launch to observe blocks and threads at runtime. On non-T4 targets, equality with the native base supports that the scoped change emits the same code for those tested architectures. It does not establish performance on those GPUs.

## Additional comparison with the former sm_75 baseline

A CPU-only disassembly of the saved native 160 × 256 baseline was also compared with the scoped 512-thread build. The complete 56,755,005-byte SASS outputs are identical, with SHA-256 `30ecd37d0a7c00e8aea8bb803755d4e3a99de4d1e5aaeb9eac797f747c3b5b32`. The downloaded CUDA 13 disassembler matched the tool hash recorded by the validator. The comparison command, binary hashes, tool hashes, sizes and equality result are preserved in `raw/2026-10-24-t4-scoped-geometry-validation-baseline256-sass.json`.

Thus the altered sm_75 launch bounds did not change the emitted instruction stream in this build. The maximum permitted block size metadata changes, while other sm_75 boards still receive the existing 256-thread runtime launch. Performance on those other boards was not measured.

## T4 performance and workload coverage

The performance evidence comes from the separate native T4 timing reports, not these correctness/SASS validations. The fresh 200M-pack confirmation measured all e0–e15 at both shops with 20 alternating pairs per item/shop. The geomean of per-item paired median ratios was **+1.432%** at shop 1 (50,000 stratified paired-bootstrap 95% CI **[+1.293%, +1.599%]**) and **+6.957%** at shop 340 (**[+6.860%, +7.093%]**). All item medians were positive. See [the full T4 confirmation report](2026-10-24-turing-geometry-512-native-confirm.md).

A separate saved coverage run compared the default geometry for e0, e6, and e15 at 1M, 5M, and 15M packs, and used 200M packs with explicit 160 blocks as a neutral control. There were 24 workload/shop parity cells, all kept/sold-equal, and 240 paired timing observations / 480 timed samples. Each cell has ten alternating pairs. The small runs use 1M warmup; the 200M control uses 20M warmup. Throughput entries below are medians across those ten rounds, in billions of packs per second; the change is the median of the paired per-round throughput ratios.

| Workload and mode | Shop | Item | Base B packs/s | Candidate B packs/s | Paired-median change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1M default | 1 | e0 | 1.8013 | 1.8594 | +3.273% |
| 1M default | 1 | e6 | 1.8066 | 1.8410 | +1.578% |
| 1M default | 1 | e15 | 1.9004 | 1.9324 | +1.626% |
| 1M default | 340 | e0 | 1.4042 | 1.4592 | +4.059% |
| 1M default | 340 | e6 | 1.3762 | 1.4628 | +6.206% |
| 1M default | 340 | e15 | 1.3279 | 1.4161 | +5.983% |
| 5M default | 1 | e0 | 2.6241 | 2.6721 | +1.481% |
| 5M default | 1 | e6 | 2.6247 | 2.6479 | +1.498% |
| 5M default | 1 | e15 | 2.6306 | 2.6605 | +1.347% |
| 5M default | 340 | e0 | 2.0366 | 2.1197 | +5.824% |
| 5M default | 340 | e6 | 2.0227 | 2.1737 | +7.068% |
| 5M default | 340 | e15 | 1.9099 | 2.0667 | +8.195% |
| 15M default | 1 | e0 | 2.4196 | 2.4605 | +1.678% |
| 15M default | 1 | e6 | 2.7296 | 2.7688 | +1.614% |
| 15M default | 1 | e15 | 2.8014 | 2.8482 | +1.611% |
| 15M default | 340 | e0 | 2.1765 | 2.3643 | +7.926% |
| 15M default | 340 | e6 | 1.9199 | 2.0679 | +8.019% |
| 15M default | 340 | e15 | 2.0902 | 2.2193 | +8.485% |
| 200M explicit 160 | 1 | e0 | 2.1938 | 2.2001 | +0.442% |
| 200M explicit 160 | 1 | e6 | 2.1479 | 2.1481 | −0.074% |
| 200M explicit 160 | 1 | e15 | 2.1539 | 2.1595 | +0.591% |
| 200M explicit 160 | 340 | e0 | 1.7455 | 1.7375 | −0.497% |
| 200M explicit 160 | 340 | e6 | 1.7612 | 1.7679 | +0.352% |
| 200M explicit 160 | 340 | e15 | 1.6881 | 1.6823 | −0.461% |

Group effects are geometric means of each item's paired median ratio. The 95% intervals use 50,000 stratified paired-bootstrap replicates: resample the ten rounds independently within each of e0/e6/e15, calculate each item's median ratio, then take the equal-weight geometric mean across the three items. Seeds are deterministic per group in the analyzer.

| Packs / mode | Shop | Group change (95% CI) |
| --- | ---: | ---: |
| 1M default | 1 | +2.156% [+0.259%, +3.320%] |
| 1M default | 340 | +5.411% [+3.590%, +7.430%] |
| 5M default | 1 | +1.442% [+0.885%, +1.972%] |
| 5M default | 340 | +7.025% [+5.698%, +8.168%] |
| 15M default | 1 | +1.635% [+0.100%, +2.069%] |
| 15M default | 340 | +8.143% [+7.092%, +8.725%] |
| 200M explicit 160 | 1 | +0.319% [−0.388%, +1.401%] |
| 200M explicit 160 | 340 | −0.203% [−0.548%, +0.113%] |

The small-workload groups are positive for these three selected items and both shops. The explicit-160 control is neutral at both shops, as expected when the candidate's default geometry is bypassed. The coverage sample does not include all 16 items, and 1M samples are short GPU intervals; these results should be treated as targeted coverage rather than a full item-population estimate. The bootstrap intervals condition on the three selected items and resample timing rounds; they do not capture uncertainty from choosing other items.

## Scope recommendation and rollback

The evidence supports considering the **T4-only runtime default**: the fresh 200M confirmation is positive, the selected 1M–15M workload groups are positive, explicit 160-block behavior is neutral, all correctness/override checks pass, and the T4 scoped build's SASS exactly matches the measured native T4 80 × 512 build. The negative L4 and Blackwell default-512 screens reinforce that the policy must stay device-scoped; this report does not support a global geometry change. A100 validation confirms the T4-scoped candidate leaves the tested A100's SASS equal to its 160 × 256 base, but is not a timing result for this scoped policy.

Keep the dispatch guard exact: CC 7.5, 40 SMs, and device name containing `T4`, only for an implicit default. Positive argument/environment overrides must continue to win. All sm_75 builds receive `(512,2)` launch bounds, including non-T4 boards where dispatch remains 160 × 256. The additional baseline comparison establishes an identical instruction stream for this native build; those other boards were not performance-tested. If the policy is later withdrawn, revert the integrating scoped dispatcher/kernel change to the baseline source hashes; keep the independent native build-profile commit separate.

## Reproduction and raw artifacts

From the repository root, recompute the workload table and bootstrap intervals and validate each compressed scope result, raw and normalized SASS hash, correctness count, source hash, and compressed/original JSON checksum with:

```sh
python3 docs/benchmarks/raw/2026-10-24-t4-scoped-geometry-validation-analyze.py
```

The analyzer uses 50,000 replicates and NumPy. Full validation JSON files, including both embedded SASS outputs, are stored losslessly as `docs/benchmarks/raw/2026-10-24-t4-scoped-geometry-validation-{t4,l4,a100,blackwell}.json.gz`. The analyzer verifies each decompressed JSON SHA-256 and each gzip SHA-256. Original / compressed hashes are:

| Device | Original JSON SHA-256 | Gzip SHA-256 |
| --- | --- | --- |
| T4 | `2073b9a83537e2ddfad6386852622e8b250a1d62334a01b02487a04801c47c9f` | `27df310341203aff6be27ee8600d91703a5d46f1fa6d3bb418464a2922283d7f` |
| L4 | `23554f36798d6c29edb94b63535ab7afd172bfc3851bef1fbb0e06675238d73e` | `08d18ffe6abb49e77f6fbb0b0a937884a12054af217e352f6592266dca8c2ed0` |
| A100 | `68ab5bcc25c087335fda2cd9ecba5503073aab4dca5e539d02da364082714195` | `0750eac6b6b42dd870e3a26573480afd206e404e556660cf3d5df38cd2d733c4` |
| Blackwell | `284df5b122287737809ae38dd5a30ef7ae42d44b09ae665a6715aa1b337a46fc` | `6712e976be0c4736a68fd1459fd50a042dac1669a82de512486a8c05739c88ce` |

The coverage timing JSON, summary, runner log, executed runner, scoped-validation runner, source patch, and candidate/reference build logs are stored under the same raw-artifact prefix. The coverage JSON SHA-256 is `59d74a11dc43a0892aec5735e59df3534f8c923e7c0af6e29ddbbece3a115fef`. The candidate patch is retained for review; this report does not modify or promote source.
