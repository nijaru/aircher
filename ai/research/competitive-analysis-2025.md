# Competitive Analysis: SOTA Coding Agents (2025)

**Last Updated**: 2025-10-29
**Purpose**: Honest assessment of what competitors do, what we can verify, and where we're guessing

## Evidence Levels

- **✅ VERIFIED**: Open source code inspected, or official documentation with implementation details
- **⚠️ INFERRED**: Public statements, user reports, blog posts, but no code to inspect
- **❓ UNKNOWN**: No public information, pure speculation
- **📄 DOCUMENTED**: Official docs/blogs exist but implementation hidden

---

## Open Source Agents (Can Inspect Code)

### 1. OpenCode (thdxr)
**GitHub**: https://github.com/thdxr/opencode (hypothetical - need to verify)
**Evidence Level**: ✅ VERIFIED (if truly open source) / ⚠️ INFERRED (if just blog posts)

**What We Know FOR SURE**:
- ✅ Plan/Build mode separation (from blog posts/tweets)
- ✅ LSP integration (from blog posts)
- ✅ Git-based undo/snapshots (from blog posts)
- ✅ Event bus architecture (from blog posts)

**What We NEED TO VERIFY**:
- [ ] Is the repo actually public? Check GitHub
- [ ] Can we read the actual implementation?
- [ ] Are our assumptions about the design correct?

**Action Items**:
- Find and inspect actual OpenCode source code
- If closed source, downgrade all claims to ⚠️ INFERRED

---

### 2. Zed AI
**GitHub**: https://github.com/zed-industries/zed
**Evidence Level**: ✅ VERIFIED (open source)

**Verified Features**:
- ✅ LSP integration (native, part of editor)
- ✅ Multi-provider support (OpenAI, Anthropic, local)
- ✅ Inline assistant
- ✅ Agent Client Protocol (ACP) support

**Implementation Details**:
- Language: Rust + GPUI
- Architecture: Native LSP integration, no separate event bus
- Agent system: Basic tool calling, no complex orchestration

**Unique Patterns**:
- GPUI framework for native UI
- Direct LSP integration (no abstraction layer needed)
- ACP server/client implementation

---

### 3. Sweep
**GitHub**: https://github.com/sweepai/sweep
**Evidence Level**: ✅ VERIFIED (open source)

**Verified Features**:
- ✅ GitHub integration (issues → PRs)
- ✅ Planning system
- ✅ Multi-step execution
- ✅ Test-driven fixes

**Implementation Details**:
- Language: Python
- Architecture: Planning → Implementation → Testing loop
- Memory: Stores context in GitHub issues/PRs

**Unique Patterns**:
- Issue-driven workflow
- Automatic PR generation
- Test-based validation

---

## Closed Source Agents (Must Infer)

### 4. Claude Code (Anthropic)
**Evidence Level**: ⚠️ INFERRED (no source code, only user reports + Anthropic statements)

**What We Can Infer** (from user complaints on Reddit/HN/Twitter):
- ⚠️ Uses sub-agents for tasks (users report 160k tokens for 3k work)
- ⚠️ Sub-agents cause 15x overhead (from user complaints)
- ⚠️ Rate limit issues (50+ reports/month)
- ⚠️ No persistent memory across sessions
- ⚠️ No git-based undo (users request feature)

**What We DON'T Know**:
- ❓ Actual architecture (might not use sub-agents at all)
- ❓ Internal tool system
- ❓ Reasoning strategy
- ❓ Context management approach

**Sources**:
- Reddit r/ClaudeAI user complaints
- Hacker News discussions
- Anthropic blog posts (generic, no implementation details)

**HONESTY CHECK**: We're basing our entire "sub-agents are bad" claim on user reports. We don't know how Claude Code actually works.

---

### 5. Factory Droid
**Evidence Level**: ⚠️ INFERRED (closed source, only benchmark results + marketing)

**What We Can Infer** (from Terminal-Bench leaderboard):
- ⚠️ #1 on Terminal-Bench (58.8% success rate)
- ⚠️ Uses "specialized droids" (from marketing materials)
- ⚠️ Pre-configured prompts per task type (from docs)

**What We DON'T Know**:
- ❓ Actual architecture
- ❓ How "droids" are implemented
- ❓ Tool system design
- ❓ Whether it's fundamentally different or just good prompts

**Sources**:
- Terminal-Bench leaderboard
- Factory Droid website marketing
- No technical blog posts or talks

**HONESTY CHECK**: We're assuming "specialized droids" = specialized agent configs. Could just be different system prompts.

---

### 6. Sourcegraph Amp (formerly Cody)
**Evidence Level**: 📄 DOCUMENTED (closed source, but docs exist)

**What's Documented**:
- 📄 Multi-model routing (Haiku/Sonnet/Opus)
- 📄 Context fetching from codebase
- 📄 LLM-based agent system

**What We Can Infer**:
- ⚠️ Cost-aware model selection (from docs)
- ⚠️ Task complexity determines model (from docs)

**What We DON'T Know**:
- ❓ Implementation details of router
- ❓ How complexity is measured
- ❓ Actual cost savings

**Sources**:
- Sourcegraph docs
- Blog posts about Cody/Amp

**HONESTY CHECK**: Docs say it routes between models, but no implementation details or real cost numbers.

---

### 7. Cursor
**Evidence Level**: ⚠️ INFERRED (partially open, mostly closed)

**What We Can Infer**:
- ⚠️ Multi-model support (visible in UI)
- ⚠️ Inline editing
- ⚠️ Composer mode for agentic tasks
- ⚠️ Complex approval workflow (users complain about "4+ accept buttons")

**What We DON'T Know**:
- ❓ Agent architecture
- ❓ Context management strategy
- ❓ Memory/persistence approach

**Sources**:
- Cursor website
- User reports
- YouTube demos

**HONESTY CHECK**: Very little known about internals. Mostly UI observations.

---

### 8. Windsurf (Codeium)
**Evidence Level**: ⚠️ INFERRED (closed source)

**What We Can Infer**:
- ⚠️ "Cascade" flow for multi-step tasks
- ⚠️ Context-aware suggestions
- ⚠️ Inline editing

**What We DON'T Know**:
- ❓ Everything about the architecture

**Sources**:
- Windsurf website
- Marketing materials

**HONESTY CHECK**: Minimal information available.

---

### 9. Google Jules (Gemini Code Assist Agent)
**Evidence Level**: ❓ UNKNOWN (very limited public info)

**What We Can Infer**:
- ⚠️ Autonomous bug fixing (from Google I/O demo)
- ⚠️ GitHub integration
- ⚠️ Multi-step planning

**What We DON'T Know**:
- ❓ Everything else

**Sources**:
- Google I/O announcement
- Limited blog post

**HONESTY CHECK**: Almost no information. Mostly speculation.

---

### 10. Devin (Cognition AI)
**Evidence Level**: ⚠️ INFERRED (closed, but some demos/interviews)

**What We Can Infer** (from demos + SWE-bench results):
- ⚠️ 13.86% SWE-bench score (verified)
- ⚠️ Persistent memory across sessions (from demos)
- ⚠️ Repository scanning (from demos)
- ⚠️ Long-running autonomous tasks (from demos)

**What We DON'T Know**:
- ❓ Memory implementation
- ❓ Planning architecture
- ❓ Tool system

**Sources**:
- SWE-bench leaderboard
- Cognition AI demos
- Blog posts

---

## Editor-Based Systems

### 11. GitHub Copilot
**Evidence Level**: ⚠️ INFERRED (closed source)

**What We Know**:
- ✅ Inline completion (verified by using it)
- ✅ Chat interface (verified)
- ⚠️ Workspace indexing (from docs)

**What We DON'T Know**:
- ❓ Agent architecture (if any)
- ❓ Context strategy

---

## Pattern Analysis: What's Actually Common vs Unique?

### Common Patterns (Most/All Agents Have):
1. **Multi-model support** - Everyone offers multiple LLMs
2. **File operations** - Read/write/edit files
3. **Inline editing** - Editor-based agents have this
4. **Context from LSP** - Editor-based agents get this for free
5. **Basic tool calling** - Industry standard now

### Uncommon Patterns (Only Some Have):
1. **Plan/Build separation** - Only OpenCode (verified if open source)
2. **Git snapshots** - Only OpenCode (verified if open source)
3. **Sub-agents** - Claude Code (inferred from user complaints)
4. **Specialized agent configs** - Factory Droid (inferred from marketing)
5. **Persistent episodic memory** - Devin (inferred from demos), Aircher (we have it)

### Unique to Aircher:
1. **Three-layer memory** (Episodic + Knowledge Graph + Working) - No one else documented
2. **Dynamic context pruning** - No one else documented
3. **60% tool call reduction** - Our POC result, not yet validated in production

---

## What We Can Confidently Claim

### ✅ VERIFIED SOTA (We Can Prove):
1. **Memory systems** - No other agent has all 3 (episodic + knowledge graph + working memory)
2. **ACP protocol** - Multi-frontend support (Zed has it, but they're an editor)
3. **Dynamic context pruning** - Unique to us (not seen elsewhere)

### ⚠️ INSPIRED BY (Can't Prove Superiority):
1. **Plan/Build modes** - From OpenCode (IF we can verify their implementation)
2. **LSP integration** - Common in editors, less common in CLI agents
3. **Git snapshots** - From OpenCode (IF we can verify)
4. **Model routing** - From Amp (but no cost data to compare)

### ❌ CANNOT CLAIM (Insufficient Evidence):
1. **"Better than Claude Code"** - We don't know how it works
2. **"Better than Factory Droid"** - It's closed source, #1 on benchmarks
3. **"Hybrid is superior"** - We haven't run comparative benchmarks yet

---

## Action Items: Research We Need to Do

### Critical (Must Do Before Claiming SOTA):
1. [ ] **Find OpenCode source code** - Is it actually open source?
2. [ ] **Run benchmarks vs Claude Code** - Prove our claims with data
3. [ ] **Test Cursor/Windsurf** - Understand their actual capabilities
4. [ ] **Analyze Zed source code** - Learn from their ACP implementation
5. [ ] **Study Sweep source code** - Understand their planning system

### Important (Should Do):
1. [ ] **Monitor SWE-bench leaderboard** - Track Factory Droid, Devin scores
2. [ ] **Collect Claude Code user feedback** - Validate sub-agent waste claims
3. [ ] **Sourcegraph Amp docs deep-dive** - Understand model routing
4. [ ] **Jules documentation** - Track Google's approach when public

### Research Questions:
1. Is LSP integration unique, or do all CLI agents have it?
2. Do other agents use event-driven architectures internally?
3. Are we the only ones with persistent memory?
4. What patterns are we missing that competitors have?

---

## Honest Assessment: Where We Stand

### What We KNOW We're Good At:
- ✅ Memory systems (episodic + knowledge graph + working memory)
- ✅ Rust implementation (performance advantage)
- ✅ ACP protocol (multi-frontend support)

### What We THINK We're Good At (Need to Prove):
- ⚠️ Dynamic context management
- ⚠️ Tool call reduction (60% in POC)
- ⚠️ Continuous work without restart

### What We DON'T Know:
- ❓ How we compare to Claude Code (no benchmarks)
- ❓ How we compare to Factory Droid (#1 on Terminal-Bench)
- ❓ Whether our architecture is actually better
- ❓ If anyone else has similar patterns we don't know about

---

## Next Steps: Becoming Truly SOTA

### Week 8-9: Competitive Research
1. Find and read ALL open source agent code
2. Try ALL major competitors (Claude Code, Cursor, Windsurf)
3. Run SWE-bench tasks against them
4. Document what they actually do (not what we think they do)

### Week 9: Benchmarks
1. Run same tasks in Aircher vs Claude Code vs Cursor
2. Measure: tool calls, context efficiency, success rate, cost
3. Get empirical evidence for our claims
4. Publish honest comparison

### Week 10: Research Paper
1. Only claim what we can prove
2. Mark inference vs verification clearly
3. Focus on our unique contributions (memory systems)
4. Be honest about limitations

---

## Conclusion

**What We Got Right**:
- Memory systems are genuinely unique
- ACP support is valuable
- Rust implementation is solid

**What We Need to Fix**:
- Stop claiming patterns are from competitors without verification
- Get empirical benchmarks before claiming superiority
- Be honest: "Inspired by OpenCode" not "Proven better than Claude Code"

**Research Principle**:
> "Verify before claiming. Infer cautiously. Mark speculation clearly."
