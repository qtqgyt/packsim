#pragma once

#include "src/cpu/core/tables.hpp"
#include "src/cpu/sim/god.hpp"
#include "src/cpu/sim/direct_exp01.hpp"

/**
 * @brief Multi-chunk simulation loop for Exp 0 & Exp 1 collections (Commons, Rares, Epics, Legendaries).
 *
 * Runs direct-to-histogram key generation with chunk synchronization, progress updates,
 * and checkpoint state tracking.
 */
template<int EXP_ID, int POOL_SIZE, typename SyncChunkFn>
static PACK_ALWAYS_INLINE void run_sim_direct_exp01(
    int ti, Rng &rng, int shop, uint64_t effective_chunks, uint64_t start_chunk,
    uint64_t base_piece, uint64_t rem_piece, int &rc,
    const Params *pack_params, const Thr *thresholds,
    uint32_t * const K_main, uint32_t * const K_ghost,
    const int *pristine_pool, std::vector<uint64_t> &states,
    const SyncChunkFn &sync_chunk) {

    const Params p0 = pack_params[0];
    const Thr th0 = thresholds[0];

    for (uint64_t c = start_chunk; c < effective_chunks && rc == 0; c++) {
        uint64_t piece = base_piece + (c < rem_piece ? 1 : 0);
        for (uint64_t p = 0; p < piece; p++) {
            int god = roll_god(rng, shop);
            if (PACK_LIKELY(god == 0)) {
                uint32_t k[7];
                generate_keys_exp01<EXP_ID, POOL_SIZE, true>(rng, th0, pristine_pool, k);
                K_main[k[0]]++;
                K_main[k[1]]++;
                K_main[k[2]]++;
                K_main[k[3]]++;
                K_main[k[4]]++;
                K_main[k[5]]++;

                if (shop > 1) {
                    int gate = (EXP_ID == 0) ? (int)rng.below(20000) : (int)rng.below(10000);
                    if (PACK_LIKELY(gate >= 20)) {
                        K_main[k[6]]++;
                    } else {
                        // Rare ghost kicker branch (0.15% gate hit)
                        uint32_t gk[7];
                        roll_ghost_pack_keys<true>(rng, p0, th0, gk);
                        int si = (int)rng.below(7);
                        K_ghost[gk[si]]++;
                    }
                } else {
                    K_main[k[6]]++;
                }
            } else {
                const Params p2 = pack_params[god];
                const Thr th = thresholds[god];
                uint32_t k[7];
                generate_keys_exp01<EXP_ID, POOL_SIZE, false>(rng, th, pristine_pool, k, god);

                if (shop > 1) {
                    int gate = (EXP_ID == 0) ? (int)rng.below(20000) : (int)rng.below(10000);
                    if (god == 11 || god == 12) gate = 0;
                    if (gate >= 20) {
                        K_main[k[0]]++;
                        K_main[k[1]]++;
                        K_main[k[2]]++;
                        K_main[k[3]]++;
                        K_main[k[4]]++;
                        K_main[k[5]]++;
                        K_main[k[6]]++;
                    } else {
                        uint32_t gk[7];
                        roll_ghost_pack_keys<false>(rng, p2, th, gk, god);
                        if (god == 11 || god == 12) {
                            #pragma GCC unroll 7
                            for (int i = 0; i < 7; i++) K_ghost[gk[i]]++;
                        } else {
                            K_main[k[0]]++;
                            K_main[k[1]]++;
                            K_main[k[2]]++;
                            K_main[k[3]]++;
                            K_main[k[4]]++;
                            K_main[k[5]]++;
                            int si = (int)rng.below(7);
                            K_ghost[gk[si]]++;
                        }
                    }
                } else {
                    K_main[k[0]]++;
                    K_main[k[1]]++;
                    K_main[k[2]]++;
                    K_main[k[3]]++;
                    K_main[k[4]]++;
                    K_main[k[5]]++;
                    K_main[k[6]]++;
                }
            }
        }
        states[(size_t)ti] = rng.s;
        sync_chunk(c);
    }
}
