#!/usr/bin/env python3
"""Extract packopener_tables.json -> Rust source tables (packmap + exp tables)."""
import json, os, sys

T = os.environ.get('PACKOPENER_TABLES', os.path.expanduser('~/.local/share/Steam/steamapps/common/TCG Card Shop Simulator/BepInEx/config/packopener_tables.json'))
OUT = sys.argv[1] if len(sys.argv) > 1 else 'tables_gen.rs'

with open(T) as f:
    t = json.load(f)
packmap = {}
for p in t['packs']:
    if p['collection'] >= 0 and p['exp'] >= 0:
        packmap[p['eitem']] = (p['collection'], p['exp'])

lines = []
lines.append('#[allow(dead_code)]')
lines.append('pub static PACKMAP: &[(i32, i32, i32)] = &[')
for e in sorted(packmap):
    c, x = packmap[e]
    lines.append(f'    ({e}, {c}, {x}),')
lines.append('];')
lines.append('')
lines.append('#[allow(dead_code)]')
lines.append('pub static EXPS: &[(i32, bool, bool, i32, &[i32])] = &[')
for e in sorted(x['exp'] for x in t['expansions']):
    d = next(x for x in t['expansions'] if x['exp'] == e)
    r = ','.join(map(str, d['rarities']))
    lines.append(f'    ({e}, {"true" if d["useRarity"] else "false"}, false, {d["perMonster"]}, &[{r}]),')
lines.append('];')
lines.append('')
lines.append('pub static SHOP_LEVEL_FALLBACK: i32 = %d;' % t.get('shopLevel', 1))

with open(OUT, 'w') as f:
    f.write('\n'.join(lines) + '\n')
print(f'wrote {OUT}: {len(packmap)} packs, {len(t["expansions"])} expansions')
