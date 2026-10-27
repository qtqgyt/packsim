#pragma once

#include "src/cpu/core/tables.hpp"

static PACK_ALWAYS_INLINE int roll_border_god0(Rng &rng) {
    if (PACK_UNLIKELY((int)rng.below(10000) < 25)) return 5;
    if (PACK_UNLIKELY((int)rng.below(10000) < 100)) return 4;
    if (PACK_UNLIKELY((int)rng.below(10000) < 400)) return 3;
    if (PACK_UNLIKELY((int)rng.below(10000) < 800)) return 2;
    if (PACK_UNLIKELY((int)rng.below(10000) < 2000)) return 1;
    return 0;
}

static PACK_ALWAYS_INLINE int roll_border_generic(Rng &rng, const Thr &th) {
    if ((int)rng.below(10000) < th.b_full_art) return 5;
    if ((int)rng.below(10000) < th.b_ex) return 4;
    if ((int)rng.below(10000) < th.b_gold) return 3;
    if ((int)rng.below(10000) < th.b_silver) return 2;
    if ((int)rng.below(10000) < th.b_first_ed) return 1;
    return 0;
}

static PACK_ALWAYS_INLINE int roll_god(Rng &rng, int shop) {
    if (shop <= 1) return 0;
    int num = (int)rng.below(100000);
    if (PACK_UNLIKELY(num >= 1000 && num <= 2950)) {
        if (num == 1000) return 12;
        if (num <= 1051) return 11;
        if (num <= 1060) return 10;
        if (num <= 1081) return 9;
        if (num <= 1130) return 8;
        if (num <= 1230) return 7;
        if (num >= 1240 && num <= 1360) return 6;
        if (num >= 1370 && num <= 1570) return 13;
        if (num >= 1580 && num <= 1650) return 5;
        if (num >= 1660 && num <= 1860) return 4;
        if (num >= 1870 && num <= 2120) return 3;
        if (num >= 2130 && num <= 2440) return 2;
        if (num >= 2450) return 1;
    }
    return 0;
}
