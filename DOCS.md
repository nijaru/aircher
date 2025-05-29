# Aircher Project Documentation Reference

This file serves as a comprehensive guide to all project documentation. Include this file in context to give AI assistants complete awareness of where to find information and how to keep documentation current.

## 📁 Project Documentation Structure

### Core Project Files
```
aircher/
├── DOCS.md              # 👈 This file - Documentation reference guide
├── README.md            # Project overview, installation, and quick start
├── OUTLINE.md           # Project vision, features, and high-level roadmap
├── SPEC.md              # Technical specification and architecture details
├── STATUS.md            # Current implementation status and progress
├── TASKS.md             # Detailed implementation task list and tracking
├── AIRCHER.md           # Project memory system for development context
├── example.config.toml  # Configuration file template
└── go.mod               # Go dependencies and tool management
```

### UI/UX Documentation
```
aircher/
├── UI_IMPROVEMENTS.md   # UI/UX design improvements and rationale
├── demo.md              # TUI demonstration script and features
└── test-claude.md       # Claude provider testing guide
```

### Development Files
```
aircher/
├── Makefile             # Build automation and development commands
├── .gitignore           # Git ignore patterns
├── go.sum               # Go dependency checksums
└── examples/            # Usage examples and templates
```

## 📚 Documentation Purpose and Scope

### 🎯 README.md
**Purpose**: Primary project introduction and user-facing documentation
**Scope**: 
- Project overview and key features
- Installation instructions
- Quick start guide and usage examples
- Configuration basics
- Go 1.24 feature highlights
**When to Update**: 
- New major features
- Installation changes
- API updates
- Performance improvements

### 🗺️ OUTLINE.md
**Purpose**: Comprehensive project vision and feature overview
**Scope**:
- Project philosophy and vision
- Complete feature list with descriptions
- Architecture overview
- Key differentiators vs competitors
- Usage scenarios and target users
- Implementation roadmap phases
- AIRCHER.md memory system explanation
**When to Update**:
- New feature concepts
- Architecture changes
- Roadmap adjustments
- Competitive analysis updates

### ⚙️ SPEC.md
**Purpose**: Technical specification and implementation details
**Scope**:
- System architecture with Go type definitions
- Database schemas and storage architecture
- LLM provider system specifications
- Context management algorithms
- MCP integration details
- Configuration system schema
- Implementation status tracking
**When to Update**:
- Architecture changes
- New technical components
- API design updates
- Implementation progress

### 📊 STATUS.md
**Purpose**: Current project status and progress tracking
**Scope**:
- Implementation progress overview
- Completed vs in-progress vs planned components
- Code metrics (lines of code, test coverage)
- Manual testing results
- Next priorities and milestones
- Current capabilities demonstration
**When to Update**:
- After completing major features
- Weekly progress updates
- Before releases
- When testing milestones

### ✅ TASKS.md
**Purpose**: Detailed implementation task breakdown and tracking
**Scope**:
- Granular task lists organized by component
- Implementation phase tracking
- Completion status markers (✅ 🚧 ❌)
- Task dependencies and priorities
**When to Update**:
- When completing tasks
- Adding new implementation requirements
- Reorganizing development priorities
- Phase completion milestones

### 🧠 AIRCHER.md
**Purpose**: Project development memory and context
**Scope**:
- Development conventions and guidelines
- Architecture decisions and rationale
- Command reference and workflows
- Go 1.24 feature usage
- Documentation maintenance guidelines
- Current development priorities
**When to Update**:
- New development conventions
- Architecture decisions
- Tool updates
- Process improvements

### 🎨 UI_IMPROVEMENTS.md
**Purpose**: UI/UX design documentation and improvements
**Scope**:
- Design philosophy and principles
- Specific improvements implemented
- Before/after comparisons
- Technical implementation details
- User experience enhancements
**When to Update**:
- UI/UX design changes
- New interface features
- User experience improvements
- Design pattern updates

### 🎬 demo.md
**Purpose**: Demonstration script for TUI features
**Scope**:
- Feature demonstration flow
- Key visual highlights
- Technical capabilities showcase
- Comparison points
**When to Update**:
- New TUI features
- Interface improvements
- Demo flow changes

### 🧪 test-claude.md
**Purpose**: Provider testing documentation
**Scope**:
- Testing procedures for Claude provider
- Expected behaviors and outputs
- Troubleshooting guide
**When to Update**:
- Provider implementation changes
- New testing requirements
- API updates

## 📝 Documentation Maintenance Guidelines

### Status Markers
Use consistent status markers across all documentation:
- ✅ **Completed**: Feature fully implemented and tested
- 🚧 **In Progress**: Implementation started, framework complete
- ❌ **Not Started**: Planned but not yet implemented
- 📊 **Metrics**: Quantitative measurements
- 🎯 **Goals**: Objectives and targets

### Cross-References
When updating documentation, check these cross-references:
1. **Feature additions**: Update README.md, OUTLINE.md, SPEC.md
2. **Architecture changes**: Update SPEC.md, AIRCHER.md, STATUS.md
3. **Implementation progress**: Update STATUS.md, TASKS.md
4. **UI changes**: Update UI_IMPROVEMENTS.md, demo.md
5. **Configuration changes**: Update README.md, example.config.toml

### Documentation Consistency Rules
1. **Keep version numbers synchronized** across all files
2. **Update progress markers** when completing tasks
3. **Maintain feature parity** between OUTLINE.md and SPEC.md
4. **Update metrics** in STATUS.md after significant work
5. **Cross-link related sections** between documents

## 🔄 Documentation Update Workflow

### When implementing new features:
1. **Plan**: Update TASKS.md with new tasks
2. **Design**: Update SPEC.md with technical details
3. **Implement**: Update progress markers in TASKS.md and STATUS.md
4. **Test**: Update testing documentation as needed
5. **Document**: Update README.md and OUTLINE.md with user-facing changes
6. **Finalize**: Update AIRCHER.md with any new conventions

### When completing phases:
1. **Review all documentation** for accuracy
2. **Update status markers** consistently
3. **Refresh metrics** in STATUS.md
4. **Update roadmap** in OUTLINE.md
5. **Check cross-references** between documents

## 🎯 Quick Reference for AI Assistants

### To find information about:
- **Project overview**: README.md
- **Feature descriptions**: OUTLINE.md
- **Technical details**: SPEC.md
- **Current status**: STATUS.md
- **Implementation tasks**: TASKS.md
- **Development context**: AIRCHER.md
- **UI/UX design**: UI_IMPROVEMENTS.md
- **Demonstrations**: demo.md
- **Testing procedures**: test-claude.md

### To update documentation:
1. **Identify affected files** using the cross-reference guide above
2. **Update status markers** consistently across files
3. **Maintain cross-links** between related sections
4. **Check for version/metric consistency**
5. **Update DOCS.md** if adding new documentation files

### Common documentation patterns:
- **Use consistent headings** and emoji for visual organization
- **Include code examples** with proper syntax highlighting
- **Provide context** for why changes were made
- **Link to related sections** in other documentation files
- **Update dates and versions** when making significant changes

This reference system ensures comprehensive, consistent, and current project documentation that supports both human developers and AI assistants in understanding and maintaining the Aircher project.