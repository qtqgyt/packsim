#pragma once

#include <cstdint>
#include <cstdlib>
#include <cstdio>
#include <cstring>

#if defined(_MSC_VER)
    #define PACK_ALWAYS_INLINE __forceinline
    #define PACK_LIKELY(x) (x)
    #define PACK_UNLIKELY(x) (x)
    #define PACK_ASSUME_ALIGNED(ptr, alignment) (ptr)
#elif defined(__GNUC__) || defined(__clang__)
    #define PACK_ALWAYS_INLINE __attribute__((always_inline)) inline
    #define PACK_LIKELY(x) __builtin_expect(!!(x), 1)
    #define PACK_UNLIKELY(x) __builtin_expect(!!(x), 0)
    #define PACK_ASSUME_ALIGNED(ptr, alignment) __builtin_assume_aligned((ptr), (alignment))
#else
    #define PACK_ALWAYS_INLINE inline
    #define PACK_LIKELY(x) (x)
    #define PACK_UNLIKELY(x) (x)
    #define PACK_ASSUME_ALIGNED(ptr, alignment) (ptr)
#endif

// Portable aligned allocation and deallocation helper
static inline void *pack_aligned_alloc(size_t alignment, size_t size) {
#if defined(_MSC_VER)
    return _aligned_malloc(size, alignment);
#elif defined(_ISOC11_SOURCE) || (defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L) || (__cplusplus >= 201703L)
    size_t rounded = (size + alignment - 1) & ~(alignment - 1);
    return std::aligned_alloc(alignment, rounded);
#elif defined(_POSIX_C_SOURCE) && _POSIX_C_SOURCE >= 200112L
    void *ptr = nullptr;
    if (posix_memalign(&ptr, alignment, size) != 0) return nullptr;
    return ptr;
#else
    void *ptr = nullptr;
    if (posix_memalign(&ptr, alignment, size) != 0) return nullptr;
    return ptr;
#endif
}

static inline void pack_aligned_free(void *ptr) {
#if defined(_MSC_VER)
    _aligned_free(ptr);
#else
    std::free(ptr);
#endif
}

// Single-stream SplitMix64 RNG shared between CPU and GPU.
// Must remain byte-for-byte identical across architectures for golden parity.
#ifndef PACK_BOTH
#define PACK_BOTH PACK_DEVICE
#endif
struct Rng {
    uint64_t s;
    PACK_BOTH Rng() : s(0) {}
    PACK_BOTH Rng(uint64_t seed) : s(seed) {}
    PACK_BOTH uint64_t next() {
        // splitmix64. the add at the top is the whole state advance,
        // everything after is just scrambling for output quality.
        s += 0x9E3779B97F4A7C15ULL;
        uint64_t z = s;
        z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
        z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
        return z ^ (z >> 31);
    }
    // yes % introduces modulo bias. no, do not "fix" it with rejection
    // sampling. the game code does % so we do % and the bias is load bearing.
    PACK_BOTH uint64_t below(uint64_t n) { return next() % n; }
};
// guaranteed slots are ints, everything else is a percent that later
// gets scaled by 100 and rounded in thr(). floats here are fine because
// thr() is the only place they turn into ints.
struct Params {
    int commons, rares, epics, legendaries;
    float rare_casc, epic_casc, leg_casc;
    float foil, b_first_ed, b_silver, b_gold, b_ex, b_full_art;
};
struct Thr {
    int leg_casc, epic_casc, rare_casc, foil, b_first_ed, b_silver, b_gold, b_ex, b_full_art;
};
// base pack is 1 common plus tiny cascade chances. each collection
// then overrides it. paired cases (1/5, 2/6, 3/7) are the same pack
// at different shop tiers so they share numbers on purpose.
PACK_BOTH static Params params_for(int collection) {
    Params p{1,0,0,0, 10.f,2.f,0.1f, 5.f,20.f,8.f,4.f,1.f,0.25f};
    switch (collection) {
    case 1: case 5: p.commons=0; p.rares=7; p.rare_casc+=45.f; p.epic_casc+=2.f; p.leg_casc+=1.f; break;
    case 2: case 6: p.commons=0; p.rares=1; p.epics=7; p.legendaries=0; p.rare_casc+=65.f; p.epic_casc+=45.f; p.leg_casc+=3.f; break;
    case 3: case 7: p.commons=0; p.rares=0; p.epics=1; p.legendaries=7; p.rare_casc+=65.f; p.epic_casc+=55.f; p.leg_casc+=35.f; break;
    case 0: case 4: p.commons=7; break;
    // collection 15 is the odd one out, flat 25s across the cascades.
    // that is the ghost-adjacent pack, not a typo.
    case 15: p.rare_casc=25.f; p.epic_casc=25.f; p.leg_casc=25.f; break;
    default: break;
    }
    return p;
}
// god wipes borders and foil first, then re-enables one thing.
// 11 is missing from the switch on purpose, it is handled as
// ghost-only in open_one_pack and needs no param change here.
PACK_BOTH static void apply_god(Params &p, int god) {
    if (god > 0) { p.foil=0; p.b_first_ed=0; p.b_silver=0; p.b_gold=0; p.b_ex=0; p.b_full_art=0; }
    switch (god) {
    case 1: p.b_first_ed=100.f; break;
    case 2: p.b_silver=100.f; break;
    case 3: p.b_gold=100.f; break;
    case 4: p.b_ex=100.f; break;
    case 5: p.b_full_art=100.f; break;
    case 6: p.foil=10000.f; p.b_first_ed=100.f; break;
    case 7: p.foil=10000.f; p.b_silver=100.f; break;
    case 8: p.foil=10000.f; p.b_gold=100.f; break;
    case 9: p.foil=10000.f; p.b_ex=100.f; break;
    case 10: p.foil=10000.f; p.b_full_art=100.f; break;
    case 12: case 13: p.foil=10000.f; break;
    default: break;
    }
}
// shop 1 never rolls gods, early out keeps the stream aligned with the game.
// the gaps between ranges (1231-1239 etc) are intentional misses, not bugs.
// watch the 1130 overlap, 8 is checked before 7 so 1130 lands on 8.
PACK_BOTH static inline int god_roll(Rng &rng, int shop) {
    if (shop <= 1) return 0;
    int num = (int)rng.below(100000);
    if (num == 1000) return 12;
    if (num >= 1001 && num <= 1051) return 11;
    if (num >= 1052 && num <= 1060) return 10;
    if (num >= 1061 && num <= 1081) return 9;
    if (num >= 1082 && num <= 1130) return 8;
    if (num >= 1130 && num <= 1230) return 7;
    if (num >= 1240 && num <= 1360) return 6;
    if (num >= 1370 && num <= 1570) return 13;
    if (num >= 1580 && num <= 1650) return 5;
    if (num >= 1660 && num <= 1860) return 4;
    if (num >= 1870 && num <= 2120) return 3;
    if (num >= 2130 && num <= 2440) return 2;
    if (num >= 2450 && num <= 2950) return 1;
    return 0;
}
// nearbyintf follows current rounding mode (ties to even). swapping in
// a plain cast or roundf shifts .5 cases and breaks cpu vs gpu parity.
PACK_BOTH static int thr(float rate) {
    return (int)nearbyintf(rate);
}
static const int GHOST_EXP = 2;
// int32 cap the game uses. kept clamps here, overflow goes to sold.
static const int64_t INTMAX32 = 2147483647LL;
// 18 bit keys, 262k entries. layout is exp:4 save:12 dest:1 foil:1
// packed in make_key below. KEY_N sizes every histogram in the codebase.
static const int KEY_BITS = 18;
static const int KEY_N = (1 << KEY_BITS);
PACK_BOTH static inline uint32_t make_key(int exp, int save, int dest, int foil) {
    return ((uint32_t)exp << 14) | (((uint32_t)save) << 2) | (((uint32_t)dest) << 1) | (uint32_t)foil;
}
// only ghost consumes rng here, so call order matters for stream parity.
// exps 0 and 1 are pinned destinies, everything else except ghost is 0.
PACK_BOTH static inline int destiny_of(int exp_id, Rng &rng) {
    if (exp_id == 0) return 0;
    if (exp_id == 1) return 1;
    if (exp_id == GHOST_EXP) return (rng.below(100) < 50) ? 1 : 0;
    return 0;
}

/**
 * @brief Serializes simulation histograms to disk in standardized text format:
 *        `<exp> <subvariant> <destiny> <foil> <count>`
 */
static inline void pack_write_results(const char *results_path, const char *sold_path,
                                      const uint64_t *kept, const uint64_t *sold) {
    if (results_path && results_path[0]) {
        FILE *f = fopen(results_path, "wb");
        if (f) {
            for (int k = 0; k < KEY_N; k++) {
                if (kept[k]) {
                    int exp = k >> 14;
                    int subvariant = (k >> 2) & 0xFFF;
                    int dest = (k >> 1) & 1;
                    int foil = k & 1;
                    fprintf(f, "%d %d %d %d %llu\n", exp, subvariant, dest, foil, (unsigned long long)kept[k]);
                }
            }
            fclose(f);
        }
    }
    if (sold_path && sold_path[0]) {
        FILE *f = fopen(sold_path, "wb");
        if (f) {
            for (int k = 0; k < KEY_N; k++) {
                if (sold[k]) {
                    int exp = k >> 14;
                    int subvariant = (k >> 2) & 0xFFF;
                    int dest = (k >> 1) & 1;
                    int foil = k & 1;
                    fprintf(f, "%d %d %d %d %llu\n", exp, subvariant, dest, foil, (unsigned long long)sold[k]);
                }
            }
            fclose(f);
        }
    }
}

static const uint64_t SEED_BASE = 0x12345678abcdef01ULL;

// fixed at 1000 on purpose. scaling it with count would reshuffle the
// per-thread split and invalidate resume files, so leave it alone.
static uint64_t ckpt_chunks(uint64_t count) {
    (void)count;
    return 1000;
}

// checkpoint is a straight binary dump with an 8 byte magic. write to
// tmp then rename so a crash mid-write never leaves a half file behind.
// windows rename will not overwrite, hence the remove first up there.
static int ckpt_save(const char *path, uint32_t tag, int eitem, int shop, uint64_t target,
                     uint64_t done_chunks, uint64_t chunks, uint64_t threads, uint64_t seed,
                     const uint64_t *kept, const uint64_t *sold,
                     const uint64_t *states, uint64_t nstates) {
    char tmp[4096];
    int snprintf_res = snprintf(tmp, sizeof(tmp), "%s.tmp", path);
    if (snprintf_res < 0 || (size_t)snprintf_res >= sizeof(tmp)) {
        return -1;
    }
    FILE *f = fopen(tmp, "wb");
    if (!f) return -1;
    fwrite("PKCKPT03", 1, 8, f);
    int32_t ei = eitem, sh = shop;
    fwrite(&tag, 4, 1, f);
    fwrite(&ei, 4, 1, f);
    fwrite(&sh, 4, 1, f);
    fwrite(&target, 8, 1, f);
    fwrite(&done_chunks, 8, 1, f);
    fwrite(&chunks, 8, 1, f);
    fwrite(&threads, 8, 1, f);
    fwrite(&seed, 8, 1, f);
    fwrite(&nstates, 8, 1, f);
    fwrite(kept, 8, KEY_N, f);
    fwrite(sold, 8, KEY_N, f);
    if (nstates) fwrite(states, 8, nstates, f);
    fclose(f);
#ifdef _WIN32
    remove(path);
#endif
    if (rename(tmp, path) != 0) {
        remove(tmp);
        return -1;
    }
    return 0;
}

// strict loader. tag/eitem/shop/count/chunks/threads all have to match
// or we return 1 and the caller starts fresh. that is the safe choice,
// a half compatible resume would silently scramble the streams.
static int ckpt_load(const char *path, uint32_t tag, int eitem, int shop, uint64_t target,
                     uint64_t *done_chunks_out, uint64_t *chunks_out, uint64_t *threads_out,
                     uint64_t *kept, uint64_t *sold,
                     uint64_t *states, uint64_t states_cap, uint64_t *nstates_out,
                     uint64_t *seed_out = nullptr) {
    FILE *f = fopen(path, "rb");
    if (!f) return 1;
    char magic[8];
    uint32_t tag_f = 0;
    int32_t ei = 0, sh = 0;
    uint64_t tt = 0, dc = 0, ch = 0, th = 0, ns = 0, sd = SEED_BASE;
    if (fread(magic, 1, 8, f) != 8) { fclose(f); return 1; }

    bool is_v3 = (memcmp(magic, "PKCKPT03", 8) == 0);
    bool is_v2 = (memcmp(magic, "PKCKPT02", 8) == 0);
    if (!is_v3 && !is_v2) { fclose(f); return 1; }

    bool ok = fread(&tag_f, 4, 1, f) == 1 && fread(&ei, 4, 1, f) == 1 && fread(&sh, 4, 1, f) == 1
        && fread(&tt, 8, 1, f) == 1 && fread(&dc, 8, 1, f) == 1 && fread(&ch, 8, 1, f) == 1
        && fread(&th, 8, 1, f) == 1;

    if (is_v3) {
        ok = ok && (fread(&sd, 8, 1, f) == 1);
    }
    ok = ok && (fread(&ns, 8, 1, f) == 1);
    ok = ok && (tag_f == tag && ei == eitem && sh == shop && tt == target && dc < ch && ns == th && th <= states_cap);
    ok = ok && (fread(kept, 8, KEY_N, f) == (size_t)KEY_N);
    ok = ok && (fread(sold, 8, KEY_N, f) == (size_t)KEY_N);
    ok = ok && (ns == 0 || fread(states, 8, ns, f) == ns);
    fclose(f);
    if (!ok) return 1;
    *done_chunks_out = dc;
    *chunks_out = ch;
    *threads_out = th;
    *nstates_out = ns;
    if (seed_out) *seed_out = sd;
    return 0;
}
