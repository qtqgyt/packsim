# CUDA GPU build profiles

PackSim uses CMake to build the GPU shared library (`libpackopen_gpu.so`). By default,
CMake builds native cubins for architectures `75 80 86 89 90` if not overridden.
Select specific target architectures with `-DCMAKE_CUDA_ARCHITECTURES`, a semicolon-
or space-separated list of compute capabilities. These controls change the GPU library's
compiled code coverage; they do not change runtime backend selection, launch geometry,
or performance policy.

When changing architecture targets, re-run CMake configure and build:

```sh
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES="75"
cmake --build build --target packopen_gpu
```

## Native build commands

Run `nvcc --list-gpu-arch` with the selected toolkit first. Each command below
requests a native cubin for one representative GPU generation; use a toolkit
that lists the requested architecture.

```sh
# Turing: Tesla T4
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=75 && cmake --build build --target packopen_gpu

# Ampere: A100
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=80 && cmake --build build --target packopen_gpu

# Ampere: RTX 30 series (device dependent; sm_86 is a common target)
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=86 && cmake --build build --target packopen_gpu

# Ada: L4
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=89 && cmake --build build --target packopen_gpu

# Hopper: H100
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=90 && cmake --build build --target packopen_gpu
```

For multiple native cubins in one library, specify multiple targets separated
by semicolons (or spaces in quotes), for example:
```sh
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES="75;80;86;89;90"
cmake --build build --target packopen_gpu
```
Virtual PTX architectures can be requested using CMake syntax (e.g. `90-virtual` or `90-real;90-virtual`).

Blackwell targets depend on the exact GPU and CUDA toolkit. If the toolkit's `nvcc --list-gpu-arch` lists `compute_100` and/or
`compute_120`, request the matching target,
for example:
```sh
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=100 && cmake --build build --target packopen_gpu
# or
cmake -S . -B build -DCMAKE_CUDA_ARCHITECTURES=120 && cmake --build build --target packopen_gpu
```
Do not include an architecture that the installed compiler does not list. PTX JIT
support is subject to the CUDA driver's compatibility rules.

The GPU runtime currently selects the Turing kernel backend at compute
capability 7.5 and newer, while the GPU probe rejects devices below 7.5.
Adding a cubin target alone does not establish runtime support for a GPU
family. Validate runtime support and correctness on the actual hardware before
claiming a new generation is supported. The documented examples are build
profiles, not performance recommendations; launch defaults remain unchanged.

## Launch defaults

The current dispatcher selects 80 blocks of 512 threads on supported devices
with compute-capability major version below 12, for counts through
49,085,317,412 packs inclusive. Larger counts, devices with major version 12
or newer, and failed device-property queries use 160 × 256. This selection
has no SM-count, model-name, or minimum-pack-count condition.

Both implicit configurations contain 40,960 logical RNG threads. Positive API
block counts and `PACKSIM_BLOCKS` use 256 threads per block;
`PACKSIM_BLOCKS=160` selects 160 × 256. `PACKSIM_BLOCKS=80` means 80 × 256
and does not select the implicit 80 × 512 configuration.

The saved [T4 validation report](benchmarks/2026-10-24-t4-scoped-geometry-validation.md)
and [A100 validation report](benchmarks/2026-10-25-t4-a100-scoped-geometry-validation.md)
describe earlier, more narrowly scoped policies. Their timing evidence does
not validate the current broader selection on every device or workload.
Treat performance on those additional cases as **EXPERIMENTAL** until measured
on the target GPU. A real 50-billion-pack case verified the 160 × 256 fallback
on T4 and A100 with nonempty sold output in the scoped validation.

## Measured explicit grid settings

Native builds on the tested NVIDIA L4 and RTX PRO 6000 Blackwell Server Edition
benefited from explicit block settings at 256 threads per block:

| Tested GPU | Setting | Fresh all-item aggregate change |
| --- | --- | --- |
| L4 (`sm_89`) | `PACKSIM_BLOCKS=232` | +9.658% (95% CI +9.380% to +9.930%) |
| RTX PRO 6000 Blackwell Server Edition (`sm_120`) | `PACKSIM_BLOCKS=188` | +16.893% (95% CI +16.874% to +16.913%) |

These settings change the number of logical RNG threads, so a fixed seed can
produce different card counts than the 160-block default. They remain explicit
options. Each confirmation covered e0–e15 at shops 1 and 340, 200M packs per
sample, with additional targeted small-count checks. See the
[L4 confirmation](benchmarks/2026-10-24-ada-l4-native-grid-confirm.md) and
[Blackwell confirmation](benchmarks/2026-10-24-blackwell-native-grid-confirm.md).
