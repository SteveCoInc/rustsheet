"""R4.1 hold-out for T-4328. Pre-registered before scoring (written 2026-11-07, before either binary's values were read):
- Cells: every Excel-valued cell (any suite) whose whole formula is one SKEW(, SKEW.P( or KURT( call.
- Split: sha256(case_id + '!' + cell) first byte < 77 (about 30%) is hold-out, the rest train.
- Candidates: A = this branch (stored x87, (z*z)*(z*z)); B = main (once-rounded binary64 per operation, chained powers).
- Metrics per half: bit-exact, within 4 ulp, within 15 significant digits.
- Accept A only if on hold-out A's bit-exact count beats B's by at least 5 cells and A loses no 15-digit cell B keeps.
"""
import hashlib
import math
import re
import struct
import subprocess
import sys

BR, MAIN, ROOT = sys.argv[1], sys.argv[2], sys.argv[3]
MOMENT = re.compile(r'^=(SKEW|SKEW\.P|KURT)\((?:[^()]|\([^()]*\))*\)$')
def show(binary, case):
    out = subprocess.run([binary, '--root', ROOT, 'show', case, '--all'], capture_output=True, text=True, check=True).stdout
    cells, cur = {}, None
    for line in out.splitlines():
        m = re.match(r'^(\S+!\S+)\s+(\S+)', line)
        if m and not line.startswith(' '): cur = m[1]; cells[cur] = {}; continue
        m = re.match(r'^\s+(formula|expected|actual)\s*: (.*)$', line)
        if m and cur: cells[cur][m[1]] = m[2]
    return cells
def ulps(a, b):
    ia, ib = (struct.unpack('<q', struct.pack('<d', x))[0] for x in (a, b))
    ia, ib = [i if i >= 0 else -(1 << 63) - i for i in (ia, ib)]
    return abs(ia - ib)
def num(s):
    try: return float(s)
    except ValueError: return None
cases = sys.stdin.read().split()
rows = []
for case in cases:
    a, b = show(BR, case), show(MAIN, case)
    for cell, d in a.items():
        f = d.get('formula', '')
        if not MOMENT.match(f) or cell not in b: continue
        e, x, y = num(d.get('expected', '')), num(d.get('actual', '')), num(b[cell].get('actual', ''))
        if e is None: continue
        hold = hashlib.sha256(f'{case}!{cell}'.encode()).digest()[0] < 77
        rows.append((case, cell, f.split('(')[0][1:], hold, e, x, y))
def score(v, e):
    if v is None: return (0, 0, 0)
    exact = v == e
    u = ulps(v, e) if math.isfinite(v) and math.isfinite(e) else (0 if exact else 1 << 62)
    d15 = exact or (e != 0 and abs(v - e) <= 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14))
    return (int(exact), int(u <= 4), int(d15))
for half in ('train', 'hold-out'):
    sel = [r for r in rows if r[3] == (half == 'hold-out')]
    A = [sum(score(r[5], r[4])[i] for r in sel) for i in range(3)]
    B = [sum(score(r[6], r[4])[i] for r in sel) for i in range(3)]
    lost = sum(1 for r in sel if score(r[6], r[4])[2] and not score(r[5], r[4])[2])
    print(f'{half}: n={len(sel)}  A stored-x87 exact/4ulp/15dig = {A}  B once-rounded = {B}  15-digit cells A loses vs B: {lost}')
by = {}
for r in rows: by.setdefault(r[2], 0); by[r[2]] += 1
print('cells per function', by, 'cases', len({r[0] for r in rows}))
