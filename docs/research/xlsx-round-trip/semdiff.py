"""Compare two rustsheet-eval snapshot JSONs: sheets, cell formulas, values."""
import json, sys
a, b = (json.load(open(x)) for x in sys.argv[1:3])
out = {'sheets_a': len(a['sheets']), 'sheets_b': len(b['sheets'])}
na = [s['name'] for s in a['sheets']]; nb = [s['name'] for s in b['sheets']]
out['sheet_names_equal'] = na == nb
fd = vd = miss = tot = 0; ex = []
bs = {s['name']: s['cells'] for s in b['sheets']}
for s in a['sheets']:
    cb = bs.get(s['name'], {})
    for ref, c in s['cells'].items():
        tot += 1
        d = cb.get(ref)
        if d is None:
            miss += 1; ex.append(('missing', s['name'], ref)); continue
        if c.get('formula') != d.get('formula'):
            fd += 1; ex.append(('formula', s['name'], ref, c.get('formula'), d.get('formula')))
        elif c.get('value') != d.get('value'):
            vd += 1; ex.append(('value', s['name'], ref, c.get('value'), d.get('value')))
out.update(cells=tot, missing=miss, formula_diff=fd, value_diff=vd, examples=ex[:5])
print(json.dumps(out, default=str))
