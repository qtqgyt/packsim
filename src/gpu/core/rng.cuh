#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/math.cuh"
#include "src/common/packopen_common.hpp"
#include <random>

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
