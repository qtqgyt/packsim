#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/rng.cuh"
#include "src/common/tables_gen.hpp"

struct GodInfo {
    int foil;
    int border;
    uint64_t step;
};

__device__ static const GodInfo GOD_INFO[14] = {
    {0, 0, 0},                                // god 0: not a god pack
    {0, 1, 6ULL * 0x9E3779B97F4A7C15ULL},     // god 1: First Edition (border 1), 6 steps
    {0, 2, 5ULL * 0x9E3779B97F4A7C15ULL},     // god 2: Silver (border 2), 5 steps
    {0, 3, 4ULL * 0x9E3779B97F4A7C15ULL},     // god 3: Gold (border 3), 4 steps
    {0, 4, 3ULL * 0x9E3779B97F4A7C15ULL},     // god 4: EX (border 4), 3 steps
    {0, 5, 2ULL * 0x9E3779B97F4A7C15ULL},     // god 5: Full Art (border 5), 2 steps
    {1, 1, 6ULL * 0x9E3779B97F4A7C15ULL},     // god 6: Foil + First Ed, 6 steps
    {1, 2, 5ULL * 0x9E3779B97F4A7C15ULL},     // god 7: Foil + Silver, 5 steps
    {1, 3, 4ULL * 0x9E3779B97F4A7C15ULL},     // god 8: Foil + Gold, 4 steps
    {1, 4, 3ULL * 0x9E3779B97F4A7C15ULL},     // god 9: Foil + EX, 3 steps
    {1, 5, 2ULL * 0x9E3779B97F4A7C15ULL},     // god 10: Foil + Full Art, 2 steps
    {0, 0, 6ULL * 0x9E3779B97F4A7C15ULL},     // god 11: All Ghost, 6 steps
    {1, 0, 6ULL * 0x9E3779B97F4A7C15ULL},     // god 12: All Foil Ghost, 6 steps
    {1, 0, 6ULL * 0x9E3779B97F4A7C15ULL}      // god 13: All Foil, 6 steps
};

__device__ __constant__ const unsigned char c_comm[27] = {
    0, 4, 8, 12, 16, 20, 24, 28, 38, 42, 43, 46, 55, 59, 63, 67, 71, 75, 78, 81, 84, 93, 102, 105, 111, 113, 117
};

__device__ __constant__ const unsigned char c_rar_lists[121] = {
    3, 7, 11, 15, 19, 23, 27, 31, 41, 49, 50, 51, 52, 53, 58, 62, 66, 70, 74, 88, 91, 99, 101, 110, 116, 120, // 26 leg
    2, 6, 10, 14, 18, 22, 26, 30, 33, 34, 40, 45, 48, 54, 57, 61, 65, 69, 73, 77, 80, 83, 86, 87, 90, 95, 97, 98, 100, 104, 107, 109, 115, 119, // 34 epic
    1, 5, 9, 13, 17, 21, 25, 29, 32, 35, 36, 37, 39, 44, 47, 56, 60, 64, 68, 72, 76, 79, 82, 85, 89, 92, 94, 96, 103, 106, 108, 112, 114, 118, // 34 rare
    0, 4, 8, 12, 16, 20, 24, 28, 38, 42, 43, 46, 55, 59, 63, 67, 71, 75, 78, 81, 84, 93, 102, 105, 111, 113, 117 // 27 comm
};

PACK_BOTH static inline int slab_slots_for(int exp_id) {
    switch (exp_id) {
    case 0: case 1: case 6: case 7: case 8: return 121 * 48; // 5808
    case 2: return 156;
    case 3: return 113 * 48; // 5424
    case 4: return 50 * 48;  // 2400
    case 5: return 40 * 48;  // 1920
    default: return KEY_N;
    }
}

PACK_BOTH static inline constexpr uint64_t exp_all_cards(int exp_id) {
    switch (exp_id) {
        case 2: return 20;
        case 3: return 113;
        case 4: return 50;
        case 5: return 40;
        default: return 121;
    }
}

PACK_BOTH static inline bool run_has_ghost(int exp_id) {
    return exp_id == 0 || exp_id == 1 || exp_id == 7;
}

PACK_BOTH static inline uint32_t slab_key(uint32_t slot, int exp_id, int main_slots) {
    if (slot >= (uint32_t)main_slots) return ((uint32_t)GHOST_EXP << 14) | (slot - main_slots);
    return ((uint32_t)exp_id << 14) | slot;
}

static inline int find_pack_host(int eitem, int &collection, int &exp) {
    if (eitem == 129) { collection = 15; exp = 7; return 1; }
    for (int i = 0; i < NPACKMAP; i++) {
        if (PACKMAP[i].eitem == eitem) {
            collection = PACKMAP[i].collection;
            exp = PACKMAP[i].exp;
            return 1;
        }
    }
    return 0;
}
