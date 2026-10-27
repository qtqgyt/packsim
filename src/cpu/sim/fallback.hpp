#pragma once

#include "src/cpu/sim/god.hpp"

template<int EXP_ID, bool GOD_IS_ZERO>
static PACK_ALWAYS_INLINE int roll_content_t(
    Rng &rng, const ExpT *e, const Params &prm, const Thr &th,
    int *o_pos, int *o_bor, int *o_dst, int *o_foil,
    int *tmp_leg, int *tmp_epic, int *tmp_rare, int *tmp_comm, int *tmp_all,
    Prebuilt *pre, unsigned char *ulo, int god = 0) {
    (void)tmp_all;

    // --- Specialized Path: Exp 2 (Ghost) ---
    if constexpr (EXP_ID == 2) {
        constexpr uint64_t const_n = (uint64_t)ExpTraits<EXP_ID>::const_n;
        #pragma GCC unroll 7
        for (int s = 0; s < 7; s++) {
            if (s >= prm.legendaries) {
                rng.s += 0x9E3779B97F4A7C15ULL;
            }
            int pos = (int)(rng.next() % const_n);
            int foil;
            if constexpr (GOD_IS_ZERO) {
                foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
                roll_border_god0(rng);
            } else {
                foil = GOD_INFO[god].foil;
                rng.s += GOD_INFO[god].step;
            }
            int border = 0;
            int dst_val = ((rng.next() % 100ULL) < 50) ? 1 : 0;
            o_pos[s] = pos;
            o_bor[s] = border;
            o_dst[s] = dst_val;
            o_foil[s] = foil;
        }
        return 7;
    // --- Specialized Path: Exp 3..7 (No-Rarity Uniform Draws) ---
    } else if constexpr (EXP_ID >= 3 && EXP_ID <= 7) {
        constexpr uint64_t const_n = (uint64_t)ExpTraits<EXP_ID>::const_n;
        #pragma GCC unroll 7
        for (int s = 0; s < 7; s++) {
            if (s >= prm.commons) {
                rng.s += 0x9E3779B97F4A7C15ULL;
            }
            int pos = (int)(rng.next() % const_n);
            int foil;
            int border;
            if constexpr (GOD_IS_ZERO) {
                foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
                border = roll_border_god0(rng);
            } else {
                foil = GOD_INFO[god].foil;
                border = GOD_INFO[god].border;
                rng.s += GOD_INFO[god].step;
            }
            o_pos[s] = pos;
            o_bor[s] = border;
            o_dst[s] = 0;
            o_foil[s] = foil;
        }
        return 7;
    }

    // --- General Fallback Path ---
    constexpr bool HAVEDUP = ExpTraits<EXP_ID>::haveDup;
    int nleg = HAVEDUP ? pre->n[0] : e->n_leg;
    int nep  = HAVEDUP ? pre->n[1] : e->n_epic;
    int nra  = HAVEDUP ? pre->n[2] : e->n_rare;
    int ncm  = HAVEDUP ? pre->n[3] : e->n_common;
    int nall = HAVEDUP ? pre->n[4] : e->n;

    int *vleg = HAVEDUP ? pre->leg : tmp_leg;
    int *vepic = HAVEDUP ? pre->epic : tmp_epic;
    int *vrare = HAVEDUP ? pre->rare : tmp_rare;
    int *vcomm = HAVEDUP ? pre->comm : tmp_comm;

    constexpr int b_epic = ExpTraits<EXP_ID>::useRarity ? ExpTraits<EXP_ID>::const_n - 34 - 34 - 27 : 0;
    constexpr int b_rare = ExpTraits<EXP_ID>::useRarity ? ExpTraits<EXP_ID>::const_n - 34 - 27 : 0;
    constexpr int b_comm = ExpTraits<EXP_ID>::useRarity ? ExpTraits<EXP_ID>::const_n - 27 : 0;

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

        int pos = 0;
        if constexpr (EXP_ID == 2 || EXP_ID == 3 || EXP_ID == 4 || EXP_ID == 5 || EXP_ID == 6 || EXP_ID == 7) {
            constexpr uint64_t const_n = (uint64_t)ExpTraits<EXP_ID>::const_n;
            pos = (int)(rng.next() % const_n);
        } else if constexpr (EXP_ID == 0 || EXP_ID == 1 || EXP_ID == 8) {
            int *v; int *nv; int vb;
            if (rarity == 3) { v = vleg; nv = &nleg; vb = 0; }
            else if (rarity == 2) { v = vepic; nv = &nep; vb = b_epic; }
            else if (rarity == 1) { v = vrare; nv = &nra; vb = b_rare; }
            else { v = vcomm; nv = &ncm; vb = b_comm; }
            if (*nv == 0) continue;
            int idx = (int)fastmod_u64(rng.next(), (uint64_t)*nv);
            if constexpr (HAVEDUP) {
                pos = v[idx];
            } else {
                ulo[out] = (unsigned char)(vb + idx);
                int t = v[idx];
                v[idx] = v[*nv - 1]; // Swap-remove
                pos = t;
                (*nv)--;
            }
        }

        int foil;
        int border;
        if constexpr (GOD_IS_ZERO) {
            foil = (((int)rng.below(10000)) < 500) ? 1 : 0;
            border = roll_border_god0(rng);
        } else {
            foil = GOD_INFO[god].foil;
            border = GOD_INFO[god].border;
            rng.s += GOD_INFO[god].step;
        }
        if constexpr (EXP_ID == GHOST_EXP) border = 0;

        int dst_val;
        if constexpr (EXP_ID == 0) {
            dst_val = 0;
        } else if constexpr (EXP_ID == 1) {
            dst_val = 1;
        } else if constexpr (EXP_ID == GHOST_EXP) {
            dst_val = ((rng.next() % 100ULL) < 50) ? 1 : 0;
        } else {
            dst_val = 0;
        }

        o_pos[out] = pos;
        o_bor[out] = border;
        o_dst[out] = dst_val;
        o_foil[out] = foil;
        out++;
    }

    if constexpr (!HAVEDUP) {
        for (int j = out - 1; j >= 0; j--) tmp_leg[ulo[j]] = o_pos[j];
    }
    return out;
}

template<int EXP_ID>
static PACK_ALWAYS_INLINE int open_one_pack(
    Rng &rng, int shop,
    const Params *p2t, const Thr *tht,
    int *pos, int *bor, int *dst, int *foil,
    int *spos, int *sbor, int *sdst, int *sfoil,
    int *leg, int *epic, int *rare, int *comm, int *all,
    int *out_e, int *out_s, int *out_d, int *out_f,
    unsigned char *ulo) {

    const ExpT *exptab = &EXPS[EXP_ID];
    int god = roll_god(rng, shop);
    constexpr int per = ExpTraits<EXP_ID>::perMonster;

    if (PACK_LIKELY(god == 0)) {
        Params p2 = p2t[0];
        Thr th = tht[0];
        int n = roll_content_t<EXP_ID, true>(rng, exptab, p2, th, pos, bor, dst, foil, leg, epic, rare, comm, all, &GPRE[EXP_ID], ulo);

        if constexpr (EXP_ID == 0 || EXP_ID == 1 || EXP_ID == 7) {
            if (shop > 1) {
                int gate = (EXP_ID == 0) ? (int)rng.below(20000) : (int)rng.below(10000);
                if (PACK_UNLIKELY(gate < 20)) {
                    const ExpT *ge = &EXPS[GHOST_EXP];
                    int ns = roll_content_t<GHOST_EXP, true>(rng, ge, p2, th, spos, sbor, sdst, sfoil, leg, epic, rare, comm, all, &GPRE[GHOST_EXP], ulo);
                    if (ns > 0) {
                        constexpr int gper = ExpTraits<GHOST_EXP>::perMonster;
                        int keep = (n > 0) ? n - 1 : 0;
                        for (int i = 0; i < keep; i++) {
                            out_e[i] = EXP_ID;
                            out_s[i] = pos[i] * per + bor[i];
                            out_d[i] = dst[i];
                            out_f[i] = foil[i];
                        }
                        int si = (int)fastmod_u64(rng.next(), (uint64_t)ns);
                        out_e[keep] = GHOST_EXP;
                        out_s[keep] = spos[si] * gper + sbor[si];
                        out_d[keep] = sdst[si];
                        out_f[keep] = sfoil[si];
                        return keep + 1;
                    }
                }
            }
        }
        for (int i = 0; i < n; i++) {
            out_e[i] = EXP_ID;
            out_s[i] = pos[i] * per + bor[i];
            out_d[i] = dst[i];
            out_f[i] = foil[i];
        }
        return n;
    } else {
        Params p2 = p2t[god];
        Thr th = tht[god];
        int n = roll_content_t<EXP_ID, false>(rng, exptab, p2, th, pos, bor, dst, foil, leg, epic, rare, comm, all, &GPRE[EXP_ID], ulo, god);

        if constexpr (EXP_ID == 0 || EXP_ID == 1 || EXP_ID == 7) {
            if (shop > 1) {
                int gate = (EXP_ID == 0) ? (int)rng.below(20000) : (int)rng.below(10000);
                if (god == 11 || god == 12) gate = 0;
                if (gate < 20) {
                    const ExpT *ge = &EXPS[GHOST_EXP];
                    int ns = roll_content_t<GHOST_EXP, false>(rng, ge, p2, th, spos, sbor, sdst, sfoil, leg, epic, rare, comm, all, &GPRE[GHOST_EXP], ulo, god);
                    if (ns > 0) {
                        constexpr int gper = ExpTraits<GHOST_EXP>::perMonster;
                        if (god == 11 || god == 12) {
                            for (int i = 0; i < ns; i++) {
                                out_e[i] = GHOST_EXP;
                                out_s[i] = spos[i] * gper + sbor[i];
                                out_d[i] = sdst[i];
                                out_f[i] = sfoil[i];
                            }
                            return ns;
                        } else {
                            int keep = (n > 0) ? n - 1 : 0;
                            for (int i = 0; i < keep; i++) {
                                out_e[i] = EXP_ID;
                                out_s[i] = pos[i] * per + bor[i];
                                out_d[i] = dst[i];
                                out_f[i] = foil[i];
                            }
                            int si = (int)fastmod_u64(rng.next(), (uint64_t)ns);
                            out_e[keep] = GHOST_EXP;
                            out_s[keep] = spos[si] * gper + sbor[si];
                            out_d[keep] = sdst[si];
                            out_f[keep] = sfoil[si];
                            return keep + 1;
                        }
                    }
                }
            }
        }
        for (int i = 0; i < n; i++) {
            out_e[i] = EXP_ID;
            out_s[i] = pos[i] * per + bor[i];
            out_d[i] = dst[i];
            out_f[i] = foil[i];
        }
        return n;
    }
}
