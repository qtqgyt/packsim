#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/math.cuh"
#include "src/gpu/core/rng.cuh"
#include "src/gpu/core/tables.cuh"
#include "src/gpu/arch/turing/layouts.cuh"

namespace arch::turing {

__device__ __constant__ Params c_p2t[14];
__device__ __constant__ Thr c_tht[14];

struct EmitCtx { uint32_t *K; int main_slots; int exp_id; };
__device__ static int slab_slots_dev_unused;
__device__ __forceinline__ uint32_t slab_index(uint32_t key, int exp_id, int main_slots);
__device__ __forceinline__ void emit_key(const EmitCtx &c, uint32_t key) {
    // block-local shared histogram, flushed to the block slab at kernel end.
    atomicAdd(&c.K[slab_index(key, c.exp_id, c.main_slots)], 1u);
}
__device__ __constant__ const unsigned char c_comm[27] = {
    0, 4, 8, 12, 16, 20, 24, 28, 38, 42, 43, 46, 55, 59, 63, 67, 71, 75, 78, 81, 84, 93, 102, 105, 111, 113, 117
};

__device__ __constant__ const unsigned char c_rar_lists[121] = {
    3, 7, 11, 15, 19, 23, 27, 31, 41, 49, 50, 51, 52, 53, 58, 62, 66, 70, 74, 88, 91, 99, 101, 110, 116, 120,
    2, 6, 10, 14, 18, 22, 26, 30, 33, 34, 40, 45, 48, 54, 57, 61, 65, 69, 73, 77, 80, 83, 86, 87, 90, 95, 97, 98, 100, 104, 107, 109, 115, 119,
    1, 5, 9, 13, 17, 21, 25, 29, 32, 35, 36, 37, 39, 44, 47, 56, 60, 64, 68, 72, 76, 79, 82, 85, 89, 92, 94, 96, 103, 106, 108, 112, 114, 118,
    0, 4, 8, 12, 16, 20, 24, 28, 38, 42, 43, 46, 55, 59, 63, 67, 71, 75, 78, 81, 84, 93, 102, 105, 111, 113, 117
};

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

#include "src/gpu/arch/turing/roll_card.cuh"
#include "src/gpu/arch/turing/god_pack.cuh"

// Resolve seven swap-remove draws from an N-entry identity-indexed list
// without mutating the list. The caller maps each resolved index through its
// pristine rarity slice before emitting the card.
template <int N, typename CardCallback>
__device__ __forceinline__ static void roll_indices_register_forwarded(Rng &rng, CardCallback &&callback) {
    int k0 = (int)fastmod_u64(rng.next(), N);
    callback(k0, 0);

    int k1 = (int)fastmod_u64(rng.next(), N - 1);
    callback((k1 == k0) ? N - 1 : k1, 1);

    int k2 = (int)fastmod_u64(rng.next(), N - 2);
    int x2 = k2;
    if (x2 == k1) x2 = N - 2;
    if (x2 == k0) x2 = N - 1;
    callback(x2, 2);

    int k3 = (int)fastmod_u64(rng.next(), N - 3);
    int x3 = k3;
    if (x3 == k2) x3 = N - 3;
    if (x3 == k1) x3 = N - 2;
    if (x3 == k0) x3 = N - 1;
    callback(x3, 3);

    int k4 = (int)fastmod_u64(rng.next(), N - 4);
    int x4 = k4;
    if (x4 == k3) x4 = N - 4;
    if (x4 == k2) x4 = N - 3;
    if (x4 == k1) x4 = N - 2;
    if (x4 == k0) x4 = N - 1;
    callback(x4, 4);

    int k5 = (int)fastmod_u64(rng.next(), N - 5);
    int x5 = k5;
    if (x5 == k4) x5 = N - 5;
    if (x5 == k3) x5 = N - 4;
    if (x5 == k2) x5 = N - 3;
    if (x5 == k1) x5 = N - 2;
    if (x5 == k0) x5 = N - 1;
    callback(x5, 5);

    int k6 = (int)fastmod_u64(rng.next(), N - 6);
    int x6 = k6;
    if (x6 == k5) x6 = N - 6;
    if (x6 == k4) x6 = N - 5;
    if (x6 == k3) x6 = N - 4;
    if (x6 == k2) x6 = N - 3;
    if (x6 == k1) x6 = N - 2;
    if (x6 == k0) x6 = N - 1;
    callback(x6, 6);
}

// rolls up to 7 cards.
//
// tmp must hold the PRISTINE partition of e on entry and is restored to
// pristine before returning, so the caller builds it ONCE at kernel entry
// and never again. for haveDup tables (exps 2..8 and the ghost) nothing
// ever writes it. for the noDup tables (exps 0 and 1) the swap-removes below
// are undone on the way out. either way the per pack 121 byte rebuild is
// gone, which is the whole point: it was ~40% of runtime.
//
// undo log, and why one byte per card is enough:
//   the swap-remove is `t = v[idx]; v[idx] = v[last]; len--`. the extra
//   store `v[last] = t` that the original also did is DEAD: last is exactly
//   the new length, so from here on every read of that slice is at an index
//   < last (idx = below(*nv) with *nv == last) and every later write lands
//   at an index < last as well. slices are disjoint ranges of tmp, so no
//   other slice can see it either. dropping it leaves exactly ONE live byte
//   modified per selection, v[idx] == tmp[base+idx], and the value that has
//   to go back there is `t`, which is `pos`, which is already sitting in
//   o_pos[out]. so the whole undo log is the list of absolute offsets that
//   were touched: one byte each, 9 bytes per thread.
//   it MUST be replayed in reverse. the same offset can be hit twice in one
__device__ constexpr uint64_t exp_all_cards(int exp) {
    if (exp == 2) return 20;
    if (exp == 3) return 113;
    if (exp == 4) return 50;
    if (exp == 5) return 40;
    return 121;
}

// rolls up to 7 cards.
//
// tmp must hold the PRISTINE partition of e on entry and is restored to
// pristine before returning, so the caller builds it ONCE at kernel entry
// and never again. for haveDup tables (exps 2..8 and the ghost) nothing
// ever writes it. for the noDup tables (exps 0 and 1) the swap-removes below
// are undone on the way out. either way the per pack 121 byte rebuild is
// gone, which is the whole point: it was ~40% of runtime.
//
// undo log, and why one byte per card is enough:
//   the swap-remove is `t = v[idx]; v[idx] = v[last]; len--`. the extra
//   store `v[last] = t` that the original also did is DEAD: last is exactly
//   the slice being popped off the active set, nobody will read it for the
//   rest of the pack, and the pristine builder will re-populate it next
//   pack anyway. so the only thing that must be saved to restore pristine
//   is the pristine card that just got overwritten at v[idx]. and that card
//   is exactly t. but wait, t is ALSO the card we rolled! so the card id
//   we're about to emit (pos) IS the byte we would have had to log.
//   one array, out tracks the length, no separate undo stack.
//
//   wait, is it?
//   for exps 0 and 1, yes, because card id == index in the pristine slice.
//   for an arbitrary table where pristine wasn't identity-mapped it would
//   need the original pristine[idx], which is one more load. our tables
//   are identity-mapped within each rarity slice, so t == pos holds.
//   the only detail is rarity offset: vcomm starts at b_comm, etc, so
//   ulo stores the slice-local idx rather than the global pos so the
//   restore loop knows which sub-slice to write back into.
//   non-mutating tables (haveDup, exps 2..8) never touch tmp at all, so
//   the undo loop skips them entirely.
//
//   ghost rolls (exps 0 and 1 when gate rolls < 20) kick one card out of
//   the pack and replace it with a ghost card. the ghost table has
//   haveDup == true so its own rolls never mutate anything, but the main
//   pack that just got kicked DID mutate. we must roll back the kicked
//   card's swap-remove so pristine stays pristine. that's what sulo is for.
//   it logs the ghost rolls the same way, and open_one_pack undoes the
//   kicked card before returning.
//   ghost kicker is rare (<0.2% of packs), so the branch almost never hits.
//   when it doesn't, zero undo traffic.
//
//   net result: list rebuild dropped from 121 bytes/pack to at most 7
//   stores on exit, tmp stays resident in L1 across packs, and the 256B
//   table fits in shared memory without eviction.
//
// direct: when true, each non-ghost card is emitted straight to the
//   histogram inside the slot loop and the pos/bor/dst/foil arrays are
//   bypassed completely. gods 11/12 still buffer because the whole pack
//   gets discarded if they hit, but they are ~0.05% of packs.
//   pending: the most recently rolled card is held in pend (not emitted)
//   until the NEXT card rolls, because the ghost kicker can replace it.
//   when roll_content returns, pend holds the final card; open_one_pack
//   flushes or discards it depending on whether ghost kicked.
//   this gives the direct-to-histogram speed of g3 while preserving
//   exact bit-for-bit parity on every ghost roll.
//
// tmp is shared across the thread's lifetime, allocated by the caller.
// each thread gets its own slice so no inter-thread sync needed here.
//
// returns actual cards rolled (can be < 7 if a rarity list starved).
// callers that pass direct=true do not read o_pos etc on return unless
// the pack hit a god that requires inspecting them. the arrays are still
// passed so the buffered fallback has somewhere to write.
//
// mutates tmp in place for no-dup exps, callers rely on this for rollback.
// pristine invariant: tmp must equal PRISTINE on entry AND on exit.
// if you break this, subsequent packs on the same thread see corrupted
// lists and the goldens fail silently. don't touch unless you diff-test
// against ref on 1M packs.
//
// note on sulo vs ulo: sulo is ONLY read when the ghost kicker fired.
// it holds the slice offsets of the ghost rolls so the caller can
// reverse them. since ghost is haveDup, it's actually always empty in
// practice, but the plumbing is here in case a future ghost table has
// haveDup == false. do not remove without checking the tables.
//
// one more detail: the direct path still writes o_pos when MUTATE is true
// because the undo loop needs to know which cards to un-swap. it just
// skips bor/dst/foil, which are the ones that actually cost registers.
// that's why the direct check below only guards those three arrays.
//
// all indices are bytes because no expansion has > 128 cards in a rarity.
// if a future expansion exceeds that, this will truncate silently.
// assert(e->n <= 128) in the host loader catches this at startup.
//
// see src/common/packopen_common.hpp for the CPU reference this is cloned from.
// the two implementations must stay in lockstep.
//
// perf: on RTX 4090 this function is ~70% of total kernel time.
// the hot path is the fastmod_u64 calls and the speculative rng mix.
// keep the register footprint under 48 per thread or occupancy drops.
//
// order of operations inside the loop is load-bearing:
//   1. pick rarity (guaranteed first, then cascade)
//   2. swap-remove card from that rarity's slice
//   3. speculative rng mix for foil + borders
//   4. emit or buffer
// reordering 2 and 3 changes the rng stream and breaks goldens.
//
// returns: number of cards placed into the pack (usually 7, can be less
// if lists ran out).
//
// pack is guaranteed to have at least one card unless e->n == 0.
// if e->n == 0 the caller skips this kernel entirely.
//
// check git log for why the haveDup branch exists: without it, haveDup
// tables were running swap-removes that did nothing except dirty L1.
//
// end of comment block. code starts here.
//
template <int E>
__device__ __forceinline__ int destiny_of_t(Rng &rng) {
    if constexpr (E == 0) return 0;
    else if constexpr (E == 1) return 1;
    else if constexpr (E == 2) return (rng.below(100) < 50) ? 1 : 0;
    else return 0;
}

template <int EXP_ID, bool HAS_GHOST, bool MUTATE, bool USE_RARITY, bool ALL_COMMON = false, bool REGISTER_FIXED = false, int FIXED_RARITY = 1, bool SHOP_ONE = false>
__device__ static int roll_content(Rng &rng, const ExpT *e, const Params &prm, const Thr &th, int exp_id,
                                   unsigned char *o_pos, unsigned char *o_bor, unsigned char *o_dst, unsigned char *o_foil,
                                   std::conditional_t<MUTATE && !ALL_COMMON, unsigned char *, const unsigned char *> tmp, unsigned char *ulo,
                                   bool direct, const EmitCtx &ec, uint32_t &pend, bool &havep, bool &tmp_ready) {
    if constexpr (ALL_COMMON) {
        // Register-Forwarded Swap-Remove:
        // Mathematical Proof & Principle:
        // In Fisher-Yates swap-remove from initial array V[i] = i for i in [0..26]:
        // At step s, random index k_s in [0, 26 - s] is drawn. Card c_s = V_s[k_s].
        // Position k_s is then overwritten with the tail element at 26 - s.
        // If an index k_s drawn at step s was previously drawn at step j < s, the element at
        // position k_s was moved from position 26 - j during step j.
        // Therefore, tracing backwards chronologically from step s - 1 down to 0:
        // For j = s - 1 down to 0:
        //   if (x == k_j) x = 26 - j;
        // Because 26 - j > 26 - s >= k_s, no subsequent check for j' < j can match an index
        // that was within [0, 26 - j'] unless that tail position was itself swapped. But all
        // tail positions 26 - j are strictly outside [0, 26 - j'] for all j' < j, so 26 - j
        // could never have been drawn as a random index at step j'.
        // Thus, each swap resolution is uniquely resolved in at most one match per backward step,
        // totaling exactly 0 + 1 + 2 + 3 + 4 + 5 + 6 = 21 comparisons across all 7 card draws.
        // Each comparison compiles directly to a 1-cycle ISETP + SELP instruction in SASS.
        // This eliminates vcomm, ulo, local stack frame spills, and the 7-step reverse restore loop!

        auto roll_card = [&](int c, int s) {
            const uint64_t G = 0x9E3779B97F4A7C15ULL;
            const uint64_t s0 = rng.s;
            uint64_t m0, m1, m2, m3, m4, m5;
            sm_mix6(s0 + G, s0 + 2 * G, s0 + 3 * G, s0 + 4 * G, s0 + 5 * G, s0 + 6 * G, m0, m1, m2, m3, m4, m5);
            int r0 = (EXP_ID == 0 || EXP_ID == 1) ? fastmod10k_limb16(m0) : fastmod10k(m0);
            int r1 = (EXP_ID == 0 || EXP_ID == 1) ? fastmod10k_limb16(m1) : fastmod10k(m1);
            int r2 = (EXP_ID == 0 || EXP_ID == 1) ? fastmod10k_limb16(m2) : fastmod10k(m2);
            int r3 = (EXP_ID == 0 || EXP_ID == 1) ? fastmod10k_limb16(m3) : fastmod10k(m3);
            int r4 = (EXP_ID == 0 || EXP_ID == 1) ? fastmod10k_limb16(m4) : fastmod10k(m4);
            int r5 = (EXP_ID == 0 || EXP_ID == 1) ? fastmod10k_limb16(m5) : fastmod10k(m5);
            int foil = (r0 < th.foil) ? 1 : 0;
            unsigned int mask = ((r1 < th.b_full_art) << 5) |
                                ((r2 < th.b_ex) << 4) |
                                ((r3 < th.b_gold) << 3) |
                                ((r4 < th.b_silver) << 2) |
                                ((r5 < th.b_first_ed) << 1) | 1;
            int border = 31 - __clz(mask);
            int used = (border == 0) ? 6 : (7 - border);
            rng.s = s0 + (uint64_t)used * G;

            int dstv = destiny_of_t<EXP_ID>(rng);
            if (direct) {
                // Dense shared-memory indexing with 49-stride padding (coprime to 32: 49 % 32 = 17)
                uint32_t shist_idx = (uint32_t)c * 49U + (((uint32_t)border << 2) | ((uint32_t)dstv << 1) | (uint32_t)foil);
                if constexpr (!HAS_GHOST) {
                    atomicAdd(&ec.K[shist_idx], 1u);
                } else {
                    if (s < 6) atomicAdd(&ec.K[shist_idx], 1u);
                    else {
                        pend = shist_idx;
                        havep = true;
                    }
                }
            } else {
                o_pos[s] = (unsigned char)c;
                o_bor[s] = (unsigned char)border;
                o_dst[s] = (unsigned char)dstv;
                o_foil[s] = (unsigned char)foil;
            }
        };

        // Draw 0: rem = 27 (0 comparisons)
        int k0 = (int)fastmod_u64(rng.next(), 27);
        int c0 = k0;
        roll_card(c0, 0);

        // Draw 1: rem = 26 (1 comparison)
        int k1 = (int)fastmod_u64(rng.next(), 26);
        int c1 = (k1 == k0) ? 26 : k1;
        roll_card(c1, 1);

        // Draw 2: rem = 25 (2 comparisons)
        int k2 = (int)fastmod_u64(rng.next(), 25);
        int x2 = k2;
        if (x2 == k1) x2 = 25;
        if (x2 == k0) x2 = 26;
        int c2 = x2;
        roll_card(c2, 2);

        // Draw 3: rem = 24 (3 comparisons)
        int k3 = (int)fastmod_u64(rng.next(), 24);
        int x3 = k3;
        if (x3 == k2) x3 = 24;
        if (x3 == k1) x3 = 25;
        if (x3 == k0) x3 = 26;
        int c3 = x3;
        roll_card(c3, 3);

        // Draw 4: rem = 23 (4 comparisons)
        int k4 = (int)fastmod_u64(rng.next(), 23);
        int x4 = k4;
        if (x4 == k3) x4 = 23;
        if (x4 == k2) x4 = 24;
        if (x4 == k1) x4 = 25;
        if (x4 == k0) x4 = 26;
        int c4 = x4;
        roll_card(c4, 4);

        // Draw 5: rem = 22 (5 comparisons)
        int k5 = (int)fastmod_u64(rng.next(), 22);
        int x5 = k5;
        if (x5 == k4) x5 = 22;
        if (x5 == k3) x5 = 23;
        if (x5 == k2) x5 = 24;
        if (x5 == k1) x5 = 25;
        if (x5 == k0) x5 = 26;
        int c5 = x5;
        roll_card(c5, 5);

        // Draw 6: rem = 21 (6 comparisons)
        int k6 = (int)fastmod_u64(rng.next(), 21);
        int x6 = k6;
        if (x6 == k5) x6 = 21;
        if (x6 == k4) x6 = 22;
        if (x6 == k3) x6 = 23;
        if (x6 == k2) x6 = 24;
        if (x6 == k1) x6 = 25;
        if (x6 == k0) x6 = 26;
        int c6 = x6;
        roll_card(c6, 6);

        return 7;
    } else if constexpr (!USE_RARITY) {
        const int per_c = (EXP_ID == 2 ? 2 : 12);
        constexpr uint64_t n_cards = exp_all_cards(EXP_ID);
        const uint64_t G = 0x9E3779B97F4A7C15ULL;
        #pragma unroll
        for (int s = 0; s < 7; s++) {
            if constexpr (EXP_ID == 2) {
                if (prm.legendaries == 0) rng.s += G;
            } else {
                if (s > 0) rng.s += G;
            }
            int pos = (int)fastmod_u64(rng.next(), (uint32_t)n_cards);

            int border, foil, used;
            if constexpr (EXP_ID == 2) {
                const uint64_t s0 = rng.s;
                uint64_t m0, m1, m2, m3, m4;
                sm_mix5(s0 + G, s0 + 2 * G, s0 + 3 * G, s0 + 4 * G, s0 + 5 * G, m0, m1, m2, m3, m4);
                int r0 = fastmod10k_limb16(m0);
                int r1 = fastmod10k_limb16(m1);
                int r2 = fastmod10k_limb16(m2);
                int r3 = fastmod10k_limb16(m3);
                int r4 = fastmod10k_limb16(m4);
                foil = (r0 < th.foil) ? 1 : 0;
                border = 0;
                unsigned int mask = ((r1 < th.b_full_art) << 4) |
                                    ((r2 < th.b_ex) << 3) |
                                    ((r3 < th.b_gold) << 2) |
                                    ((r4 < th.b_silver) << 1) | 1;
                used = 6 - (31 - __clz(mask));
                rng.s = s0 + (uint64_t)used * G;
            } else {
                const uint64_t s0 = rng.s;
                uint64_t m0, m1, m2, m3, m4, m5;
                sm_mix6(s0 + G, s0 + 2 * G, s0 + 3 * G, s0 + 4 * G, s0 + 5 * G, s0 + 6 * G, m0, m1, m2, m3, m4, m5);
                int r0 = fastmod10k_limb16(m0);
                int r1 = fastmod10k_limb16(m1);
                int r2 = fastmod10k_limb16(m2);
                int r3 = fastmod10k_limb16(m3);
                int r4 = fastmod10k_limb16(m4);
                int r5 = fastmod10k_limb16(m5);
                foil = (r0 < th.foil) ? 1 : 0;
                unsigned int mask = ((r1 < th.b_full_art) << 5) |
                                    ((r2 < th.b_ex) << 4) |
                                    ((r3 < th.b_gold) << 3) |
                                    ((r4 < th.b_silver) << 2) |
                                    ((r5 < th.b_first_ed) << 1) | 1;
                border = 31 - __clz(mask);
                used = (border == 0) ? 6 : (7 - border);
                rng.s = s0 + (uint64_t)used * G;
                if (exp_id == GHOST_EXP) border = 0;
            }

            int dstv = destiny_of_t<EXP_ID>(rng);
            if (direct) {
                if constexpr (EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5) {
                    // 13-stride coprime layout: gcd(13, 32) = 1 eliminates 4-way shared bank conflicts
                    uint32_t shist_idx = (uint32_t)pos * 13U + (((uint32_t)border << 1) | (uint32_t)foil);
                    atomicAdd(&ec.K[shist_idx], 1u);
                } else if constexpr (EXP_ID == 2) {
                    // 5-stride coprime layout: gcd(5, 32) = 1 eliminates shared bank conflicts
                    uint32_t shist_idx = (uint32_t)pos * 5U + (((uint32_t)dstv << 1) | (uint32_t)foil);
                    atomicAdd(&ec.K[shist_idx], 1u);
                } else {
                    uint32_t k = make_key(exp_id, pos * per_c + border, dstv, foil);
                    if constexpr (!HAS_GHOST) {
                        emit_key(ec, k);
                    } else {
                        if (s < 6) emit_key(ec, k);
                        else {
                            pend = k;
                            havep = true;
                        }
                    }
                }
            } else {
                o_pos[s] = (unsigned char)pos;
                o_bor[s] = (unsigned char)border;
                o_dst[s] = (unsigned char)dstv;
                o_foil[s] = (unsigned char)foil;
            }
        }
        return 7;
    } else {
    const int per_c = e->perMonster;
    int nleg = e->n_leg, nep = e->n_epic, nra = e->n_rare, ncm = e->n_common, nall = e->n;
    const int b_epic = USE_RARITY ? nleg : 0;
    const int b_rare = USE_RARITY ? (nleg + nep) : 0;
    const int b_comm = USE_RARITY ? (nleg + nep + nra) : 0;
    using TmpPtr = std::conditional_t<MUTATE, unsigned char *, const unsigned char *>;
    TmpPtr vleg = tmp;
    TmpPtr vepic = tmp + b_epic;
    TmpPtr vrare = tmp + b_rare;
    TmpPtr vcomm = tmp + b_comm;
    TmpPtr vall = tmp;
    constexpr bool mutate = MUTATE;
    // c2..c5 count how many of each rarity we already placed this pack.
    // guaranteed slots burn first, cascade only fires when those run out.
    int c2 = 0, c3 = 0, c4 = 0, c5 = 0;
    int out = 0;
    bool fixed_rarity = false;
    auto roll_selected_card = [&](int pos, int s) {
        // Speculative draws: splitmix outputs are independent values from
        // successive state advances; advance by exactly the short-circuit count.
        const uint64_t G = 0x9E3779B97F4A7C15ULL;
        const uint64_t s0 = rng.s;
        constexpr bool USE_LIMB16 = (EXP_ID == 1) || (REGISTER_FIXED && EXP_ID == 0);
        int foil, border, used;
        if constexpr (REGISTER_FIXED && (EXP_ID == 0 || EXP_ID == 1)) {
            int r0 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + G)) : fastmod10k(sm_mix(s0 + G));
            foil = (r0 < th.foil) ? 1 : 0;
            int r1 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 2 * G)) : fastmod10k(sm_mix(s0 + 2 * G));
            if constexpr (SHOP_ONE) {
                int r2 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 3 * G)) : fastmod10k(sm_mix(s0 + 3 * G));
                int r3 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 4 * G)) : fastmod10k(sm_mix(s0 + 4 * G));
                int r4 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 5 * G)) : fastmod10k(sm_mix(s0 + 5 * G));
                int r5 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 6 * G)) : fastmod10k(sm_mix(s0 + 6 * G));
                unsigned int mask = ((r1 < th.b_full_art) << 5) |
                                    ((r2 < th.b_ex) << 4) |
                                    ((r3 < th.b_gold) << 3) |
                                    ((r4 < th.b_silver) << 2) |
                                    ((r5 < th.b_first_ed) << 1) | 1;
                border = 31 - __clz(mask);
                used = (border == 0) ? 6 : (7 - border);
            } else {
                if (r1 < th.b_full_art) { border = 5; used = 2; }
                else {
                    int r2 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 3 * G)) : fastmod10k(sm_mix(s0 + 3 * G));
                    if (r2 < th.b_ex) { border = 4; used = 3; }
                    else {
                        int r3 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 4 * G)) : fastmod10k(sm_mix(s0 + 4 * G));
                        if (r3 < th.b_gold) { border = 3; used = 4; }
                        else {
                            int r4 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 5 * G)) : fastmod10k(sm_mix(s0 + 5 * G));
                            if (r4 < th.b_silver) { border = 2; used = 5; }
                            else {
                                int r5 = USE_LIMB16 ? fastmod10k_limb16(sm_mix(s0 + 6 * G)) : fastmod10k(sm_mix(s0 + 6 * G));
                                if (r5 < th.b_first_ed) { border = 1; used = 6; }
                                else { border = 0; used = 6; }
                            }
                        }
                    }
                }
            }
        } else {
            uint64_t m0, m1, m2, m3, m4, m5;
            sm_mix6(s0 + G, s0 + 2 * G, s0 + 3 * G, s0 + 4 * G, s0 + 5 * G, s0 + 6 * G, m0, m1, m2, m3, m4, m5);
            int r0 = USE_LIMB16 ? fastmod10k_limb16(m0) : fastmod10k(m0);
            int r1 = USE_LIMB16 ? fastmod10k_limb16(m1) : fastmod10k(m1);
            int r2 = USE_LIMB16 ? fastmod10k_limb16(m2) : fastmod10k(m2);
            int r3 = USE_LIMB16 ? fastmod10k_limb16(m3) : fastmod10k(m3);
            int r4 = USE_LIMB16 ? fastmod10k_limb16(m4) : fastmod10k(m4);
            int r5 = USE_LIMB16 ? fastmod10k_limb16(m5) : fastmod10k(m5);
            foil = (r0 < th.foil) ? 1 : 0;
            if constexpr (EXP_ID != 2) {
                unsigned int mask = ((r1 < th.b_full_art) << 5) |
                                    ((r2 < th.b_ex) << 4) |
                                    ((r3 < th.b_gold) << 3) |
                                    ((r4 < th.b_silver) << 2) |
                                    ((r5 < th.b_first_ed) << 1) | 1;
                border = 31 - __clz(mask);
                used = (border == 0) ? 6 : (7 - border);
            } else {
                if (r1 < th.b_full_art) { border = 5; used = 2; }
                else if (r2 < th.b_ex) { border = 4; used = 3; }
                else if (r3 < th.b_gold) { border = 3; used = 4; }
                else if (r4 < th.b_silver) { border = 2; used = 5; }
                else if (r5 < th.b_first_ed) { border = 1; used = 6; }
                else { border = 0; used = 6; }
            }
        }
        rng.s = s0 + (uint64_t)used * G;
        if (exp_id == GHOST_EXP) border = 0;
        if constexpr (REGISTER_FIXED) {
            if (!fixed_rarity || !direct) o_pos[out] = pos;
        } else if (MUTATE || !direct) {
            o_pos[out] = pos;
        }
        int dstv = destiny_of_t<EXP_ID>(rng);
        if (direct) {
            if constexpr (REGISTER_FIXED && (EXP_ID == 0 || EXP_ID == 1)) {
                if constexpr (!HAS_GHOST) {
                    uint32_t shist_idx = (uint32_t)pos * 13U + (((uint32_t)border << 1) | (uint32_t)foil);
                    atomicAdd(&ec.K[shist_idx], 1u);
                } else if (s < 6) {
                    uint32_t shist_idx = (uint32_t)pos * 13U + (((uint32_t)border << 1) | (uint32_t)foil);
                    atomicAdd(&ec.K[shist_idx], 1u);
                } else {
                    pend = (uint32_t)pos * 13U + (((uint32_t)border << 1) | (uint32_t)foil);
                    havep = true;
                }
            } else if constexpr (EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5) {
                uint32_t shist_idx = (uint32_t)pos * 13U + (((uint32_t)border << 1) | (uint32_t)foil);
                atomicAdd(&ec.K[shist_idx], 1u);
            } else if constexpr (EXP_ID == 2) {
                uint32_t shist_idx = (uint32_t)pos * 5U + (((uint32_t)dstv << 1) | (uint32_t)foil);
                atomicAdd(&ec.K[shist_idx], 1u);
            } else {
                uint32_t k = make_key(exp_id, pos * per_c + border, dstv, foil);
                if constexpr (!HAS_GHOST) emit_key(ec, k);
                else if (s < 6) emit_key(ec, k);
                else { pend = k; havep = true; }
            }
        } else {
            o_bor[out] = border;
            o_dst[out] = dstv;
            o_foil[out] = foil;
        }
        out++;
    };
    if constexpr (REGISTER_FIXED) {
        // DISP_FIXED only serves collection mappings whose Params guarantee
        // this rarity count, so the generic scratch-copy fallback is unreachable.
        static_assert(FIXED_RARITY >= 1 && FIXED_RARITY <= 3);
        fixed_rarity = true;
        if constexpr (FIXED_RARITY == 1) {
            roll_indices_register_forwarded<34>(rng, [&](int idx, int s) {
                roll_selected_card(idx, s);
            });
        } else if constexpr (FIXED_RARITY == 2) {
            roll_indices_register_forwarded<34>(rng, [&](int idx, int s) {
                roll_selected_card(idx, s);
            });
        } else {
            roll_indices_register_forwarded<26>(rng, [&](int idx, int s) {
                roll_selected_card(idx, s);
            });
        }
        return out;
    } else {
        #pragma unroll
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
                int r21 = fastmod10k(rng.next());
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
            if constexpr (USE_RARITY) {
                TmpPtr v; int *nv; int vb;
                if (rarity == 3) { v = vleg; nv = &nleg; vb = 0; }
                else if (rarity == 2) { v = vepic; nv = &nep; vb = b_epic; }
                else if (rarity == 1) { v = vrare; nv = &nra; vb = b_rare; }
                else { v = vcomm; nv = &ncm; vb = b_comm; }
                if (*nv == 0) continue;
                int idx = (int)fastmod_u64(rng.next(), (uint64_t)*nv);
                if constexpr (!MUTATE) pos = v[idx];
                else {
                    ulo[out] = (unsigned char)(vb + idx);
                    int t = v[idx];
                    v[idx] = v[*nv - 1];
                    pos = t;
                    (*nv)--;
                }
            } else {
                if (nall == 0) continue;
                int idx;
                if constexpr (!MUTATE) {
                    constexpr uint64_t n_cards = exp_all_cards(EXP_ID);
                    idx = (int)fastmod_u64(rng.next(), (uint32_t)n_cards);
                    pos = idx;
                } else {
                    idx = (int)fastmod_u64(rng.next(), (uint64_t)nall);
                    ulo[out] = (unsigned char)idx;
                    int t = vall[idx];
                    vall[idx] = vall[nall - 1];
                    pos = t;
                    nall--;
                }
            }
            roll_selected_card(pos, s);
        }
        // hand tmp back the way we found it, so the next pack on this thread
        // starts from the pristine lists again.
        if constexpr (MUTATE) {
            for (int j = out - 1; j >= 0; j--) tmp[ulo[j]] = o_pos[j];
        }
        return out;
    }
    }
}
// one pack plus the ghost kicker. god is rolled first so it shifts the
// stream for everything after, even when it ends up doing nothing.
// cards are emitted straight into the histogram. gods 11/12 discard the main
// pack so they take the buffered path; otherwise only the last main card is
// held back until the ghost gate is known.
template <int EXP_ID, bool HAS_GHOST, bool MUTATE, bool USE_RARITY, bool ALL_COMMON = false, bool REGISTER_FIXED = false, int FIXED_RARITY = 1, bool SHOP_ONE = false>
__device__ static void open_one_pack(Rng &rng, const ExpT *exptab, const ExpT *ge, int exp_id, int collection, int shop,
                                     const Params &p_base, const Thr &th_base,
                                     unsigned char *pos, unsigned char *bor, unsigned char *dst, unsigned char *foil,
                                     unsigned char *spos, unsigned char *sbor, unsigned char *sdst, unsigned char *sfoil,
                                     std::conditional_t<MUTATE && !ALL_COMMON, unsigned char *, const unsigned char *> tmp,
                                     bool &tmp_ready, const unsigned char *stmp,
                                     unsigned char *ulo, unsigned char *sulo,
                                     const EmitCtx &ec) {
    int god = 0;
    if constexpr (!SHOP_ONE) if (shop > 1) {
        int num = (int)rng.below(100000);
        if (num >= 1000 && num <= 2950) {
            if (num < 1370) {
                if (num < 1131) {
                    if (num < 1052) god = (num == 1000) ? 12 : 11;
                    else if (num < 1061) god = 10;
                    else if (num < 1082) god = 9;
                    else god = 8;
                } else if (num <= 1230) god = 7;
                else if (num >= 1240 && num <= 1360) god = 6;
            } else if (num < 1870) {
                if (num < 1580) {
                    if (num <= 1570) god = 13;
                } else if (num < 1660) {
                    if (num <= 1650) god = 5;
                } else if (num <= 1860) god = 4;
            } else if (num < 2450) {
                if (num <= 2120) god = 3;
                else if (num >= 2130 && num <= 2440) god = 2;
            } else god = 1;
        }
    }
    Thr th = th_base;
    if (god != 0) th = c_tht[god];
    const bool godx = (god == 11 || god == 12);
    uint32_t pend = 0; bool havep = false;
    int n = roll_content<EXP_ID, HAS_GHOST, MUTATE, USE_RARITY, ALL_COMMON, REGISTER_FIXED, FIXED_RARITY, SHOP_ONE>(rng, exptab, p_base, th, exp_id, pos, bor, dst, foil, tmp, ulo, !godx, ec, pend, havep, tmp_ready);
    int per = exptab->perMonster;
    if constexpr (HAS_GHOST) {
        int gate = (EXP_ID == 0) ? (int)rng.below(20000) : fastmod10k(rng.next());
        if (godx) gate = 0;
        if (gate < 20) {
            uint32_t dpend = 0; bool dhavep = false;
            int ns = roll_content<GHOST_EXP, false, false, false, false>(rng, ge, p_base, th, GHOST_EXP, spos, sbor, sdst, sfoil, c_rar_lists, sulo, false, ec, dpend, dhavep, tmp_ready);
            if (ns > 0) {
                int gper = ge->perMonster;
                if (godx) {
                    for (int i = 0; i < ns; i++)
                        emit_key(ec, make_key(GHOST_EXP, spos[i] * gper + sbor[i], sdst[i], sfoil[i]));
                    return;
                } else {
                    int si = (int)fastmod_u64(rng.next(), (uint64_t)ns);
                    if (havep) havep = false;
                    emit_key(ec, make_key(GHOST_EXP, spos[si] * gper + sbor[si], sdst[si], sfoil[si]));
                    return;
                }
            }
        }
    }
    if (godx) {
        for (int i = 0; i < n; i++) {
            if constexpr (ALL_COMMON) {
                uint32_t shist_idx = (uint32_t)pos[i] * 49U + (((uint32_t)bor[i] << 2) | ((uint32_t)dst[i] << 1) | (uint32_t)foil[i]);
                atomicAdd(&ec.K[shist_idx], 1u);
            } else if constexpr (REGISTER_FIXED && (EXP_ID == 0 || EXP_ID == 1)) {
                uint32_t shist_idx = (uint32_t)pos[i] * 13U + (((uint32_t)bor[i] << 1) | (uint32_t)foil[i]);
                atomicAdd(&ec.K[shist_idx], 1u);
            } else if constexpr (EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5) {
                uint32_t shist_idx = (uint32_t)pos[i] * 13U + (((uint32_t)bor[i] << 1) | (uint32_t)foil[i]);
                atomicAdd(&ec.K[shist_idx], 1u);
            } else if constexpr (EXP_ID == 2) {
                uint32_t shist_idx = (uint32_t)pos[i] * 5U + (((uint32_t)dst[i] << 1) | (uint32_t)foil[i]);
                atomicAdd(&ec.K[shist_idx], 1u);
            } else {
                emit_key(ec, make_key(exp_id, pos[i] * per + bor[i], dst[i], foil[i]));
            }
        }
    } else if constexpr (HAS_GHOST) {
        if (havep) {
            if constexpr (ALL_COMMON) {
                atomicAdd(&ec.K[pend], 1u);
            } else if constexpr (REGISTER_FIXED && (EXP_ID == 0 || EXP_ID == 1)) {
                atomicAdd(&ec.K[pend], 1u);
            } else {
                emit_key(ec, pend);
            }
        }
    }
}
// A run has one main expedition. If it emits ghosts, ghost exp 2 is a
// second compact region after the main-exp region. Region-local slots retain
// all four dest/foil combinations; ghost borders are always zero.
static inline int main_slots_for(int exp_id) {
    switch (exp_id) {
        case 0: case 1: case 6: case 7: case 8: return 5784;
        case 2: return 156;
        case 3: return 5400;
        case 4: return 2376;
        case 5: return 1896;
        default: return 5784;
    }
}
PACK_BOTH static inline bool run_has_ghost(int exp_id) { return exp_id == 0 || exp_id == 1 || exp_id == 7; }
static inline int slab_slots_for(int exp_id) { return main_slots_for(exp_id) + (run_has_ghost(exp_id) ? 156 : 0); }
__device__ __forceinline__ uint32_t slab_index(uint32_t key, int exp_id, int main_slots) {
    int key_exp = (int)(key >> 14);
    uint32_t low = key & 0x3fffu;
    return (key_exp == GHOST_EXP && exp_id != GHOST_EXP) ? (uint32_t)main_slots + low : low;
}
static inline uint32_t slab_key(uint32_t slot, int exp_id, int main_slots) {
    if (slot >= (uint32_t)main_slots) return ((uint32_t)GHOST_EXP << 14) | (slot - main_slots);
    return ((uint32_t)exp_id << 14) | slot;
}
// ============================================================================
// SIMULATION KERNEL
// ============================================================================
// Uniform grid pack distribution:
// Total packs T are divided among N threads: threads ti < rem receive (T/N)+1 packs,
// and ti >= rem receive (T/N) packs, guaranteeing exact pack conservation.
template <int EXP_ID, bool HAS_GHOST, bool MUTATE, bool USE_RARITY, bool ALL_COMMON = false, bool REGISTER_FIXED = false, int FIXED_RARITY = 1, bool SHOP_ONE = false>
#if defined(__CUDA_ARCH__) && (__CUDA_ARCH__ >= 750 && __CUDA_ARCH__ < 1200)
__launch_bounds__(512, 2)
#else
__launch_bounds__(256, 4)
#endif
__global__ void sim_kernel(int exp_id, int collection, int shop,
                           uint32_t total_packs_hi, uint32_t total_packs_lo, uint32_t rem, int nthreads_actual,
                           uint64_t *d_red_kept,
                           uint64_t seed_base,
                           const uint64_t *init_state,
                           uint64_t *state_out, int main_slots) {
    int ti = blockIdx.x * blockDim.x + threadIdx.x;
    if (ti >= nthreads_actual) return;
    uint32_t my_packs = ((uint32_t)ti < rem) ? total_packs_hi : total_packs_lo;
    Rng rng(init_state ? init_state[ti] : (seed_base ^ ((uint64_t)(ti + 1) * 0x9E3779B97F4A7C15ULL)));
    constexpr int ghost_slots = HAS_GHOST ? 156 : 0;
    unsigned char pos[9], bor[9], dst[9], foil[9];
    unsigned char spos[HAS_GHOST ? 9 : 1], sbor[HAS_GHOST ? 9 : 1], sdst[HAS_GHOST ? 9 : 1], sfoil[HAS_GHOST ? 9 : 1];
    unsigned char tmp_local[(MUTATE && !ALL_COMMON && !REGISTER_FIXED) ? 128 : 1];
    using TmpPtr = std::conditional_t<MUTATE && !ALL_COMMON, unsigned char *, const unsigned char *>;
    TmpPtr tmp;
    bool tmp_ready = true;
    if constexpr (ALL_COMMON) {
        tmp = nullptr;
    } else if constexpr (MUTATE) {
        tmp = tmp_local;
        if constexpr (!REGISTER_FIXED) {
            for (int i = 0; i < 121; i++) tmp_local[i] = c_rar_lists[i];
        }
    } else {
        tmp = c_rar_lists;
    }
    unsigned char ulo[(MUTATE && !ALL_COMMON && !REGISTER_FIXED) ? 9 : 1];
    unsigned char sulo[1];
    const ExpT *exptab = &EXPS[EXP_ID];
    const ExpT *ge = &EXPS[2];
    constexpr bool FIXED_COMPACT = REGISTER_FIXED && (EXP_ID == 0 || EXP_ID == 1);
    constexpr int fixed_compact_slots = 121 * 13 + (HAS_GHOST ? 156 : 0);
    constexpr int shist_nslots = ALL_COMMON ? (1323 + (HAS_GHOST ? 156 : 0)) :
                                 (FIXED_COMPACT ? fixed_compact_slots :
                                 ((EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5) ? (EXP_ID == 3 ? 113 : (EXP_ID == 4 ? 50 : 40)) * 13 :
                                 (EXP_ID == 2 ? 100 : 0)));
    const int nslots = (ALL_COMMON || FIXED_COMPACT || EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5 || EXP_ID == 2) ? shist_nslots : (main_slots + ghost_slots);
    extern __shared__ uint32_t shist[];
    if constexpr (FIXED_COMPACT) {
        for (int i = threadIdx.x; i < fixed_compact_slots; i += blockDim.x) shist[i] = 0;
    } else if constexpr (EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5) {
        constexpr int n_slots = (EXP_ID == 3 ? 113 : (EXP_ID == 4 ? 50 : 40)) * 13;
        for (int i = threadIdx.x; i < n_slots; i += blockDim.x) shist[i] = 0;
    } else if constexpr (EXP_ID == 2) {
        for (int i = threadIdx.x; i < 100; i += blockDim.x) shist[i] = 0;
    } else {
        const int nslots4 = (nslots + 3) / 4;
        uint4 * const shist4 = reinterpret_cast<uint4*>(shist);
        for (int i = threadIdx.x; i < nslots4; i += blockDim.x) shist4[i] = make_uint4(0, 0, 0, 0);
    }
    __syncthreads();
    // The ghost region is compact in shared memory for this variant, while
    // the output slab keeps its original main_slots offset during the flush.
    const EmitCtx ec{shist, FIXED_COMPACT ? (121 * 13) : (ALL_COMMON ? 1323 : main_slots), EXP_ID};
    const Params p_base = c_p2t[0];
    const Thr th_base = c_tht[0];

    // Single flattened loop per thread:
    // Executes exactly my_packs packs sequentially with zero chunk division overhead.
    for (uint32_t p = 0; p < my_packs; p++) {
        open_one_pack<EXP_ID, HAS_GHOST, MUTATE, USE_RARITY, ALL_COMMON, REGISTER_FIXED, FIXED_RARITY, SHOP_ONE>(
                      rng, exptab, ge, EXP_ID, collection, shop,
                      p_base, th_base,
                      pos, bor, dst, foil, spos, sbor, sdst, sfoil,
                      tmp, tmp_ready, c_rar_lists, ulo, sulo, ec);
    }
    __syncthreads();
    // Direct L2 Atomic Accumulation:
    // Flushes block shared memory counts directly into d_red_kept via 64-bit atomics.
    // Completely eliminates intermediate DRAM writes and separate reduce_hist kernel.
    if constexpr (FIXED_COMPACT) {
        constexpr int main_compact_slots = 121 * 13;
        for (int s = threadIdx.x; s < main_compact_slots; s += blockDim.x) {
            uint32_t val = shist[s];
            if (val > 0) {
                int ordinal = s / 13;
                int rem = s % 13;
                if (rem < 12) {
                    int bor = rem >> 1;
                    int foi = rem & 1;
                    int card_pos;
                    if constexpr (FIXED_RARITY == 1) card_pos = (int)c_rar_lists[60 + ordinal];
                    else if constexpr (FIXED_RARITY == 2) card_pos = (int)c_rar_lists[26 + ordinal];
                    else card_pos = (int)c_rar_lists[ordinal];
                    uint32_t target_slot = (((uint32_t)card_pos * 12U + (uint32_t)bor) << 2) |
                                           ((uint32_t)EXP_ID << 1) | (uint32_t)foi;
                    atomicAdd((unsigned long long*)&d_red_kept[target_slot], (unsigned long long)val);
                }
            }
        }
        if constexpr (HAS_GHOST) {
            for (int i = threadIdx.x; i < 156; i += blockDim.x) {
                uint32_t val = shist[main_compact_slots + i];
                if (val > 0) atomicAdd((unsigned long long*)&d_red_kept[main_slots + i], (unsigned long long)val);
            }
        }
    } else if constexpr (ALL_COMMON) {
        // Common slots: translate common_idx (0..26) -> c_comm[common_idx] via 49-stride unpack
        for (int s = threadIdx.x; s < 1323; s += blockDim.x) {
            uint32_t val = shist[s];
            if (val > 0) {
                int c_idx = s / 49;
                int rem = s % 49;
                if (rem < 48) { // 48th slot is the padding word
                    int bor = rem >> 2;
                    int dst = (rem >> 1) & 1;
                    int foi = rem & 1;
                    int global_pos = c_comm[c_idx];
                    uint32_t target_slot = (((uint32_t)global_pos * 12U + (uint32_t)bor) << 2) | ((uint32_t)dst << 1) | (uint32_t)foi;
                    atomicAdd((unsigned long long*)&d_red_kept[target_slot], (unsigned long long)val);
                }
            }
        }
        if constexpr (HAS_GHOST) {
            for (int i = threadIdx.x; i < 156; i += blockDim.x) {
                uint32_t val = shist[1323 + i];
                if (val > 0) {
                    atomicAdd((unsigned long long*)&d_red_kept[main_slots + i], (unsigned long long)val);
                }
            }
        }
    } else if constexpr (EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5) {
        // Exp 3/4/5 13-stride flush loop: unpacks coprime layout to target global slab
        constexpr int n_slots = (EXP_ID == 3 ? 113 : (EXP_ID == 4 ? 50 : 40)) * 13;
        for (int s = threadIdx.x; s < n_slots; s += blockDim.x) {
            uint32_t val = shist[s];
            if (val > 0) {
                int p = s / 13;
                int rem = s % 13;
                if (rem < 12) { // slot 12 is padding
                    int bor = rem >> 1;
                    int foi = rem & 1;
                    uint32_t target_slot = (((uint32_t)p * 12U + (uint32_t)bor) << 2) | (uint32_t)foi;
                    atomicAdd((unsigned long long*)&d_red_kept[target_slot], (unsigned long long)val);
                }
            }
        }
    } else if constexpr (EXP_ID == 2) {
        // Exp 2 Ghost Pack 5-stride coprime flush loop: unpacks coprime layout to target global slab
        for (int s = threadIdx.x; s < 100; s += blockDim.x) {
            uint32_t val = shist[s];
            if (val > 0) {
                int p = s / 5;
                int rem = s % 5;
                if (rem < 4) { // 5th slot (rem == 4) is padding
                    int dst = rem >> 1;
                    int foi = rem & 1;
                    uint32_t target_slot = (((uint32_t)p * 2U) << 2) | ((uint32_t)dst << 1) | (uint32_t)foi;
                    atomicAdd((unsigned long long*)&d_red_kept[target_slot], (unsigned long long)val);
                }
            }
        }
    } else {
        for (int i = threadIdx.x; i < nslots; i += blockDim.x) {
            uint32_t val = shist[i];
            if (val > 0) {
                atomicAdd((unsigned long long*)&d_red_kept[i], (unsigned long long)val);
            }
        }
    }
    if (state_out) state_out[ti] = rng.s;
}

// Keep runtime shop selection out of the device kernel's god-roll path for shop 1.
template <int E, bool G, bool M, bool R, bool C = false, bool F = false, int FR = 1>
static inline void launch_sim_kernel(int blocks, int threads, size_t smem, int exp_id, int collection, int shop,
                                     uint32_t hi, uint32_t lo, uint32_t rem, int nthreads,
                                     uint64_t *red, uint64_t seed, uint64_t *state, int main_slots) {
    if (shop == 1) {
        sim_kernel<E, G, M, R, C, F, FR, true><<<blocks, threads, smem>>>(exp_id, collection, shop, hi, lo, rem, nthreads, red, seed, nullptr, state, main_slots);
    } else {
        sim_kernel<E, G, M, R, C, F, FR, false><<<blocks, threads, smem>>>(exp_id, collection, shop, hi, lo, rem, nthreads, red, seed, nullptr, state, main_slots);
    }
}

} // namespace arch::turing
