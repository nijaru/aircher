use anyhow::{Context, Result};
use arrow::array::{ArrayRef, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema};
use chrono::{DateTime, Utc};
use duckdb::{Connection as DuckDBConnection, params};
use lance::{Dataset, dataset::WriteParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Action taken by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub tool: String,
    pub params: serde_json::Value,
    pub result: ActionResult,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success(String),
    Failure(String),
    Partial(String),
}

/// File change tracked during pattern execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { from: String },
}

/// Intelligent pattern with full context and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentPattern {
    pub id: String,
    pub description: String,
    pub embedding: Vec<f32>,

    // Context
    pub trigger_context: String,
    pub action_sequence: Vec<Action>,
    pub files_changed: Vec<FileChange>,

    // Outcomes
    pub immediate_success: bool,
    pub long_term_success: Option<f32>,
    pub side_effects: Vec<String>,

    // Relationships
    pub preceded_by: Vec<String>,
    pub followed_by: Vec<String>,
    pub co_occurred_with: Vec<String>,

    // Metadata
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub model_used: String,
}

/// Predicted next action based on historical patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedAction {
    pub action: String,
    pub confidence: f32,
    pub similar_contexts: usize,
    pub average_success_rate: f32,
    pub typical_duration_ms: u64,
}

/// File relationship analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRelationship {
    pub file: String,
    pub related_file: String,
    pub co_occurrence_count: usize,
    pub correlation_strength: f32,
    pub typical_change_order: ChangeOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeOrder {
    Before,
    After,
    Simultaneous,
    Independent,
}

/// Trend analysis for pattern effectiveness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub pattern_type: String,
    pub current_success_rate: f32,
    pub trend_direction: TrendDirection,
    pub trend_strength: f32,
    pub usage_frequency: f32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

/// Intelligent memory system combining lance-rs and DuckDB
pub struct IntelligentMemory {
    base_dir: PathBuf,
    lance_db: Option<Dataset>,
    duck_db: Arc<RwLock<DuckDBConnection>>,
}

impl IntelligentMemory {
    /// Create new intelligent memory system
    pub async fn new(project_root: &Path) -> Result<Self> {
        let base_dir = project_root.join(".aircher").join("intelligence");
        std::fs::create_dir_all(&base_dir)?;

        // Initialize lance for vector storage
        let lance_db = Self::init_lance(&base_dir).await?;

        // Initialize DuckDB for analytics
        let duck_db = Self::init_duckdb(&base_dir).await?;

        Ok(Self {
            base_dir,
            lance_db: Some(lance_db),
            duck_db: Arc::new(RwLock::new(duck_db)),
        })
    }

    async fn init_lance(base_dir: &Path) -> Result<Dataset> {
        let path = base_dir.join("patterns.lance");

        if path.exists() {
            Dataset::open(&path).await.context("Failed to open lance dataset")
        } else {
            // Create schema for intelligent patterns
            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("description", DataType::Utf8, false),
                Field::new("embedding", DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    384
                ), false),
                Field::new("trigger_context", DataType::Utf8, false),
                Field::new("action_sequence", DataType::Utf8, false), // JSON
                Field::new("files_changed", DataType::Utf8, false), // JSON
                Field::new("immediate_success", DataType::Boolean, false),
                Field::new("long_term_success", DataType::Float32, true),
                Field::new("timestamp", DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None), false),
                Field::new("session_id", DataType::Utf8, false),
                Field::new("model_used", DataType::Utf8, false),
            ]));

            // Create empty dataset
            let empty_batch = RecordBatch::new_empty(schema.clone());
            Dataset::write(empty_batch, &path, Some(WriteParams::default()))
                .await
                .context("Failed to create lance dataset")
        }
    }

    async fn init_duckdb(base_dir: &Path) -> Result<DuckDBConnection> {
        let db_path = base_dir.join("analytics.duckdb");
        let conn = DuckDBConnection::open(&db_path)?;

        // Create tables for analytics
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patterns (
                id VARCHAR PRIMARY KEY,
                description TEXT,
                trigger_context TEXT,
                action_count INTEGER,
                file_count INTEGER,
                immediate_success BOOLEAN,
                long_term_success DOUBLE,
                duration_ms BIGINT,
                timestamp TIMESTAMP,
                session_id VARCHAR,
                model_used VARCHAR
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS action_sequences (
                pattern_id VARCHAR,
                sequence_index INTEGER,
                tool VARCHAR,
                params JSON,
                success BOOLEAN,
                duration_ms BIGINT,
                FOREIGN KEY (pattern_id) REFERENCES patterns(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_changes (
                pattern_id VARCHAR,
                file_path VARCHAR,
                change_type VARCHAR,
                lines_added INTEGER,
                lines_removed INTEGER,
                FOREIGN KEY (pattern_id) REFERENCES patterns(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS pattern_relationships (
                pattern_id VARCHAR,
                related_pattern_id VARCHAR,
                relationship_type VARCHAR,
                strength DOUBLE,
                FOREIGN KEY (pattern_id) REFERENCES patterns(id)
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_patterns_timestamp ON patterns(timestamp)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_patterns_session ON patterns(session_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_file_changes_path ON file_changes(file_path)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_action_sequences_tool ON action_sequences(tool)", [])?;

        Ok(conn)
    }

    /// Record an intelligent pattern with full context
    pub async fn record_pattern(&mut self, pattern: IntelligentPattern) -> Result<()> {
        // Store in lance for vector search
        self.store_pattern_lance(&pattern).await?;

        // Store in DuckDB for analytics
        self.store_pattern_duckdb(&pattern).await?;

        Ok(())
    }

    async fn store_pattern_lance(&mut self, pattern: &IntelligentPattern) -> Result<()> {
        // Implementation for lance storage
        // Convert pattern to arrow format and append to dataset
        Ok(())
    }

    async fn store_pattern_duckdb(&self, pattern: &IntelligentPattern) -> Result<()> {
        let db = self.duck_db.write().await;

        // Insert main pattern record
        db.execute(
            "INSERT INTO patterns (id, description, trigger_context, action_count, file_count,
             immediate_success, long_term_success, duration_ms, timestamp, session_id, model_used)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &pattern.id,
                &pattern.description,
                &pattern.trigger_context,
                pattern.action_sequence.len() as i32,
                pattern.files_changed.len() as i32,
                pattern.immediate_success,
                pattern.long_term_success,
                pattern.action_sequence.iter().map(|a| a.duration_ms).sum::<u64>() as i64,
                pattern.timestamp.to_rfc3339(),
                &pattern.session_id,
                &pattern.model_used,
            ],
        )?;

        // Insert action sequence
        for (idx, action) in pattern.action_sequence.iter().enumerate() {
            let success = matches!(action.result, ActionResult::Success(_));
            db.execute(
                "INSERT INTO action_sequences (pattern_id, sequence_index, tool, params, success, duration_ms)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    &pattern.id,
                    idx as i32,
                    &action.tool,
                    serde_json::to_string(&action.params)?,
                    success,
                    action.duration_ms as i64,
                ],
            )?;
        }

        // Insert file changes
        for file_change in &pattern.files_changed {
            let change_type = match &file_change.change_type {
                ChangeType::Created => "created",
                ChangeType::Modified => "modified",
                ChangeType::Deleted => "deleted",
                ChangeType::Renamed { .. } => "renamed",
            };

            db.execute(
                "INSERT INTO file_changes (pattern_id, file_path, change_type, lines_added, lines_removed)
                 VALUES (?, ?, ?, ?, ?)",
                params![
                    &pattern.id,
                    &file_change.path,
                    change_type,
                    file_change.lines_added as i32,
                    file_change.lines_removed as i32,
                ],
            )?;
        }

        Ok(())
    }

    /// Predict next actions based on current context
    pub async fn predict_next_actions(
        &self,
        context_embedding: &[f32],
        limit: usize
    ) -> Result<Vec<PredictedAction>> {
        // 1. Find similar contexts using lance
        let similar_patterns = self.find_similar_patterns(context_embedding, 20).await?;

        // 2. Analyze successful action sequences in DuckDB
        let db = self.duck_db.read().await;

        let pattern_ids: Vec<String> = similar_patterns.iter()
            .map(|p| format!("'{}'", p.id))
            .collect();

        if pattern_ids.is_empty() {
            return Ok(vec![]);
        }

        let query = format!(
            "WITH successful_actions AS (
                SELECT
                    s.tool,
                    COUNT(*) as usage_count,
                    AVG(CASE WHEN s.success THEN 1.0 ELSE 0.0 END) as success_rate,
                    AVG(s.duration_ms) as avg_duration,
                    COUNT(DISTINCT s.pattern_id) as unique_patterns
                FROM action_sequences s
                JOIN patterns p ON s.pattern_id = p.id
                WHERE p.id IN ({})
                  AND p.immediate_success = true
                GROUP BY s.tool
                HAVING COUNT(*) > 2
            )
            SELECT
                tool as action,
                success_rate,
                unique_patterns as similar_contexts,
                avg_duration
            FROM successful_actions
            ORDER BY success_rate DESC, usage_count DESC
            LIMIT ?",
            pattern_ids.join(", ")
        );

        let mut stmt = db.prepare(&query)?;
        let predictions = stmt.query_map([limit], |row| {
            Ok(PredictedAction {
                action: row.get(0)?,
                confidence: row.get::<_, f64>(1)? as f32,
                similar_contexts: row.get::<_, i32>(2)? as usize,
                average_success_rate: row.get::<_, f64>(1)? as f32,
                typical_duration_ms: row.get::<_, i64>(3)? as u64,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(predictions)
    }

    /// Find file relationships (what changes together)
    pub async fn find_file_relationships(&self, file: &str) -> Result<Vec<FileRelationship>> {
        let db = self.duck_db.read().await;

        let query = "
            WITH file_patterns AS (
                SELECT DISTINCT pattern_id
                FROM file_changes
                WHERE file_path = ?
            ),
            related_files AS (
                SELECT
                    fc.file_path as related_file,
                    COUNT(*) as co_occurrence_count,
                    AVG(p.immediate_success::INT) as success_correlation
                FROM file_changes fc
                JOIN file_patterns fp ON fc.pattern_id = fp.pattern_id
                JOIN patterns p ON fc.pattern_id = p.id
                WHERE fc.file_path != ?
                GROUP BY fc.file_path
                HAVING COUNT(*) > 2
            )
            SELECT
                related_file,
                co_occurrence_count,
                success_correlation
            FROM related_files
            ORDER BY co_occurrence_count DESC
            LIMIT 10";

        let mut stmt = db.prepare(query)?;
        let relationships = stmt.query_map(params![file, file], |row| {
            Ok(FileRelationship {
                file: file.to_string(),
                related_file: row.get(0)?,
                co_occurrence_count: row.get::<_, i32>(1)? as usize,
                correlation_strength: row.get::<_, f64>(2)? as f32,
                typical_change_order: ChangeOrder::Simultaneous, // Simplified
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(relationships)
    }

    /// Analyze pattern trends over time
    pub async fn analyze_pattern_trend(&self, pattern_type: &str) -> Result<TrendAnalysis> {
        let db = self.duck_db.read().await;

        let query = "
            WITH weekly_stats AS (
                SELECT
                    DATE_TRUNC('week', timestamp) as week,
                    AVG(immediate_success::INT) as success_rate,
                    COUNT(*) as usage_count
                FROM patterns
                WHERE description LIKE '%' || ? || '%'
                  AND timestamp > CURRENT_TIMESTAMP - INTERVAL '90 days'
                GROUP BY week
                ORDER BY week DESC
            ),
            trend_calc AS (
                SELECT
                    success_rate,
                    usage_count,
                    LAG(success_rate, 1) OVER (ORDER BY week DESC) as prev_rate,
                    AVG(success_rate) OVER (ORDER BY week DESC ROWS BETWEEN 3 PRECEDING AND CURRENT ROW) as moving_avg
                FROM weekly_stats
            )
            SELECT
                AVG(success_rate) as current_rate,
                AVG(success_rate - COALESCE(prev_rate, success_rate)) as trend_delta,
                SUM(usage_count) as total_usage
            FROM trend_calc";

        let mut stmt = db.prepare(query)?;
        let mut rows = stmt.query_map([pattern_type], |row| {
            let current_rate: f64 = row.get(0)?;
            let trend_delta: f64 = row.get(1)?;
            let total_usage: i64 = row.get(2)?;

            let trend_direction = if trend_delta > 0.05 {
                TrendDirection::Improving
            } else if trend_delta < -0.05 {
                TrendDirection::Degrading
            } else {
                TrendDirection::Stable
            };

            let recommendation = match trend_direction {
                TrendDirection::Improving => "Continue using this pattern, it's becoming more effective",
                TrendDirection::Degrading => "Consider alternative approaches, effectiveness is declining",
                TrendDirection::Stable => "Pattern performance is consistent",
            }.to_string();

            Ok(TrendAnalysis {
                pattern_type: pattern_type.to_string(),
                current_success_rate: current_rate as f32,
                trend_direction,
                trend_strength: trend_delta.abs() as f32,
                usage_frequency: (total_usage as f32) / 90.0, // Per day average
                recommendation,
            })
        })?;

        rows.next()
            .ok_or_else(|| anyhow::anyhow!("No trend data found"))?
            .map_err(Into::into)
    }

    /// Find similar patterns using vector search
    async fn find_similar_patterns(&self, embedding: &[f32], limit: usize) -> Result<Vec<IntelligentPattern>> {
        // Use lance for similarity search
        let dataset = self.lance_db.as_ref()
            .context("Lance dataset not initialized")?;

        let results = dataset
            .scan()
            .nearest("embedding", embedding, limit)?
            .try_collect::<Vec<_>>()
            .await?;

        // Convert results to IntelligentPattern
        let mut patterns = Vec::new();
        for batch in results {
            // Extract pattern data from batch
            // ... conversion logic
        }

        Ok(patterns)
    }

    /// Get patterns that worked well for specific files
    pub async fn get_successful_file_patterns(&self, file: &str, limit: usize) -> Result<Vec<IntelligentPattern>> {
        let db = self.duck_db.read().await;

        let query = "
            SELECT DISTINCT p.id
            FROM patterns p
            JOIN file_changes fc ON p.id = fc.pattern_id
            WHERE fc.file_path = ?
              AND p.immediate_success = true
              AND (p.long_term_success IS NULL OR p.long_term_success > 0.7)
            ORDER BY p.timestamp DESC
            LIMIT ?";

        let mut stmt = db.prepare(query)?;
        let pattern_ids: Vec<String> = stmt.query_map(params![file, limit], |row| {
            row.get(0)
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Fetch full patterns from lance
        // ... implementation

        Ok(vec![])
    }

    /// Update long-term success metrics
    pub async fn update_long_term_success(&self, pattern_id: &str, success_score: f32) -> Result<()> {
        let db = self.duck_db.write().await;

        db.execute(
            "UPDATE patterns SET long_term_success = ? WHERE id = ?",
            params![success_score, pattern_id],
        )?;

        Ok(())
    }
}

/// Agent-friendly API wrapper
impl IntelligentMemory {
    /// Simple API: What should I do next?
    pub async fn suggest_next_action(&self, context: &str, embedding: &[f32]) -> Result<String> {
        let predictions = self.predict_next_actions(embedding, 3).await?;

        if predictions.is_empty() {
            return Ok("No specific suggestions based on past patterns".to_string());
        }

        let suggestions = predictions.iter()
            .map(|p| format!("{} ({}% confidence from {} similar cases)",
                p.action, (p.confidence * 100.0) as u32, p.similar_contexts))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!("Based on similar situations, consider:\n{}", suggestions))
    }

    /// Simple API: What files might need changes?
    pub async fn suggest_related_files(&self, current_file: &str) -> Result<Vec<String>> {
        let relationships = self.find_file_relationships(current_file).await?;

        Ok(relationships.into_iter()
            .filter(|r| r.correlation_strength > 0.5)
            .map(|r| r.related_file)
            .collect())
    }

    /// Simple API: Is this approach getting better or worse?
    pub async fn check_pattern_health(&self, pattern: &str) -> Result<String> {
        let trend = self.analyze_pattern_trend(pattern).await?;

        Ok(format!(
            "{} ({}% success rate, trend: {:?}). {}",
            pattern,
            (trend.current_success_rate * 100.0) as u32,
            trend.trend_direction,
            trend.recommendation
        ))
    }
}
