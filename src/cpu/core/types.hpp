#pragma once

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cstdint>
#include <cmath>
#include <chrono>
#include <vector>
#include <algorithm>
#include <atomic>
#include <mutex>

#ifdef _OPENMP
#include <omp.h>
#endif

#define PACK_DEVICE
#include "src/common/tables_gen.hpp"
#include "src/common/packopen_common.hpp"
#include "src/cpu/core/math.hpp"

#define make_key(e, s, d, f) (((uint32_t)(e) << 14) | ((uint32_t)(s) << 2) | ((uint32_t)(d) << 1) | (uint32_t)(f))
#define key_e(k) ((int)((k) >> 14))
#define key_s(k) ((int)(((k) >> 2) & 0xFFF))
#define key_d(k) ((int)(((k) >> 1) & 1))
#define key_f(k) ((int)((k) & 1))

/**
 * @brief Prebuilt static card partitions for expansions with duplicate cards allowed (haveDup == true).
 */
struct Prebuilt {
    int leg[128], epic[128], rare[128], comm[128], all[256];
    int n[5]; ///< Element counts for [legendary, epic, rare, common, all]
};

static Prebuilt GPRE[NEXPS];
static std::once_flag GPRE_ONCE;

/**
 * @brief Thread-safe one-time initialization of static Prebuilt tables for all expansions.
 */
static void prebuilt_init(void) {
    for (int x = 0; x < NEXPS; x++) {
        const ExpT *e = &EXPS[x];
        Prebuilt &g = GPRE[x];
        g.n[0] = g.n[1] = g.n[2] = g.n[3] = g.n[4] = 0;
        for (int i = 0; i < e->n; i++) {
            int r = e->rar[i];
            g.all[g.n[4]++] = i;
            if (r == 3) g.leg[g.n[0]++] = i;
            else if (r == 2) g.epic[g.n[1]++] = i;
            else if (r == 1) g.rare[g.n[2]++] = i;
            else g.comm[g.n[3]++] = i;
        }
    }
}

typedef void (*packopen_progress_cb)(unsigned long long done, unsigned long long total, void *ctx);

/**
 * @brief Unified simulation context passed across CPU worker dispatchers.
 */
struct CpuSimContext {
    int eitem;
    int exp_id;
    int collection;
    int shop;
    int threads;
    unsigned long long count;
    const Params *pack_params;
    const Thr *thresholds;
    uint64_t chunks;
    const std::vector<uint64_t> &share;
    std::vector<uint64_t> &states;
    std::vector<uint64_t> &acc_kept;
    std::vector<uint64_t> &acc_sold;
    uint64_t start_chunk;
    uint32_t *dense_kept;
    size_t thread_stride;
    int main_slots;
    bool can_ghost;
    const char *ckpt_path;
    uint64_t ckpt_every;
    uint64_t fold_every;
    bool use_ckpt;
    packopen_progress_cb progress;
    void *progress_ctx;
    uint64_t &done_packs;
    int &rc;
    uint64_t seed;
};
