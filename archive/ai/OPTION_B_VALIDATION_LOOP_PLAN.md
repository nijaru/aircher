# Option B: AutoGen Validation Loop Implementation

**Goal**: Fix SWE-bench 0/2 location identification failures via multi-agent validation

**Timeline**: 1-2 days (8-16 hours)

**Expected Impact**: Could improve SWE-bench success rate from 0% to 20-30%

## Problem Statement

**Current failure pattern** (0/2 tasks):
```
User: "Fix bug"
  → Agent: Generates patch (wrong location)
  → Done ❌
```

**Task 0**: Changed loop logic instead of assignment (wrong location in file)
**Task 6**: Changed `__init__.py` instead of `global_settings.py` (wrong file entirely)

**Root cause**: No verification that target location contains expected code

## Solution: AutoGen Validation Loop

**New pattern**:
```
User: "Fix bug"
  → Research Agent: Finds potential location(s)
  → Coding Agent: Proposes patch for location
  → Research Agent: Verifies "Does this file/line contain expected code?"
    → If NO: Loop back, try different location (max 3 attempts)
    → If YES: Submit patch
  → Done ✅
```

## Architecture Design

### 1. Agent Roles (Using Existing AgentType Enum)

```rust
// From ai/SYSTEM_DESIGN_2025.md - already designed
pub enum AgentType {
    Explorer,    // ← Research/verification role
    Builder,     // ← Coding/patch generation role
    Debugger,
    Refactorer,
}
```

**Validation loop uses**:
- **Explorer**: Finds bug location, verifies proposed patches
- **Builder**: Generates patches for verified locations

### 2. Validation Loop Coordinator

```rust
// New file: src/agent/validation_loop.rs

pub struct ValidationLoopCoordinator {
    explorer: Arc<Agent>,   // Research/verification
    builder: Arc<Agent>,    // Patch generation
    max_attempts: usize,    // Default: 3
    session: Arc<Session>,
}

pub struct LocationCandidate {
    file_path: PathBuf,
    line_number: Option<usize>,
    confidence: f32,        // 0.0 to 1.0
    reasoning: String,      // Why this location?
}

pub struct PatchProposal {
    location: LocationCandidate,
    patch: String,          // Unified diff
    reasoning: String,
}

pub struct VerificationResult {
    is_correct: bool,
    reasoning: String,
    issues: Vec<String>,    // What's wrong if not correct
}

impl ValidationLoopCoordinator {
    pub async fn fix_bug_with_validation(
        &self,
        bug_description: &str,
        repository_path: &Path,
    ) -> Result<ValidatedPatch> {
        // Phase 1: Research - Find location candidates
        let candidates = self.explorer.find_bug_locations(
            bug_description,
            repository_path,
        ).await?;

        // Sort by confidence (highest first)
        let mut candidates = candidates;
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Phase 2: Iterate through candidates with validation
        for (attempt, candidate) in candidates.iter().enumerate().take(self.max_attempts) {
            info!("Attempt {}/{}: Trying location {:?} (confidence: {:.1}%)",
                  attempt + 1, self.max_attempts, candidate.file_path, candidate.confidence * 100.0);

            // Builder generates patch for this location
            let proposal = self.builder.generate_patch(
                bug_description,
                candidate,
            ).await?;

            // Explorer verifies the patch
            let verification = self.explorer.verify_patch_location(
                &proposal,
                bug_description,
            ).await?;

            if verification.is_correct {
                info!("✓ Location verified on attempt {}", attempt + 1);
                return Ok(ValidatedPatch {
                    patch: proposal.patch,
                    location: candidate.clone(),
                    attempts: attempt + 1,
                });
            }

            warn!("✗ Verification failed: {}", verification.reasoning);
            for issue in &verification.issues {
                warn!("  - {}", issue);
            }
        }

        Err(anyhow!("Failed to find correct location after {} attempts", self.max_attempts))
    }
}
```

### 3. Explorer Agent: Location Finding

```rust
// Addition to src/agent/core.rs

impl Agent {
    /// Find potential bug locations (Explorer agent role)
    pub async fn find_bug_locations(
        &self,
        bug_description: &str,
        repository_path: &Path,
    ) -> Result<Vec<LocationCandidate>> {
        let prompt = format!(
            r#"You are a code investigator finding bug locations.

Bug Description:
{bug_description}

Repository: {repository_path}

Task: Find the most likely file(s) and line(s) where this bug exists.

IMPORTANT STEPS:
1. Use grep to search for relevant code patterns mentioned in bug description
2. Use read_file to examine candidate files
3. Identify the EXACT location (file + line number if possible)
4. Verify the code at that location matches the bug description

Return 1-3 location candidates ranked by confidence.

For each candidate provide:
- file_path: Exact path to file
- line_number: Line number if identifiable (or null)
- confidence: 0.0 to 1.0 (how sure are you?)
- reasoning: Why you think the bug is here

Output JSON array of candidates.
"#,
            bug_description = bug_description,
            repository_path = repository_path.display()
        );

        let response = self.execute_with_tools(&prompt).await?;

        // Parse JSON response into Vec<LocationCandidate>
        let candidates: Vec<LocationCandidate> = serde_json::from_str(&response)?;

        Ok(candidates)
    }

    /// Verify proposed patch targets correct location (Explorer agent role)
    pub async fn verify_patch_location(
        &self,
        proposal: &PatchProposal,
        bug_description: &str,
    ) -> Result<VerificationResult> {
        let prompt = format!(
            r#"You are a code reviewer verifying patch correctness.

Bug Description:
{bug_description}

Proposed Patch:
File: {file_path}
Line: {line_number}
Patch:
{patch}

Reasoning from patch author:
{reasoning}

Task: Verify this patch targets the CORRECT location.

VERIFICATION STEPS:
1. Read the target file: {file_path}
2. Check if line {line_number} contains the code mentioned in bug description
3. Verify the patch makes sense for this location
4. Confirm this is the ACTUAL bug location (not just related code)

Answer:
- is_correct: true/false
- reasoning: Explain your verification
- issues: List any problems if not correct (e.g., "File doesn't contain variable X", "Line is in documentation not implementation")

Output JSON.
"#,
            bug_description = bug_description,
            file_path = proposal.location.file_path.display(),
            line_number = proposal.location.line_number.map_or("unknown".to_string(), |n| n.to_string()),
            patch = proposal.patch,
            reasoning = proposal.reasoning,
        );

        let response = self.execute_with_tools(&prompt).await?;

        let verification: VerificationResult = serde_json::from_str(&response)?;

        Ok(verification)
    }
}
```

### 4. Builder Agent: Patch Generation

```rust
impl Agent {
    /// Generate patch for verified location (Builder agent role)
    pub async fn generate_patch(
        &self,
        bug_description: &str,
        location: &LocationCandidate,
    ) -> Result<PatchProposal> {
        let prompt = format!(
            r#"You are a code fixer generating patches.

Bug Description:
{bug_description}

Target Location (verified by investigator):
File: {file_path}
Line: {line_number}
Confidence: {confidence:.1}%
Reasoning: {reasoning}

Task: Generate a unified diff patch to fix this bug.

IMPORTANT:
1. Read the target file: {file_path}
2. Understand the context around line {line_number}
3. Generate MINIMAL patch (smallest change that fixes bug)
4. Output valid unified diff format

Provide:
- patch: Unified diff string
- reasoning: Explain what your patch does and why
"#,
            bug_description = bug_description,
            file_path = location.file_path.display(),
            line_number = location.line_number.map_or("unknown".to_string(), |n| n.to_string()),
            confidence = location.confidence * 100.0,
            reasoning = location.reasoning,
        );

        let response = self.execute_with_tools(&prompt).await?;

        // Parse response
        let proposal: PatchProposal = serde_json::from_str(&response)?;

        Ok(proposal)
    }
}
```

## Implementation Steps

### Day 1: Core Infrastructure (4-6 hours)

**Morning (2-3 hours): Data Structures**
- [ ] Create `src/agent/validation_loop.rs`
- [ ] Define structs: `LocationCandidate`, `PatchProposal`, `VerificationResult`, `ValidatedPatch`
- [ ] Define `ValidationLoopCoordinator` struct
- [ ] Add tests for struct serialization/deserialization

**Afternoon (2-3 hours): Agent Methods**
- [ ] Implement `find_bug_locations()` in `src/agent/core.rs`
- [ ] Implement `verify_patch_location()` in `src/agent/core.rs`
- [ ] Implement `generate_patch()` in `src/agent/core.rs`
- [ ] Add unit tests (mock responses)

### Day 2: Integration & Testing (4-6 hours)

**Morning (2-3 hours): Coordinator Implementation**
- [ ] Implement `ValidationLoopCoordinator::fix_bug_with_validation()`
- [ ] Wire into existing agent execution flow
- [ ] Add logging for debugging
- [ ] Integration tests with mock LLM

**Afternoon (2-3 hours): SWE-bench Validation**
- [ ] Re-run Task 0 (astropy) with validation loop
- [ ] Re-run Task 6 (django) with validation loop
- [ ] Compare: 0/2 (old) vs ?/2 (new)
- [ ] Document improvements in evaluation files

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_candidate_serialization() {
        let candidate = LocationCandidate {
            file_path: PathBuf::from("src/auth.rs"),
            line_number: Some(42),
            confidence: 0.95,
            reasoning: "Variable name matches bug description".to_string(),
        };

        let json = serde_json::to_string(&candidate).unwrap();
        let deserialized: LocationCandidate = serde_json::from_str(&json).unwrap();

        assert_eq!(candidate.file_path, deserialized.file_path);
    }

    #[test]
    fn test_validation_loop_max_attempts() {
        // Mock 3 failed verifications
        // Assert loop stops after max_attempts
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_validation_loop_with_mock_llm() {
    // Mock LLM responses:
    // 1. find_bug_locations returns 2 candidates
    // 2. First verification fails
    // 3. Second verification succeeds
    // Assert: Returns ValidatedPatch with attempts=2
}
```

### SWE-bench Validation

**Re-test with validation loop**:
- Task 0 (astropy__astropy-12907)
- Task 6 (django__django-10914)

**Measure**:
- Success rate: 0/2 → ?/2
- Attempts needed: How many locations tried?
- Time overhead: Validation adds ~30s per attempt?

## Prompt Engineering

### Critical Prompt Components

**1. Explicit Navigation Steps**:
```
IMPORTANT STEPS:
1. Use grep to search for relevant code patterns
2. Use read_file to examine candidate files
3. Identify EXACT location (file + line number)
4. Verify code matches bug description
```

**2. Verification Requirements**:
```
VERIFICATION STEPS:
1. Read the target file
2. Check if line contains code from bug description
3. Verify patch makes sense for this location
4. Confirm this is ACTUAL bug (not just related code)
```

**3. JSON Output Format**:
- Structured responses for parsing
- Clear field names
- Required vs optional fields

## Expected Outcomes

### Success Metrics

**Minimum success (acceptable)**:
- 1/2 SWE-bench tasks pass (50% improvement)
- Validation catches wrong locations
- Max 3 attempts needed per task

**Good success (target)**:
- 2/2 SWE-bench tasks pass (100% fix)
- Average 1.5 attempts per task
- Clear verification reasoning

**Excellent success (stretch)**:
- 2/2 SWE-bench + 2 more tasks = 4/4
- Validation loop generalizes to new tasks

### Failure Cases to Handle

**1. All candidates fail verification**:
- Log: "No correct location found in top 3 candidates"
- Fallback: Return best attempt with warning
- User can review and override

**2. Grep/read_file tools fail**:
- Retry with different search terms
- Fallback to semantic search
- Log errors for debugging

**3. LLM refuses to verify**:
- Timeout after 30s per verification
- Log refusal reason
- Try next candidate

## Integration with Existing Code

### Minimal Changes to Current Architecture

**New file**: `src/agent/validation_loop.rs` (~300 lines)
**Modified**: `src/agent/core.rs` (+150 lines for 3 new methods)
**Modified**: `src/agent/mod.rs` (export validation_loop module)

**No changes needed**:
- ✅ Existing tool infrastructure works as-is
- ✅ Session management unchanged
- ✅ Memory systems unchanged
- ✅ ACP protocol unchanged

### Wire Into SWE-bench Runner

```rust
// In swe_bench_runner.py or equivalent

// Old approach:
let patch = agent.execute(prompt).await?;

// New approach with validation:
let coordinator = ValidationLoopCoordinator::new(explorer, builder, 3);
let validated_patch = coordinator.fix_bug_with_validation(
    &task.problem_statement,
    &repository_path,
).await?;
```

## Risks & Mitigations

### Risk 1: Validation adds too much latency
**Impact**: 3 attempts × 30s = 90s overhead per task
**Mitigation**: Accept for now, optimize later (async validation, caching)

### Risk 2: Explorer agent also wrong
**Impact**: Garbage in → garbage out (both agents wrong)
**Mitigation**: Use different system prompts, different models for diversity

### Risk 3: LLM can't parse JSON reliably
**Impact**: Parsing errors break validation loop
**Mitigation**: Add retry with "fix your JSON" prompt, fallback to text parsing

## Success Criteria

**Must have**:
- [ ] Validation loop executes without crashes
- [ ] Can re-run SWE-bench tasks 0 and 6
- [ ] Success rate improves (0/2 → at least 1/2)

**Should have**:
- [ ] Clear logging of each validation step
- [ ] Verification reasoning makes sense
- [ ] Max 3 attempts respected

**Nice to have**:
- [ ] 2/2 SWE-bench success
- [ ] Pattern generalizes to other tasks
- [ ] <60s total overhead per task

## Files to Create/Modify

**New**:
- `src/agent/validation_loop.rs` (coordinator logic)
- `tests/validation_loop_test.rs` (comprehensive tests)

**Modified**:
- `src/agent/core.rs` (add 3 methods: find_locations, verify_patch, generate_patch)
- `src/agent/mod.rs` (export validation_loop)
- `Cargo.toml` (if adding dependencies)

**Documentation**:
- Update `ai/STATUS.md` with validation loop status
- Create evaluation files for re-tested SWE-bench tasks
- Document improvements in `docs/TECH_SPEC.md`

## Next Steps After Completion

**If successful (1-2/2)**:
- Test on 2 more SWE-bench tasks (get to 4 total)
- Document pattern in research paper
- Consider it validated approach

**If unsuccessful (0/2)**:
- Analyze why verification failed
- Try different model for Explorer vs Builder
- Consider different verification prompts

**Either way**:
- Proceed to Option C (Week 8 sub-agents)
- Validation loop is independent feature (keep or refine)
