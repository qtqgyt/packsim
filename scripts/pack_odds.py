#!/usr/bin/env python3
import json
import argparse
import sys
import math
import os

def load_tables():
    # If explicitly passed via env
    if "PACKOPENER_TABLES" in os.environ:
        path = os.environ["PACKOPENER_TABLES"]
        if not os.path.exists(path):
            print(f"Error: {path} not found.")
            sys.exit(1)
        with open(path) as f:
            return json.load(f)

    # Try a few common paths
    paths = [
        "packopener_tables.json",
        "../packopener_tables.json",
        os.path.expanduser("~/.local/share/Steam/steamapps/common/TCG Card Shop Simulator/BepInEx/config/packopener_tables.json")
    ]
    for p in paths:
        if os.path.exists(p):
            with open(p) as f:
                return json.load(f)

    print("Error: packopener_tables.json not found. Please set PACKOPENER_TABLES env var or run in the correct directory.")
    sys.exit(1)

def params_for(collection):
    p = {
        "commons": 1, "rares": 0, "epics": 0, "legendaries": 0,
        "rare_casc": 10.0, "epic_casc": 2.0, "leg_casc": 0.1,
        "foil": 5.0, "b_first_ed": 20.0, "b_silver": 8.0, "b_gold": 4.0, "b_ex": 1.0, "b_full_art": 0.25
    }

    if collection in (1, 5):
        p["commons"] = 0
        p["rares"] = 7
        p["rare_casc"] += 45.0
        p["epic_casc"] += 2.0
        p["leg_casc"] += 1.0
    elif collection in (2, 6):
        p["commons"] = 0
        p["rares"] = 1
        p["epics"] = 7
        p["legendaries"] = 0
        p["rare_casc"] += 65.0
        p["epic_casc"] += 45.0
        p["leg_casc"] += 3.0
    elif collection in (3, 7):
        p["commons"] = 0
        p["rares"] = 0
        p["epics"] = 1
        p["legendaries"] = 7
        p["rare_casc"] += 65.0
        p["epic_casc"] += 55.0
        p["leg_casc"] += 35.0
    elif collection in (0, 4):
        p["commons"] = 7
    elif collection == 15:
        p["rare_casc"] = 25.0
        p["epic_casc"] = 25.0
        p["leg_casc"] = 25.0

    return p

# God roll logic from god_roll() in src/common/packopen_common.hpp
def get_god_odds():
    # 100,000 total possibilities
    odds = {
        12: 1 / 100000.0,    # 1000 -> 12 (1)
        11: 51 / 100000.0,   # 1001-1051 -> 11 (51)
        10: 9 / 100000.0,    # 1052-1060 -> 10 (9)
        9: 21 / 100000.0,    # 1061-1081 -> 9 (21)
        8: 49 / 100000.0,    # 1082-1130 -> 8 (49)
        7: 101 / 100000.0,   # 1130-1230 -> 7 (101) -- Note: overlaps on 1130. 1130 is 8, so 1131-1230 is 100 actually
                             # Looking closely at code:
                             # if (num >= 1082 && num <= 1130) return 8;
                             # if (num >= 1130 && num <= 1230) return 7;
                             # The first one triggers for 1130, so 1131-1230 -> 100.
        6: 121 / 100000.0,   # 1240-1360 -> 6 (121)
        13: 201 / 100000.0,  # 1370-1570 -> 13 (201)
        5: 71 / 100000.0,    # 1580-1650 -> 5 (71)
        4: 201 / 100000.0,   # 1660-1860 -> 4 (201)
        3: 251 / 100000.0,   # 1870-2120 -> 3 (251)
        2: 311 / 100000.0,   # 2130-2440 -> 2 (311)
        1: 501 / 100000.0    # 2450-2950 -> 1 (501)
    }
    odds[7] = 100 / 100000.0
    total_god_prob = sum(odds.values())
    return odds, total_god_prob

# Simulate card roll in a pack to find probabilities of each rarity per slot
def get_slot_probabilities(prm, has_comm, has_rare, has_epic, has_leg):
    # Returns a list of dicts with rarity probs per slot (7 slots)

    dp = {(0, 0, 0, 0): 1.0} # (c5, c4, c3, c2): prob

    for slot_idx in range(7):
        new_dp = {}

        for state, prob in dp.items():
            c5, c4, c3, c2 = state

            # Guaranteed logic:
            if prm["legendaries"] - c5 > 0 and has_leg:
                new_state = (c5+1, c4, c3, c2)
                new_dp[new_state] = new_dp.get(new_state, 0) + prob
            elif prm["epics"] - c4 > 0 and has_epic:
                new_state = (c5, c4+1, c3, c2)
                new_dp[new_state] = new_dp.get(new_state, 0) + prob
            elif prm["rares"] - c3 > 0 and has_rare:
                new_state = (c5, c4, c3+1, c2)
                new_dp[new_state] = new_dp.get(new_state, 0) + prob
            elif prm["commons"] - c2 > 0 and has_comm:
                new_state = (c5, c4, c3, c2+1)
                new_dp[new_state] = new_dp.get(new_state, 0) + prob
            else:
                # Random roll
                # thr does round_ties_even
                t_leg = int(round(prm["leg_casc"] * 100.0))
                t_epic = int(round(prm["epic_casc"] * 100.0))
                t_rare = int(round(prm["rare_casc"] * 100.0))

                p_leg = t_leg / 10000.0 if (has_leg and c5 < 4) else 0.0
                p_epic = t_epic / 10000.0 if (has_epic and c4 < 4) else 0.0
                p_rare = t_rare / 10000.0 if (has_rare and c3 < 4) else 0.0
                p_comm = 1.0 - p_leg - p_epic - p_rare

                if p_leg > 0:
                    new_state = (c5+1, c4, c3, c2)
                    new_dp[new_state] = new_dp.get(new_state, 0) + prob * p_leg
                if p_epic > 0:
                    new_state = (c5, c4+1, c3, c2)
                    new_dp[new_state] = new_dp.get(new_state, 0) + prob * p_epic
                if p_rare > 0:
                    new_state = (c5, c4, c3+1, c2)
                    new_dp[new_state] = new_dp.get(new_state, 0) + prob * p_rare
                if p_comm > 0:
                    new_state = (c5, c4, c3, c2+1)
                    new_dp[new_state] = new_dp.get(new_state, 0) + prob * p_comm

        dp = new_dp

    # Summarize expectations
    expected_c5 = sum(c5 * p for (c5, c4, c3, c2), p in dp.items())
    expected_c4 = sum(c4 * p for (c5, c4, c3, c2), p in dp.items())
    expected_c3 = sum(c3 * p for (c5, c4, c3, c2), p in dp.items())
    expected_c2 = sum(c2 * p for (c5, c4, c3, c2), p in dp.items())

    return {
        3: expected_c5,
        2: expected_c4,
        1: expected_c3,
        0: expected_c2
    }

def expected_packs_for_set(N, prob_per_pack):
    """
    Expected packs to collect N unique items, each with prob_per_pack
    being the expected number of such items per pack (assuming uniform distribution among the N).
    Using the classic coupon collector approximation: (N / expected_yield) * H_N.
    """
    if N == 0 or prob_per_pack <= 0:
        return float('inf')

    H_n = sum(1.0 / i for i in range(1, N + 1))
    return (N * H_n) / prob_per_pack

def get_ghost_kicker_odds(shop_level):
    if shop_level <= 1:
        return 0.0, 0.0
    return 20 / 20000.0, 20 / 10000.0


def main():
    parser = argparse.ArgumentParser(description="Calculate theoretical pack opening probabilities")
    parser.add_argument("--markdown", action="store_true", help="Output results as clean markdown tables")
    parser.add_argument("--shop-level", type=int, default=None, help="Override shop level from file")
    args = parser.parse_args()

    tables = load_tables()

    shop_level = args.shop_level if args.shop_level is not None else tables.get("shopLevel", 1)

    # Calculate god pack odds
    god_odds_dict, total_god_prob = get_god_odds()
    if shop_level <= 1:
        total_god_prob = 0.0
        god_odds_dict = {}

    expected_packs_per_god = 1.0 / total_god_prob if total_god_prob > 0 else float('inf')

    # Calculate ghost kicker odds
    ghost_base, ghost_destiny = get_ghost_kicker_odds(shop_level)

    if args.markdown:
        print(f"## Global Modifiers (Shop Level {shop_level})")
        if total_god_prob > 0:
            print(f"- **God Pack Odds**: {total_god_prob * 100:.4f}% (1 in {expected_packs_per_god:.1f} packs)")
        else:
            print("- **God Pack Odds**: Disabled (Shop Level <= 1)")

        if shop_level > 1:
            print(f"- **Ghost Card Kicker (Basic)**: {ghost_base * 100:.4f}% (1 in {1/ghost_base if ghost_base > 0 else float('inf'):.1f} packs)")
            print(f"- **Ghost Card Kicker (Destiny)**: {ghost_destiny * 100:.4f}% (1 in {1/ghost_destiny if ghost_destiny > 0 else float('inf'):.1f} packs)")
        else:
            print("- **Ghost Card Kicker**: Disabled (Shop Level <= 1)")
        print()

        print("## Expansion Odds")
    else:
        print(f"Global Modifiers (Shop Level {shop_level}):")
        if total_god_prob > 0:
            print(f"  God Pack Odds: {total_god_prob * 100:.4f}% (1 in {expected_packs_per_god:.1f} packs)")
        else:
            print("  God Pack Odds: Disabled (Shop Level <= 1)")

        if shop_level > 1:
            print(f"  Ghost Kicker (Basic): {ghost_base * 100:.4f}% (1 in {1/ghost_base if ghost_base > 0 else float('inf'):.1f} packs)")
            print(f"  Ghost Kicker (Destiny): {ghost_destiny * 100:.4f}% (1 in {1/ghost_destiny if ghost_destiny > 0 else float('inf'):.1f} packs)")
        else:
            print("  Ghost Kicker: Disabled (Shop Level <= 1)")
        print()

    # Collect stats for all packs
    for pack in sorted(tables["packs"], key=lambda p: p.get("eitem", 0)):
        eitem = pack.get("eitem")
        collection = pack.get("collection", -1)
        exp_id = pack.get("exp", -1)
        price = pack.get("price", 0.0)

        if collection < 0 or exp_id < 0:
            continue

        # Find expansion
        exp_data = next((e for e in tables["expansions"] if e["exp"] == exp_id), None)
        if not exp_data:
            continue

        rarities = exp_data.get("rarities", [])
        has_comm = 0 in rarities
        has_rare = 1 in rarities
        has_epic = 2 in rarities
        has_leg = 3 in rarities

        c_comm = rarities.count(0)
        c_rare = rarities.count(1)
        c_epic = rarities.count(2)
        c_leg = rarities.count(3)

        prm = params_for(collection)
        exp_yields = get_slot_probabilities(prm, has_comm, has_rare, has_epic, has_leg)

        # Ghost pack overrides?
        ghost_kicker_prob = 0.0
        if shop_level > 1 and exp_id in (0, 1, 7):
            ghost_kicker_prob = ghost_base if exp_id == 0 else ghost_destiny

        # Coupon collector calculation

        exp_comm_packs = expected_packs_for_set(c_comm, exp_yields[0]) if c_comm > 0 and exp_yields[0] > 0 else 0
        exp_rare_packs = expected_packs_for_set(c_rare, exp_yields[1]) if c_rare > 0 and exp_yields[1] > 0 else 0
        exp_epic_packs = expected_packs_for_set(c_epic, exp_yields[2]) if c_epic > 0 and exp_yields[2] > 0 else 0
        exp_leg_packs  = expected_packs_for_set(c_leg, exp_yields[3]) if c_leg > 0 and exp_yields[3] > 0 else 0

        expected_base_packs = max(exp_comm_packs, exp_rare_packs, exp_epic_packs, exp_leg_packs)

        # Foil sets
        # Foil chance is prm['foil'] / 100
        # Wait, foil logic: r21 < thr(prm.foil * 100.0) -> prm.foil is a float %.
        # thr(5.0 * 100) = 500, out of 10000 = 5%.
        # So foil_rate is prm["foil"] / 100.0, meaning 5.0 -> 0.05
        foil_rate = prm["foil"] / 100.0

        # But god packs alter this!
        # God packs 6,7,8,9,10,12,13 have foil = 100% (10000/10000)
        god_foil_odds = sum(god_odds_dict.get(g, 0) for g in [6,7,8,9,10,12,13])
        god_foil_none = total_god_prob - god_foil_odds

        effective_foil_rate = (1.0 - total_god_prob) * foil_rate + god_foil_odds * 1.0 + god_foil_none * 0.0

        exp_comm_foil_packs = expected_packs_for_set(c_comm, exp_yields[0] * effective_foil_rate) if c_comm > 0 and exp_yields[0] > 0 else 0
        exp_rare_foil_packs = expected_packs_for_set(c_rare, exp_yields[1] * effective_foil_rate) if c_rare > 0 and exp_yields[1] > 0 else 0
        exp_epic_foil_packs = expected_packs_for_set(c_epic, exp_yields[2] * effective_foil_rate) if c_epic > 0 and exp_yields[2] > 0 else 0
        exp_leg_foil_packs  = expected_packs_for_set(c_leg, exp_yields[3] * effective_foil_rate) if c_leg > 0 and exp_yields[3] > 0 else 0

        expected_foil_packs = max(exp_comm_foil_packs, exp_rare_foil_packs, exp_epic_foil_packs, exp_leg_foil_packs)

        if args.markdown:
            print(f"### Pack `eitem: {eitem}` (Collection {collection}, Exp {exp_id}, Price ${price:.2f})")

            print("| Rarity | Cards in Set | Expected per Pack |")
            print("|---|---|---|")
            if c_comm > 0: print(f"| Common | {c_comm} | {exp_yields[0]:.4f} |")
            if c_rare > 0: print(f"| Rare | {c_rare} | {exp_yields[1]:.4f} |")
            if c_epic > 0: print(f"| Epic | {c_epic} | {exp_yields[2]:.4f} |")
            if c_leg > 0:  print(f"| Legendary | {c_leg} | {exp_yields[3]:.4f} |")
            print()

            print(f"**Coupon Collector (Expected Packs & Cost to Complete Set)**")
            print("| Set Type | Expected Packs | Expected Cost |")
            print("|---|---|---|")
            if expected_base_packs > 0 and expected_base_packs != float('inf'):
                print(f"| Full Base Set | {expected_base_packs:,.0f} | ${expected_base_packs * price:,.2f} |")
            if expected_foil_packs > 0 and expected_foil_packs != float('inf'):
                print(f"| Full Foil Set | {expected_foil_packs:,.0f} | ${expected_foil_packs * price:,.2f} |")
            print()
        else:
            print(f"Pack eitem={eitem} (Collection={collection}, Exp={exp_id}, Price=${price:.2f})")
            print("  Expected yield per pack:")
            if c_comm > 0: print(f"    Common:    {exp_yields[0]:.4f}")
            if c_rare > 0: print(f"    Rare:      {exp_yields[1]:.4f}")
            if c_epic > 0: print(f"    Epic:      {exp_yields[2]:.4f}")
            if c_leg > 0:  print(f"    Legendary: {exp_yields[3]:.4f}")
            print("  Coupon Collector Expected Completion:")
            if expected_base_packs > 0 and expected_base_packs != float('inf'):
                print(f"    Base Set: {expected_base_packs:,.0f} packs (${expected_base_packs * price:,.2f})")
            if expected_foil_packs > 0 and expected_foil_packs != float('inf'):
                print(f"    Foil Set: {expected_foil_packs:,.0f} packs (${expected_foil_packs * price:,.2f})")
            print()

if __name__ == "__main__":
    main()
