# Query Intelligence

Aircher's semantic search includes advanced query intelligence features that help you find code more effectively, even when you don't know the exact terminology or make typos.

## Features

### 1. Typo Correction

Automatically corrects common programming typos:
- `fucntion` → `function`
- `authentification` → `authentication`
- `databse` → `database`
- `conifg` → `config`
- `hanlder` → `handler`
- `errer` → `error`
- `responce` → `response`
- `reqeust` → `request`

When a typo is detected, Aircher will show:
```
💡 Did you mean: function
🔍 Searching for: 'function'
```

### 2. Synonym Expansion

Automatically expands queries to include synonymous terms for broader coverage:

#### Programming Terms
- `error` → also searches: `exception`, `failure`, `panic`, `fault`, `bug`
- `auth` → also searches: `authentication`, `authorization`, `login`, `signin`, `security`
- `database` → also searches: `db`, `sql`, `storage`, `persistence`, `repository`
- `config` → also searches: `configuration`, `settings`, `options`, `preferences`, `setup`
- `function` → also searches: `method`, `fn`, `procedure`, `routine`, `callback`

#### Pattern Expansion
- Queries containing `handling` expand to include `error handling`, `exception handling`, `handle`, `handler`
- Queries containing `logic` expand to include `implementation`, `algorithm`, `process`, `flow`

### 3. Query Analysis

Aircher analyzes your queries to provide helpful suggestions:

#### Complexity Levels
- **Simple**: 1-word queries (e.g., "error")
- **Moderate**: 2-3 word queries (e.g., "error handling")
- **Complex**: 4+ word queries (e.g., "async function error handling implementation")

#### Specificity Detection
- **High**: Contains specific programming terms (function, class, async, etc.)
- **Medium**: Somewhat specific but could be clearer
- **Low**: Very vague, mostly generic terms

#### Suggestions
Based on analysis, Aircher may suggest:
- Adding more context for single-word queries
- Using more specific programming terms
- Being more precise instead of vague terms like "stuff" or "thing"

## How It Works

1. **Query Processing**: Your search query is first analyzed for typos and common patterns
2. **Expansion Generation**: Up to 3 synonym-based variations are created
3. **Multi-Search Execution**: All query variants are searched in parallel
4. **Result Deduplication**: Duplicate results across queries are removed
5. **Relevance Sorting**: Final results are sorted by similarity score

## Examples

### Basic Typo Correction
```bash
$ aircher search query "fucntion implementation"
💡 Did you mean: function
🔍 Searching for: 'function'
```

### Synonym Expansion
```bash
$ aircher search query "auth" --debug-filters
🔍 Searching for: 'auth'
🔍 Expanding search with synonyms: ["authentication", "authorization", "login"]
```

### Complex Query with Analysis
```bash
$ aircher search query "error" --debug-filters
📊 Query analysis:
   Complexity: Simple
   Specificity: Medium
   💡 Tips:
      - Try adding more context (e.g., 'error handling' instead of just 'error')
   Related terms: exception, failure, panic, fault, bug
```

## Performance Impact

Query expansion typically adds minimal overhead:
- Typo correction: < 1ms
- Synonym expansion: 2-3 additional searches
- Deduplication: < 10ms for typical result sets

The benefits of finding more relevant results usually outweigh the small performance cost.

## Customization

Currently, the synonym mappings and typo corrections are built-in. Future versions may support:
- Custom synonym dictionaries
- Project-specific terminology mappings
- Machine learning-based query improvements

## Tips for Best Results

1. **Use natural language**: "error handling" instead of "err_handle"
2. **Include context**: "database connection" instead of just "connection"
3. **Use programming terms**: "async function" instead of "asynchronous method"
4. **Don't worry about typos**: They'll be corrected automatically
5. **Start broad**: Let synonym expansion help you discover related code