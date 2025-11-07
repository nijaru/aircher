# Aircher Licensing & Model Policy

## Aircher License: Elastic License 2.0

Aircher is licensed under Elastic License 2.0, which:
- ✅ **Permits**: Free use, modification, distribution, commercial products
- ❌ **Prohibits**: Offering Aircher as a managed service (SaaS)
- 🎯 **Purpose**: Prevent exploitation while keeping it free for users

## Embedding Model Licensing

### Default: Commercial-Compatible Models
Aircher defaults to Apache 2.0/MIT licensed models:
- `all-MiniLM-L6-v2` - Apache 2.0 (default)
- `all-mpnet-base-v2` - Apache 2.0
- `bge-small-en-v1.5` - MIT
- `gte-small` - Apache 2.0

### Optional: Research-Grade Models
Advanced models available with restrictions:
- `SweRankEmbed-Small` - CC BY-NC 4.0 (non-commercial only)

## User Choice

### Commercial Users
- Default configuration is fully commercial-compatible
- No license concerns with Apache 2.0/MIT models
- Clear warnings if selecting restricted models

### Research/Personal Users
- Can choose any model including CC BY-NC
- Clear license information shown during selection
- Must accept terms explicitly for restricted models

### License Compliance
```bash
# Check your current model's license
aircher model current

# Verify commercial compatibility
aircher model check-license

# Switch to commercial-safe model
aircher model install all-MiniLM-L6-v2
```

## First-Run Experience

1. New users see model selection menu
2. Default choice (press Enter) = commercial-compatible
3. Advanced users can choose specialized models
4. License terms clearly displayed
5. Configuration saved for future use

## Summary

- **Aircher**: Elastic License 2.0 (commercial use OK)
- **Default models**: Apache 2.0/MIT (commercial use OK)
- **Optional models**: CC BY-NC 4.0 (research/personal only)
- **Your choice**: Select based on your intended use