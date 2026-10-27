#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/math.cuh"
#include "src/gpu/core/rng.cuh"
#include "src/gpu/core/tables.cuh"

namespace arch::generic {

__device__ static void build_lists(const ExpT *e, unsigned char *tmp) {
    const int *__restrict__ rar = e->rar;
    int nleg = e->n_leg, nep = e->n_epic, nra = e->n_rare;
    unsigned char *vleg = tmp;
    unsigned char *vepic = tmp + nleg;
    unsigned char *vrare = tmp + nleg + nep;
    unsigned char *vcomm = tmp + nleg + nep + nra;
    unsigned char *vall = tmp;
    if (e->useRarity) {
        int wleg = 0, wepic = 0, wrare = 0, wcomm = 0;
        for (int i = 0; i < e->n; i++) {
            int r = rar[i];
            if (r == 3) vleg[wleg++] = i;
            else if (r == 2) vepic[wepic++] = i;
            else if (r == 1) vrare[wrare++] = i;
            else vcomm[wcomm++] = i;
        }
    } else {
        for (int i = 0; i < e->n; i++) vall[i] = i;
    }
}

// rolls up to 7 cards. skip_build lets the caller reuse the lists from
// kernel entry when haveDup means they never get mutated.
__device__ static int roll_content(Rng &rng, const ExpT *e, const Params &prm, const Thr &th, int exp_id,
                                   unsigned char *o_pos, unsigned char *o_bor, unsigned char *o_dst, unsigned char *o_foil,
                                   unsigned char *tmp, bool skip_build) {
    if (!skip_build) build_lists(e, tmp);
    int nleg = e->n_leg, nep = e->n_epic, nra = e->n_rare, ncm = e->n_common, nall = e->n;
    unsigned char *vleg = tmp;
    unsigned char *vepic = tmp + nleg;
    unsigned char *vrare = tmp + nleg + nep;
    unsigned char *vcomm = tmp + nleg + nep + nra;
    unsigned char *vall = tmp;
    // c2..c5 count how many of each rarity we already placed this pack.
    // guaranteed slots burn first, cascade only fires when those run out.
    int c2 = 0, c3 = 0, c4 = 0, c5 = 0;
    int out = 0;
    for (int s = 0; s < 7; s++) {
        if (nall == 0) break;
        int rarity;
        if (prm.legendaries - c5 > 0 && nleg > 0) { rarity = 3; c5++; }
        else if (prm.epics - c4 > 0 && nep > 0) { rarity = 2; c4++; }
        else if (prm.rares - c3 > 0 && nra > 0) { rarity = 1; c3++; }
        else if (prm.commons - c2 > 0 && ncm > 0) { rarity = 0; c2++; }
        else {
            // one shared r21 for the whole cascade. order matters here,
            // leg gets first crack and the 4 - cX caps keep it to 4 total.
            int r21 = (int)rng.below(10000);
            bool done = false;
            rarity = 0;
            if (!done && prm.leg_casc > 0.0f && nleg > 0 && 4 - c5 > 0) {
                if (r21 < th.leg_casc) { done = true; rarity = 3; c5++; }
            }
            if (!done && prm.epic_casc > 0.0f && nep > 0 && 4 - c4 > 0) {
                if (r21 < th.epic_casc) { done = true; rarity = 2; c4++; }
            }
            if (!done && prm.rare_casc > 0.0f && nra > 0 && 4 - c3 > 0) {
                if (r21 < th.rare_casc) { done = true; rarity = 1; c3++; }
            }
            if (!done) { rarity = 0; c2++; }
        }
        int pos;
        if (e->useRarity) {
            unsigned char *v; int *nv;
            if (rarity == 3) { v = vleg; nv = &nleg; }
            else if (rarity == 2) { v = vepic; nv = &nep; }
            else if (rarity == 1) { v = vrare; nv = &nra; }
            else { v = vcomm; nv = &ncm; }
            if (*nv == 0) continue;
            int idx = (int)rng.below((uint64_t)*nv);
            if (e->haveDup) pos = v[idx];
            // swap with last and shrink. each swap is its own inverse,
            // which is why an undo log could rebuild this exactly.
            else { int last = *nv - 1; int t = v[idx]; v[idx] = v[last]; v[last] = t; pos = t; (*nv)--; }
        } else {
            if (nall == 0) continue;
            int idx = (int)rng.below((uint64_t)nall);
            if (e->haveDup) pos = vall[idx];
            else { int last = nall - 1; int t = vall[idx]; vall[idx] = vall[last]; vall[last] = t; pos = t; nall--; }
        }
        int foil = (((int)rng.below(10000)) < th.foil) ? 1 : 0;
        // border chain runs high to low so full art wins ties.
        // short circuit means draw count varies per card, and that
        // variance is part of the stream contract. do not flatten it.
        int border = 0;
        bool bdone = false;
        if (((int)rng.below(10000)) < th.b_full_art) { border = 5; bdone = true; }
        if (!bdone && ((int)rng.below(10000)) < th.b_ex) { border = 4; bdone = true; }
        if (!bdone && ((int)rng.below(10000)) < th.b_gold) { border = 3; bdone = true; }
        if (!bdone && ((int)rng.below(10000)) < th.b_silver) { border = 2; bdone = true; }
        if (!bdone && ((int)rng.below(10000)) < th.b_first_ed) { border = 1; bdone = true; }
        // ghost cards never keep a border, even if one rolled.
        if (!bdone || exp_id == GHOST_EXP) border = 0;
        o_pos[out] = pos;
        o_bor[out] = border;
        o_dst[out] = destiny_of(exp_id, rng);
        o_foil[out] = foil;
        out++;
    }
    return out;
}
// one pack plus the ghost kicker. god is rolled first so it shifts the
// stream for everything after, even when it ends up doing nothing.
__device__ static int open_one_pack(Rng &rng, const ExpT *exptab, const ExpT *ge, int exp_id, int collection, int shop,
                                    const Params *p2t, const Thr *tht,
                                    unsigned char *pos, unsigned char *bor, unsigned char *dst, unsigned char *foil,
                                    unsigned char *spos, unsigned char *sbor, unsigned char *sdst, unsigned char *sfoil,
                                    unsigned char *tmp, unsigned char *stmp, bool preMain, bool preGhost,
                                    unsigned char *out_e, unsigned short *out_s, unsigned char *out_d, unsigned char *out_f,
                                    int *remove_last) {
    int god = god_roll(rng, shop);
    // tables are prebuilt on host for this collection, one entry per god.
    // same ints as computing inline, just no float work per pack.
    Params p2 = p2t[god];
    Thr th = tht[god];
    int n = roll_content(rng, exptab, p2, th, exp_id, pos, bor, dst, foil, tmp, preMain);
    int per = exptab->perMonster;
    *remove_last = 0;
    // ghost only exists for these three exps and only past shop 1.
    // gate is out of 20000 for exp 0, out of 10000 for the other two.
    if ((exp_id == 0 || exp_id == 1 || exp_id == 7) && shop > 1) {
        int gate = (exp_id == 0) ? (int)rng.below(20000) : (int)rng.below(10000);
        if (god == 11 || god == 12) gate = 0;
        if (gate < 20) {
            int ns = roll_content(rng, ge, p2, th, GHOST_EXP, spos, sbor, sdst, sfoil, stmp, preGhost);
            if (ns > 0) {
                int gper = ge->perMonster;
                // gods 11 and 12 throw the whole pack away and return ghosts only.
                if (god == 11 || god == 12) {
                    for (int i = 0; i < ns; i++) {
                        out_e[i] = GHOST_EXP;
                        out_s[i] = spos[i] * gper + sbor[i];
                        out_d[i] = sdst[i];
                        out_f[i] = sfoil[i];
                    }
                    return ns;
                } else {
                    // normal case keeps all but the last main card and
                    // swaps in one random ghost from the ghost roll.
                    int keep = (n > 0) ? n - 1 : 0;
                    for (int i = 0; i < keep; i++) {
                        out_e[i] = exp_id;
                        out_s[i] = pos[i] * per + bor[i];
                        out_d[i] = dst[i];
                        out_f[i] = foil[i];
                    }
                    int si = (int)rng.below((uint64_t)ns);
                    out_e[keep] = GHOST_EXP;
                    out_s[keep] = spos[si] * gper + sbor[si];
                    out_d[keep] = sdst[si];
                    out_f[keep] = sfoil[si];
                    *remove_last = 0;
                    return keep + 1;
                }
            }
        }
    }
    for (int i = 0; i < n; i++) {
        out_e[i] = exp_id;
        out_s[i] = pos[i] * per + bor[i];
        out_d[i] = dst[i];
        out_f[i] = foil[i];
    }
    return n;
}
// one thread owns share[ti] packs and never touches another thread's.
// ti also picks the seed, so lane count is output identity. changing
// blocks changes results by design, that is what the goldens pin down.
__global__ void sim_kernel(int exp_id, int collection, int shop,
                           const Params *p2t, const Thr *tht,
                           const uint64_t *shares, uint64_t chunk, uint64_t chunks, int nthreads_actual,
                           uint32_t *kept, unsigned long long *sold,
                           uint64_t seed_base,
                           const uint64_t *init_state,
                           uint64_t *state_out) {
    int ti = blockIdx.x * blockDim.x + threadIdx.x;
    if (ti >= nthreads_actual) return;
    uint64_t total = shares[ti];
    // even split of this thread's packs across chunks, first chunks get the extra one.
    uint64_t n = total / chunks + (chunk < total % chunks ? 1 : 0);
    Rng rng(init_state ? init_state[ti] : (seed_base ^ ((uint64_t)(ti + 1) * 0x9E3779B97F4A7C15ULL)));
    int bi = blockIdx.x;
    // per block slabs, not per thread. 256 threads share one K and one S
    // so atomics stay inside the block. 3 MiB per block at KEY_N.
    uint32_t *K = kept + (size_t)bi * KEY_N;
    unsigned long long *S = sold + (size_t)bi * KEY_N;
    unsigned char pos[9], bor[9], dst[9], foil[9];
    unsigned char spos[9], sbor[9], sdst[9], sfoil[9];
    // 9 slots covers 7 cards plus ghost spillover. card ids, borders,
    // destinies and foils all fit in bytes, save ids need 16 bits.
    // 128 covers max n=121 with room to spare, but assert if tables
    // ever grow past it.
    unsigned char tmp[128], stmp[128];
    unsigned char oe[9], od[9], of[9];
    unsigned short os[9];
    int remove_last;
    const ExpT *exptab = nullptr;
    for (int i = 0; i < NEXPS; i++) if (EXPS[i].exp == exp_id) { exptab = &EXPS[i]; break; }
    const ExpT *ge = nullptr;
    for (int i = 0; i < NEXPS; i++) if (EXPS[i].exp == GHOST_EXP) { ge = &EXPS[i]; break; }
    // haveDup lists never mutate so build once here. noDup lists get
    // rebuilt per pack inside roll_content because swap-remove eats them.
    bool preMain = (exptab->haveDup != 0);
    bool preGhost = (ge->haveDup != 0);
    if (preMain) build_lists(exptab, tmp);
    if (preGhost) build_lists(ge, stmp);
    for (uint64_t p = 0; p < n; p++) {
        int nc = open_one_pack(rng, exptab, ge, exp_id, collection, shop,
                               p2t, tht,
                               pos, bor, dst, foil, spos, sbor, sdst, sfoil,
                               tmp, stmp, preMain, preGhost, oe, os, od, of, &remove_last);
        // warp aggregated atomics. lanes showing the same key elect the
        // lowest lane to issue one atomic for the group. totals match the
        // one atomic per card path exactly, there are just fewer of them.
        // sm_70 and up only, older cards keep the plain loop below.
#if defined(__CUDA_ARCH__) && __CUDA_ARCH__ >= 700
        int lane = threadIdx.x & 31;
        for (int i = 0; i < 9; i++) {
            unsigned active_m = __ballot_sync(0xFFFFFFFFu, i < nc);
            if (active_m == 0) continue;
            uint32_t key = (i < nc) ? make_key(oe[i], os[i], od[i], of[i]) : 0xFFFFFFFFu;
            unsigned same = __match_any_sync(active_m, key);
            int leader = __ffs(same) - 1;
            if (lane == leader && i < nc) {
                int cnt = __popc(same);
                uint32_t old = atomicAdd(&K[key], (uint32_t)cnt);
                uint64_t tot = (uint64_t)old + (uint64_t)(uint32_t)cnt;
                if (tot > (uint64_t)INTMAX32) {
                    uint32_t over = (uint32_t)(tot - (uint64_t)INTMAX32);
                    atomicSub(&K[key], over);
                    atomicAdd(&S[key], (unsigned long long)over);
                }
            }
        }
#else
        for (int i = 0; i < nc; i++) {
            uint32_t key = make_key(oe[i], os[i], od[i], of[i]);
            // kept clamps at int32 max. on overflow undo the add and spill
            // to the 64 bit side. order between threads does not matter,
            // the totals come out the same after the fold.
            uint32_t old = atomicAdd(&K[key], 1u);
            if (old >= (uint32_t)INTMAX32) {
                atomicSub(&K[key], 1u);
                atomicAdd(&S[key], 1ULL);
            }
        }
#endif
    }
    if (state_out) state_out[ti] = rng.s;
}
// one thread per key, sums that key across all block slabs.
// split into 32 and 64 bit versions because the slabs are different types.
__global__ void reduce_hist(const uint32_t *per, uint64_t *single, int nslabs) {
    int k = blockIdx.x * blockDim.x + threadIdx.x;
    if (k >= KEY_N) return;
    uint64_t s = 0;
    for (int t = 0; t < nslabs; t++) s += per[(size_t)t * KEY_N + k];
    single[k] = s;
}
__global__ void reduce_hist64(const unsigned long long *per, uint64_t *single, int nslabs) {
    int k = blockIdx.x * blockDim.x + threadIdx.x;
    if (k >= KEY_N) return;
    uint64_t s = 0;
    for (int t = 0; t < nslabs; t++) s += per[(size_t)t * KEY_N + k];
    single[k] = s;
}
// eitem 129 is special cased, everything else goes through PACKMAP.

} // namespace arch::generic
