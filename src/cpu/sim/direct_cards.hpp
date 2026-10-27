#pragma once

#include "src/cpu/sim/god.hpp"

/**
 * @brief Direct-to-Histogram Pack Generation for Exp 2..6 and 8.
 */
template<int EXP_ID, bool GOD_IS_ZERO>
static PACK_ALWAYS_INLINE void open_pack_cards_direct(
    Rng &rng, const Params &prm, const Thr &th, uint32_t *K_main, int god = 0) {

    // --- Direct Path: Exp 2 (Ghost) ---
    if constexpr (EXP_ID == 2) {
        constexpr uint64_t const_n = (uint64_t)ExpTraits<2>::const_n;
        const int foil_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].foil;
        const uint64_t step_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].step;
#if defined(__GNUC__) || defined(__clang__)
        #pragma GCC unroll 7
#endif
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
            uint32_t sk = ((uint32_t)pos << 2) | ((uint32_t)dst_val << 1) | (uint32_t)foil;
            K_main[sk]++;
        }
    // --- Direct Path: Exp 3..6 (No-Rarity Uniform Expansions) ---
    } else if constexpr (EXP_ID >= 3 && EXP_ID <= 6) {
        constexpr uint64_t const_n = (uint64_t)ExpTraits<EXP_ID>::const_n;
        const int foil_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].foil;
        const int bor_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].border;
        const uint64_t step_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].step;
#if defined(__GNUC__) || defined(__clang__)
        #pragma GCC unroll 7
#endif
        for (int s = 0; s < 7; s++) {
            if (s >= prm.commons) {
                rng.s += 0x9E3779B97F4A7C15ULL;
            }
            int pos = (int)(rng.next() % const_n);
            int foil;
            int border;
            if constexpr (GOD_IS_ZERO) {
                foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
                border = roll_border_god0(rng);
            } else {
                foil = foil_c;
                border = bor_c;
                rng.s += step_c;
            }
            // Dense non-ghost indexing: (pos * 6 + border) * 2 + foil -> const_n * 12 slots
            uint32_t sk = (uint32_t)((pos * 6 + border) * 2 + foil);
            K_main[sk]++;
        }
    // --- Direct Path: Exp 8 (Immutable Prebuilt Rarity Distribution) ---
    } else if constexpr (EXP_ID == 8) {
        const int foil_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].foil;
        const int bor_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].border;
        const uint64_t step_c = GOD_IS_ZERO ? 0 : GOD_INFO[god].step;
        Prebuilt *pre = &GPRE[8];
        int nleg = pre->n[0], nep = pre->n[1], nra = pre->n[2], ncm = pre->n[3], nall = pre->n[4];
        int *vleg = pre->leg, *vepic = pre->epic, *vrare = pre->rare, *vcomm = pre->comm;
        int c2 = 0, c3 = 0, c4 = 0, c5 = 0;
        for (int s = 0; s < 7; s++) {
            if (nall == 0) break;
            int rarity;
            if (prm.legendaries - c5 > 0 && nleg > 0) { rarity = 3; c5++; }
            else if (prm.epics - c4 > 0 && nep > 0) { rarity = 2; c4++; }
            else if (prm.rares - c3 > 0 && nra > 0) { rarity = 1; c3++; }
            else if (prm.commons - c2 > 0 && ncm > 0) { rarity = 0; c2++; }
            else {
                int r21 = (int)rng.below(10000);
                bool done = false;
                rarity = 0;
                if (!done && prm.leg_casc > 0.0f && nleg > 0 && 4 - c5 > 0) {
                    if (r21 < th.leg_casc) { done = true; rarity = 3; c5++; }
                }
                if (!done && prm.epic_casc > 0.0f && nep > 0 && 4 - c4 > 0) {
                    if (r21 < th.epic_casc) { done = true; rarity = 2; c4++; }
                }
                if (!done && prm.rare_casc > 0.0f && nra > 0 && 4 - c3 > 0) {
                    if (r21 < th.rare_casc) { done = true; rarity = 1; c3++; }
                }
                if (!done) { rarity = 0; c2++; }
            }

            int pos = 0;
            int *v; int *nv;
            if (rarity == 3) { v = vleg; nv = &nleg; }
            else if (rarity == 2) { v = vepic; nv = &nep; }
            else if (rarity == 1) { v = vrare; nv = &nra; }
            else { v = vcomm; nv = &ncm; }
            if (*nv == 0) continue;
            int idx = (int)fastmod_u64(rng.next(), (uint64_t)*nv);
            pos = v[idx];

            int foil;
            int border;
            if constexpr (GOD_IS_ZERO) {
                foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
                border = roll_border_god0(rng);
            } else {
                foil = foil_c;
                border = bor_c;
                rng.s += step_c;
            }
            uint32_t sk = (uint32_t)((pos * 6 + border) * 2 + foil);
            K_main[sk]++;
        }
    }
}
