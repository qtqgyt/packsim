# Scoped GPU geometry: native correctness evidence

## Scope and evidence identity

This report audits native guard-validation artifacts for T4, A100, L4, and Blackwell. It does not make a timing or speedup claim: these runs set `timing.performed` to false. The combined T4/A100/L4/Blackwell campaign bundle SHA-256 is `e916ef2c9ad4ebfc92bc82e1ee85c12fff1d2c756e0b357c7b2dcaa23427ffde`; the exact T4-only rerun used a separate `codex/t4-safe-default` source bundle with SHA-256 `ddcd6b51cab104d3d214b3f202e7ab91c52445fec0b0e12f8bddea5855260b07`. The offline analyzer verifies all 13 GPU source hashes against each matching bundle. Both candidates are based on `44cf2e748df641e66f03f28b6103688ba4a3a706`, distinct from the documentation repository's current HEAD (`892b84b`).

An offline analyzer rechecks the copied JSON, source bundle contents, result-output byte counts and SHA-256s, candidate-library hashes, SASS archive members and hashes, and ptxas spill totals. It needs Python 3 and no GPU:

```sh
python3 docs/benchmarks/raw/2026-10-25-scoped-geometry-guard/analyze_scoped_geometry_guard.py
```

Raw inputs, native logs, candidate/reference libraries (gzip-compressed), SASS archives, campaign state, source bundle, and run/validation scripts are under [`raw/2026-10-25-scoped-geometry-guard`](raw/2026-10-25-scoped-geometry-guard/). Each preserved file is below 50 MB.

## Result summary

| Target | Native build | 12 goldens | Default parity | Override parity | Boundary matrix | 50B fallback case | SASS match |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| Tesla T4 (sm_75) | passed | 12/12 | 32/32 | 18/18 | not applicable | passed | expected 80x512 candidate |
| A100-SXM4-40GB (sm_80) | passed | 12/12 | 32/32 | 18/18 | 30/30 | passed | expected 80x512 candidate |
| NVIDIA L4 (sm_89) | passed | 12/12 | 32/32 | 18/18 | not applicable | not run | expected 160x256 base |
| RTX PRO 6000 Blackwell (sm_120) | passed | 12/12 | 32/32 | 18/18 | not applicable | not run | expected 160x256 base |

Default and override parity counts were checked against both output byte lengths and SHA-256 digests, with successful candidate/reference call return codes. The 12 goldens also report matching kept/sold output. The T4 row uses the later exact-only T4 rerun, which completed at `2026-10-25T17:31:04Z`; its candidate library SHA-256 is `2f44b82c84baefc930aef62484f04ddb0ac3703cdb17f0febc895fdcb75f095e` and its SASS archive SHA-256 is `53d3ffb045dfbca46c85a6372e8a87ad4d5dea2396eb52d1ace5d2b9633893e5`. Its 13 source hashes match the distinct T4 bundle. Results JSON SHA-256 is `0a5cf3de9032942ddb6fd8385c1ef1de9079a657babb07584a1a6dbb8090e416`. No timing was performed.

### A100 boundary matrix

The A100 result covers 3 item IDs (`0`, `6`, `15`) × 2 shops (`1`, `340`) × 5 pack counts (`1,000,000`, `4,999,999`, `5,000,000`, `15,000,000`, `200,000,000`) for 30 cases. Candidate and reference kept/sold output lengths and SHA-256 digests match in all 30. The recorded reference geometry is the existing 160x256 base below 5,000,000 and the 80x512 candidate from 5,000,000 upward.

### 50B count on T4 and A100

Both native candidate runs used the guarded 160x256 fallback for 50,000,000,000 packs and exactly matched the base reference's two output files byte-for-byte. In each run, kept output was 6,385 bytes and sold output was 1,032 bytes; sold output was nonempty. Candidate and reference totals were 175,963,664,586 kept plus 174,036,335,414 sold, summing to the required 350,000,000,000 emissions. This is a correctness-only large-count case, not a performance measurement.

## Dispatcher guard and counter bound

The reviewed dispatcher selects 80x512 only for an implicit default and only when the active device matches the exact board predicate: T4 (`CC 7.5`, 40 SMs, name contains `T4`) or A100-SXM4-40GB (`CC 8.0`, 108 SMs, name contains `A100-SXM4-40GB`). The A100 branch additionally requires count at least 5,000,000. Both branches require count no greater than `49,085,317,412`; otherwise they retain 160x256. Positive API block counts and positive `PACKSIM_BLOCKS` remain on the existing 256-thread geometry. Blackwell does not match either predicate and remains 160x256.

The counter limit follows the source's maximum of seven card emissions per pack into a `uint32_t` shared histogram slot. `floor((2^32-1)/7) = 613,566,756` packs is the maximum safe first-CTA work. With 40,960 logical RNG threads and 512 threads per CTA, the exact count limit is 49,085,317,412: at the limit the first CTA receives 613,566,756 packs; one more pack can exceed the per-slot counter bound. The reviewed static assertion pins this value. The 50B checks exercise the fallback path; they do not prove safety above the pre-existing 160x256 geometry's separate bound.

## Build resource evidence

Each candidate build log records 63 ptxas function-property entries. T4, A100, and L4 show zero spill stores and zero spill loads across those entries. Blackwell records 4 spill-store bytes and 4 spill-load bytes in aggregate across the 63 entries. Nonzero stack-frame sizes appear in the logs and are not counted as spills by themselves. This evidence describes compiler-reported function resources for these builds; it is not a runtime memory-traffic measurement.

The T4 SASS archive is 17,244,706 bytes (SHA-256 `dbff1c5c93c0f6bfc7e25b3ae81acad4500c80bd5f9adb8e9edf137aa259e1c9`); normalized candidate and candidate-reference SASS match. The A100 archive is 17,061,016 bytes (SHA-256 `ab31d2e03dcc51d123485c8852fbc9c33a9389391821bdf152b67a385515804c`) with the same comparison target and result. L4's archive is 17,060,259 bytes (SHA-256 `6f22b1b1593e2492f68cb9db400a67d1d68c8914e8f6e536f470b16933081061`); normalized candidate SASS matches the 160x256 base. Blackwell's archive is 14,529,074 bytes (SHA-256 `a6a8737f1a74beb04967c1bd6aea1bf5a8f68471ae636306425c65fd668dade8`); normalized candidate SASS matches the base 160x256 SASS. The analyzer recomputes the archive digests and normalized member hashes from the saved tarballs.

## Acceptance status and limits

The exact-only T4 rerun is complete with `correctness_passed_sass_equal`: 12 goldens, 32 default parity cases, 18 override cases, the 50B correctness case, and candidate SASS equal to the native T4 candidate reference. At 50B, candidate and base output lengths (6,385 kept; 1,032 sold) and hashes match; sold is nonempty and the total is 350B. This is the final exact-T4 evidence in this report.

The fresh L4 recovery completed successfully (`correctness_passed_sass_equal` at `2026-10-25T17:30:01Z`): sm_89 native build, 12 goldens, 32 default parity cases, 18 override cases, and normalized SASS equality against the base. Its results JSON SHA-256 is `556f54e10449083e56ae4cd6aa6fd186e03d4c1d2f8bc80cf598841465005b46`; candidate library SHA-256 is `30daba72c93c442a160339a326a2ab7fa78674587b27f0be62ee688733848a89`. The earlier `20261025-l4-guard` attempt failed in collection before producing guard results; its campaign log, state, and manifest are retained under `raw/2026-10-25-scoped-geometry-guard/history-l4-guard-failed/` as history. L4 uses the 160x256 base geometry; its large-count case was not applicable and not run. Blackwell evidence is scoped to native correctness/default parity and SASS equality; it did not run the A100 boundary matrix or 50B safety case.

## Source integration conditions

The evidence validates both steps of the intended integration. The T4-only bundle (`codex/t4-safe-default`, SHA-256 `ddcd6b51cab104d3d214b3f202e7ab91c52445fec0b0e12f8bddea5855260b07`) is the independently tested first T4 change. The combined bundle (`codex/a100-geometry-scoped`, SHA-256 `e916ef2c9ad4ebfc92bc82e1ee85c12fff1d2c756e0b357c7b2dcaa23427ffde`) includes exact T4 selection and exact A100-SXM4-40GB selection with the 5M lower bound; its 13 source hashes are verified against native campaign evidence on T4, A100, L4, and Blackwell. The combined-bundle T4 guard result JSON is SHA-256 `77d42315b43024ffbb40617c56eef857af141308868c6c63797cb2171cc9fe88` and passes 12 goldens, 32 defaults, 18 overrides, 50B parity, and SASS equality. Its kernel launch bounds include sm_75 and sm_80. Thus the combined source evidence already covers the final T4+A100 guard logic; the exact-only T4 bundle separately validates the initial T4 commit step.

The validated dispatch conditions are: implicit T4 uses 80x512 through `49,085,317,412`; implicit A100-SXM4-40GB uses 80x512 from `5,000,000` through that ceiling; L4 and Blackwell remain at 160x256; positive API block counts and positive `PACKSIM_BLOCKS` remain on the existing 256-thread path. Above the wide-geometry ceiling, T4/A100 fall back to 160x256. The matrix and 50B correctness cases exercise the documented boundaries and fallback.
