#pragma once

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <chrono>
#include <vector>
#include <cuda_runtime.h>

#ifndef PACK_DEVICE
#define PACK_DEVICE __device__
#endif

#ifndef PACK_HOST
#define PACK_HOST __host__
#endif

#ifndef PACK_BOTH
#define PACK_BOTH __host__ __device__
#endif

PACK_BOTH static inline int key_e(uint32_t k) { return (int)((k >> 11) & 0x7); }
PACK_BOTH static inline int key_s(uint32_t k) { return (int)((k >> 2) & 0x1ff); }
PACK_BOTH static inline int key_d(uint32_t k) { return (int)((k >> 1) & 0x1); }
PACK_BOTH static inline int key_f(uint32_t k) { return (int)(k & 0x1); }
