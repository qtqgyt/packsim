#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/math.cuh"
#include "src/gpu/core/rng.cuh"
#include "src/gpu/core/tables.cuh"

namespace arch::turing {


/**
 * @brief Branchless evaluation of foil and border via SplitMix64 ILP and clz.
 */
__device__ __forceinline__ static void evaluate_borders_branchless(
    Rng &rng, const Thr &th, int &foil, int &border, int &used) {
    const uint64_t G = 0x9E3779B97F4A7C15ULL;
    const uint64_t s0 = rng.s;
    uint64_t m0, m1, m2, m3, m4, m5;
    sm_mix6(s0 + G, s0 + 2 * G, s0 + 3 * G, s0 + 4 * G, s0 + 5 * G, s0 + 6 * G, m0, m1, m2, m3, m4, m5);
    
    int r0 = fastmod10k(m0);
    int r1 = fastmod10k(m1);
    int r2 = fastmod10k(m2);
    int r3 = fastmod10k(m3);
    int r4 = fastmod10k(m4);
    int r5 = fastmod10k(m5);

    foil = (r0 < th.foil) ? 1 : 0;
    unsigned int mask = ((r1 < th.b_full_art) << 5) |
                        ((r2 < th.b_ex) << 4) |
                        ((r3 < th.b_gold) << 3) |
                        ((r4 < th.b_silver) << 2) |
                        ((r5 < th.b_first_ed) << 1) | 1;
    border = 31 - __clz(mask);
    used = (border == 0) ? 6 : (7 - border);
    rng.s = s0 + (uint64_t)used * G;
}

/**
 * @brief Evaluates 7 cards for ALL_COMMON using register-forwarded swap-remove.
 *
 * Eliminates thread stack frame and local memory spills completely.
 * 21 comparisons resolve the 7 card indices directly in scalar registers.
 */
template <typename CardCallback>
__device__ __forceinline__ static void roll_cards_register_forwarded(Rng &rng, CardCallback &&callback) {
    // Draw 0: rem = 27 (0 comparisons)
    int k0 = (int)fastmod_u64(rng.next(), 27);
    int c0 = k0;
    callback(c0, 0);

    // Draw 1: rem = 26 (1 comparison)
    int k1 = (int)fastmod_u64(rng.next(), 26);
    int c1 = (k1 == k0) ? 26 : k1;
    callback(c1, 1);

    // Draw 2: rem = 25 (2 comparisons)
    int k2 = (int)fastmod_u64(rng.next(), 25);
    int x2 = k2;
    if (x2 == k1) x2 = 25;
    if (x2 == k0) x2 = 26;
    int c2 = x2;
    callback(c2, 2);

    // Draw 3: rem = 24 (3 comparisons)
    int k3 = (int)fastmod_u64(rng.next(), 24);
    int x3 = k3;
    if (x3 == k2) x3 = 24;
    if (x3 == k1) x3 = 25;
    if (x3 == k0) x3 = 26;
    int c3 = x3;
    callback(c3, 3);

    // Draw 4: rem = 23 (4 comparisons)
    int k4 = (int)fastmod_u64(rng.next(), 23);
    int x4 = k4;
    if (x4 == k3) x4 = 23;
    if (x4 == k2) x4 = 24;
    if (x4 == k1) x4 = 25;
    if (x4 == k0) x4 = 26;
    int c4 = x4;
    callback(c4, 4);

    // Draw 5: rem = 22 (5 comparisons)
    int k5 = (int)fastmod_u64(rng.next(), 22);
    int x5 = k5;
    if (x5 == k4) x5 = 22;
    if (x5 == k3) x5 = 23;
    if (x5 == k2) x5 = 24;
    if (x5 == k1) x5 = 25;
    if (x5 == k0) x5 = 26;
    int c5 = x5;
    callback(c5, 5);

    // Draw 6: rem = 21 (6 comparisons)
    int k6 = (int)fastmod_u64(rng.next(), 21);
    int x6 = k6;
    if (x6 == k5) x6 = 21;
    if (x6 == k4) x6 = 22;
    if (x6 == k3) x6 = 23;
    if (x6 == k2) x6 = 24;
    if (x6 == k1) x6 = 25;
    if (x6 == k0) x6 = 26;
    int c6 = x6;
    callback(c6, 6);
}

} // namespace arch::turing
