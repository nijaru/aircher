# Double-Key Timing Research

## Common Double-Key Timings in Applications:

### **Double-Click Standards:**
- **Windows**: 500ms (default)
- **macOS**: 500ms (default) 
- **Most apps**: 400-600ms range

### **Keyboard Double-Press:**
- **Vim**: ~500ms for double-key commands
- **VS Code**: ~400ms for quick actions
- **Terminal apps**: Often 300-500ms

### **Human Factors:**
- **Fast typers**: Can easily do 200-300ms
- **Average users**: Comfortable at 400-500ms  
- **Slower users**: Need 500-700ms
- **Accidental prevention**: < 800ms avoids most accidents

### **Recommended for Aircher:**
- **400ms**: Good balance of accessibility and intentionality
- Matches typical user expectations from other applications
- Fast enough to feel responsive
- Slow enough for deliberate use

### **Current Issue:**
```rust
// BUG: 500ms exactly doesn't trigger!
if now.duration_since(last_escape).as_millis() < 500 {
```

Should be:
```rust  
if now.duration_since(last_escape).as_millis() <= 400 {
```