#!/usr/bin/env bash
# smoke.sh — CLI smoke tests against real mainnet data
set -euo pipefail

BIN="./target/release/pump-cli"
PASS=0
FAIL=0
SKIP=0

KNOWN_WALLET="675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"

red()    { printf "\033[31m%s\033[0m\n" "$1"; }
green()  { printf "\033[32m%s\033[0m\n" "$1"; }
yellow() { printf "\033[33m%s\033[0m\n" "$1"; }

run_test() {
    local name="$1"; shift
    printf "%-55s " "$name"
    if output=$("$@" 2>&1); then
        green "PASS"; PASS=$((PASS + 1))
    else
        red "FAIL"; echo "  cmd: $*"; echo "  out: $(echo "$output" | head -3)"; FAIL=$((FAIL + 1))
    fi
}

run_test_json() {
    local name="$1"; shift
    printf "%-55s " "$name"
    if output=$("$@" 2>&1) && echo "$output" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
        green "PASS"; PASS=$((PASS + 1))
    else
        red "FAIL"; echo "  cmd: $*"; echo "  out: $(echo "$output" | head -3)"; FAIL=$((FAIL + 1))
    fi
}

run_test_json_field() {
    local name="$1"; local field="$2"; shift 2
    printf "%-55s " "$name"
    if output=$("$@" 2>&1) && val=$(echo "$output" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d['$field'])" 2>/dev/null); then
        if [ -n "$val" ] && [ "$val" != "None" ] && [ "$val" != "null" ]; then
            green "PASS ($field=$val)"; PASS=$((PASS + 1))
        else
            red "FAIL ($field is empty/null)"; FAIL=$((FAIL + 1))
        fi
    else
        red "FAIL"; echo "  cmd: $*"; echo "  out: $(echo "$output" | head -3)"; FAIL=$((FAIL + 1))
    fi
}

run_test_contains() {
    local name="$1"; local needle="$2"; shift 2
    printf "%-55s " "$name"
    if output=$("$@" 2>&1) && echo "$output" | grep -qi "$needle"; then
        green "PASS"; PASS=$((PASS + 1))
    else
        red "FAIL (expected '$needle')"; echo "  cmd: $*"; echo "  out: $(echo "$output" | head -3)"; FAIL=$((FAIL + 1))
    fi
}

run_test_fails() {
    local name="$1"; shift
    printf "%-55s " "$name"
    if "$@" >/dev/null 2>&1; then
        red "FAIL (expected error)"; FAIL=$((FAIL + 1))
    else
        green "PASS"; PASS=$((PASS + 1))
    fi
}

skip_test() {
    local name="$1"; local reason="$2"
    printf "%-55s " "$name"
    yellow "SKIP ($reason)"; SKIP=$((SKIP + 1))
}

# Probe candidate mints to find one with a readable bonding curve.
# Mints may graduate or become unreachable over time.
MINT_CANDIDATES=(
    "CGKzVhKKYhP5qNrbnD8gKfPtLipsMQX3YkXhHaTVJGnH"
    "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU"
    "HeLp6NuQkmYB4pYWo2zYs22mESHXPQYzXbB8n4V98jwC"
)
ACTIVE_MINT=""
for candidate in "${MINT_CANDIDATES[@]}"; do
    for attempt in 1 2 3; do
        if $BIN -f json info "$candidate" >/dev/null 2>&1; then
            ACTIVE_MINT="$candidate"
            break 2
        fi
        sleep 1
    done
done

echo "================================================"
echo "  pump-cli smoke tests (mainnet)"
if [ -n "$ACTIVE_MINT" ]; then
    echo "  active mint: $ACTIVE_MINT"
else
    echo "  active mint: NONE FOUND (curve-info tests will skip)"
fi
echo "================================================"
echo ""

echo "── Binary ──"
run_test "binary exists"                         test -x "$BIN"
run_test "help flag works"                       $BIN --help
echo ""

echo "── Config ──"
run_test "config list (table)"                   $BIN config list
run_test_json "config list (json)"               $BIN -f json config list
echo ""

echo "── Keys ──"
run_test "keys generate"                         $BIN keys generate smoke-test-$$
run_test_json "keys generate (json)"             $BIN -f json keys generate smoke-json-$$
run_test_json "keys list (json)"                 $BIN -f json keys list
run_test "keys use"                              $BIN keys use smoke-test-$$
run_test_contains "keys list shows active"       "active" $BIN keys list
echo ""

echo "── Bonding Curve Info ──"
if [ -n "$ACTIVE_MINT" ]; then
    run_test "info table"                            $BIN info "$ACTIVE_MINT"
    run_test_json "info json"                        $BIN -f json info "$ACTIVE_MINT"
    run_test_json_field "price_sol"                   "price_sol" $BIN -f json info "$ACTIVE_MINT"
    run_test_json_field "market_cap_sol"              "market_cap_sol" $BIN -f json info "$ACTIVE_MINT"
    run_test_json_field "progress_pct"                "progress_pct" $BIN -f json info "$ACTIVE_MINT"
    run_test_json_field "creator"                     "creator" $BIN -f json info "$ACTIVE_MINT"
    run_test_contains "shows Price"                  "Price" $BIN info "$ACTIVE_MINT"
    run_test_contains "shows Progress"               "Progress" $BIN info "$ACTIVE_MINT"
    run_test_contains "shows Complete"               "Complete" $BIN info "$ACTIVE_MINT"
    run_test_contains "shows Mayhem"                 "Mayhem" $BIN info "$ACTIVE_MINT"
    run_test_contains "shows Cashback"               "Cashback" $BIN info "$ACTIVE_MINT"
    run_test_json_field "is_mayhem_mode"              "is_mayhem_mode" $BIN -f json info "$ACTIVE_MINT"
    run_test_json_field "is_cashback_coin"            "is_cashback_coin" $BIN -f json info "$ACTIVE_MINT"
else
    for t in "info table" "info json" "price_sol" "market_cap_sol" "progress_pct" \
             "creator" "shows Price" "shows Progress" "shows Complete" "shows Mayhem" \
             "shows Cashback" "is_mayhem_mode" "is_cashback_coin"; do
        skip_test "$t" "no active mint"
    done
fi
echo ""

echo "── Balance ──"
run_test "sol balance by address"                $BIN balance --address "$KNOWN_WALLET"
run_test_json "sol balance json"                 $BIN -f json balance --address "$KNOWN_WALLET"
if [ -n "$ACTIVE_MINT" ]; then
    run_test "token balance by address"          $BIN balance --address "$KNOWN_WALLET" "$ACTIVE_MINT"
else
    skip_test "token balance by address" "no active mint"
fi
echo ""

echo "── Create commands ──"
run_test "help for create (legacy)"              $BIN create --help
run_test "help for create-v2"                    $BIN create-v2 --help
run_test_contains "create-v2 has --mayhem"       "mayhem" $BIN create-v2 --help
run_test_contains "create-v2 has --cashback"     "cashback" $BIN create-v2 --help
echo ""

echo "── Error handling ──"
run_test_fails "info with bad mint"              $BIN info not-a-real-mint
run_test_fails "info with random valid pubkey"   $BIN info 11111111111111111111111111111111
run_test_fails "buy with no args"                $BIN buy
run_test_fails "sell with no args"               $BIN sell
run_test_fails "unknown command"                 $BIN foobar
echo ""

echo "── Help pages ──"
run_test "help for buy"                          $BIN buy --help
run_test "help for sell"                         $BIN sell --help
run_test "help for swap"                         $BIN swap --help
run_test "help for watch"                        $BIN watch --help
echo ""

echo "================================================"
printf "  PASS: "; green "$PASS"
printf "  FAIL: "; red "$FAIL"
printf "  SKIP: "; yellow "$SKIP"
echo "================================================"

rm -f "$HOME/Library/Application Support/pump/keys/smoke-test-$$.json"
rm -f "$HOME/Library/Application Support/pump/keys/smoke-json-$$.json"

exit $FAIL
