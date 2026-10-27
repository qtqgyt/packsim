#include "src/gpu/core/types.cuh"
#include "src/gpu/core/tables.cuh"
#include "src/gpu/arch/dispatcher.cuh"

extern "C" {

typedef void (*packopen_progress_cb)(unsigned long long done, unsigned long long total, void *ctx);

int packopen_probe(void) {
    int n = 0;
    cudaError_t err = cudaGetDeviceCount(&n);
    if (err != cudaSuccess || n <= 0) return 0;
    cudaDeviceProp prop;
    if (cudaGetDeviceProperties(&prop, 0) != cudaSuccess) return 0;
    int cc = prop.major * 10 + prop.minor;
    // Requires Turing (sm_75) or newer when compiled for modern architectures.
    if (cc < 75) return 0;
    return 1;
}

int packopen_sim_seed_ckpt(int eitem, unsigned long long count, int blocks, int shop,
                           const char *results_path, const char *sold_path,
                           const char *ckpt_path, double *out_seconds,
                           packopen_progress_cb progress, void *progress_ctx,
                           unsigned long long seed) {
    if (!packopen_probe()) return 1;
    int collection, exp_id;
    if (!find_pack_host(eitem, collection, exp_id) || count == 0) return 2;
    return arch::dispatch_sim(eitem, count, blocks, shop, results_path, sold_path,
                              ckpt_path, out_seconds, progress, progress_ctx, (uint64_t)seed);
}

int packopen_sim_ckpt(int eitem, unsigned long long count, int blocks, int shop,
                      const char *results_path, const char *sold_path,
                      const char *ckpt_path, double *out_seconds,
                      packopen_progress_cb progress, void *progress_ctx) {
    return packopen_sim_seed_ckpt(eitem, count, blocks, shop, results_path, sold_path,
                                  ckpt_path, out_seconds, progress, progress_ctx, resolve_seed());
}

int packopen_sim_seed(int eitem, unsigned long long count, int blocks, int shop,
                      const char *results_path, const char *sold_path,
                      double *out_seconds, unsigned long long seed) {
    return packopen_sim_seed_ckpt(eitem, count, blocks, shop, results_path, sold_path,
                                  nullptr, out_seconds, nullptr, nullptr, seed);
}

int packopen_sim(int eitem, unsigned long long count, int blocks, int shop,
                 const char *results_path, const char *sold_path,
                 double *out_seconds) {
    return packopen_sim_seed_ckpt(eitem, count, blocks, shop, results_path, sold_path,
                                  nullptr, out_seconds, nullptr, nullptr, resolve_seed());
}

}
