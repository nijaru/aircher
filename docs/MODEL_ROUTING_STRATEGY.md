# Model Routing Strategy Analysis & Recommendations

⚠️ **DEPRECATED**: This file contains outdated model recommendations based on training data from Jan 2025.

**Current Plan**: See `ai/MODEL_CONFIG_PLAN.md` for up-to-date model routing strategy based on Oct 2025 models:
- **Current models**: claude-sonnet-4.5, claude-haiku-4.5, claude-opus-4.1 (Anthropic), GPT-5-Codex (OpenAI - released), gemini-2.5-pro/flash/flash-lite (Google)
- **Key change**: Sonnet 4.5 is better than Opus 4.1 for most/all tasks (use Sonnet 4.5 for 90%+ of work)
- **Provider flexibility**: Anthropic, OpenAI, Google, OpenRouter (unified gateway with exacto premium), Ollama
- **Configuration**: Smart task-based routing (default, zero config), single model override, provider-specific routing tables

---

**Status** (OUTDATED - Oct 29, 2025): Week 7 implementation review + recommendations for improvements

⚠️ The recommendations below reference outdated model names and should not be used. See `ai/MODEL_CONFIG_PLAN.md` for current plan.

## Current Configuration (As Implemented)

### Default Routing Table

**Current defaults** (from `src/agent/model_router.rs:223-329`):

| Agent Type | Low Complexity | Medium Complexity | High Complexity |
|------------|---------------|-------------------|-----------------|
| **Explorer** | Haiku | Sonnet 4 | Opus 4.1 |
| **Builder** | Sonnet 4 | Sonnet 4 | Opus 4.1 |
| **Debugger** | Sonnet 4 | Opus 4.1 | Opus 4.1 |
| **Refactorer** | Sonnet 4 | Sonnet 4 | Opus 4.1 |
| **Sub-agents** | Haiku | Haiku | Haiku |

**Fallback**: Sonnet 4 (line 359)

### Problems with Current Configuration

#### 1. **Incorrect Model Names** ❌
```rust
// Current (WRONG - these don't exist yet):
model: "claude-opus-4.1"      // Doesn't exist
model: "claude-sonnet-4"      // Should be "claude-sonnet-4-20250514"
model: "claude-haiku"         // Should be "claude-3-5-haiku-20241022"
```

**Actual Anthropic models** (as of Jan 2025):
- `claude-opus-4-20250514` - Latest Opus (released May 2024)
- `claude-sonnet-4-20250514` - Latest Sonnet (released May 2024)
- `claude-3-5-haiku-20241022` - Latest Haiku (Oct 2024)
- Legacy: `claude-3-opus-20240229`, `claude-3-5-sonnet-20241022`

#### 2. **Too Much Opus Usage** ⚠️
Current routing uses Opus for:
- High complexity Explorer (expensive for reading)
- High complexity Builder (maybe justified)
- Medium/High Debugger (very expensive)
- High Refactorer (expensive)

**Result**: 40% cost reduction target may be hard to hit

#### 3. **No Sonnet 3.7 Option** ⚠️
Sonnet 3.7 (Oct 2024) is faster and cheaper than Sonnet 4:
- $3/1M input vs $3/1M (same cost)
- But faster inference (computer use optimized)
- Good for tool-heavy tasks

#### 4. **No Provider Flexibility** ⚠️
Routing table hardcodes Anthropic only. Missing:
- OpenAI (GPT-4o, GPT-5-Codex when released)
- DeepSeek (70% cost savings)
- Ollama (free, local)
- Google Gemini 2.0

## Recommended Configuration

### Option 1: Conservative (Quality-First)

**Philosophy**: Use best model that makes sense for task, prioritize quality over cost

```rust
// Recommended defaults (prioritize Claude)
Explorer:
  Low:    claude-3-5-haiku-20241022    // Fast queries
  Medium: claude-sonnet-4-20250514     // Most analysis
  High:   claude-sonnet-4-20250514     // Even deep analysis (Opus rarely needed)

Builder:
  Low:    claude-sonnet-4-20250514     // Always use Sonnet for code generation
  Medium: claude-sonnet-4-20250514     // Most building tasks
  High:   claude-opus-4-20250514       // Only for complex architecture

Debugger:
  Low:    claude-sonnet-4-20250514     // Simple fixes
  Medium: claude-sonnet-4-20250514     // Most debugging (Opus overkill)
  High:   claude-opus-4-20250514       // Only for deep bugs

Refactorer:
  Low:    claude-sonnet-4-20250514     // All refactoring uses Sonnet
  Medium: claude-sonnet-4-20250514
  High:   claude-sonnet-4-20250514     // Rarely need Opus

Sub-agents (all):
  All:    claude-3-5-haiku-20241022    // Cheap parallelization
```

**Expected savings**: ~50% vs always Opus (more Sonnet, less Opus)

### Option 2: Cost-Optimized (Balance)

**Philosophy**: Use cheapest model that can handle task, accept some quality tradeoff

```rust
// Cost-optimized (more aggressive)
Explorer:
  Low:    claude-3-5-haiku-20241022    // Cheap queries
  Medium: claude-3-5-haiku-20241022    // Most analysis (Haiku is pretty good)
  High:   claude-sonnet-4-20250514     // Only complex needs Sonnet

Builder:
  Low:    claude-3-5-haiku-20241022    // Simple code gen (comments, docs)
  Medium: claude-sonnet-4-20250514     // Most building
  High:   claude-sonnet-4-20250514     // Rarely use Opus

Debugger:
  Low:    claude-3-5-haiku-20241022    // Syntax errors
  Medium: claude-sonnet-4-20250514     // Most bugs
  High:   claude-sonnet-4-20250514     // Avoid Opus unless desperate

Refactorer:
  Low:    claude-3-5-haiku-20241022    // Formatting, simple changes
  Medium: claude-sonnet-4-20250514     // Most refactors
  High:   claude-sonnet-4-20250514     // Keep it cheap

Sub-agents (all):
  All:    claude-3-5-haiku-20241022    // Always cheap
```

**Expected savings**: ~65% vs always Opus (lots more Haiku)

### Option 3: Multi-Provider (Flexibility)

**Philosophy**: Use best model from ANY provider based on task requirements

```rust
// Multi-provider routing
Explorer (read-only, no code gen):
  Low:    deepseek-chat (70% cheaper than Claude)
  Medium: claude-sonnet-4-20250514
  High:   claude-sonnet-4-20250514

Builder (code generation critical):
  Low:    claude-sonnet-4-20250514     // Claude best for code
  Medium: claude-sonnet-4-20250514
  High:   claude-opus-4-20250514 OR gpt-5-codex (when available)

Debugger (reasoning critical):
  Low:    claude-sonnet-4-20250514
  Medium: claude-sonnet-4-20250514
  High:   claude-opus-4-20250514 OR gemini-2.0-flash-thinking

Refactorer (code understanding):
  Low:    deepseek-coder (specialized, cheap)
  Medium: claude-sonnet-4-20250514
  High:   claude-sonnet-4-20250514

Sub-agents (cheap parallelization):
  All:    ollama/qwen2.5-coder (FREE!) OR claude-haiku
```

**Expected savings**: ~75% vs always Opus (use free/cheap where possible)

## Provider-Specific Recommendations

### Anthropic Claude (Primary)

**Use for**:
- ✅ All code generation (Builder agent)
- ✅ Complex reasoning (Debugger high complexity)
- ✅ Nuanced analysis (Explorer high complexity)

**Models**:
- **Sonnet 4** (`claude-sonnet-4-20250514`): **Default for 80% of tasks**
  - Best balance of quality/cost
  - Excellent at code generation
  - Good enough for most debugging
  - $3/$15 per 1M tokens (input/output)

- **Haiku 3.5** (`claude-3-5-haiku-20241022`): **Use for 15% of tasks**
  - Simple queries, read operations
  - Sub-agent parallelization (cheap)
  - Quick analysis tasks
  - $0.25/$1.25 per 1M tokens

- **Opus 4** (`claude-opus-4-20250514`): **Use for <5% of tasks**
  - Only for truly complex architecture decisions
  - Deep bug fixing with complex state
  - Novel algorithm design
  - $15/$75 per 1M tokens (5x more than Sonnet!)

### OpenAI (Secondary)

**Use for**:
- Multi-modal tasks (images, diagrams)
- JSON mode (structured output)
- GPT-5-Codex when released (code-specific)

**Models**:
- **GPT-4o** (`gpt-4o`): Similar to Sonnet in capability
  - $2.5/$10 per 1M tokens (slightly cheaper than Sonnet)
  - Good fallback if Claude unavailable

- **GPT-5-Codex** (when released): Watch for this
  - Code-specific model (potentially better than Sonnet for code)
  - Use for Builder agent if benchmarks show advantage

### DeepSeek (Cost Leader)

**Use for**:
- ✅ Read-only analysis (Explorer low/medium)
- ✅ Sub-agents (70% cheaper than Claude)
- ✅ High-volume tasks where cost matters

**Models**:
- **DeepSeek V3** (`deepseek-chat`):
  - $0.14/$0.28 per 1M tokens (95% cheaper than Claude!)
  - Reasonable quality for analysis
  - NOT recommended for code generation

- **DeepSeek Coder V3** (`deepseek-coder`):
  - Code-specialized
  - Good for refactoring analysis
  - $0.14/$0.28 per 1M tokens

### Ollama (Free Local)

**Use for**:
- ✅ Development testing (no API costs)
- ✅ Sub-agents if acceptable latency
- ✅ Privacy-sensitive codebases

**Models**:
- **qwen2.5-coder**: Best coding model
  - Tool calling support
  - Good for sub-agents
  - FREE (local execution)

- **deepseek-r1**: Reasoning-focused
  - Free alternative to Opus
  - Slower but free
  - Good for debugging

### Google Gemini (Experimental)

**Use for**:
- Thinking mode (Gemini 2.0 Flash Thinking)
- Very long context (2M tokens)
- Multimodal analysis

**Models**:
- **Gemini 2.0 Flash Thinking**: New reasoning model
  - Good for complex debugging
  - Potentially cheaper than Opus
  - Watch benchmarks

### Others Worth Watching

- **GLM-4.6**: Chinese market, cheap
- **Kimi K2**: Long context (200K+ tokens)
- **Alibaba Qwen**: Strong coding performance

## Claude Max / Pro OAuth Support

**Question**: Can we use Claude Max subscriptions like OpenCode?

**Answer**: **Partially possible** with caveats:

### What OpenCode Does

OpenCode authenticates to `claude.ai` via OAuth2 PKCE flow:
1. Opens browser to `https://claude.ai/authorize`
2. User logs in with Claude Max/Pro account
3. Gets OAuth token for API access
4. Uses token for 5 concurrent Claude requests

**Advantages**:
- Uses user's existing Claude Max subscription ($20/month unlimited)
- No separate API key needed
- Higher rate limits than free API

**Disadvantages**:
- Unofficial API (not guaranteed stable)
- Terms of service gray area (technically consumer product, not API)
- Rate limits still apply (5 concurrent, slower than API)

### Implementation for Aircher

**Option 1: Direct OAuth (Like OpenCode)**

Pros:
- Users can use Claude Max subscription
- No API key setup needed
- Cheaper for heavy users ($20/month vs $100s in API costs)

Cons:
- Violates Claude TOS (API meant for this, not consumer product)
- Risk of account suspension
- Unofficial API may break
- Slower than official API

**Recommendation**: **Add as optional provider, warn about TOS**

```rust
// Example configuration
[providers.anthropic_oauth]
enabled = false  # Default off
auth_method = "oauth2_pkce"
base_url = "https://claude.ai/api"
warning = "Using Claude consumer account may violate Terms of Service"
```

**Option 2: Official API Only (Recommended)**

Pros:
- Complies with TOS
- Stable, supported API
- Fast, reliable
- No risk of suspension

Cons:
- Users pay per-token (can get expensive)
- Separate from Claude Max subscription

**Recommendation**: **Default to official API, document alternatives**

## Configuration Recommendations

### Default Strategy (Recommended for v0.1.0)

**Single model default**: `claude-sonnet-4-20250514`
- Use for everything unless user configures routing table
- Simple, predictable, good quality
- Let users opt into cost optimization

**Configuration**:
```toml
[model]
# Simple default (single model for everything)
default_model = "claude-sonnet-4-20250514"
default_provider = "anthropic"

# Optional: Enable smart routing
smart_routing = false  # Off by default

# Optional: Override per agent type
[model.overrides]
# builder = "claude-opus-4-20250514"  # Uncomment to use Opus for building
# explorer_low = "claude-3-5-haiku-20241022"  # Uncomment for cheap exploration
# sub_agents = "ollama/qwen2.5-coder"  # Uncomment to use free local model
```

### Advanced Strategy (Power Users)

**Enable smart routing**:
```toml
[model]
default_model = "claude-sonnet-4-20250514"  # Fallback
smart_routing = true  # Enable cost-aware routing

# Routing strategy: "conservative" | "balanced" | "aggressive"
routing_strategy = "balanced"

# Conservative: Quality-first (Sonnet default, Opus for hard tasks)
# Balanced: Cost-aware (more Haiku, less Opus) - RECOMMENDED
# Aggressive: Cost-optimized (lots of Haiku, Opus rare)

[model.providers]
# Enable multiple providers
anthropic = true
openai = true
deepseek = true
ollama = true  # Free local models

[model.provider_preferences]
# Prefer Claude for code generation
builder = ["anthropic"]

# Allow cheaper providers for analysis
explorer = ["deepseek", "anthropic"]

# Use free local for sub-agents
sub_agents = ["ollama", "anthropic"]
```

## Implementation Priorities

### Immediate (Before v0.1.0)

1. **Fix model names** ✅ CRITICAL
   - Update to actual Anthropic model names
   - Test with current API
   - Ensure backward compatibility

2. **Reduce Opus usage** ✅ IMPORTANT
   - Change Debugger medium → Sonnet (not Opus)
   - Change Explorer high → Sonnet (not Opus)
   - Reserve Opus for <5% of tasks

3. **Add provider override support** ✅ IMPORTANT
   - Allow user to specify provider in config
   - Support per-agent provider preferences
   - Document in user guide

### Short-term (v0.2.0)

1. **Add DeepSeek provider**
   - 95% cost savings for analysis tasks
   - Good quality for read-only operations
   - Easy win for cost optimization

2. **Multi-provider routing**
   - Use best provider for each task type
   - DeepSeek for analysis, Claude for code gen
   - Document cost savings

3. **Ollama integration improvements**
   - Better tool calling support
   - Sub-agent optimization
   - Local-first option for privacy

### Medium-term (v0.3.0)

1. **Claude Max OAuth support** (optional)
   - Add as experimental provider
   - Warn about TOS implications
   - Let users decide

2. **GPT-5-Codex support** (when released)
   - Test vs Claude Sonnet for code generation
   - Add to Builder agent if better
   - Document benchmarks

3. **Gemini 2.0 Thinking mode**
   - Test for complex debugging
   - Potentially cheaper than Opus
   - Add as Debugger option

## Cost Analysis

### Current Routing (Needs Fix)

Assume 100 tasks:
- 40% Explorer (lots of Opus currently)
- 30% Builder (lots of Opus)
- 20% Debugger (lots of Opus)
- 10% Refactorer (some Opus)

**Estimated cost**: ~$80-120 per 100M tokens (too much Opus)

### Recommended "Balanced" Routing

Same 100 tasks:
- 80% use Sonnet ($3/$15 per 1M)
- 15% use Haiku ($0.25/$1.25 per 1M)
- 5% use Opus ($15/$75 per 1M)

**Estimated cost**: ~$35-50 per 100M tokens (60% savings vs current!)

### Aggressive Multi-Provider

Same 100 tasks:
- 40% use DeepSeek ($0.14/$0.28 per 1M)
- 40% use Claude Sonnet
- 15% use Ollama (free)
- 5% use Claude Opus

**Estimated cost**: ~$15-25 per 100M tokens (75% savings!)

## Summary & Action Items

### Current Problems
1. ❌ Model names incorrect (don't exist in API)
2. ❌ Too much Opus usage (expensive)
3. ❌ No provider flexibility
4. ❌ No single-model default option

### Recommended Defaults
- **Single model**: `claude-sonnet-4-20250514` (use for everything by default)
- **Smart routing off by default** (let users opt in)
- **When routing enabled**: Use "balanced" strategy (80% Sonnet, 15% Haiku, 5% Opus)
- **Provider priority**: Claude > OpenAI > DeepSeek > Ollama

### Quick Wins
1. Fix model names (5 min)
2. Reduce Opus usage (10 min)
3. Add single-model default config (15 min)
4. Document cost analysis (30 min)

### For Future
- Add DeepSeek provider (70% cost savings)
- Add Claude Max OAuth (optional, with warnings)
- Add GPT-5-Codex when released
- Test Gemini 2.0 Thinking for debugging

**Bottom line**: Use Claude Sonnet 4 for 80% of tasks, Haiku for simple stuff, Opus only when truly needed. This hits 60% cost savings while maintaining quality.
