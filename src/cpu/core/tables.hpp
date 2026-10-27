#pragma once

#include "src/cpu/core/types.hpp"

/**
 * @brief Template traits defining microarchitectural and mathematical properties per expansion.
 */
template<int EXP_ID>
struct ExpTraits;

template<> struct ExpTraits<0> {
    static constexpr bool haveDup = false;
    static constexpr bool useRarity = true;
    static constexpr int const_n = 121;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 1452;
};

template<> struct ExpTraits<1> {
    static constexpr bool haveDup = false;
    static constexpr bool useRarity = true;
    static constexpr int const_n = 121;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 1452;
};

template<> struct ExpTraits<2> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = false;
    static constexpr int const_n = 20;
    static constexpr int perMonster = 2;
    static constexpr int main_n = 80;
};

template<> struct ExpTraits<3> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = false;
    static constexpr int const_n = 113;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 1356;
};

template<> struct ExpTraits<4> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = false;
    static constexpr int const_n = 50;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 600;
};

template<> struct ExpTraits<5> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = false;
    static constexpr int const_n = 40;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 480;
};

template<> struct ExpTraits<6> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = false;
    static constexpr int const_n = 121;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 1452;
};

template<> struct ExpTraits<7> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = false;
    static constexpr int const_n = 121;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 1452;
};

template<> struct ExpTraits<8> {
    static constexpr bool haveDup = true;
    static constexpr bool useRarity = true;
    static constexpr int const_n = 121;
    static constexpr int perMonster = 12;
    static constexpr int main_n = 1452;
};

struct GodOutcome {
    int foil;
    int border;
    uint64_t step;
};

static constexpr uint64_t SPLITMIX_STEP = 0x9E3779B97F4A7C15ULL;

static constexpr GodOutcome GOD_INFO[14] = {
    {0, 0, 0ULL},                     // god 0: standard pack
    {0, 1, 6ULL * SPLITMIX_STEP},     // god 1: foil=0, border=1, used=6
    {0, 2, 5ULL * SPLITMIX_STEP},     // god 2: foil=0, border=2, used=5
    {0, 3, 4ULL * SPLITMIX_STEP},     // god 3: foil=0, border=3, used=4
    {0, 4, 3ULL * SPLITMIX_STEP},     // god 4: foil=0, border=4, used=3
    {0, 5, 2ULL * SPLITMIX_STEP},     // god 5: foil=0, border=5, used=2
    {1, 1, 6ULL * SPLITMIX_STEP},     // god 6: foil=1, border=1, used=6
    {1, 2, 5ULL * SPLITMIX_STEP},     // god 7: foil=1, border=2, used=5
    {1, 3, 4ULL * SPLITMIX_STEP},     // god 8: foil=1, border=3, used=4
    {1, 4, 3ULL * SPLITMIX_STEP},     // god 9: foil=1, border=4, used=3
    {1, 5, 2ULL * SPLITMIX_STEP},     // god 10: foil=1, border=5, used=2
    {0, 0, 6ULL * SPLITMIX_STEP},     // god 11: foil=0, border=0, used=6
    {1, 0, 6ULL * SPLITMIX_STEP},     // god 12: foil=1, border=0, used=6
    {1, 0, 6ULL * SPLITMIX_STEP},     // god 13: foil=1, border=0, used=6
};

// True positions of the 27 common cards in Exp 0 & Exp 1
static constexpr int PRISTINE_COMMONS[27] = {
    0, 4, 8, 12, 16, 20, 24, 28, 38, 42, 43, 46, 55, 59, 63, 67, 71, 75, 78, 81, 84, 93, 102, 105, 111, 113, 117
};

// Dense pre-multiplied common offsets (c_idx * 12) for 324-slot (1.26 KB) dense slab
static constexpr int PRISTINE_COMMONS_C12[27] = {
    0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 180, 192, 204, 216, 228, 240, 252, 264, 276, 288, 300, 312
};

// True positions of Rares (34 cards) in Exp 0/1:
static constexpr int PRISTINE_RARES[34] = {
    1, 5, 9, 13, 17, 21, 25, 29, 32, 35, 36, 37, 39, 44, 47, 56, 60, 64, 68, 72, 76, 79, 82, 85, 89, 92, 94, 96, 103, 106, 108, 112, 114, 118
};

// Dense pre-multiplied rare offsets (idx * 12) for 408-slot (1.60 KB) dense slab
static constexpr int PRISTINE_RARES_C12[34] = {
    0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 180, 192, 204, 216, 228, 240, 252, 264, 276, 288, 300, 312, 324, 336, 348, 360, 372, 384, 396
};

// True positions of Epics (34 cards) in Exp 0/1:
static constexpr int PRISTINE_EPICS[34] = {
    2, 6, 10, 14, 18, 22, 26, 30, 33, 34, 40, 45, 48, 54, 57, 61, 65, 69, 73, 77, 80, 83, 86, 87, 90, 95, 97, 98, 100, 104, 107, 109, 115, 119
};

// Dense pre-multiplied epic offsets (idx * 12) for 408-slot (1.60 KB) dense slab
static constexpr int PRISTINE_EPICS_C12[34] = {
    0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 180, 192, 204, 216, 228, 240, 252, 264, 276, 288, 300, 312, 324, 336, 348, 360, 372, 384, 396
};

// True positions of Legendaries (26 cards) in Exp 0/1:
static constexpr int PRISTINE_LEGS[26] = {
    3, 7, 11, 15, 19, 23, 27, 31, 41, 49, 50, 51, 52, 53, 58, 62, 66, 70, 74, 88, 91, 99, 101, 110, 116, 120
};

// Dense pre-multiplied legendary offsets (idx * 12) for 312-slot (1.22 KB) dense slab
static constexpr int PRISTINE_LEGS_C12[26] = {
    0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 180, 192, 204, 216, 228, 240, 252, 264, 276, 288, 300
};

static inline int find_pack_host(int eitem, int &collection, int &exp) {
    if (eitem == 129) { collection = 15; exp = 7; return 1; }
    for (int i = 0; i < NPACKMAP; i++) if (PACKMAP[i].eitem == eitem) {
        collection = PACKMAP[i].collection; exp = PACKMAP[i].exp; return 1;
    }
    return 0;
}
