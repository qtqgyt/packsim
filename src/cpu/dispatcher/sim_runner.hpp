#pragma once

#include "src/cpu/core/tables.hpp"
#include "src/cpu/core/rng.hpp"
#include "src/cpu/sim/god.hpp"
#include "src/cpu/sim/direct_exp01.hpp"
#include "src/cpu/sim/direct_cards.hpp"
#include "src/cpu/sim/fallback.hpp"
#include "src/cpu/dispatcher/writer.hpp"
#include "src/cpu/dispatcher/sim_direct_exp01.hpp"

template<int EXP_ID, typename SyncChunkFn>
static void run_sim_fallback(int ti, Rng &rng, int shop, uint64_t chunks, uint64_t start_chunk,
                             const std::vector<uint64_t> &share, std::vector<uint64_t> &states, int &rc,
                             const Params *pack_params, const Thr *thresholds,
                             uint32_t * const K_main, int main_n, bool can_ghost,
                             const SyncChunkFn &sync_chunk) {
    uint32_t * const K_ghost = (can_ghost) ? (K_main + main_n) : nullptr;
    int pos[9], bor[9], dst[9], foil[9];
    int spos[9], sbor[9], sdst[9], sfoil[9];
    int leg[128], epic[128], rare[128], comm[128], all[256];
    int oe[9], os[9], od[9], of[9];
    unsigned char ulo[9];
    if constexpr (!ExpTraits<EXP_ID>::haveDup) {
        const ExpT *me = &EXPS[EXP_ID];
        if (me->useRarity) {
            int w[4] = {0, me->n_leg, me->n_leg + me->n_epic, me->n_leg + me->n_epic + me->n_rare};
            for (int i = 0; i < me->n; i++) {
                int r = me->rar[i];
                leg[w[r == 3 ? 0 : r == 2 ? 1 : r == 1 ? 2 : 3]++] = i;
            }
        } else {
            for (int i = 0; i < me->n; i++) leg[i] = i;
        }
    }

    uint64_t base_piece = share[(size_t)ti] / chunks;
    uint64_t rem_piece = share[(size_t)ti] % chunks;
    for (uint64_t c = start_chunk; c < chunks && rc == 0; c++) {
        uint64_t piece = base_piece + (c < rem_piece ? 1 : 0);
        for (uint64_t p = 0; p < piece; p++) {
            int nc = open_one_pack<EXP_ID>(rng, shop,
                                           pack_params, thresholds,
                                           pos, bor, dst, foil, spos, sbor, sdst, sfoil,
                                           leg, epic, rare, comm, all, oe, os, od, of, ulo);
            for (int i = 0; i < nc; i++) {
                if (!can_ghost || PACK_LIKELY(oe[i] == EXP_ID)) {
                    if constexpr (EXP_ID == 2) {
                        int pos = os[i] >> 1; // Ghost perMonster is 2
                        uint32_t sk = ((uint32_t)pos << 2) | ((uint32_t)od[i] << 1) | (uint32_t)of[i];
                        K_main[sk]++;
                    } else {
                        int bor = os[i] % 12;
                        int pos = os[i] / 12;
                        uint32_t sk = (uint32_t)((pos * 6 + bor) * 2 + of[i]);
                        K_main[sk]++;
                    }
                } else {
                    int pos = os[i] >> 1; // Ghost perMonster is 2
                    uint32_t sk = ((uint32_t)pos << 2) | ((uint32_t)od[i] << 1) | (uint32_t)of[i];
                    K_ghost[sk]++;
                }
            }
        }
        states[(size_t)ti] = rng.s;
        sync_chunk(c);
    }
}

template<int EXP_ID>
static void run_sim(CpuSimContext &ctx) {
    constexpr int ghost_n = 80;
    constexpr bool EXP_CAN_GHOST = (EXP_ID == 0 || EXP_ID == 1 || EXP_ID == 7);
    const uint64_t effective_chunks = (ctx.use_ckpt || ctx.progress) ? ctx.chunks : 1;

    const int threads = ctx.threads;
    uint32_t *dense_kept = ctx.dense_kept;
    const size_t thread_stride = ctx.thread_stride;
    const int main_n = ctx.main_slots;
    const bool can_ghost = ctx.can_ghost;
    const bool use_ckpt = ctx.use_ckpt;
    const uint64_t ckpt_every = ctx.ckpt_every;
    const uint64_t fold_every = ctx.fold_every;
    const uint64_t chunks = ctx.chunks;
    const uint64_t start_chunk = ctx.start_chunk;
    const char *ckpt_path = ctx.ckpt_path;
    const packopen_progress_cb progress = ctx.progress;
    void *progress_ctx = ctx.progress_ctx;
    uint64_t &done_packs = ctx.done_packs;
    int &rc = ctx.rc;
    const uint64_t seed = ctx.seed;
    const Params *pack_params = ctx.pack_params;
    const Thr *thresholds = ctx.thresholds;
    const auto &share = ctx.share;
    auto &states = ctx.states;
    auto &acc_kept = ctx.acc_kept;
    auto &acc_sold = ctx.acc_sold;
    const int shop = ctx.shop;
    const int eitem = ctx.eitem;
    const unsigned long long count = ctx.count;

    // Persistent OpenMP parallel region across all chunks
    #pragma omp parallel num_threads(threads)
    {
        int ti =
#ifdef _OPENMP
            omp_get_thread_num();
#else
            0;
#endif
        Rng rng;
        rng.s = states[(size_t)ti];
        uint32_t * const K_main = (uint32_t *)PACK_ASSUME_ALIGNED(dense_kept + (size_t)ti * thread_stride, 64);

        uint64_t base_piece = share[(size_t)ti] / effective_chunks;
        uint64_t rem_piece = share[(size_t)ti] % effective_chunks;

        // Lambda handling chunk synchronization, progress updates, folding, and checkpointing
        auto sync_chunk = [&](uint64_t c) {
            bool last = (c + 1 == effective_chunks);
            bool ckpt_now = use_ckpt && ((c + 1) % ckpt_every == 0);
            bool safety_due = ((c + 1 - start_chunk) % fold_every == 0);
            bool do_fold = last || safety_due || ckpt_now;

            if (do_fold || ckpt_now || progress) {
#pragma omp barrier
#if defined(_OPENMP) && _OPENMP >= 202011L
#pragma omp masked
#else
#pragma omp master
#endif
                {
                    uint64_t chunk_packs = 0;
                    for (int t = 0; t < threads; t++) chunk_packs += share[(size_t)t] / effective_chunks + (c < share[(size_t)t] % effective_chunks ? 1 : 0);
                    done_packs += chunk_packs;
                    if (progress) progress(done_packs, count, progress_ctx);

                    if (do_fold) {
                        // Dense decoding: map dense per-thread sk indices back to 14-bit packed keys
                        if constexpr (EXP_ID != 2) {
                            constexpr int dst = (EXP_ID == 1) ? 1 : 0;
                            if constexpr (EXP_ID == 0 || EXP_ID == 1) {
                                if (main_n == 324) {
                                    for (int sk = 0; sk < 324; sk++) {
                                        uint64_t sum = 0;
                                        for (int t = 0; t < threads; t++) {
                                            sum += dense_kept[(size_t)t * thread_stride + sk];
                                        }
                                        if (sum > 0) {
                                            int foil = sk & 1;
                                            int pb = sk >> 1;
                                            int border = pb % 6;
                                            int c_idx = pb / 6;
                                            int pos = PRISTINE_COMMONS[c_idx];
                                            uint32_t key = make_key(EXP_ID, pos * 12 + border, dst, foil);
                                            acc_kept[key] += sum;
                                            if (acc_kept[key] > (uint64_t)INTMAX32) {
                                                acc_sold[key] += acc_kept[key] - (uint64_t)INTMAX32;
                                                acc_kept[key] = (uint64_t)INTMAX32;
                                            }
                                        }
                                    }
                                } else if (main_n == 408 || main_n == 312) {
                                    const bool is_rare = (pack_params[0].rares == 7);
                                    const bool is_epic = (pack_params[0].epics == 7);
                                    for (int sk = 0; sk < main_n; sk++) {
                                        uint64_t sum = 0;
                                        for (int t = 0; t < threads; t++) {
                                            sum += dense_kept[(size_t)t * thread_stride + sk];
                                        }
                                        if (sum > 0) {
                                            int foil = sk & 1;
                                            int pb = sk >> 1;
                                            int border = pb % 6;
                                            int idx = pb / 6;
                                            int pos = (is_rare ? PRISTINE_RARES[idx] : (is_epic ? PRISTINE_EPICS[idx] : PRISTINE_LEGS[idx]));
                                            uint32_t key = make_key(EXP_ID, pos * 12 + border, dst, foil);
                                            acc_kept[key] += sum;
                                            if (acc_kept[key] > (uint64_t)INTMAX32) {
                                                acc_sold[key] += acc_kept[key] - (uint64_t)INTMAX32;
                                                acc_kept[key] = (uint64_t)INTMAX32;
                                            }
                                        }
                                    }
                                } else {
                                    for (int sk = 0; sk < main_n; sk++) {
                                        uint64_t sum = 0;
                                        for (int t = 0; t < threads; t++) {
                                            sum += dense_kept[(size_t)t * thread_stride + sk];
                                        }
                                        if (sum > 0) {
                                            int foil = sk & 1;
                                            int pb = sk >> 1;
                                            int border = pb % 6;
                                            int pos = pb / 6;
                                            uint32_t key = make_key(EXP_ID, pos * 12 + border, dst, foil);
                                            acc_kept[key] += sum;
                                            if (acc_kept[key] > (uint64_t)INTMAX32) {
                                                acc_sold[key] += acc_kept[key] - (uint64_t)INTMAX32;
                                                acc_kept[key] = (uint64_t)INTMAX32;
                                            }
                                        }
                                    }
                                }
                            } else {
                                for (int sk = 0; sk < main_n; sk++) {
                                    uint64_t sum = 0;
                                    for (int t = 0; t < threads; t++) {
                                        sum += dense_kept[(size_t)t * thread_stride + sk];
                                    }
                                    if (sum > 0) {
                                        int foil = sk & 1;
                                        int pb = sk >> 1;
                                        int border = pb % 6;
                                        int pos = pb / 6;
                                        uint32_t key = make_key(EXP_ID, pos * 12 + border, dst, foil);
                                        acc_kept[key] += sum;
                                        if (acc_kept[key] > (uint64_t)INTMAX32) {
                                            acc_sold[key] += acc_kept[key] - (uint64_t)INTMAX32;
                                            acc_kept[key] = (uint64_t)INTMAX32;
                                        }
                                    }
                                }
                            }
                        } else {
                            for (int sk = 0; sk < main_n; sk++) {
                                uint64_t sum = 0;
                                for (int t = 0; t < threads; t++) {
                                    sum += dense_kept[(size_t)t * thread_stride + sk];
                                }
                                if (sum > 0) {
                                    int pos = sk >> 2;
                                    int dst_val = (sk >> 1) & 1;
                                    int foil = sk & 1;
                                    uint32_t key = make_key(2, pos * 2, dst_val, foil);
                                    acc_kept[key] += sum;
                                    if (acc_kept[key] > (uint64_t)INTMAX32) {
                                        acc_sold[key] += acc_kept[key] - (uint64_t)INTMAX32;
                                        acc_kept[key] = (uint64_t)INTMAX32;
                                    }
                                }
                            }
                        }
                        if constexpr (EXP_CAN_GHOST) {
                            if (can_ghost && EXP_ID != GHOST_EXP) {
                                for (int sk = 0; sk < ghost_n; sk++) {
                                    uint64_t sum = 0;
                                    for (int t = 0; t < threads; t++) {
                                        sum += dense_kept[(size_t)t * thread_stride + main_n + sk];
                                    }
                                    if (sum > 0) {
                                        int pos = sk >> 2;
                                        int dst_val = (sk >> 1) & 1;
                                        int foil = sk & 1;
                                        uint32_t key = make_key(GHOST_EXP, pos * 2, dst_val, foil);
                                        acc_kept[key] += sum;
                                        if (acc_kept[key] > (uint64_t)INTMAX32) {
                                            acc_sold[key] += acc_kept[key] - (uint64_t)INTMAX32;
                                            acc_kept[key] = (uint64_t)INTMAX32;
                                        }
                                    }
                                }
                            }
                        }
                        // Zero all active slabs across threads
                        for (int t = 0; t < threads; t++) {
                            memset(dense_kept + (size_t)t * thread_stride, 0, (size_t)(main_n + (can_ghost ? ghost_n : 0)) * sizeof(uint32_t));
                        }
                    }
                    if (ckpt_now && !last) {
                        if (ckpt_save(ckpt_path, 1, eitem, shop, count, c + 1, chunks, (uint64_t)threads, seed,
                                      acc_kept.data(), acc_sold.data(), states.data(), (uint64_t)threads) != 0) {
                            // Checkpoint write failure handled gracefully
                        }
                    }
                }
#pragma omp barrier
            }
        };

        // --- Fast Path: Non-Ghost Direct-to-Histogram Path ---
        if constexpr (!EXP_CAN_GHOST) {
            const Params p0 = pack_params[0];
            const Thr th0 = thresholds[0];
            for (uint64_t c = start_chunk; c < effective_chunks && rc == 0; c++) {
                uint64_t piece = base_piece + (c < rem_piece ? 1 : 0);
                for (uint64_t p = 0; p < piece; p++) {
                    int god = roll_god(rng, shop);
                    if (PACK_LIKELY(god == 0)) {
                        open_pack_cards_direct<EXP_ID, true>(rng, p0, th0, K_main);
                    } else {
                        open_pack_cards_direct<EXP_ID, false>(rng, pack_params[god], thresholds[god], K_main, god);
                    }
                }
                states[(size_t)ti] = rng.s;
                sync_chunk(c);
            }
        // --- Direct-to-Histogram Path for Exp 0 & Exp 1 (All Collections 0..7) ---
        } else if constexpr (EXP_ID == 0 || EXP_ID == 1) {
            uint32_t * const K_ghost = (can_ghost) ? (K_main + main_n) : nullptr;
            if (pack_params[0].commons == 7) {
                run_sim_direct_exp01<EXP_ID, 27>(ti, rng, shop, effective_chunks, start_chunk,
                                                 base_piece, rem_piece, rc, pack_params, thresholds,
                                                 K_main, K_ghost, PRISTINE_COMMONS_C12, states, sync_chunk);
            } else if (pack_params[0].rares == 7) {
                run_sim_direct_exp01<EXP_ID, 34>(ti, rng, shop, effective_chunks, start_chunk,
                                                 base_piece, rem_piece, rc, pack_params, thresholds,
                                                 K_main, K_ghost, PRISTINE_RARES_C12, states, sync_chunk);
            } else if (pack_params[0].epics == 7) {
                run_sim_direct_exp01<EXP_ID, 34>(ti, rng, shop, effective_chunks, start_chunk,
                                                 base_piece, rem_piece, rc, pack_params, thresholds,
                                                 K_main, K_ghost, PRISTINE_EPICS_C12, states, sync_chunk);
            } else if (pack_params[0].legendaries == 7) {
                run_sim_direct_exp01<EXP_ID, 26>(ti, rng, shop, effective_chunks, start_chunk,
                                                 base_piece, rem_piece, rc, pack_params, thresholds,
                                                 K_main, K_ghost, PRISTINE_LEGS_C12, states, sync_chunk);
            } else {
                run_sim_fallback<EXP_ID>(ti, rng, shop, effective_chunks, start_chunk, share, states, rc,
                                        pack_params, thresholds, K_main, main_n, can_ghost, sync_chunk);
            }
        // --- Full Fallback Path: Exp 7 ---
        } else {
            run_sim_fallback<EXP_ID>(ti, rng, shop, effective_chunks, start_chunk, share, states, rc,
                                    pack_params, thresholds, K_main, main_n, can_ghost, sync_chunk);
        }
    }
}
