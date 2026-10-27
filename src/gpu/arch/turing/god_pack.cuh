#pragma once

#include "src/gpu/core/types.cuh"
#include "src/gpu/core/tables.cuh"
#include "src/gpu/arch/turing/layouts.cuh"

namespace arch::turing {

template <int EXP_ID, bool HAS_GHOST, bool ALL_COMMON, typename EmitDirectFn, typename EmitSlotFn>
__device__ __forceinline__ static void open_god_pack_direct(
    int god, Rng &rng, int shop,
    unsigned char *pos, unsigned char *bor, unsigned char *dst, unsigned char *foil,
    unsigned char *spos, unsigned char *sbor, unsigned char *sdst, unsigned char *sfoil,
    uint32_t &pend, bool &havep,
    EmitDirectFn &&emit_direct, EmitSlotFn &&emit_slot) {

    const ExpT *exptab = &EXPS[EXP_ID];
    const ExpT *ge = &EXPS[2];
    int nall = exptab->n;
    int gnall = ge->n;
    int gfoil = (god == 12) ? 1 : 0;

    if (god == 11 || god == 12) {
        // God 11/12: replace ALL 7 cards with ghost cards
        for (int s = 0; s < 7; s++) {
            int p = (int)(rng.next() % gnall);
            int d = ((rng.next() % 100ULL) < 50) ? 1 : 0;
            emit_slot(make_key(2, p * 2, d, gfoil));
        }
        return;
    }

    // Gods 1..10, 13
    int p_bor = GOD_INFO[god].border;
    int p_foil = GOD_INFO[god].foil;
    uint64_t step = GOD_INFO[god].step;

    for (int s = 0; s < 7; s++) {
        int p;
        if constexpr (ALL_COMMON) {
            p = (int)c_comm[(int)(rng.next() % 27)];
        } else {
            p = (int)(rng.next() % nall);
        }
        rng.s += step;
        int d = (EXP_ID == 1) ? 1 : 0;
        pos[s] = (unsigned char)p;
        bor[s] = (unsigned char)p_bor;
        dst[s] = (unsigned char)d;
        foil[s] = (unsigned char)p_foil;
    }

    if constexpr (HAS_GHOST) {
        int gate = (int)rng.below(EXP_ID == 0 ? 20000 : 10000);
        if (gate < 20) {
            int gp = (int)(rng.next() % gnall);
            int gd = ((rng.next() % 100ULL) < 50) ? 1 : 0;
            int gf = (((int)rng.below(10000)) < 500) ? 1 : 0;
            spos[0] = (unsigned char)gp;
            sbor[0] = 0;
            sdst[0] = (unsigned char)gd;
            sfoil[0] = (unsigned char)gf;
            emit_slot(make_key(2, gp * 2, gd, gf));
            for (int s = 0; s < 6; s++) {
                emit_slot(make_key(EXP_ID, (int)pos[s] * 12 + (int)bor[s], (int)dst[s], (int)foil[s]));
            }
            return;
        }
    }

    for (int s = 0; s < 7; s++) {
        emit_slot(make_key(EXP_ID, (int)pos[s] * 12 + (int)bor[s], (int)dst[s], (int)foil[s]));
    }
}

} // namespace arch::turing
