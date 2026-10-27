#!/usr/bin/env python3
import sys
import re
import os
import json

def parse_cpp_packmap(filepath):
    packmap = []
    in_packmap = False
    with open(filepath, 'r') as f:
        for line in f:
            if 'PACKMAP[]' in line:
                in_packmap = True
                continue
            if in_packmap:
                if '}' in line and not line.strip().startswith('{'):
                    break
                m = re.search(r'\{\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\}', line)
                if m:
                    packmap.append((int(m.group(1)), int(m.group(2)), int(m.group(3))))
    return packmap

def parse_rs_packmap(filepath):
    packmap = []
    in_packmap = False
    with open(filepath, 'r') as f:
        for line in f:
            if 'PACKMAP' in line and '=' in line:
                in_packmap = True
                continue
            if in_packmap:
                if ']' in line:
                    break
                m = re.search(r'\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)', line)
                if m:
                    packmap.append((int(m.group(1)), int(m.group(2)), int(m.group(3))))
    return packmap

def load_json_packmap(filepath):
    try:
        with open(filepath, 'r') as f:
            t = json.load(f)
        packmap = {}
        for p in t.get('packs', []):
            if p['collection'] >= 0 and p['exp'] >= 0:
                packmap[p['eitem']] = (p['collection'], p['exp'])
        return [(e, packmap[e][0], packmap[e][1]) for e in sorted(packmap)]
    except FileNotFoundError:
        return None

def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    cpp_path = os.path.join(root, 'src/common/tables_gen.hpp')
    rs_path = os.path.join(root, 'cli', 'src', 'tables_gen.rs')
    json_path = os.path.join(root, 'packopener_tables.json')
    alt_json_path = os.path.expanduser('~/.local/share/Steam/steamapps/common/TCG Card Shop Simulator/BepInEx/config/packopener_tables.json')

    cpp_packmap = parse_cpp_packmap(cpp_path)
    rs_packmap = parse_rs_packmap(rs_path)

    if not cpp_packmap:
        print("FAIL table verification (C++ packmap empty)")
        sys.exit(1)
    if not rs_packmap:
        print("FAIL table verification (Rust packmap empty)")
        sys.exit(1)

    if cpp_packmap != rs_packmap:
        print("FAIL table verification (C++ and Rust tables do not match)")
        sys.exit(1)

    json_packmap = load_json_packmap(json_path)
    if json_packmap is None:
        json_packmap = load_json_packmap(alt_json_path)

    if json_packmap is not None:
        if cpp_packmap != json_packmap:
            print("FAIL table verification (Generated tables do not match JSON)")
            sys.exit(1)
    else:
        print("WARN: packopener_tables.json not found, skipping JSON verification")

    print("PASS table verification")
    sys.exit(0)

if __name__ == '__main__':
    main()
