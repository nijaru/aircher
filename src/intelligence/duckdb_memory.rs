use anyhow::Result;
use chrono::{DateTime, Utc};
use duckdb::{Connection as DuckDBConnection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Action taken by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub tool: String,
    pub params: serde_json::Value,
    pub success: bool,
    pub duration_ms: u64,
    pub result_summary: String,
}

/// Pattern learned from agent interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub description: String,
    pub context: String,
    pub actions: Vec<AgentAction>,
    pub files_involved: Vec<String>,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub embedding_text: String, // Text to generate embedding from
    pub embedding: Vec<f32>, // Actual embedding vector
}

/// Predicted action based on patterns
#[derive(Debug, Clone)]
pub struct Prediction {
    pub action: String,
    pub confidence: f32,
    pub reason: String,
}

/// DuckDB-based intelligent memory system
pub struct DuckDBMemory {
    _base_dir: PathBuf,
    db: Arc<Mutex<DuckDBConnection>>,
}

impl DuckDBMemory {
    /// Create new DuckDB-based memory
    pub async fn new(project_root: &Path) -> Result<Self> {
        let base_dir = project_root.join(".aircher").join("intelligence");
        std::fs::create_dir_all(&base_dir)?;
        
        let db_path = base_dir.join("intelligence.duckdb");
        let conn = DuckDBConnection::open(&db_path)?;
        
        // Initialize schema
        Self::init_schema(&conn)?;
        
        Ok(Self {
            _base_dir: base_dir,
            db: Arc::new(Mutex::new(conn)),
        })
    }
    
    fn init_schema(conn: &DuckDBConnection) -> Result<()> {
        // Main patterns table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patterns (
                id VARCHAR PRIMARY KEY,
                description TEXT,
                context TEXT,
                embedding_text TEXT,
                embedding BLOB,
                success BOOLEAN,
                timestamp TIMESTAMP,
                session_id VARCHAR,
                files_count INTEGER,
                actions_count INTEGER,
                total_duration_ms BIGINT
            )",
            [],
        )?;
        
        // Actions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS actions (
                pattern_id VARCHAR,
                sequence_index INTEGER,
                tool VARCHAR,
                params JSON,
                success BOOLEAN,
                duration_ms BIGINT,
                result_summary TEXT,
                PRIMARY KEY (pattern_id, sequence_index),
                FOREIGN KEY (pattern_id) REFERENCES patterns(id)
            )",
            [],
        )?;
        
        // Files table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                pattern_id VARCHAR,
                file_path VARCHAR,
                PRIMARY KEY (pattern_id, file_path),
                FOREIGN KEY (pattern_id) REFERENCES patterns(id)
            )",
            [],
        )?;
        
        // Pattern similarity table (pre-computed similarities)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pattern_similarity (
                pattern_a VARCHAR,
                pattern_b VARCHAR,
                similarity DOUBLE,
                PRIMARY KEY (pattern_a, pattern_b),
                FOREIGN KEY (pattern_a) REFERENCES patterns(id),
                FOREIGN KEY (pattern_b) REFERENCES patterns(id)
            )",
            [],
        )?;
        
        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_patterns_timestamp ON patterns(timestamp DESC)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_patterns_success ON patterns(success)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_files_path ON files(file_path)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_actions_tool ON actions(tool)", [])?;
        
        Ok(())
    }
    
    /// Record a pattern from agent execution
    pub async fn record_pattern(&self, pattern: Pattern) -> Result<()> {
        let db = self.db.clone();
        let pattern = pattern.clone();
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            let db = db.blocking_lock();
            
            // Insert main pattern
            let total_duration: u64 = pattern.actions.iter().map(|a| a.duration_ms).sum();
            
            // Serialize embedding as bytes for storage
            let embedding_bytes = bincode::serialize(&pattern.embedding)?;
            
            db.execute(
                "INSERT INTO patterns (id, description, context, embedding_text, embedding, success, 
                 timestamp, session_id, files_count, actions_count, total_duration_ms)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &pattern.id,
                    &pattern.description,
                    &pattern.context,
                    &pattern.embedding_text,
                    embedding_bytes,
                    pattern.success,
                    pattern.timestamp.to_rfc3339(),
                    &pattern.session_id,
                    pattern.files_involved.len() as i32,
                    pattern.actions.len() as i32,
                    total_duration as i64,
                ],
            )?;
            
            // Insert actions
            for (idx, action) in pattern.actions.iter().enumerate() {
                db.execute(
                    "INSERT INTO actions (pattern_id, sequence_index, tool, params, 
                     success, duration_ms, result_summary)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![
                        &pattern.id,
                        idx as i32,
                        &action.tool,
                        serde_json::to_string(&action.params)?,
                        action.success,
                        action.duration_ms as i64,
                        &action.result_summary,
                    ],
                )?;
            }
            
            // Insert files
            for file in &pattern.files_involved {
                db.execute(
                    "INSERT INTO files (pattern_id, file_path) VALUES (?, ?)",
                    params![&pattern.id, file],
                )?;
            }
            
            // Update similarities
            Self::update_similarities_sync(&*db, &pattern)?;
            
            Ok(())
        }).await?
    }
    
    fn update_similarities_sync(db: &DuckDBConnection, pattern: &Pattern) -> Result<()> {
        // Simple text similarity based on shared words
        // In production, you'd use actual embeddings
        let recent_patterns = db.prepare(
            "SELECT id, embedding_text FROM patterns 
             WHERE id != ? 
             ORDER BY timestamp DESC 
             LIMIT 100"
        )?
        .query_map([&pattern.id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        for (other_id, other_text) in recent_patterns {
            let similarity = Self::compute_text_similarity(&pattern.embedding_text, &other_text);
            
            if similarity > 0.3 {
                db.execute(
                    "INSERT OR REPLACE INTO pattern_similarity (pattern_a, pattern_b, similarity)
                     VALUES (?, ?, ?)",
                    params![&pattern.id, &other_id, similarity],
                )?;
            }
        }
        
        Ok(())
    }
    
    fn compute_text_similarity(text_a: &str, text_b: &str) -> f64 {
        // Simple Jaccard similarity for demonstration
        let words_a: std::collections::HashSet<_> = text_a.split_whitespace().collect();
        let words_b: std::collections::HashSet<_> = text_b.split_whitespace().collect();
        
        if words_a.is_empty() || words_b.is_empty() {
            return 0.0;
        }
        
        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();
        
        intersection as f64 / union as f64
    }
    
    /// Find patterns similar to current context
    pub async fn find_similar_patterns(&self, context: &str, limit: usize) -> Result<Vec<Pattern>> {
        let db = self.db.clone();
        let context = context.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<Pattern>> {
            let db = db.blocking_lock();
            
            // First, find patterns with similar context text
            let query = "
                SELECT DISTINCT p.id, p.description, p.context, p.success, 
                       p.timestamp, p.session_id, p.embedding_text, p.embedding
                FROM patterns p
                WHERE p.success = true
                ORDER BY p.timestamp DESC
                LIMIT ?";
            
            let patterns = db.prepare(query)?
                .query_map([limit], |row| {
                    // Deserialize embedding from bytes
                    let embedding_bytes: Vec<u8> = row.get(7)?;
                    let embedding: Vec<f32> = if embedding_bytes.is_empty() {
                        vec![]
                    } else {
                        bincode::deserialize(&embedding_bytes).unwrap_or_else(|_| vec![])
                    };
                    
                    Ok(Pattern {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        context: row.get(2)?,
                        actions: vec![],
                        files_involved: vec![],
                        success: row.get(3)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                            .unwrap()
                            .with_timezone(&Utc),
                        session_id: row.get(5)?,
                        embedding_text: row.get(6)?,
                        embedding,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            
            // Filter by text similarity
            let mut scored_patterns: Vec<(f64, Pattern)> = patterns
                .into_iter()
                .map(|p| {
                    let similarity = Self::compute_text_similarity(&context, &p.context);
                    (similarity, p)
                })
                .filter(|(sim, _)| *sim > 0.2)
                .collect();
            
            scored_patterns.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            
            Ok(scored_patterns.into_iter()
                .take(limit)
                .map(|(_, p)| p)
                .collect())
        }).await?
    }
    
    /// Predict next actions based on patterns
    pub async fn predict_next_actions(&self, context: &str) -> Result<Vec<Prediction>> {
        let similar = self.find_similar_patterns(context, 10).await?;
        
        if similar.is_empty() {
            return Ok(vec![]);
        }
        
        let db = self.db.clone();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<Prediction>> {
            let db = db.blocking_lock();
            
            // Analyze common actions in similar patterns
            let pattern_ids: Vec<String> = similar.iter()
                .map(|p| format!("'{}'", p.id))
                .collect();
            
            let query = format!(
                "SELECT tool, COUNT(*) as count, AVG(CASE WHEN success THEN 1.0 ELSE 0.0 END) as success_rate
                 FROM actions
                 WHERE pattern_id IN ({})
                 GROUP BY tool
                 HAVING COUNT(*) > 1
                 ORDER BY count DESC, success_rate DESC
                 LIMIT 5",
                pattern_ids.join(", ")
            );
            
            let predictions = db.prepare(&query)?
                .query_map([], |row| {
                    let tool: String = row.get(0)?;
                    let count: i32 = row.get(1)?;
                    let success_rate: f64 = row.get(2)?;
                    
                    Ok(Prediction {
                        action: tool.clone(),
                        confidence: (success_rate * (count as f64 / 10.0).min(1.0)) as f32,
                        reason: format!("Used {} times with {:.0}% success in similar contexts", 
                                      count, success_rate * 100.0),
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(predictions)
        }).await?
    }
    
    /// Find files that often change together
    pub async fn find_related_files(&self, file: &str) -> Result<Vec<String>> {
        let db = self.db.clone();
        let file = file.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
            let db = db.blocking_lock();
            
            let query = "
                WITH target_patterns AS (
                    SELECT DISTINCT pattern_id
                    FROM files
                    WHERE file_path = ?
                )
                SELECT f.file_path, COUNT(*) as co_occurrence
                FROM files f
                JOIN target_patterns tp ON f.pattern_id = tp.pattern_id
                WHERE f.file_path != ?
                GROUP BY f.file_path
                HAVING COUNT(*) > 2
                ORDER BY co_occurrence DESC
                LIMIT 10";
            
            let related = db.prepare(query)?
                .query_map(params![&file, &file], |row| {
                    row.get::<_, String>(0)
                })?
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(related)
        }).await?
    }
    
    /// Get successful patterns for a file
    pub async fn get_file_patterns(&self, file: &str) -> Result<Vec<Pattern>> {
        let db = self.db.clone();
        let file = file.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<Vec<Pattern>> {
            let db = db.blocking_lock();
            
            let query = "
                SELECT DISTINCT p.id, p.description, p.context, p.success,
                       p.timestamp, p.session_id, p.embedding_text, p.embedding
                FROM patterns p
                JOIN files f ON p.id = f.pattern_id
                WHERE f.file_path = ?
                  AND p.success = true
                ORDER BY p.timestamp DESC
                LIMIT 10";
            
            let patterns = db.prepare(query)?
                .query_map([&file], |row| {
                    // Deserialize embedding from bytes
                    let embedding_bytes: Vec<u8> = row.get(7)?;
                    let embedding: Vec<f32> = if embedding_bytes.is_empty() {
                        vec![]
                    } else {
                        bincode::deserialize(&embedding_bytes).unwrap_or_else(|_| vec![])
                    };
                    
                    Ok(Pattern {
                        id: row.get(0)?,
                        description: row.get(1)?,
                        context: row.get(2)?,
                        actions: vec![],
                        files_involved: vec![],
                        success: row.get(3)?,
                        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                            .unwrap()
                            .with_timezone(&Utc),
                        session_id: row.get(5)?,
                        embedding_text: row.get(6)?,
                        embedding,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(patterns)
        }).await?
    }
    
    /// Analyze pattern effectiveness trends
    pub async fn analyze_trends(&self, pattern_type: &str) -> Result<String> {
        let db = self.db.clone();
        let pattern_type = pattern_type.to_string();
        
        tokio::task::spawn_blocking(move || -> Result<String> {
            let db = db.blocking_lock();
            
            let query = "
                WITH weekly_stats AS (
                    SELECT 
                        DATE_TRUNC('week', timestamp) as week,
                        COUNT(*) as usage_count,
                        AVG(CASE WHEN success THEN 1.0 ELSE 0.0 END) as success_rate
                    FROM patterns
                    WHERE description LIKE '%' || ? || '%'
                      AND timestamp > CURRENT_TIMESTAMP - INTERVAL '30 days'
                    GROUP BY week
                    ORDER BY week DESC
                )
                SELECT 
                    AVG(success_rate) as avg_success,
                    SUM(usage_count) as total_usage
                FROM weekly_stats";
            
            let mut stmt = db.prepare(query)?;
            let mut rows = stmt.query_map([&pattern_type], |row| {
                let avg_success: f64 = row.get(0)?;
                let total_usage: i64 = row.get(1)?;
                
                Ok(format!(
                    "Pattern '{}': {:.0}% success rate, used {} times in last 30 days",
                    pattern_type, avg_success * 100.0, total_usage
                ))
            })?;
            
            rows.next()
                .ok_or_else(|| anyhow::anyhow!("No data found"))?
                .map_err(Into::into)
        }).await?
    }
}

/// Simple agent-friendly API
impl DuckDBMemory {
    /// Record that something worked
    pub async fn record_success(
        &self,
        description: &str,
        context: &str,
        actions: Vec<AgentAction>,
        files: Vec<String>,
    ) -> Result<()> {
        let pattern = Pattern {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.to_string(),
            context: context.to_string(),
            actions,
            files_involved: files,
            success: true,
            timestamp: Utc::now(),
            session_id: "current".to_string(),
            embedding_text: format!("{} {}", description, context),
            embedding: vec![], // No embedding generation in record_success helper
        };
        
        self.record_pattern(pattern).await
    }
    
    /// Get suggestions for what to do next
    pub async fn suggest_next(&self, context: &str) -> Result<String> {
        let predictions = self.predict_next_actions(context).await?;
        
        if predictions.is_empty() {
            return Ok("No suggestions based on past patterns".to_string());
        }
        
        let suggestions = predictions.iter()
            .map(|p| format!("- {} ({}% confidence: {})", 
                           p.action, (p.confidence * 100.0) as u32, p.reason))
            .collect::<Vec<_>>()
            .join("\n");
        
        Ok(format!("Suggested actions:\n{}", suggestions))
    }
    
    /// Check what files might need changes
    pub async fn check_related_files(&self, current_file: &str) -> Result<Vec<String>> {
        self.find_related_files(current_file).await
    }
}