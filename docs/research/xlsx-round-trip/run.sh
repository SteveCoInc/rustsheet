#!/bin/bash
# run.sh <listfile> <outdir>
S=/tmp/claude-0/-home-user-rustsheet/7fa474b6-91aa-563f-bd80-6e7f80786bd6/scratchpad
EV=/home/user/rustsheet/target/release/rustsheet-eval
mkdir -p $2/x; : > $2/status.tsv; : > $2/diff.tsv; : > $2/sem.tsv
while read f; do
  id=$(echo "$f" | md5sum | cut -c1-10); o=$2/x/$id.xlsx
  r=$(timeout 120 env $RTENV $S/rt/target/release/rt "$f" "$o" 2>&1); rc=$?; r=$(echo "$r" | tail -1)
  if [ $rc -ne 0 ]; then printf '%s\tFAIL%s\t%s\n' "$f" "$rc" "$r" >> $2/status.tsv; continue; fi
  printf '%s\tOK\n' "$f" >> $2/status.tsv
  printf '%s\t%s\n' "$f" "$(timeout 120 python3 -I $S/tools/rtdiff.py "$f" "$o" 2>&1 | tail -1)" >> $2/diff.tsv
  timeout 120 $EV snapshot "$f" > $2/x/$id.a.json 2>/dev/null && timeout 120 $EV snapshot "$o" > $2/x/$id.b.json 2>/dev/null && \
    printf '%s\t%s\n' "$f" "$(python3 -I $S/tools/semdiff.py $2/x/$id.a.json $2/x/$id.b.json 2>&1 | tail -1)" >> $2/sem.tsv
  rm -f $2/x/$id.a.json $2/x/$id.b.json
done < $1
