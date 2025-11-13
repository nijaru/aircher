# Managing Reference Repositories

## Current References

- **Zed** (`/Users/nick/github/zed-industries/zed`) - ACP implementation reference
- **agent-contexts** (submodule) - Development patterns and standards

## Management Strategy

### For Heavy References (like Zed)
Use `gh` commands for on-demand access:
```bash
# Clone when needed
gh repo clone zed-industries/zed ../zed-industries/zed

# Sync to latest
cd ../zed-industries/zed && git pull

# Search for patterns
rg "ACP" ../zed-industries/zed/crates/
```

**Pros**:
- No repo bloat
- Always latest when pulled
- Can delete when not needed

**Cons**:
- Need to clone first time
- Not versioned with our code

### For Lightweight References (like agent-contexts)
Keep as submodules:
```bash
# Already added as submodule
git submodule update --init --recursive

# Update to latest
git submodule update --remote external/agent-contexts
```

**Pros**:
- Versioned with our code
- Always available
- Small size (documentation only)

**Cons**:
- Needs submodule management
- Can get out of sync

## Recommendation

1. **Keep agent-contexts as submodule** - It's lightweight and we reference it constantly
2. **Use gh commands for Zed** - It's large and we only need it occasionally
3. **Document in CLAUDE.md** - Add note about using gh commands for reference repos

## Useful gh Commands

```bash
# Clone a specific directory only (if supported)
gh repo clone zed-industries/zed -- --depth=1 --filter=blob:none --sparse
cd zed && git sparse-checkout set crates/acp_*

# View file directly without cloning
gh repo view zed-industries/zed --web crates/acp_tools/src/acp_tools.rs

# Search without cloning
gh search code "ACP" --repo zed-industries/zed
```

## Adding to Global Claude Instructions

Consider adding to `~/.claude/CLAUDE.md`:
```markdown
## Reference Repository Management

When working with external reference repositories:
- Use `gh repo clone` for temporary access to large repos
- Keep lightweight documentation repos as submodules
- Search with `gh search code` before cloning
- Clean up cloned references after use with `rm -rf`

Common references:
- Zed (ACP): `gh repo clone zed-industries/zed`
- Cursor: `gh search code "pattern" --repo getcursor/cursor`
```
