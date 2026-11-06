"""Feature-level round-trip loss table from rtdiff output + original presence."""
import json, re, sys, zipfile, collections
sys.path.insert(0, sys.argv[1]); import rtdiff
F = [ # name, kind, part-kind regex, element path (E key) or None for part
 ('Charts', 'P', r'xl/charts/chartN\.xml', None),
 ('Drawings (shapes, pictures anchors)', 'P', r'xl/drawings/drawingN\.xml', None),
 ('Images / media', 'P', r'xl/media/', None),
 ('Legacy comments (notes)', 'P', r'xl/commentsN\.xml|xl/comments/', None),
 ('Threaded comments', 'P', r'xl/threadedcomments/', None),
 ('VML (notes, form controls)', 'P', r'xl/drawings/vmldrawingN\.vml', None),
 ('Pivot tables', 'P', r'xl/pivottables/pivottableN\.xml', None),
 ('Pivot caches', 'P', r'xl/pivotcache/pivotcachedefinitionN\.xml', None),
 ('Slicers', 'P', r'xl/slicers/', None),
 ('External link parts', 'P', r'xl/externallinks/externallinkN\.xml', None),
 ('VBA project', 'P', r'xl/vbaproject\.bin', None),
 ('ActiveX / form control props', 'P', r'xl/activex/|xl/ctrlprops/', None),
 ('Embedded OLE objects', 'P', r'xl/embeddings/', None),
 ('Chartsheets (whole sheet)', 'P', r'xl/chartsheets/sheetN\.xml', None),
 ('Macro sheets (whole sheet)', 'P', r'xl/macrosheets/', None),
 ('Custom document properties', 'P', r'docprops/custom\.xml', None),
 ('customXml data parts', 'P', r'customxml/itemN\.xml', None),
 ('Web extensions (add-ins)', 'P', r'xl/webextensions/webextensionN\.xml', None),
 ('Printer settings', 'P', r'xl/printersettings/', None),
 ('Query tables / connections', 'P', r'xl/querytables/|xl/connections\.xml', None),
 ('Thumbnail', 'P', r'docprops/thumbnail', None),
 ('calcChain', 'P', r'xl/calcchain\.xml', None),
 ('Data validation', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/dataValidations'),
 ('Data validation (x14 ext)', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/extLst/ext[{CCE6A557-97BC-4b89-ADB6-D9C93CAAB3DF}]'),
 ('Sparklines (x14 ext)', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/extLst/ext[{05C60535-1F16-4fd2-B633-F4F36F0B64E0}]'),
 ('Conditional formatting', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/conditionalFormatting'),
 ('Sheet autoFilter', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/autoFilter'),
 ('Page setup', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/pageSetup'),
 ('Page margins', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/pageMargins'),
 ('Header / footer', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/headerFooter'),
 ('Print options', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/printOptions'),
 ('Row/col page breaks', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/rowBreaks'),
 ('Sheet protection', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/sheetProtection'),
 ('Protected ranges', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/protectedRanges'),
 ('Sheet properties (tab color, outline, codeName)', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/sheetPr'),
 ('Tab color', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/sheetPr/tabColor'),
 ('Outline settings', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/sheetPr/outlinePr'),
 ('sheetFormatPr (default row height etc.)', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/sheetFormatPr'),
 ('Ignored errors', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/ignoredErrors'),
 ('Phonetic properties', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/phoneticPr'),
 ('Sheet extLst (any)', 'E', r'xl/worksheets/sheetN\.xml', '/worksheet/extLst'),
 ('Workbook protection', 'E', r'xl/workbook\.xml', '/workbook/workbookProtection'),
 ('fileVersion', 'E', r'xl/workbook\.xml', '/workbook/fileVersion'),
 ('workbookPr', 'E', r'xl/workbook\.xml', '/workbook/workbookPr'),
 ('Workbook extLst (any)', 'E', r'xl/workbook\.xml', '/workbook/extLst'),
 ('externalReferences', 'E', r'xl/workbook\.xml', '/workbook/externalReferences'),
 ('Rich text runs (sharedStrings)', 'E', r'xl/sharedstrings\.xml', '/sst/si/r'),
 ('Styles: indexed color palette', 'E', r'xl/styles\.xml', '/styleSheet/colors/indexedColors'),
 ('Styles: cell protection (locked/hidden)', 'E', r'xl/styles\.xml', '/styleSheet/cellXfs/xf/protection'),
 ('Styles: extLst (slicer/timeline styles)', 'E', r'xl/styles\.xml', '/styleSheet/extLst'),
 ('Styles: custom table styles', 'E', r'xl/styles\.xml', '/styleSheet/tableStyles/tableStyle'),
 ('Theme: format scheme (fills/effects)', 'E', r'xl/theme/themeN\.xml', '/a:theme/a:themeElements/a:fmtScheme/a:effectStyleLst'),
]
tag = {}
for l in open(sys.argv[2]):
    t, p = l.rstrip('\n').split('\t'); tag[p] = t
pres = collections.Counter(); lostc = collections.Counter(); partial = collections.Counter(); nfiles = collections.Counter()
for l in open(sys.argv[3]):
    p, _, js = l.rstrip('\n').partition('\t')
    try: d = json.loads(js)
    except Exception: continue
    t = tag.get(p, '?'); nfiles[t] += 1
    A = rtdiff.ziprd(p)
    kinds = collections.Counter(rtdiff.kind(n) for n in A)
    census = {}
    for n, data in A.items():
        k = rtdiff.kind(n)
        if k in rtdiff.DEPTH:
            c = rtdiff.census(data, rtdiff.DEPTH[k])
            if c: census.setdefault(k, collections.Counter()).update(c)
    for name, kd, prx, el in F:
        if kd == 'P':
            have = any(re.match(prx, k) for k in kinds)
            if not have: continue
            pres[(t, name)] += 1
            if any(re.match(prx, k) for k in d['lost_parts']): lostc[(t, name)] += 1
        else:
            ks = [k for k in census if re.match(prx, k)]
            have = any(('E ' + el) in census[k] for k in ks)
            if not have: continue
            pres[(t, name)] += 1
            for k in ks:
                v = d['lost'].get(f'{k} E {el}')
                if v:
                    if v[1] == 0: lostc[(t, name)] += 1
                    else: partial[(t, name)] += 1
corp = ['poi', 'calipers', 'cb', 'local']
print('| Feature | ' + ' | '.join(f'{c} ({nfiles[c]})' for c in corp) + ' | all: present / lost |')
print('|---|' + '---|' * (len(corp) + 1))
for name, *_ in F:
    cells = []
    tp = tl = 0
    for c in corp:
        p = pres[(c, name)]; lo = lostc[(c, name)] + partial[(c, name)]; tp += p; tl += lo
        cells.append(f'{lo}/{p}' if p else '–')
    if tp: print(f'| {name} | ' + ' | '.join(cells) + f' | {tp} / {tl} |')
