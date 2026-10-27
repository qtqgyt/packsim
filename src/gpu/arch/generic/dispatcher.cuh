#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/tables.cuh"
#include "src/gpu/arch/generic/kernel.cuh"
#include <chrono>
#include <vector>

namespace arch::generic {

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
    if (blk <= 0) {
        const char *eb = getenv("PACKSIM_BLOCKS");
        if (eb && eb[0]) blk = atoi(eb);
    }
    if (blk <= 0) {
        blk = 48;
    }

    int nthreads_actual = blk * threads_per_block;
    uint64_t chunks = ckpt_chunks(count);
    uint64_t ckpt_every = (chunks + 63) / 64;
    if (ckpt_every < 1) ckpt_every = 1;
    bool use_ckpt = ckpt_path && ckpt_path[0];

    uint64_t base = count / (uint64_t)nthreads_actual;
    uint64_t rem = count % (uint64_t)nthreads_actual;
    std::vector<uint64_t> share((size_t)nthreads_actual);
    for (int i = 0; i < nthreads_actual; i++) {
        share[(size_t)i] = base + ((uint64_t)i < rem ? 1 : 0);
    }

    std::vector<uint64_t> acc_kept(KEY_N, 0), acc_sold(KEY_N, 0);
    uint64_t start_chunk = 0;
    std::vector<uint64_t> states((size_t)nthreads_actual);
    if (use_ckpt) {
        uint64_t dc = 0, ch = 0, th = 0, ns = 0, ckpt_seed = 0;
        std::vector<uint64_t> rstates((size_t)nthreads_actual);
        if (ckpt_load(ckpt_path, 2, eitem, shop, count, &dc, &ch, &th, acc_kept.data(), acc_sold.data(),
                      rstates.data(), (uint64_t)nthreads_actual, &ns, &ckpt_seed) == 0
            && ch == chunks && th == (uint64_t)nthreads_actual) {
            start_chunk = dc;
            states = rstates;
            seed = ckpt_seed;
            fprintf(stderr, "checkpoint: resuming at chunk %llu/%llu (seed: 0x%016llx)\n",
                    (unsigned long long)dc, (unsigned long long)chunks, (unsigned long long)ckpt_seed);
        }
    }

    size_t sz_kept = (size_t)blk * KEY_N * sizeof(uint32_t);
    size_t sz_sold = (size_t)blk * KEY_N * sizeof(unsigned long long);
    uint32_t *d_kept = nullptr;
    unsigned long long *d_sold = nullptr;
    uint64_t *d_shares = nullptr;
    Params *d_p2t = nullptr;
    Thr *d_tht = nullptr;
    uint64_t *d_state = nullptr;
    unsigned long long *d_s_kept = nullptr;
    unsigned long long *d_s_sold = nullptr;

    if (cudaMalloc(&d_kept, sz_kept) != cudaSuccess) return 3;
    if (cudaMalloc(&d_sold, sz_sold) != cudaSuccess) { cudaFree(d_kept); return 3; }
    if (cudaMalloc(&d_shares, (size_t)nthreads_actual * sizeof(uint64_t)) != cudaSuccess) {
        cudaFree(d_kept); cudaFree(d_sold); return 3;
    }
    if (cudaMalloc(&d_p2t, 14 * sizeof(Params)) != cudaSuccess) {
        cudaFree(d_kept); cudaFree(d_sold); cudaFree(d_shares); return 3;
    }
    if (cudaMalloc(&d_tht, 14 * sizeof(Thr)) != cudaSuccess) {
        cudaFree(d_kept); cudaFree(d_sold); cudaFree(d_shares); cudaFree(d_p2t); return 3;
    }
    if (cudaMalloc(&d_state, (size_t)nthreads_actual * sizeof(uint64_t)) != cudaSuccess) {
        cudaFree(d_kept); cudaFree(d_sold); cudaFree(d_shares); cudaFree(d_p2t); cudaFree(d_tht); return 3;
    }
    if (cudaMalloc(&d_s_kept, (size_t)KEY_N * sizeof(unsigned long long)) != cudaSuccess) {
        cudaFree(d_kept); cudaFree(d_sold); cudaFree(d_shares); cudaFree(d_p2t); cudaFree(d_tht); cudaFree(d_state); return 3;
    }
    if (cudaMalloc(&d_s_sold, (size_t)KEY_N * sizeof(unsigned long long)) != cudaSuccess) {
        cudaFree(d_kept); cudaFree(d_sold); cudaFree(d_shares); cudaFree(d_p2t); cudaFree(d_tht); cudaFree(d_state); cudaFree(d_s_kept); return 3;
    }

    Params p2t[14];
    Thr tht[14];
    for (int g = 0; g < 14; g++) {
        p2t[g] = params_for(collection);
        apply_god(p2t[g], g);
        tht[g].leg_casc = thr(p2t[g].leg_casc * 100.f);
        tht[g].epic_casc = thr(p2t[g].epic_casc * 100.f);
        tht[g].rare_casc = thr(p2t[g].rare_casc * 100.f);
        tht[g].foil = thr(p2t[g].foil * 100.f);
        tht[g].b_first_ed = thr(p2t[g].b_first_ed * 100.f);
        tht[g].b_silver = thr(p2t[g].b_silver * 100.f);
        tht[g].b_gold = thr(p2t[g].b_gold * 100.f);
        tht[g].b_ex = thr(p2t[g].b_ex * 100.f);
        tht[g].b_full_art = thr(p2t[g].b_full_art * 100.f);
    }
    cudaMemcpy(d_p2t, p2t, 14 * sizeof(Params), cudaMemcpyHostToDevice);
    cudaMemcpy(d_tht, tht, 14 * sizeof(Thr), cudaMemcpyHostToDevice);
    cudaMemcpy(d_shares, share.data(), (size_t)nthreads_actual * sizeof(uint64_t), cudaMemcpyHostToDevice);
    if (start_chunk > 0) {
        cudaMemcpy(d_state, states.data(), (size_t)nthreads_actual * sizeof(uint64_t), cudaMemcpyHostToDevice);
    }

    cudaMemset(d_kept, 0, sz_kept);
    cudaMemset(d_sold, 0, sz_sold);

    auto t0 = std::chrono::steady_clock::now();
    uint64_t done_packs = 0;
    for (int t = 0; t < nthreads_actual; t++) {
        uint64_t piece = share[(size_t)t] / chunks;
        done_packs += piece * start_chunk;
    }

    std::vector<unsigned long long> h_s_kept(KEY_N), h_s_sold(KEY_N);
    int rc = 0;
    const uint64_t fold_every = 64;

    for (uint64_t c = start_chunk; c < chunks; c++) {
        const uint64_t *init_p = (c == start_chunk && start_chunk == 0) ? nullptr : d_state;
        sim_kernel<<<blk, threads_per_block>>>(
            exp_id, collection, shop, d_p2t, d_tht, d_shares, c, chunks, nthreads_actual,
            d_kept, d_sold, seed, init_p, d_state);

        if (cudaDeviceSynchronize() != cudaSuccess) { rc = 5; break; }

        uint64_t chunk_packs = 0;
        for (int t = 0; t < nthreads_actual; t++) {
            chunk_packs += share[(size_t)t] / chunks + (c < share[(size_t)t] % chunks ? 1 : 0);
        }
        done_packs += chunk_packs;
        if (progress) progress(done_packs, count, progress_ctx);

        bool last = (c + 1 == chunks);
        bool ckpt_now = use_ckpt && ((c + 1) % ckpt_every == 0);
        bool safety_due = ((c + 1 - start_chunk) % fold_every == 0);

        if (last || safety_due || ckpt_now) {
            int red_blocks = (KEY_N + 255) / 256;
            reduce_hist<<<red_blocks, 256>>>(d_kept, (uint64_t*)d_s_kept, blk);
            reduce_hist64<<<red_blocks, 256>>>(d_sold, (uint64_t*)d_s_sold, blk);
            if (cudaMemcpy(h_s_kept.data(), d_s_kept, (size_t)KEY_N * sizeof(unsigned long long), cudaMemcpyDeviceToHost) != cudaSuccess) { rc = 5; break; }
            if (cudaMemcpy(h_s_sold.data(), d_s_sold, (size_t)KEY_N * sizeof(unsigned long long), cudaMemcpyDeviceToHost) != cudaSuccess) { rc = 5; break; }

            for (int k = 0; k < KEY_N; k++) {
                acc_kept[(size_t)k] += h_s_kept[(size_t)k];
                acc_sold[(size_t)k] += h_s_sold[(size_t)k];
                if (acc_kept[(size_t)k] > (uint64_t)INTMAX32) {
                    acc_sold[(size_t)k] += acc_kept[(size_t)k] - (uint64_t)INTMAX32;
                    acc_kept[(size_t)k] = (uint64_t)INTMAX32;
                }
            }
            cudaMemset(d_kept, 0, sz_kept);
            cudaMemset(d_sold, 0, sz_sold);
        }
        if (ckpt_now && !last) {
            if (cudaMemcpy(states.data(), d_state, (size_t)nthreads_actual * sizeof(uint64_t), cudaMemcpyDeviceToHost) != cudaSuccess) { rc = 5; break; }
            if (ckpt_save(ckpt_path, 2, eitem, shop, count, c + 1, chunks, (uint64_t)nthreads_actual, seed,
                          acc_kept.data(), acc_sold.data(), states.data(), (uint64_t)nthreads_actual) != 0) {
                fprintf(stderr, "warning: checkpoint write failed\n");
            }
        }
    }
    auto t1 = std::chrono::steady_clock::now();

    cudaFree(d_kept);
    cudaFree(d_sold);
    cudaFree(d_shares);
    cudaFree(d_p2t);
    cudaFree(d_tht);
    cudaFree(d_state);
    cudaFree(d_s_kept);
    cudaFree(d_s_sold);

    if (rc == 0) {
        if (results_path && sold_path) {
            write_results(results_path, sold_path, acc_kept.data(), acc_sold.data());
        }
        if (out_seconds) {
            *out_seconds = std::chrono::duration<double>(t1 - t0).count();
        }
    }
    return rc;
}

} // namespace arch::generic
