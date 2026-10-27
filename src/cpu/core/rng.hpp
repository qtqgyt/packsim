#pragma once

#include "src/cpu/core/types.hpp"
#include <random>

static const uint64_t SEED_STEP = 0x9E3779B97F4A7C15ULL;

static inline uint64_t resolve_seed() {
    const char *env = getenv("PACKSIM_SEED");
    if (env && *env) {
        if (strcmp(env, "static") == 0 || strcmp(env, "default") == 0 || strcmp(env, "ref") == 0) {
            return SEED_BASE;
        }
        return (uint64_t)strtoull(env, nullptr, 0);
    }
    std::random_device rd;
    return ((uint64_t)rd() << 32) | (uint64_t)rd();
}

static inline uint64_t thread_seed(int ti, uint64_t seed_base) {
    return seed_base ^ ((uint64_t)(ti + 1) * SEED_STEP);
}

static inline uint64_t thread_seed(int ti) {
    return thread_seed(ti, SEED_BASE);
}
