#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/tables.cuh"
#include "src/gpu/arch/turing/kernel.cuh"
#include <chrono>
#include <cstring>
#include <vector>

namespace arch::turing {

// A pack emits at most seven cards, so bound each uint32 shared counter by
// seven increments per pack. With 40,960 logical threads and 512-thread CTAs,
// the first CTA receives 512*base + min(rem, 512) packs. The final remainder
// below is the exact largest safe count for this geometry.
constexpr uint64_t kU32CounterMax = 0xffffffffULL;
constexpr uint64_t kMaxPacksPerCta = kU32CounterMax / 7ULL;
constexpr uint64_t kWideGeometryBlockThreads = 512ULL;
constexpr uint64_t kWideGeometryLogicalThreads = 40960ULL;
constexpr uint64_t kMaxSafePacks80x512 =
    (kMaxPacksPerCta / kWideGeometryBlockThreads) * kWideGeometryLogicalThreads +
    (kMaxPacksPerCta % kWideGeometryBlockThreads);
static_assert(kMaxSafePacks80x512 == 49085317412ULL,
              "80x512 shared histogram safety bound changed");

inline void write_results(const char *results_path, const char *sold_path, const uint64_t *kept, const uint64_t *sold) {
    pack_write_results(results_path, sold_path, kept, sold);
}

typedef void (*packopen_progress_cb)(unsigned long long done, unsigned long long total, void *ctx);

inline int run_sim(int eitem, unsigned long long count, int blocks, int shop,
                   const char *results_path, const char *sold_path,
                   const char *ckpt_path, double *out_seconds,
                   packopen_progress_cb progress, void *progress_ctx,
                   uint64_t seed = resolve_seed()) {
    cudaSetDeviceFlags(cudaDeviceScheduleBlockingSync);
    cudaDeviceSetCacheConfig(cudaFuncCachePreferL1);

    int collection, exp_id;
    if (!find_pack_host(eitem, collection, exp_id)) return 2;
    if (count == 0) return 2;

    int threads_per_block = 256;
    int blk = blocks;
    bool use_implicit_default = blk <= 0;
    if (blk <= 0) {
        const char *eb = getenv("PACKSIM_BLOCKS");
        if (eb && eb[0]) blk = atoi(eb);
        if (blk > 0) use_implicit_default = false;
    }
    if (blk <= 0) {
        blk = 160;
        if (use_implicit_default) {
            int device_id = 0;
            cudaDeviceProp prop;
            if (cudaGetDevice(&device_id) == cudaSuccess &&
                cudaGetDeviceProperties(&prop, device_id) == cudaSuccess) {
                // If the GPU architecture supports 512-thread blocks efficiently (sm_75 through sm_90)
                // and count is within the safe counter bound, use 80x512 to preserve 40,960 logical RNG threads.
                // Blackwell (sm_120+) regresses with 512-thread CTAs, so retain 160x256 there.
                const bool allow_512 = (prop.major < 12) &&
                                       count <= kMaxSafePacks80x512;
                if (allow_512) {
                    threads_per_block = 512;
                    blk = 80;
                }
            }
        }
    }

    int nthreads_actual = blk * threads_per_block;
    uint64_t chunks = ckpt_chunks(count);
    uint64_t ckpt_every = (chunks + 63) / 64;
    if (ckpt_every < 1) ckpt_every = 1;
    bool use_ckpt = ckpt_path && ckpt_path[0];

    std::vector<uint64_t> acc_kept(KEY_N, 0), acc_sold(KEY_N, 0);
    if (use_ckpt) {
        uint64_t dc = 0, ch = 0, th = 0, ns = 0, ckpt_seed = 0;
        std::vector<uint64_t> rstates((size_t)nthreads_actual);
        if (ckpt_load(ckpt_path, 2, eitem, shop, count, &dc, &ch, &th,
                      acc_kept.data(), acc_sold.data(), rstates.data(),
                      (uint64_t)nthreads_actual, &ns, &ckpt_seed) == 0 &&
            ch == chunks && th == (uint64_t)nthreads_actual) {
            seed = ckpt_seed;
            if (dc == ch && ch > 0) {
                if (progress) {
                    progress(count, count, progress_ctx);
                }
                if (results_path && sold_path) {
                    write_results(results_path, sold_path, acc_kept.data(), acc_sold.data());
                }
                if (out_seconds) {
                    *out_seconds = 0.0;
                }
                return 0;
            }
            fprintf(stderr, "warning: streaming checkpoint resume is CPU-only; starting fresh GPU simulation with seed 0x%016llx\n",
                    (unsigned long long)seed);
            std::fill(acc_kept.begin(), acc_kept.end(), 0ULL);
            std::fill(acc_sold.begin(), acc_sold.end(), 0ULL);
        } else {
            fprintf(stderr, "note: streaming checkpoints are CPU-only; checkpoint will be saved upon completion\n");
        }
    }

    uint64_t base = count / (uint64_t)nthreads_actual;
    uint64_t rem = count % (uint64_t)nthreads_actual;
    uint32_t total_packs_hi = (uint32_t)(base + 1);
    uint32_t total_packs_lo = (uint32_t)base;
    uint32_t rem_th = (uint32_t)rem;

    int main_slots = slab_slots_for(exp_id);
    int nkeys = main_slots;
    if (run_has_ghost(exp_id) && shop > 1) {
        nkeys += 156;
    }

    size_t red_sz = (size_t)nkeys * sizeof(uint64_t);
    uint64_t *d_red_kept = nullptr;
    if (cudaMalloc(&d_red_kept, red_sz) != cudaSuccess) {
        fprintf(stderr, "CUDA error: failed to allocate histogram buffer\n");
        return 3;
    }

    uint64_t *d_state = nullptr;
    if (cudaMalloc(&d_state, (size_t)nthreads_actual * sizeof(uint64_t)) != cudaSuccess) {
        fprintf(stderr, "CUDA error: failed to allocate RNG state buffer\n");
        cudaFree(d_red_kept);
        return 3;
    }

    Params pack_params[14];
    Thr thresholds[14];
    for (int g = 0; g < 14; g++) {
        pack_params[g] = params_for(collection);
        apply_god(pack_params[g], g);
        thresholds[g].leg_casc = thr(pack_params[g].leg_casc * 100.f);
        thresholds[g].epic_casc = thr(pack_params[g].epic_casc * 100.f);
        thresholds[g].rare_casc = thr(pack_params[g].rare_casc * 100.f);
        thresholds[g].foil = thr(pack_params[g].foil * 100.f);
        thresholds[g].b_first_ed = thr(pack_params[g].b_first_ed * 100.f);
        thresholds[g].b_silver = thr(pack_params[g].b_silver * 100.f);
        thresholds[g].b_gold = thr(pack_params[g].b_gold * 100.f);
        thresholds[g].b_ex = thr(pack_params[g].b_ex * 100.f);
        thresholds[g].b_full_art = thr(pack_params[g].b_full_art * 100.f);
    }
    if (cudaMemcpyToSymbol(c_p2t, pack_params, sizeof(pack_params)) != cudaSuccess ||
        cudaMemcpyToSymbol(c_tht, thresholds, sizeof(thresholds)) != cudaSuccess ||
        cudaMemset(d_red_kept, 0, red_sz) != cudaSuccess) {
        fprintf(stderr, "CUDA error: symbol copy or buffer memset failed\n");
        cudaFree(d_red_kept);
        cudaFree(d_state);
        return 4;
    }

    auto t0 = std::chrono::steady_clock::now();
    std::vector<uint64_t> tmp_kept((size_t)nkeys);
    int rc = 0;
    const bool has_ghost = run_has_ghost(exp_id) && (shop > 1);

    const size_t default_smem = (size_t)nkeys * sizeof(uint32_t);
    const size_t common_smem = ((1323 + (has_ghost ? 156 : 0) + 3) / 4) * 4 * sizeof(uint32_t);
    const size_t fixed_pool_smem = (size_t)(121 * 13 + (has_ghost ? 156 : 0)) * sizeof(uint32_t);

#define LAUNCH_SIM(E, GHOST, ONE_RARITY, USE_RARITY, ALL_COMM, FIXED_POOL, POOL_RARITY, SMEM) \
    launch_sim_kernel<E, GHOST, ONE_RARITY, USE_RARITY, ALL_COMM, FIXED_POOL, POOL_RARITY>( \
        blk, threads_per_block, (SMEM), \
        exp_id, collection, shop, total_packs_hi, total_packs_lo, rem_th, \
        nthreads_actual, d_red_kept, seed, d_state, main_slots)

    switch (exp_id) {
        case 0:
            if (collection == 0) {
                if (has_ghost) LAUNCH_SIM(0, true, false, true, true, false, 0, common_smem);
                else LAUNCH_SIM(0, false, false, true, true, false, 0, common_smem);
            } else if (collection == 1) {
                if (has_ghost) LAUNCH_SIM(0, true, true, true, false, true, 1, fixed_pool_smem);
                else LAUNCH_SIM(0, false, true, true, false, true, 1, fixed_pool_smem);
            } else if (collection == 2) {
                if (has_ghost) LAUNCH_SIM(0, true, true, true, false, true, 2, fixed_pool_smem);
                else LAUNCH_SIM(0, false, true, true, false, true, 2, fixed_pool_smem);
            } else if (collection == 3) {
                if (has_ghost) LAUNCH_SIM(0, true, true, true, false, true, 3, fixed_pool_smem);
                else LAUNCH_SIM(0, false, true, true, false, true, 3, fixed_pool_smem);
            } else {
                if (has_ghost) LAUNCH_SIM(0, true, true, true, false, false, 0, default_smem);
                else LAUNCH_SIM(0, false, true, true, false, false, 0, default_smem);
            }
            break;
        case 1:
            if (collection == 4) {
                if (has_ghost) LAUNCH_SIM(1, true, false, true, true, false, 0, common_smem);
                else LAUNCH_SIM(1, false, false, true, true, false, 0, common_smem);
            } else if (collection == 5) {
                if (has_ghost) LAUNCH_SIM(1, true, true, true, false, true, 1, fixed_pool_smem);
                else LAUNCH_SIM(1, false, true, true, false, true, 1, fixed_pool_smem);
            } else if (collection == 6) {
                if (has_ghost) LAUNCH_SIM(1, true, true, true, false, true, 2, fixed_pool_smem);
                else LAUNCH_SIM(1, false, true, true, false, true, 2, fixed_pool_smem);
            } else if (collection == 7) {
                if (has_ghost) LAUNCH_SIM(1, true, true, true, false, true, 3, fixed_pool_smem);
                else LAUNCH_SIM(1, false, true, true, false, true, 3, fixed_pool_smem);
            } else {
                if (has_ghost) LAUNCH_SIM(1, true, true, true, false, false, 0, default_smem);
                else LAUNCH_SIM(1, false, true, true, false, false, 0, default_smem);
            }
            break;
        case 2: // Exp 2: Ghost pack
            LAUNCH_SIM(2, false, false, false, false, false, 0, 100 * sizeof(uint32_t));
            break;
        case 3: // Exp 3: Destiny pack
            LAUNCH_SIM(3, false, false, false, false, false, 0, 113 * 13 * sizeof(uint32_t));
            break;
        case 4: // Exp 4: Megabot
            LAUNCH_SIM(4, false, false, false, false, false, 0, 50 * 13 * sizeof(uint32_t));
            break;
        case 5: // Exp 5: Ultra pack
            LAUNCH_SIM(5, false, false, false, false, false, 0, 40 * 13 * sizeof(uint32_t));
            break;
        case 6: // Exp 6: Non-rarity
            LAUNCH_SIM(6, false, false, false, false, false, 0, default_smem);
            break;
        case 7:
            if (has_ghost) LAUNCH_SIM(7, true, false, false, false, false, 0, default_smem);
            else LAUNCH_SIM(7, false, false, false, false, false, 0, default_smem);
            break;
        case 8:
            if (has_ghost) LAUNCH_SIM(8, true, false, true, false, false, 0, default_smem);
            else LAUNCH_SIM(8, false, false, true, false, false, 0, default_smem);
            break;
        default:
            LAUNCH_SIM(0, false, false, false, false, false, 0, default_smem);
            break;
    }
#undef LAUNCH_SIM

    cudaError_t launch_err = cudaGetLastError();
    if (launch_err != cudaSuccess) {
        fprintf(stderr, "CUDA kernel launch error: %s\n", cudaGetErrorString(launch_err));
        rc = 5;
    } else {
        cudaError_t sync_err = cudaDeviceSynchronize();
        if (sync_err != cudaSuccess) {
            fprintf(stderr, "CUDA synchronization error: %s\n", cudaGetErrorString(sync_err));
            rc = 5;
        }
    }
    auto t1 = std::chrono::steady_clock::now();

    if (rc == 0) {
        cudaError_t cpy_err = cudaMemcpy(tmp_kept.data(), d_red_kept, red_sz, cudaMemcpyDeviceToHost);
        if (cpy_err != cudaSuccess) {
            fprintf(stderr, "CUDA DtoH memcpy error: %s\n", cudaGetErrorString(cpy_err));
            rc = 5;
        }
    }

    if (rc == 0) {
        for (int i = 0; i < nkeys; i++) {
            int k = (int)slab_key((uint32_t)i, exp_id, main_slots);
            acc_kept[k] += tmp_kept[i];
            if (acc_kept[k] > (uint64_t)INTMAX32) {
                acc_sold[k] += acc_kept[k] - (uint64_t)INTMAX32;
                acc_kept[k] = (uint64_t)INTMAX32;
            }
        }
        if (use_ckpt) {
            std::remove(ckpt_path);
        }
    }

    cudaFree(d_red_kept);
    cudaFree(d_state);

    if (rc == 0) {
        if (progress) {
            progress(count, count, progress_ctx);
        }
        if (results_path && sold_path) {
            write_results(results_path, sold_path, acc_kept.data(), acc_sold.data());
        }
        if (out_seconds) {
            *out_seconds = std::chrono::duration<double>(t1 - t0).count();
        }
    }
    return rc;
}

} // namespace arch::turing
