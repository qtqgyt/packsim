#pragma once

#include "src/cpu/sim/god.hpp"

template<int EXP_ID, int POOL_SIZE, bool GOD_IS_ZERO>
static PACK_ALWAYS_INLINE void generate_keys_exp01(
    Rng &rng, const Thr &th, const int *pristine_pool, uint32_t *k, int god = 0) {
    (void)th;

    int foil_c = 0, bor_c = 0;
    uint64_t step_c = 0;
    if constexpr (!GOD_IS_ZERO) {
        foil_c = GOD_INFO[god].foil;
        bor_c = GOD_INFO[god].border;
        step_c = GOD_INFO[god].step;
    }

    auto roll_card = [&](int c_idx, int s) {
        int c12 = pristine_pool[c_idx];
        int foil, bor;
        if constexpr (GOD_IS_ZERO) {
            foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
            bor = roll_border_god0(rng);
        } else {
            foil = foil_c;
            bor = bor_c;
            rng.s += step_c;
        }
        k[s] = (uint32_t)(c12 + (bor << 1) + foil);
    };

    // Draw 0: rem = POOL_SIZE (0 comparisons)
    int k0 = (int)fastmod_u64(rng.next(), (uint64_t)POOL_SIZE);
    int c0 = k0;
    roll_card(c0, 0);

    // Draw 1: rem = POOL_SIZE - 1 (1 comparison)
    int k1 = (int)fastmod_u64(rng.next(), (uint64_t)(POOL_SIZE - 1));
    int c1 = (k1 == k0) ? (POOL_SIZE - 1) : k1;
    roll_card(c1, 1);

    // Draw 2: rem = POOL_SIZE - 2 (2 comparisons)
    int k2 = (int)fastmod_u64(rng.next(), (uint64_t)(POOL_SIZE - 2));
    int x2 = k2;
    if (x2 == k1) x2 = POOL_SIZE - 2;
    if (x2 == k0) x2 = POOL_SIZE - 1;
    int c2 = x2;
    roll_card(c2, 2);

    // Draw 3: rem = POOL_SIZE - 3 (3 comparisons)
    int k3 = (int)fastmod_u64(rng.next(), (uint64_t)(POOL_SIZE - 3));
    int x3 = k3;
    if (x3 == k2) x3 = POOL_SIZE - 3;
    if (x3 == k1) x3 = POOL_SIZE - 2;
    if (x3 == k0) x3 = POOL_SIZE - 1;
    int c3 = x3;
    roll_card(c3, 3);

    // Draw 4: rem = POOL_SIZE - 4 (4 comparisons)
    int k4 = (int)fastmod_u64(rng.next(), (uint64_t)(POOL_SIZE - 4));
    int x4 = k4;
    if (x4 == k3) x4 = POOL_SIZE - 4;
    if (x4 == k2) x4 = POOL_SIZE - 3;
    if (x4 == k1) x4 = POOL_SIZE - 2;
    if (x4 == k0) x4 = POOL_SIZE - 1;
    int c4 = x4;
    roll_card(c4, 4);

    // Draw 5: rem = POOL_SIZE - 5 (5 comparisons)
    int k5 = (int)fastmod_u64(rng.next(), (uint64_t)(POOL_SIZE - 5));
    int x5 = k5;
    if (x5 == k4) x5 = POOL_SIZE - 5;
    if (x5 == k3) x5 = POOL_SIZE - 4;
    if (x5 == k2) x5 = POOL_SIZE - 3;
    if (x5 == k1) x5 = POOL_SIZE - 2;
    if (x5 == k0) x5 = POOL_SIZE - 1;
    int c5 = x5;
    roll_card(c5, 5);

    // Draw 6: rem = POOL_SIZE - 6 (6 comparisons)
    int k6 = (int)fastmod_u64(rng.next(), (uint64_t)(POOL_SIZE - 6));
    int x6 = k6;
    if (x6 == k5) x6 = POOL_SIZE - 6;
    if (x6 == k4) x6 = POOL_SIZE - 5;
    if (x6 == k3) x6 = POOL_SIZE - 4;
    if (x6 == k2) x6 = POOL_SIZE - 3;
    if (x6 == k1) x6 = POOL_SIZE - 2;
    if (x6 == k0) x6 = POOL_SIZE - 1;
    int c6 = x6;
    roll_card(c6, 6);
}

template<bool GOD_IS_ZERO>
static PACK_ALWAYS_INLINE void roll_ghost_pack_keys(
    Rng &rng, const Params &prm, const Thr &th, uint32_t *gk, int god = 0) {
    (void)th;
    constexpr uint64_t const_n = 20;
    const int foil_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].foil;
    const uint64_t step_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].step;
    #pragma GCC unroll 7
    for (int s = 0; s < 7; s++) {
        if (s >= prm.legendaries) {
            rng.s += 0x9E3779B97F4A7C15ULL;
        }
        int pos = (int)(rng.next() % const_n);
        int foil;
        if constexpr (GOD_IS_ZERO) {
            foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
            roll_border_god0(rng);
        } else {
            foil = foil_c;
            rng.s += step_c;
        }
        int dst_val = ((rng.next() % 100ULL) < 50) ? 1 : 0;
        // Dense Ghost indexing: pos (0..19), dst_val (0..1), foil (0..1) -> 80 slots (320 bytes)
        gk[s] = ((uint32_t)pos << 2) | ((uint32_t)dst_val << 1) | (uint32_t)foil;
    }
}
