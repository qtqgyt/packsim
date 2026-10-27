#!/bin/bash
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PACKSIM_SEED="${PACKSIM_SEED:-0x12345678abcdef01}"
export PACKSIM_BLOCKS="${PACKSIM_BLOCKS:-32}"
CLI="$ROOT/packopener-cli"
REF="$ROOT/tests/golden/ref"
EITEMS=(0 2 4 6 8 10 12 14 16 17 18 19)
PACKS=1000000
THREADS=24
BENCH_MIN=40000000
fails=0
mkdir -p "$ROOT/.testrun"
cd "$ROOT/.testrun" || exit 1
rm -f results.simple results_sold.simple results.ckpt conc_cpu.simple conc_cpu.sold conc_gpu.simple conc_gpu.sold resume_ref.simple resume_ref.sold resume.gpu resume.sold resume.ckpt par_cpu.simple par_cpu.sold par_gpu.simple par_gpu.sold par_gpu.err

for e in "${EITEMS[@]}"; do
  if ! "$CLI" "$e" $PACKS --backend cpu --threads $THREADS --dry-run >/dev/null 2>&1; then
    echo "FAIL cpu run $e"
    fails=$((fails+1))
    continue
  fi
  if cmp -s results.simple "$REF/cpu_$e.simple"; then
    echo "PASS cpu $e"
  else
    echo "FAIL cpu golden $e"
    fails=$((fails+1))
  fi
done

have_gpu=0
rm -f .test_probe.simple .test_probe.ckpt .test_probe_sold.simple
if [ -f "$ROOT/libpackopen_gpu.so" ]; then
  probe_out="$("$CLI" 0 10 --backend gpu --dry-run --results .test_probe.simple 2>&1)"
  if [ $? -eq 0 ] && [ -s .test_probe.simple ] && ! echo "$probe_out" | grep -q "falling back to CPU"; then
    have_gpu=1
  fi
fi
rm -f .test_probe.*

if [ "$have_gpu" -eq 1 ]; then
  for e in "${EITEMS[@]}"; do
    if ! "$CLI" "$e" $PACKS --backend gpu --dry-run >/dev/null 2>&1; then
      echo "FAIL gpu run $e"
      fails=$((fails+1))
      continue
    fi
    if cmp -s results.simple "$REF/gpu_$e.simple"; then
      echo "PASS gpu $e"
    else
      echo "FAIL gpu golden $e"
      echo "  vram used: $(nvidia-smi --query-gpu=memory.used --format=csv,noheader 2>/dev/null)"
      fails=$((fails+1))
    fi
  done

  "$CLI" 0 $PACKS --backend cpu --threads $THREADS --dry-run --results conc_cpu.simple --sold conc_cpu.sold >/dev/null 2>&1 &
  p1=$!
  "$CLI" 8 $PACKS --backend gpu --dry-run --results conc_gpu.simple --sold conc_gpu.sold >/dev/null 2>&1 &
  p2=$!
  if wait $p1 && cmp -s conc_cpu.simple "$REF/cpu_0.simple"; then
    echo "PASS concurrent cpu"
  else
    echo "FAIL concurrent cpu"
    fails=$((fails+1))
  fi
  if wait $p2 && cmp -s conc_gpu.simple "$REF/gpu_8.simple"; then
    echo "PASS concurrent gpu"
  else
    echo "FAIL concurrent gpu"
    fails=$((fails+1))
  fi

  "$CLI" 0 100000000 --backend gpu --dry-run --results resume_ref.simple --sold resume_ref.sold >/dev/null 2>&1
  ref_rc=$?
  "$CLI" 0 100000000 --backend gpu --dry-run --results resume.gpu --sold resume.sold >/dev/null 2>&1 &
  pr=$!
  for _ in $(seq 1 200); do
    [ -e resume.ckpt ] && break
    sleep 0.05
  done
  sleep 0.4
  kill -9 $pr 2>/dev/null
  wait $pr 2>/dev/null
  "$CLI" 0 100000000 --backend gpu --dry-run --results resume.gpu --sold resume.sold >/dev/null 2>&1
  res_rc=$?
  if [ "$ref_rc" -eq 0 ] && [ "$res_rc" -eq 0 ] && cmp -s resume.gpu resume_ref.simple; then
    echo "PASS checkpoint resume"
  else
    echo "FAIL checkpoint resume (ref=$ref_rc res=$res_rc)"
    fails=$((fails+1))
  fi

  others=$(pgrep -c -f '[p]ackopener-cli' 2>/dev/null)
  util=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null)
  others=${others:-0}
  util=${util:-0}
  if [ "$others" -gt 0 ] || [ "$util" -gt 10 ]; then
    echo "SKIP gpu throughput ($others packopener runs, ${util}% gpu util)"
  else
    rate="$("$CLI" benchmark 100000000 --backend gpu 2>&1 | tail -1 | sed 's/.*= //; s/ packs\/s//')"
    rn=$rate
    mult=1
    case "$rn" in
      *Q) mult=1000000000000000; rn="${rn%Q}" ;;
      *T) mult=1000000000000; rn="${rn%T}" ;;
      *B) mult=1000000000; rn="${rn%B}" ;;
      *M) mult=1000000; rn="${rn%M}" ;;
      *K) mult=1000; rn="${rn%K}" ;;
    esac
    rate_num=$(awk -v r="$rn" -v m="$mult" 'BEGIN{printf "%.0f", r*m}')
    if [ "$rate_num" -ge "$BENCH_MIN" ] 2>/dev/null; then
      echo "PASS gpu throughput ($rate packs/s)"
    else
      echo "FAIL gpu throughput ($rate < $BENCH_MIN)"
      fails=$((fails+1))
    fi
  fi
else
  echo "SKIP gpu goldens (no compatible CUDA GPU found)"
  "$CLI" 0 $PACKS --backend cpu --threads $THREADS --dry-run --results conc_cpu1.simple --sold conc_cpu1.sold >/dev/null 2>&1 &
  p1=$!
  "$CLI" 8 $PACKS --backend cpu --threads $THREADS --dry-run --results conc_cpu2.simple --sold conc_cpu2.sold >/dev/null 2>&1 &
  p2=$!
  if wait $p1 && cmp -s conc_cpu1.simple "$REF/cpu_0.simple" && wait $p2 && cmp -s conc_cpu2.simple "$REF/cpu_8.simple"; then
    echo "PASS concurrent cpu"
  else
    echo "FAIL concurrent cpu"
    fails=$((fails+1))
  fi
  echo "SKIP concurrent gpu (no compatible CUDA GPU found)"

  "$CLI" 0 20000000 --backend cpu --threads $THREADS --dry-run --results resume_ref.simple --sold resume_ref.sold >/dev/null 2>&1
  ref_rc=$?
  "$CLI" 0 20000000 --backend cpu --threads $THREADS --dry-run --results resume.cpu --sold resume.sold >/dev/null 2>&1 &
  pr=$!
  for _ in $(seq 1 200); do
    [ -e resume.ckpt ] && break
    sleep 0.05
  done
  sleep 0.1
  kill -9 $pr 2>/dev/null
  wait $pr 2>/dev/null
  "$CLI" 0 20000000 --backend cpu --threads $THREADS --dry-run --results resume.cpu --sold resume.sold >/dev/null 2>&1
  res_rc=$?
  if [ "$ref_rc" -eq 0 ] && [ "$res_rc" -eq 0 ] && cmp -s resume.cpu resume_ref.simple; then
    echo "PASS checkpoint resume (cpu)"
  else
    echo "FAIL checkpoint resume (cpu: ref=$ref_rc res=$res_rc)"
    fails=$((fails+1))
  fi
  echo "SKIP gpu throughput (no compatible CUDA GPU found)"
fi

if "$CLI" --help >/dev/null 2>&1; then
  echo "PASS help exit 0"
else
  echo "FAIL help"
  fails=$((fails+1))
fi

"$CLI" benchmark oops >/dev/null 2>&1
rc=$?
if [ "$rc" -eq 2 ]; then
  echo "PASS bad arg exit 2"
else
  echo "FAIL bad arg (rc=$rc)"
  fails=$((fails+1))
fi

if "$CLI" status >/dev/null 2>&1; then
  echo "PASS status"
else
  echo "FAIL status"
  fails=$((fails+1))
fi

if command -v cargo >/dev/null 2>&1; then
  if (cd "$ROOT/cli" && cargo clippy --release --offline -- -D warnings >/dev/null 2>&1); then
    echo "PASS clippy"
  else
    echo "FAIL clippy"
    fails=$((fails+1))
  fi
else
  echo "SKIP clippy (cargo not installed)"
fi

if python3 "$ROOT/scripts/verify_tables.py"; then
  :
else
  fails=$((fails+1))
fi

rm -f results.simple results_sold.simple results.ckpt conc_cpu.simple conc_cpu.sold conc_gpu.simple conc_gpu.sold resume_ref.simple resume_ref.sold resume.gpu resume.sold resume.ckpt par_cpu.simple par_cpu.sold par_gpu.simple par_gpu.sold par_gpu.err

if [ "$fails" -eq 0 ]; then
  echo "ALL TESTS PASSED"
  exit 0
fi
echo "$fails FAILURE(S)"
exit 1
