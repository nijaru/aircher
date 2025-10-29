# Toad TUI Frontend Research

**Purpose**: Understand Toad's capabilities as a potential universal frontend for Aircher via ACP

**Updated**: 2025-10-29

## Overview

**Toad** is a universal terminal UI for agentic coding built by Will McGugan (creator of Rich and Textual Python libraries). It aims to provide a jank-free, high-quality terminal interface that is provider-agnostic and works with any AI coding backend.

**Source**: https://willmcgugan.github.io/announcing-toad/

**Status** (as of 2025-10-29):
- **Private preview**: Available via $5,000/month GitHub sponsorship (https://github.com/sponsors/willmcgugan)
- **Will become open source**: Eventually
- **Actively developed**: Multiple progress reports published

## Creator: Will McGugan

**Background**:
- Creator of **Rich** (Python library for rich text and beautiful formatting in terminal)
- Creator of **Textual** (sophisticated TUI framework for Python)
- Former CEO of Textualize (startup promoting rich terminal applications)
- 5+ years obsessing over terminal interfaces
- Expert in terminal rendering techniques

**Motivation for Toad**:
Will built Toad in response to Claude Code and Gemini CLI, both of which he found technically lacking despite their AI capabilities. Built the initial prototype "across two afternoons in a nerdy caffeinated rage while listening to metal music" to demonstrate how terminal coding agents should be built.

## Technical Foundation

**Framework**: Textual (Python TUI framework)
- Mature library with thriving community
- Battle-tested terminal rendering
- Solves problems that Node-based tools (Claude Code, Gemini CLI) struggle with

**Language**: Python
- Installation via `uvx` (similar UX to `npx` for Node)
- Example: `uvx toad` (when released)

**Architecture**: Universal frontend (provider-agnostic)
- Not tied to specific AI backend
- Positions as "universal UI" for agentic coding
- Works with multiple providers (Anthropic, Google, OpenAI, etc.)

## Core Features (Verified from Demos)

### 1. Prompt Input System

**Advanced Text Editing**:
- Multi-line text input with full editing capabilities
- Text selection with both mouse and cursor
- Standard operations: cut, copy, paste, undo, redo
- Follows conventions from existing agentic coding tools (familiar UX)

**Markdown Highlighting**:
- Real-time Markdown syntax highlighting
- Code fence support with language-specific highlighting
- Supports "a variety of different languages" in code fences
- Visual indication of markdown structure

**Shell Mode**:
- Triggered by "!" or "$" prefix
- Changes prompt symbol and highlight color
- Clear visual distinction between AI prompt mode and shell mode
- "Blessed commands" that auto-trigger shell mode (configurable)

**Future Enhancements** (planned for code fences):
- Smart indentation
- LSP support (autocomplete, diagnostics, etc.)

### 2. Display & Rendering Quality

**No Flicker** (Major Advantage):
- Partial region updates as small as a single character
- **Problem with Node-based tools**: Claude Code and Gemini CLI rewrite entire terminal output
  - Even changing a single line requires removing previous lines and writing new output
  - Expensive operation, high likelihood of seeing partial frame = flicker
- **Toad's Solution**: Update only the changed regions
- Result: Smooth, jank-free experience

**Scrollback Interaction**:
- Scroll back up to view previous output
- Interact with previously written content
- Copy un-garbled output even if cropped
- **Problem with Node-based tools**: Cannot interact with scrollback

**Terminal Optimization**:
- 5+ years of Textual solving terminal rendering problems
- Addresses "jank and other glitches inherent to terminals"
- Light-weight and snappy
- Preserves muscle memory from existing tools

**Definition of "Jank"** (from Will):
> "anything happening in the UI which is unpleasant to the user"

## Feature Comparison: Toad vs Node-Based Tools

| Feature | Toad (Textual) | Claude Code / Gemini CLI (Node) |
|---------|----------------|----------------------------------|
| **Flicker** | ❌ None (partial updates) | ✅ Yes (full screen rewrites) |
| **Scrollback** | ✅ Interactive, copyable | ❌ Cannot interact |
| **Update Granularity** | Single character | Entire output |
| **Terminal Performance** | Optimized (5+ years) | Expensive operations |
| **Installation** | `uvx toad` | `npx gemini-cli` |
| **UX Polish** | Jank-free | "Jank and other glitches" |

## ACP Integration Status

**Evidence Level**: ⚠️ INFERRED (not explicitly documented, but likely)

**Why ACP Support is Likely**:
1. **Positioning**: Toad is described as "universal UI" and "provider-agnostic"
2. **Timing**: Announced July 2025, shortly after ACP launch (February 2025)
3. **Ecosystem**: ACP is gaining adoption across editors (Zed, Neovim, Emacs)
4. **Will's Focus**: Universal frontend suggests protocol-based backend communication
5. **Community Context**: Toad appears in searches alongside ACP discussions

**What We Don't Know**:
- Whether ACP support is already implemented or planned
- Timeline for ACP integration (if not already done)
- Whether Toad uses ACP natively or can support multiple protocols

**Action Item**: When Toad becomes open source or documentation is released, verify ACP support and integration details.

## Toad's Development Progress (Public Reports)

### Toad Report #1 (August 28, 2025)
**Source**: https://willmcgugan.github.io/toad-report-1/

**Focus**: Prompt input implementation

**Features Documented**:
- Text area with mouse and cursor selection
- Cut, copy, paste, undo, redo
- Markdown highlighting
- Code fence highlighting for multiple languages
- Shell mode with "!" or "$" triggers
- Blessed commands for auto-shell mode
- Future: Smart indentation, LSP support for code fences

**Quote**: "Toad is intended to be a _universal_ front-end to AI services, which is entirely agnostic to the models and providers, while providing an altogether more humane user-experience."

## What This Means for Aircher

### Advantages of Using Toad as Frontend

**1. Zero UI Development** (4-6 week savings):
- No need to build custom Ratatui TUI
- Focus entirely on agent intelligence (our value-add)
- Professional-quality UI from terminal expert

**2. Universal Reach**:
- Not just Toad users, but any ACP-compatible frontend
- Zed, Neovim, Emacs, JetBrains (when ready), marimo notebooks
- Multiple frontends = larger potential user base

**3. Quality Assurance**:
- Will McGugan has 5+ years optimizing terminal rendering
- Solves problems we'd encounter building custom TUI
- Battle-tested Textual framework

**4. Provider-Agnostic Architecture**:
- Toad designed to work with any backend
- Aircher's ACP-native design aligns perfectly
- Swappable backend = competitive advantage

### Potential Challenges

**1. Private Preview Timing**:
- Currently $5,000/month sponsorship required
- No public release date announced
- May need to wait for open source release

**2. ACP Support Unclear**:
- Not explicitly documented (yet)
- May need to verify integration approach
- Could require adapter layer if Toad uses different protocol

**3. Python Dependency**:
- Toad is Python (via Textual)
- Aircher is Rust
- Not a technical problem (ACP is language-agnostic), but deployment consideration

**4. Feature Parity**:
- Need to verify Toad supports all features Aircher needs
- Approval workflows, tool status updates, streaming responses
- May need to contribute features to Toad (if open source)

### Recommended Strategy for Aircher

**Phase 1: ACP-First Development** (Current)
- ✅ Build Aircher as ACP-native backend
- ✅ Ensure full ACP protocol compliance
- ✅ Test with existing ACP frontends (Zed, Neovim, Emacs)

**Phase 2: Monitor Toad Progress** (Ongoing)
- ⏳ Watch for Toad open source release announcement
- ⏳ Verify ACP support when documentation available
- ⏳ Test Aircher integration once Toad is accessible

**Phase 3: Toad Integration** (When Available)
- 🎯 Test Aircher with Toad frontend
- 🎯 Verify all features work (streaming, tools, approvals)
- 🎯 Contribute to Toad if needed (bug fixes, feature requests)
- 🎯 Document Toad as recommended frontend

**Phase 4: Multi-Frontend Support** (Long-term)
- 🎯 Position Aircher as "works everywhere" via ACP
- 🎯 Toad for terminal users
- 🎯 Zed/JetBrains for IDE users
- 🎯 Neovim/Emacs for editor users

### Features We Should Ensure Work in Any Frontend

**From Toad's demonstrated capabilities**:
1. **Markdown output**: Aircher responses should use Markdown
2. **Code fences**: Use code blocks for code snippets
3. **Streaming**: Progressive response updates (Toad likely supports)
4. **Tool status**: Clear indication when tools are running
5. **Shell commands**: Distinguish between AI prompts and shell execution
6. **Scrollback**: Previous conversation should be accessible

**ACP Features We Need to Support**:
1. **Streaming notifications**: Text, ToolStart, ToolProgress, ToolComplete, Thinking
2. **Approval workflows**: For dangerous operations
3. **Artifacts**: Code generation, file edits
4. **Session management**: Multi-session support
5. **Error handling**: Graceful degradation

## Questions to Answer When Toad Becomes Available

1. **ACP Support**: Does Toad use ACP natively, or a different protocol?
2. **Feature Completeness**: Does Toad support approval workflows, artifacts, streaming?
3. **Extensibility**: Can we contribute features to Toad if needed?
4. **Performance**: How does Toad handle large outputs, long sessions?
5. **Configuration**: Can users customize Toad behavior (themes, keybindings, etc.)?
6. **Deployment**: How easy is it for users to install and use Toad?
7. **Documentation**: Will Toad provide integration guides for backend developers?

## Current ACP Frontend Landscape

**From research** (Zed ACP Progress Report, Feb 2025):

**Available Now**:
- **Neovim**: CodeCompanion, avante.nvim plugins
- **Emacs**: agent-shell plugin
- **marimo**: Python notebook integration
- **Zed**: Native ACP support (first implementation)

**In Progress** (as of Feb 2025):
- JetBrains IDEs collaboration
- VS Code adapter (via ACP bridge)

**Toad Status**: Announced July 2025, private preview

**Implication**: By the time Aircher is ready (Week 10+), ACP ecosystem will be more mature, potentially including Toad.

## Technical Details for Integration

### What Aircher Needs to Provide (ACP Agent)

```rust
// Already implemented in src/agent/core.rs
pub trait Agent {
    async fn initialize(&self, request: InitializeRequest) -> Result<InitializeResponse>;
    async fn new_session(&self, request: NewSessionRequest) -> Result<NewSessionResponse>;
    async fn prompt(&self, request: PromptRequest) -> Result<PromptResponse>;
    async fn cancel(&self, request: CancelRequest) -> Result<CancelResponse>;
    async fn get_sessions(&self, request: GetSessionsRequest) -> Result<GetSessionsResponse>;
    async fn delete_session(&self, request: DeleteSessionRequest) -> Result<DeleteSessionResponse>;
}
```

### What Toad Would Provide (ACP Client)

```python
# Hypothetical Toad ACP client (if using ACP)
from acp import AgentClient

class ToadUI:
    def __init__(self, agent_command: str):
        # Launch Aircher agent via ACP
        self.client = AgentClient(command="cargo run --release -- --acp")
        self.client.initialize(...)

    async def send_prompt(self, text: str):
        # User types in Toad's prompt input
        response = await self.client.prompt(session_id=..., content=[text])

        # Display streaming response with Textual widgets
        async for chunk in response.stream():
            self.display_markdown(chunk.text)
            if chunk.tool_status:
                self.show_tool_status(chunk.tool_status)
```

### Integration Checklist

When Toad becomes available, verify:
- [ ] Can launch Aircher via `--acp` flag
- [ ] Streaming responses display correctly
- [ ] Markdown rendering works (code fences, lists, etc.)
- [ ] Tool execution status visible
- [ ] Approval workflows functional (if Toad supports)
- [ ] Multi-session support working
- [ ] Error messages displayed clearly
- [ ] Performance acceptable (no lag, smooth scrolling)
- [ ] Configuration options available

## Conclusion

**Toad is a high-quality, professionally-built terminal frontend** that aligns perfectly with Aircher's ACP-native architecture. Built by a terminal UI expert with 5+ years of experience, it solves the exact problems (flicker, jank, poor UX) that plague Node-based alternatives like Claude Code and Gemini CLI.

**For Aircher**:
- **Saves 4-6 weeks** of custom TUI development
- **Professional quality** from day one (Will McGugan's expertise)
- **Universal reach** via ACP (Toad + Zed + Neovim + Emacs + JetBrains)
- **Focus on intelligence** (our competitive advantage) instead of UI

**Timeline Strategy**:
1. **Now - Week 10**: Build ACP-native Aircher, test with Zed/Neovim/Emacs
2. **Week 10+**: Monitor Toad open source release
3. **When available**: Integrate and test with Toad
4. **Position**: "Best-in-class agent intelligence, works in your favorite frontend"

**Key Insight**: By building Aircher as an ACP-native backend, we're not betting on Toad specifically—we're betting on the ACP ecosystem, which is growing rapidly across multiple frontends. Toad is one excellent option among many.

## References

1. **Announcing Toad**: https://willmcgugan.github.io/announcing-toad/
2. **Toad Report #1**: https://willmcgugan.github.io/toad-report-1/
3. **Simon Willison's Summary**: https://simonwillison.net/2025/Jul/23/announcing-toad/
4. **Elite AI Assisted Coding Talk**: https://elite-ai-assisted-coding.dev/p/toad-will-mcgugan
5. **Textual Framework**: https://github.com/Textualize/textual
6. **Rich Library**: https://github.com/Textualize/rich
7. **ACP Progress Report**: https://zed.dev/blog/acp-progress-report
8. **GitHub Sponsors** (private preview): https://github.com/sponsors/willmcgugan

## Next Steps

1. **Continue ACP Development**: Finish Week 7-8 implementation
2. **Test with Available Frontends**: Zed, Neovim, Emacs integration
3. **Monitor Toad Updates**: Watch for open source release announcement
4. **Verify ACP in Benchmarks**: Ensure ACP interface works for Terminal-Bench/SWE-bench
5. **Document Multi-Frontend Support**: Aircher positioning as universal backend
