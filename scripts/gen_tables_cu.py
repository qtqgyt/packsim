#!/usr/bin/env python3
"""Emit C++ table initializers for packopen.cu from packopener_tables.json."""
import json, os, sys

T = os.environ.get('PACKOPENER_TABLES', os.path.expanduser('~/.local/share/Steam/steamapps/common/TCG Card Shop Simulator/BepInEx/config/packopener_tables.json'))
OUT = sys.argv[1] if len(sys.argv) > 1 else 'src/common/tables_gen.hpp'

with open(T) as f:
    t = json.load(f)
L = []
L.append('struct PackMap { int eitem, collection, exp; };')
L.append('static const PackMap PACKMAP[] = {')
for p in sorted(t['packs'], key=lambda p: p['eitem']):
    if p['collection'] >= 0 and p['exp'] >= 0:
        L.append(f"    {{{p['eitem']}, {p['collection']}, {p['exp']}}},")
L.append('};')
L.append(f'static const int NPACKMAP = {sum(1 for p in t["packs"] if p["collection"] >= 0 and p["exp"] >= 0)};')
L.append('')
L.append('struct ExpT { int exp, useRarity, haveDup, perMonster, n, n_leg, n_epic, n_rare, n_common; const int *rar; };')
for e in sorted(x['exp'] for x in t['expansions']):
    d = next(x for x in t['expansions'] if x['exp'] == e)
    L.append(f"PACK_DEVICE static const int RAR{e}[] = {{{','.join(map(str, d['rarities']))}}};")
L.append('PACK_DEVICE static const ExpT EXPS[] = {')
for e in sorted(x['exp'] for x in t['expansions']):
    d = next(x for x in t['expansions'] if x['exp'] == e)
    r = d['rarities']
    c = [sum(1 for x in r if x == k) for k in (3, 2, 1, 0)]
    L.append(f"    {{{e}, {1 if d['useRarity'] else 0}, {1 if d['haveDup'] else 0}, {d['perMonster']}, {len(r)}, {c[0]}, {c[1]}, {c[2]}, {c[3]}, RAR{e}}},")
L.append('};')
L.append(f'PACK_DEVICE static const int NEXPS = {len(t["expansions"])};')
L.append(f'static const int SHOP_LEVEL = {t.get("shopLevel", 1)};')
with open(OUT, 'w') as f:
    f.write('\n'.join(L) + '\n')
print(f'wrote {OUT}')
