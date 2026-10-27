#include "src/cpu/dispatcher/sim_runner.hpp"
#include "src/cpu/dispatcher/writer.hpp"

extern "C" {

/**
 * @brief Probes CPU execution capability. Always returns 1 for host CPU.
 */
int packopen_probe(void) {
    return 1;
}

/**
 * @brief Primary C-ABI entry point with checkpointing and progress reporting support.
 *
 * Parameters:
 * - eitem:         Pack item ID (0..19, 129).
 * - count:         Total packs to open.
 * - threads:       Worker thread count (clamped to 1..256).
 * - shop:          Shop level (controls god packs and ghost availability).
 * - results_path:  Output path for kept cards histogram.
 * - sold_path:     Output path for sold cards histogram.
 * - ckpt_path:     Checkpoint file path (enables save/resume).
 * - out_seconds:   Receives total execution wall time.
 * - progress:      Callback invoked periodically with completed pack count.
 * - progress_ctx:  Context pointer passed to progress callback.
 */
int packopen_sim_seed_ckpt(int eitem, unsigned long long count, int threads, int shop,
                           const char *results_path, const char *sold_path,
                           const char *ckpt_path, double *out_seconds,
                           packopen_progress_cb progress, void *progress_ctx,
                           unsigned long long seed) {
    int collection, exp_id;
    if (!find_pack_host(eitem, collection, exp_id)) return 2;
    if (count == 0) return 2;
    if (threads < 1) threads = 1;
    if (threads > 256) threads = 256;
    std::call_once(GPRE_ONCE, prebuilt_init);

    Params pack_params[14];
    Thr thresholds[14];
    for (int g = 0; g < 14; g++) {
        pack_params[g] = params_for(collection);
        apply_god(pack_params[g], g);
        thresholds[g] = Thr{thr(pack_params[g].leg_casc * 100.0f), thr(pack_params[g].epic_casc * 100.0f), thr(pack_params[g].rare_casc * 100.0f),
                            thr(pack_params[g].foil * 100.0f), thr(pack_params[g].b_first_ed * 100.0f), thr(pack_params[g].b_silver * 100.0f),
                            thr(pack_params[g].b_gold * 100.0f), thr(pack_params[g].b_ex * 100.0f), thr(pack_params[g].b_full_art * 100.0f)};
    }
    uint64_t chunks = ckpt_chunks(count);
    uint64_t base = count / (uint64_t)threads;
    uint64_t rem = count % (uint64_t)threads;
    std::vector<uint64_t> share((size_t)threads);
    for (int t = 0; t < threads; t++) share[t] = base + ((uint64_t)t < rem ? 1 : 0);
    std::vector<uint64_t> states((size_t)threads);
    for (int t = 0; t < threads; t++) states[t] = thread_seed(t, (uint64_t)seed);
    std::vector<uint64_t> acc_kept(KEY_N, 0), acc_sold(KEY_N, 0);
    uint64_t start_chunk = 0;
    if (ckpt_path && ckpt_path[0]) {
        uint64_t dc = 0, ch = 0, th = 0, ns = 0, ckpt_seed = 0;
        std::vector<uint64_t> rstates((size_t)threads);
        if (ckpt_load(ckpt_path, 1, eitem, shop, count, &dc, &ch, &th, acc_kept.data(), acc_sold.data(),
                      rstates.data(), (uint64_t)threads, &ns, &ckpt_seed) == 0 && ch == chunks && th == (uint64_t)threads) {
            start_chunk = dc;
            states = rstates;
            seed = ckpt_seed;
            fprintf(stderr, "checkpoint: resuming at chunk %llu/%llu (seed: 0x%016llx)\n",
                    (unsigned long long)dc, (unsigned long long)chunks, (unsigned long long)ckpt_seed);
        }
    }
    int main_n = 1452;
    if (exp_id == 0 || exp_id == 1) {
        if (pack_params[0].commons == 7) {
            main_n = 324;
        } else if (pack_params[0].rares == 7) {
            main_n = 408;
        } else if (pack_params[0].epics == 7) {
            main_n = 408;
        } else if (pack_params[0].legendaries == 7) {
            main_n = 312;
        }
    } else {
        switch (exp_id) {
            case 2: main_n = 80; break;
            case 3: main_n = 1356; break;
            case 4: main_n = 600; break;
            case 5: main_n = 480; break;
            default: main_n = 1452; break;
        }
    }
    const int ghost_n = 80;
    const bool can_ghost = ((exp_id == 0 || exp_id == 1 || exp_id == 7) && shop > 1);
    const size_t active_slots = (can_ghost ? (main_n + ghost_n) : main_n);
    // Align to 64 bytes (16 uint32_t elements) and add 64-byte (16 uint32_t elements) cache line padding
    // between worker thread slabs to eliminate L1/L2 false sharing and prefetcher interference.
    const size_t thread_stride = ((active_slots + 15) & ~15) + 16;
    const size_t total_elements = (size_t)threads * thread_stride;
    const size_t alloc_bytes = ((total_elements * sizeof(uint32_t)) + 63) & ~63;
    uint32_t *dense_kept = (uint32_t *)pack_aligned_alloc(64, alloc_bytes);
    if (!dense_kept) return 2;
    memset(dense_kept, 0, alloc_bytes);
    auto t0 = std::chrono::steady_clock::now();
    uint64_t done_packs = 0;
    for (uint64_t t = 0; t < (uint64_t)threads; t++) {
        uint64_t upto = 0;
        for (uint64_t c = 0; c < start_chunk && c < chunks; c++) {
            uint64_t piece = share[t] / chunks + (c < share[t] % chunks ? 1 : 0);
            upto += piece;
        }
        done_packs += upto;
    }
    int rc = 0;
    uint64_t ckpt_every = (chunks + 63) / 64;
    if (ckpt_every < 1) ckpt_every = 1;
    bool use_ckpt = ckpt_path && ckpt_path[0];
    uint64_t max_share = 0;
    for (int t = 0; t < threads; t++) if (share[(size_t)t] > max_share) max_share = share[(size_t)t];
    uint64_t fold_every = 1;
    if (max_share > 0) {
        fold_every = (16777216ULL * chunks) / max_share;
        if (fold_every < 1) fold_every = 1;
    }

    CpuSimContext ctx{
        eitem, exp_id, collection, shop, threads, count,
        pack_params, thresholds, chunks, share, states,
        acc_kept, acc_sold, start_chunk, dense_kept, thread_stride,
        main_n, can_ghost, ckpt_path, ckpt_every, fold_every, use_ckpt,
        progress, progress_ctx, done_packs, rc, (uint64_t)seed
    };

    switch (exp_id) {
        case 0: run_sim<0>(ctx); break;
        case 1: run_sim<1>(ctx); break;
        case 2: run_sim<2>(ctx); break;
        case 3: run_sim<3>(ctx); break;
        case 4: run_sim<4>(ctx); break;
        case 5: run_sim<5>(ctx); break;
        case 6: run_sim<6>(ctx); break;
        case 7: run_sim<7>(ctx); break;
        case 8: run_sim<8>(ctx); break;
        default: pack_aligned_free(dense_kept); return 2;
    }

    if (rc == 0) {
        write_results(results_path, sold_path, acc_kept.data(), acc_sold.data());
        if (ckpt_path && ckpt_path[0]) std::remove(ckpt_path);
    }
    pack_aligned_free(dense_kept);
    if (progress) progress(done_packs, count, progress_ctx);
    if (out_seconds) {
        auto t1 = std::chrono::steady_clock::now();
        *out_seconds = std::chrono::duration<double>(t1 - t0).count();
    }
    return rc;
}

/**
 * @brief C-ABI entry point with checkpointing and progress reporting support (resolves seed if not passed).
 */
int packopen_sim_ckpt(int eitem, unsigned long long count, int threads, int shop,
                      const char *results_path, const char *sold_path,
                      const char *ckpt_path, double *out_seconds,
                      packopen_progress_cb progress, void *progress_ctx) {
    return packopen_sim_seed_ckpt(eitem, count, threads, shop, results_path, sold_path,
                                  ckpt_path, out_seconds, progress, progress_ctx,
                                  resolve_seed());
}

/**
 * @brief Standard C-ABI entry point with custom seed.
 */
int packopen_sim_seed(int eitem, unsigned long long count, int threads, int shop,
                      const char *results_path, const char *sold_path,
                      double *out_seconds, unsigned long long seed) {
    return packopen_sim_seed_ckpt(eitem, count, threads, shop, results_path, sold_path,
                                  nullptr, out_seconds, nullptr, nullptr, seed);
}

/**
 * @brief Standard C-ABI entry point without checkpointing or progress reporting.
 */
int packopen_sim(int eitem, unsigned long long count, int threads, int shop,
                 const char *results_path, const char *sold_path,
                 double *out_seconds) {
    return packopen_sim_seed_ckpt(eitem, count, threads, shop, results_path, sold_path,
                                  nullptr, out_seconds, nullptr, nullptr, resolve_seed());
}

}
