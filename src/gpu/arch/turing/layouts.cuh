#pragma once

#include "src/gpu/core/types.cuh"

namespace arch::turing {

/**
 * @brief Bank-Conflict-Free Shared-Memory Indexing Schemes for Turing SM 7.5.
 *
 * Turing shared memory has 32 memory banks (each 4 bytes wide).
 * If consecutive thread accesses land on indices sharing the same bank (idx % 32),
 * the LSU serializes the memory requests into multiple clock cycles.
 *
 * Coprime layouts use a stride coprime to 32 (gcd(stride, 32) = 1) so that card positions
 * map to distinct, uniform banks across all 32 warp lanes.
 */

// 49-stride layout for ALL_COMMON (Exp 0 & 1 with 100% commons: e0, e8)
// 27 common cards * 49 slots = 1323 slots (5.29 KB)
// gcd(49, 32) = 1 -> zero bank conflicts
__device__ __forceinline__ static uint32_t layout_common_49(int common_idx, int border, int dstv, int foil) {
    return (uint32_t)common_idx * 49U + (((uint32_t)border << 2) | ((uint32_t)dstv << 1) | (uint32_t)foil);
}

// 13-stride layout for Single-Dest Non-Rarity Expansions (Exp 3, 4, 5: e17, e18, e19)
// 6 borders * 2 foils = 12 active states per card + 1 padding word = 13 slots
// gcd(13, 32) = 1 -> zero bank conflicts
__device__ __forceinline__ static uint32_t layout_exp345_13(int pos, int border, int foil) {
    return (uint32_t)pos * 13U + (((uint32_t)border << 1) | (uint32_t)foil);
}

// 5-stride layout for Ghost Pack (Exp 2: e16)
// 2 destinations * 2 foils = 4 active states per card + 1 padding word = 5 slots
// 20 cards * 5 slots = 100 slots (400 bytes)
// gcd(5, 32) = 1 -> zero bank conflicts
__device__ __forceinline__ static uint32_t layout_exp2_5(int pos, int dstv, int foil) {
    return (uint32_t)pos * 5U + (((uint32_t)dstv << 1) | (uint32_t)foil);
}

} // namespace arch::turing
