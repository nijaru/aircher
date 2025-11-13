# Skills System Architecture

**Status**: Design phase (Oct 30, 2025)
**Priority**: HIGH (identified from SOTA research)
**Based on**: Claude Agent Skills + SOTA analysis of 8 TUI agents

## Problem Statement

**Current limitation**: Users cannot extend Aircher's capabilities without modifying Rust code. This limits:
- Custom workflows (deployment scripts, testing frameworks)
- Domain-specific operations (company-specific CI/CD, internal tools)
- Experimental features (new AI techniques, prototype tools)
- Community contributions (users share skills without PRs)

**Solution**: Skills system enabling user-extensible agent capabilities via SKILL.md files.

## Design Principles

1. **Zero Code Required**: Users create skills with markdown + YAML
2. **Progressive Loading**: Load skill metadata cheaply, full docs when invoked
3. **MCP Compatible**: Skills can invoke MCP tools for ecosystem integration
4. **Discoverable**: Agent automatically finds and suggests relevant skills
5. **Safe by Default**: Skills execute with approval workflow
6. **Composable**: Skills can invoke other skills or tools

## SKILL.md Format

### File Structure

```markdown
---
name: search_documentation
description: Search project documentation for specific topics
author: aircher
version: 1.0.0
parameters:
  - name: query
    type: string
    description: Search query (e.g., "authentication flow")
    required: true
  - name: scope
    type: string
    description: Documentation scope (docs, api, guides, all)
    required: false
    default: all
capabilities:
  - read_files
  - semantic_search
tags:
  - documentation
  - search
  - knowledge
---

# Search Documentation

Searches project documentation using semantic search and returns relevant excerpts.

## When to Use

- User asks about project-specific concepts
- Need to find documentation on a topic
- Want to understand how something works from docs

## Implementation

1. Determine search scope from parameters
2. Use semantic_search tool with query
3. Filter results to documentation files only
4. Extract relevant excerpts with context
5. Rank by relevance and recency
6. Return top 5 results with file paths

## Example Usage

```
User: "How does authentication work in this project?"
Agent: I'll search the documentation for authentication details.
       [Invokes search_documentation skill]
```

## Output Format

Returns markdown with:
- **File**: Path to documentation file
- **Excerpt**: Relevant section with highlighting
- **Context**: Surrounding headings for navigation
- **Relevance**: Score indicating match quality

## Edge Cases

- No documentation files: Return empty with suggestion to create docs
- Ambiguous query: Ask user for clarification
- Large results: Paginate or summarize

## Dependencies

- semantic_search tool (built-in)
- read_file tool (built-in)
- Markdown parser (for extracting sections)
```

### YAML Frontmatter Specification

```yaml
---
# Required fields
name: string               # Unique skill identifier (kebab-case)
description: string        # One-line description for model selection
version: string            # Semantic version (1.0.0)

# Optional fields
author: string             # Skill creator
parameters:                # Input parameters (JSON schema style)
  - name: string
    type: string           # string, number, boolean, array, object
    description: string
    required: boolean
    default: any           # Default value if not provided
    enum: [string]         # Valid values (optional)
capabilities:              # Required Aircher capabilities
  - string                 # e.g., read_files, run_commands, semantic_search
tags:                      # Categorization for discovery
  - string
examples:                  # Quick examples for model
  - input: object          # Example parameters
    output: string         # Expected result
mcp_tools:                 # MCP tools this skill uses
  - name: string
    server: string         # MCP server providing tool
---
```

## File System Structure

```
~/.aircher/skills/                    # User's global skills
├── search_documentation/
│   └── SKILL.md
├── deploy_to_staging/
│   └── SKILL.md
└── run_integration_tests/
    └── SKILL.md

.aircher/skills/                      # Project-specific skills
├── setup_dev_environment/
│   └── SKILL.md
├── generate_api_client/
│   └── SKILL.md
└── migrate_database/
    └── SKILL.md

/usr/share/aircher/skills/            # System-wide (built-in)
├── analyze_errors/
│   └── SKILL.md
└── explain_code/
    └── SKILL.md
```

**Precedence**: Project skills > User skills > System skills

## Skill Discovery

### Scan Locations

```rust
pub struct SkillDiscovery {
    system_skills_path: PathBuf,      // /usr/share/aircher/skills/
    user_skills_path: PathBuf,        // ~/.aircher/skills/
    project_skills_path: Option<PathBuf>, // .aircher/skills/
}

impl SkillDiscovery {
    pub async fn discover_all(&self) -> Result<Vec<SkillMetadata>> {
        let mut skills = vec![];

        // Scan in order of precedence (project > user > system)
        if let Some(project_path) = &self.project_skills_path {
            skills.extend(self.scan_directory(project_path).await?);
        }
        skills.extend(self.scan_directory(&self.user_skills_path).await?);
        skills.extend(self.scan_directory(&self.system_skills_path).await?);

        // Deduplicate by name (earlier paths take precedence)
        let mut seen = HashSet::new();
        skills.retain(|skill| seen.insert(skill.name.clone()));

        Ok(skills)
    }

    async fn scan_directory(&self, path: &Path) -> Result<Vec<SkillMetadata>> {
        let mut skills = vec![];

        // Find all SKILL.md files
        let entries = tokio::fs::read_dir(path).await?;
        for entry in entries {
            let entry = entry?;
            let skill_file = entry.path().join("SKILL.md");

            if skill_file.exists() {
                // Parse only frontmatter (progressive loading)
                let metadata = self.parse_metadata(&skill_file).await?;
                skills.push(metadata);
            }
        }

        Ok(skills)
    }
}
```

### Progressive Loading

```rust
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub parameters: Vec<ParameterSchema>,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub file_path: PathBuf,

    // Not loaded until skill is invoked
    full_documentation: OnceCell<String>,
}

impl SkillMetadata {
    /// Load full documentation when skill is invoked
    pub async fn load_full_documentation(&self) -> Result<&str> {
        self.full_documentation
            .get_or_try_init(|| async {
                let content = tokio::fs::read_to_string(&self.file_path).await?;
                // Parse markdown body (skip YAML frontmatter)
                let body = self.extract_markdown_body(&content)?;
                Ok(body)
            })
            .await
    }
}
```

## Skill Execution

### Integration with Tool Registry

```rust
pub struct SkillTool {
    metadata: SkillMetadata,
    executor: SkillExecutor,
}

#[async_trait]
impl Tool for SkillTool {
    async fn execute(&self, params: Value) -> Result<ToolOutput> {
        // 1. Validate parameters against schema
        self.validate_parameters(&params)?;

        // 2. Check required capabilities
        self.check_capabilities(&self.metadata.capabilities)?;

        // 3. Load full documentation
        let docs = self.metadata.load_full_documentation().await?;

        // 4. Create enhanced prompt with skill instructions
        let enhanced_prompt = format!(
            "Execute skill: {}\n\n\
             Description: {}\n\n\
             Parameters: {}\n\n\
             Instructions:\n{}\n\n\
             Provide step-by-step execution.",
            self.metadata.name,
            self.metadata.description,
            serde_json::to_string_pretty(&params)?,
            docs
        );

        // 5. Execute via agent (skill guides the agent)
        let result = self.executor.execute_skill(
            &self.metadata,
            params,
            &enhanced_prompt
        ).await?;

        Ok(result)
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn description(&self) -> &str {
        &self.metadata.description
    }

    fn parameters_schema(&self) -> Value {
        // Convert metadata.parameters to JSON schema
        self.build_json_schema(&self.metadata.parameters)
    }
}
```

### Skill Executor

```rust
pub struct SkillExecutor {
    agent: Arc<Agent>,
    tool_registry: Arc<ToolRegistry>,
    approval_manager: Arc<ApprovalManager>,
}

impl SkillExecutor {
    pub async fn execute_skill(
        &self,
        metadata: &SkillMetadata,
        params: Value,
        instructions: &str,
    ) -> Result<ToolOutput> {
        // 1. Create skill execution context
        let context = SkillContext {
            skill_name: metadata.name.clone(),
            parameters: params.clone(),
            available_tools: self.get_available_tools(&metadata.capabilities),
        };

        // 2. Request approval if skill requires dangerous operations
        if self.requires_approval(metadata) {
            self.approval_manager
                .request_approval(&format!(
                    "Execute skill '{}' with parameters: {}",
                    metadata.name,
                    serde_json::to_string_pretty(&params)?
                ))
                .await?;
        }

        // 3. Execute skill with agent
        // Skill instructions guide agent to use appropriate tools
        let response = self.agent.execute_with_instructions(
            instructions,
            context,
        ).await?;

        Ok(response)
    }

    fn requires_approval(&self, metadata: &SkillMetadata) -> bool {
        // Check if skill capabilities include dangerous operations
        metadata.capabilities.iter().any(|cap| {
            matches!(
                cap.as_str(),
                "run_commands" | "write_files" | "delete_files" | "network_access"
            )
        })
    }
}
```

## Model Selection Enhancement

### Skill-Aware Tool Calling

```rust
// In system prompt, include skills as available tools
impl Agent {
    pub async fn prepare_system_prompt(&self) -> String {
        let mut prompt = BASE_SYSTEM_PROMPT.to_string();

        // Add built-in tools
        prompt.push_str("\n\n## Available Tools\n");
        for tool in self.tools.iter() {
            prompt.push_str(&format!(
                "- {}: {}\n",
                tool.name(),
                tool.description()
            ));
        }

        // Add discovered skills
        let skills = self.skill_manager.list_skills().await?;
        if !skills.is_empty() {
            prompt.push_str("\n\n## Available Skills\n");
            for skill in skills {
                prompt.push_str(&format!(
                    "- {}: {} (tags: {})\n",
                    skill.name,
                    skill.description,
                    skill.tags.join(", ")
                ));
            }
        }

        prompt
    }
}
```

### Skill Suggestion

```rust
impl Agent {
    /// Suggest relevant skills for user query
    pub async fn suggest_skills(&self, query: &str) -> Result<Vec<SkillMetadata>> {
        let all_skills = self.skill_manager.list_skills().await?;

        // 1. Semantic similarity (if user query matches skill description)
        let query_embedding = self.embeddings.encode(query).await?;
        let mut scored_skills: Vec<_> = all_skills
            .into_iter()
            .map(|skill| {
                let desc_embedding = self.embeddings.encode(&skill.description)?;
                let similarity = cosine_similarity(&query_embedding, &desc_embedding);
                Ok((skill, similarity))
            })
            .collect::<Result<_>>()?;

        // 2. Sort by relevance
        scored_skills.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 3. Filter threshold (>0.7 similarity)
        let relevant_skills: Vec<_> = scored_skills
            .into_iter()
            .filter(|(_, score)| *score > 0.7)
            .map(|(skill, _)| skill)
            .take(5) // Top 5
            .collect();

        Ok(relevant_skills)
    }
}
```

## MCP Integration

### Skills Can Use MCP Tools

```rust
// In SKILL.md frontmatter:
// mcp_tools:
//   - name: github_create_pr
//     server: github

pub struct SkillExecutor {
    mcp_client: Option<Arc<McpClient>>,
}

impl SkillExecutor {
    async fn get_available_tools(&self, capabilities: &[String]) -> Vec<String> {
        let mut tools = vec![];

        // Add built-in Aircher tools
        for cap in capabilities {
            if let Some(tool_name) = self.capability_to_tool(cap) {
                tools.push(tool_name);
            }
        }

        // Add MCP tools if available
        if let Some(mcp) = &self.mcp_client {
            // Skills can declare MCP tools in frontmatter
            // Agent can use them during skill execution
            let mcp_tools = mcp.list_tools().await?;
            tools.extend(mcp_tools.iter().map(|t| t.name.clone()));
        }

        tools
    }
}
```

## Example Skills

### 1. Deploy to Staging

```yaml
---
name: deploy_to_staging
description: Deploy current branch to staging environment
version: 1.0.0
parameters:
  - name: branch
    type: string
    description: Git branch to deploy
    required: false
    default: current
  - name: run_tests
    type: boolean
    description: Run tests before deploying
    required: false
    default: true
capabilities:
  - run_commands
  - read_files
tags:
  - deployment
  - staging
  - ci-cd
---

# Deploy to Staging

## Steps

1. Verify branch is clean (no uncommitted changes)
2. Run tests if run_tests=true
3. Build production assets
4. Deploy to staging server via ./deploy.sh
5. Run smoke tests
6. Report deployment status
```

### 2. Generate API Client

```yaml
---
name: generate_api_client
description: Generate API client code from OpenAPI spec
version: 1.0.0
parameters:
  - name: spec_file
    type: string
    description: Path to OpenAPI spec (YAML or JSON)
    required: true
  - name: language
    type: string
    description: Target language for client
    required: true
    enum: [typescript, python, rust, go]
  - name: output_dir
    type: string
    description: Output directory for generated code
    required: false
    default: ./generated
capabilities:
  - read_files
  - write_files
  - run_commands
mcp_tools:
  - name: openapi_generator
    server: openapi-tools
tags:
  - code-generation
  - api
  - openapi
---

# Generate API Client

## Steps

1. Read and validate OpenAPI spec
2. Choose generator based on language
3. Run openapi-generator-cli or equivalent
4. Format generated code
5. Update imports/dependencies
6. Generate usage examples
```

### 3. Setup Dev Environment

```yaml
---
name: setup_dev_environment
description: Set up development environment for project
version: 1.0.0
parameters: []
capabilities:
  - run_commands
  - read_files
  - write_files
tags:
  - setup
  - onboarding
  - development
---

# Setup Dev Environment

## Steps

1. Detect project type (Rust, Node, Python, etc.)
2. Install required dependencies
3. Set up configuration files
4. Initialize databases if needed
5. Run initial build/compile
6. Verify setup with health check
```

## Implementation Plan

### Phase 1: Core Infrastructure (Week 10 Days 1-2)

1. **SkillMetadata struct** (src/agent/skills/metadata.rs)
   - Parse YAML frontmatter
   - Validate schema
   - Progressive loading

2. **SkillDiscovery** (src/agent/skills/discovery.rs)
   - Scan directories
   - Deduplicate by precedence
   - Cache discovered skills

3. **SkillTool implementation** (src/agent/skills/tool.rs)
   - Integrate with ToolRegistry
   - Parameter validation
   - Capability checking

### Phase 2: Execution Engine (Week 10 Days 3-4)

1. **SkillExecutor** (src/agent/skills/executor.rs)
   - Skill execution context
   - Approval workflow integration
   - Error handling and rollback

2. **Agent integration** (src/agent/core.rs)
   - Load skills on initialization
   - Include skills in system prompt
   - Skill suggestion algorithm

### Phase 3: Example Skills (Week 10 Days 5-6)

1. Create 5 example skills:
   - search_documentation
   - deploy_to_staging
   - run_integration_tests
   - generate_api_client
   - setup_dev_environment

2. Test skills with real workflows

### Phase 4: Documentation (Week 10 Day 7)

1. User guide: Creating custom skills
2. API reference: SKILL.md format
3. Example skills gallery
4. Troubleshooting guide

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_skill_discovery() {
        let discovery = SkillDiscovery::new();
        let skills = discovery.discover_all().await.unwrap();
        assert!(!skills.is_empty());
    }

    #[tokio::test]
    async fn test_skill_metadata_parsing() {
        let skill_md = r#"
---
name: test_skill
description: Test skill
version: 1.0.0
parameters:
  - name: input
    type: string
    required: true
---
# Test Skill
Instructions here.
"#;
        let metadata = SkillMetadata::parse(skill_md).unwrap();
        assert_eq!(metadata.name, "test_skill");
    }

    #[tokio::test]
    async fn test_progressive_loading() {
        let metadata = SkillMetadata::from_file("test.md").await.unwrap();
        assert!(metadata.full_documentation.get().is_none());

        // Load full docs
        let docs = metadata.load_full_documentation().await.unwrap();
        assert!(!docs.is_empty());
    }
}
```

### Integration Tests

1. End-to-end skill execution
2. Approval workflow with skills
3. MCP tool integration
4. Multi-skill workflows

## Competitive Advantages

**vs Claude Code**:
- ✅ Skills work offline (if using Ollama)
- ✅ Skills can be version-controlled with project
- ✅ MCP integration for ecosystem compatibility

**vs OpenCode**:
- ✅ Skills are more structured (YAML frontmatter)
- ✅ Progressive loading (faster startup)

**vs Cursor/Zed**:
- ✅ Skills are agent-native (not just snippets)
- ✅ Skills can invoke tools and other skills

## Future Enhancements

### Skill Marketplace (Phase 2)

- Share skills via Git repositories
- Skill discovery: `aircher skill install github.com/user/skill-repo`
- Version management and updates
- Ratings and reviews

### Skill Composition (Phase 2)

- Skills can invoke other skills
- Create workflows by chaining skills
- Conditional execution and error handling

### Skill Testing (Phase 2)

- Unit tests for skills (given input, expect output)
- Integration tests with mocks
- Coverage reports

### Skill Analytics (Phase 2)

- Track skill usage
- Measure success rates
- Identify popular skills
- Suggest improvements

---

**Next Steps**:
1. Implement Phase 1 (Core Infrastructure)
2. Create 3 example skills for testing
3. Document SKILL.md format
4. Announce Skills system in Week 10 release
