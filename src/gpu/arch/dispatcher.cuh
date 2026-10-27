#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/arch/turing/dispatcher.cuh"
#include "src/gpu/arch/generic/dispatcher.cuh"

namespace arch {

enum class BackendArch {
    Generic, // Pascal sm_61 and earlier
    Turing   // Turing sm_75 and later (Ampere, Ada, Hopper, Blackwell)
};

inline BackendArch detect_backend(int device_id = 0) {
    cudaDeviceProp prop;
    if (cudaGetDeviceProperties(&prop, device_id) == cudaSuccess) {
        if (prop.major > 7 || (prop.major == 7 && prop.minor >= 5)) {
            return BackendArch::Turing;
        }
    }
    return BackendArch::Generic;
}

typedef void (*packopen_progress_cb)(unsigned long long done, unsigned long long total, void *ctx);

inline int dispatch_sim(int eitem, unsigned long long count, int blocks, int shop,
                        const char *results_path, const char *sold_path,
                        const char *ckpt_path, double *out_seconds,
                        packopen_progress_cb progress, void *progress_ctx,
                        uint64_t seed = resolve_seed()) {
    BackendArch backend = detect_backend(0);
    if (backend == BackendArch::Turing) {
        return turing::run_sim(eitem, count, blocks, shop, results_path, sold_path,
                               ckpt_path, out_seconds, progress, progress_ctx, seed);
    } else {
        return generic::run_sim(eitem, count, blocks, shop, results_path, sold_path,
                                ckpt_path, out_seconds, progress, progress_ctx, seed);
    }
}

} // namespace arch
