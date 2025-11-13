# Simple Aircher Testing (Before Full SWE-bench)

**Goal**: Basic debug/fix cycles without full benchmark setup

## Approach 1: Manual Bug Fixing (Recommended)

Create simple Python bugs and have Aircher fix them. Fast, no dependencies.

### Test 1: Syntax Error
```bash
# Create buggy file
cat > /tmp/syntax_bug.py << 'EOF'
def calculate_sum(numbers)
    total = 0
    for num in numbers:
        total += num
    return total

print(calculate_sum([1, 2, 3, 4, 5]))
EOF

# Test Aircher
./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Fix the syntax error in /tmp/syntax_bug.py and save the corrected version"
```

**Expected**: Add missing colon after function definition

### Test 2: Logic Error
```bash
cat > /tmp/logic_bug.py << 'EOF'
def find_max(numbers):
    max_num = 0  # BUG: Should be numbers[0] or -inf
    for num in numbers:
        if num > max_num:
            max_num = num
    return max_num

print(find_max([-5, -2, -10, -1]))  # Should return -1, but returns 0
EOF

./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Fix the bug in /tmp/logic_bug.py that causes find_max([-5, -2, -10, -1]) to return 0 instead of -1"
```

**Expected**: Initialize max_num to first element or negative infinity

### Test 3: Missing Import
```bash
cat > /tmp/import_bug.py << 'EOF'
def get_current_time():
    now = datetime.now()
    return now.strftime("%Y-%m-%d %H:%M:%S")

print(get_current_time())
EOF

./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Fix the missing import in /tmp/import_bug.py and make it run successfully"
```

**Expected**: Add `from datetime import datetime`

### Test 4: Type Error
```bash
cat > /tmp/type_bug.py << 'EOF'
def concatenate_strings(str1, str2, str3):
    return str1 + str2 + str3

result = concatenate_strings("Hello", " ", 42)  # BUG: 42 is int, not str
print(result)
EOF

./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Fix the type error in /tmp/type_bug.py"
```

**Expected**: Convert 42 to string

### Test 5: Off-by-One Error
```bash
cat > /tmp/offbyone_bug.py << 'EOF'
def get_last_three(items):
    # BUG: Should be items[-3:], not items[-4:-1]
    return items[-4:-1]

print(get_last_three([1, 2, 3, 4, 5, 6, 7, 8, 9]))  # Should be [7, 8, 9]
EOF

./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Fix the off-by-one error in /tmp/offbyone_bug.py so it returns the last 3 items"
```

**Expected**: Change to `items[-3:]`

## Approach 2: SWE-bench Sample (5-10 Tasks)

If you want to try real SWE-bench data without full setup:

### Quick SWE-bench Test
```bash
# Clone SWE-bench (just the dataset, not the full harness)
git clone https://github.com/princeton-nlp/SWE-bench.git /tmp/swe-bench
cd /tmp/swe-bench

# Get first 10 tasks from Lite dataset
python -c "
import json
with open('data/swe-bench-lite.json', 'r') as f:
    data = json.load(f)
    for i, task in enumerate(data[:10]):
        print(f\"Task {i+1}: {task['repo']} - {task['instance_id']}\")
        print(f\"Problem: {task['problem_statement'][:200]}...\")
        print()
"
```

**Pick 2-3 simple ones and test manually**

## Approach 3: Real Aircher Repo (Dogfooding)

Use Aircher to fix bugs in Aircher itself!

### Example: Fix Warnings
```bash
# Aircher has 79 warnings - pick one to fix
cargo build 2>&1 | grep "warning:" | head -5

# Example: Fix unused import
./target/release/aircher --provider ollama --model "gpt-oss:latest" \
  "Fix the unused import warning in src/providers/ollama.rs line 134 (OllamaError struct is never constructed)"
```

**Expected**: Remove unused struct or add `#[allow(dead_code)]`

## Metrics to Track

For each test, record:

1. **Success**: Did it identify the bug correctly?
2. **Fix**: Did it propose correct solution?
3. **Tokens**: How many tokens used?
4. **Time**: How long did it take?
5. **Attempts**: How many tries needed?

**Simple Scorecard**:
```
Test 1 (Syntax Error):      ✅ Success, 1 attempt, 150 tokens, 8s
Test 2 (Logic Error):        ✅ Success, 1 attempt, 200 tokens, 10s
Test 3 (Missing Import):     ✅ Success, 1 attempt, 100 tokens, 6s
Test 4 (Type Error):         ⚠️  Success, 2 attempts, 300 tokens, 18s
Test 5 (Off-by-One):         ❌ Failed (wrong fix)

Score: 4/5 = 80% success rate
```

## Why This Approach?

**Advantages**:
- ✅ Fast (5 tests in 10 minutes)
- ✅ No dependencies (SWE-bench setup takes hours)
- ✅ Full control (create custom test cases)
- ✅ Immediate feedback
- ✅ Easy to debug

**Disadvantages**:
- ❌ Not standardized (can't compare to other agents)
- ❌ Too simple (SWE-bench has real-world complexity)

## After Simple Tests Pass

Once 80%+ success rate on simple tests:

1. **Try 10 SWE-bench Lite tasks** (cherry-pick easiest ones)
2. **Then run full Lite** (300 tasks, ~6-12 hours)
3. **Compare to baselines** (Claude Code: 43.2%, Factory Droid: 58.8%)

## Recommendation

**Start with Approach 1** (Manual bugs): 5 tests, ~15 minutes total

**If that works well**: Try Approach 3 (Fix Aircher warnings): Real dogfooding

**If both work**: Pick 5 easy SWE-bench Lite tasks and test those

**Full SWE-bench Lite**: Only after confidence in basic capabilities
