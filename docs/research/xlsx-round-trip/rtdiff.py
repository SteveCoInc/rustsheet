"""Round-trip loss census: compare an original xlsx with RustSheet's re-save.
usage: rtdiff.py orig.xlsx out.xlsx  -> JSON line with lost parts and lost element/attr features."""
import json, re, sys, zipfile
import xml.etree.ElementTree as ET
from collections import Counter

NS = {
 'http://schemas.openxmlformats.org/spreadsheetml/2006/main': '',
 'http://schemas.microsoft.com/office/spreadsheetml/2009/9/main': 'x14:',
 'http://schemas.microsoft.com/office/spreadsheetml/2010/11/main': 'x15:',
 'http://schemas.microsoft.com/office/excel/2006/main': 'xm:',
 'http://schemas.openxmlformats.org/markup-compatibility/2006': 'mc:',
 'http://schemas.openxmlformats.org/officeDocument/2006/relationships': 'r:',
 'http://schemas.microsoft.com/office/spreadsheetml/2014/revision': 'xr:',
 'http://schemas.microsoft.com/office/spreadsheetml/2009/9/ac': 'x14ac:',
 'http://schemas.openxmlformats.org/drawingml/2006/main': 'a:',
}
def q(tag):
    if tag.startswith('{'):
        ns, _, l = tag[1:].partition('}')
        return NS.get(ns, '{%s}' % ns.rsplit('/',1)[-1] + ':') + l
    return tag

def kind(name):
    n = name.lower().lstrip('/')
    n = re.sub(r'\d+(?=\.[a-z]+$)', 'N', n)
    return n

def census(data, depth):
    c = Counter()
    try:
        root = ET.fromstring(data)
    except Exception:
        return None
    def walk(e, path, d):
        p = path + '/' + q(e.tag)
        if q(e.tag) == 'ext' and 'uri' in e.attrib:
            p += '[%s]' % e.attrib['uri']
        c['E ' + p] += 1
        for a in e.attrib:
            c['A ' + p + '@' + q(a)] += 1
        if d < depth:
            for ch in e:
                walk(ch, p, d + 1)
        else:
            if len(e):
                c['E ' + p + '/*'] += sum(1 for _ in e.iter()) - 1
    walk(root, '', 0)
    return c

DEPTH = {'xl/worksheets/sheetN.xml': 4, 'xl/workbook.xml': 4, 'xl/styles.xml': 4,
         'xl/sharedstrings.xml': 3, 'xl/theme/themeN.xml': 3}
def ziprd(p):
    z = zipfile.ZipFile(p)
    out = {}
    for i in z.infolist():
        if i.file_size > 64 << 20: continue
        if i.filename.endswith('/'): continue
        out[i.filename.replace('\\', '/').lower()] = z.read(i)
    return out

def main(a, b):
    A, B = ziprd(a), ziprd(b)
    ka, kb = Counter(kind(n) for n in A), Counter(kind(n) for n in B)
    lost_parts = {k: v - kb.get(k, 0) for k, v in ka.items() if v > kb.get(k, 0)}
    added_parts = {k: v - ka.get(k, 0) for k, v in kb.items() if v > ka.get(k, 0)}
    lost = Counter()
    # map sheets by order in the workbook is complex; aggregate per part kind instead
    agg_a, agg_b = {}, {}
    for src, agg in ((A, agg_a), (B, agg_b)):
        for n, d in src.items():
            k = kind(n)
            if k in DEPTH:
                c = census(d, DEPTH[k])
                if c is not None:
                    agg.setdefault(k, Counter()).update(c)
    for k, ca in agg_a.items():
        cb = agg_b.get(k, Counter())
        for f, v in ca.items():
            if v > cb.get(f, 0):
                lost[k + ' ' + f] = (v, cb.get(f, 0))
    return {'lost_parts': lost_parts, 'added_parts': added_parts,
            'lost': {k: list(v) for k, v in lost.items()}}

if __name__ == '__main__':
    print(json.dumps(main(sys.argv[1], sys.argv[2])))
