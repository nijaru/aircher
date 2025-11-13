use anyhow::{Context, Result};
use arrow::array::{
    Float32Array, StringArray, TimestampMillisecondArray, Float32Builder,
    StringBuilder, TimestampMillisecondBuilder, RecordBatch, ArrayRef, Int32Array, Int32Builder
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use chrono::{DateTime, Utc};
use lance::{Dataset, dataset::WriteParams};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;

/// Simplified pattern that the agent can actually use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPattern {
    pub description: String,      // What the pattern does
    pub context: String,          // When it was used
    pub success: bool,           // Did it work?
    pub files: Vec<String>,      // What files were involved
    pub timestamp: DateTime<Utc>, // When it happened
}

/// Simple memory system the agent can actually utilize
pub struct SimpleLanceMemory {
    base_dir: PathBuf,
    dataset: Option<Dataset>,
}

impl SimpleLanceMemory {
    /// Create new simple memory system
    pub async fn new(project_root: &Path) -> Result<Self> {
        let base_dir = project_root.join(".aircher").join("intelligence");
        fs::create_dir_all(&base_dir)?;

        let mut memory = Self {
            base_dir: base_dir.clone(),
            dataset: None,
        };

        memory.init_dataset().await?;
        Ok(memory)
    }

    async fn init_dataset(&mut self) -> Result<()> {
        let path = self.base_dir.join("patterns.lance");

        if path.exists() {
            self.dataset = Some(Dataset::open(&path).await?);
        } else {
            // Simple schema focused on what agent needs
            let schema = Arc::new(Schema::new(vec![
                Field::new("description", DataType::Utf8, false),
                Field::new("context", DataType::Utf8, false),
                Field::new("success", DataType::Boolean, false),
                Field::new("files", DataType::Utf8, true), // JSON array
                Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, None), false),
                Field::new("embedding", DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    384
                ), true), // Optional embedding for similarity search
            ]));

            // Create empty dataset
            let empty_batch = self.create_empty_batch(schema.clone())?;
            let params = WriteParams::default();
            self.dataset = Some(
                Dataset::write(empty_batch, &path, Some(params)).await?
            );
        }

        Ok(())
    }

    fn create_empty_batch(&self, schema: Arc<Schema>) -> Result<RecordBatch> {
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec![] as Vec<Option<String>>)) as ArrayRef,
                Arc::new(StringArray::from(vec![] as Vec<Option<String>>)) as ArrayRef,
                Arc::new(arrow::array::BooleanArray::from(vec![] as Vec<Option<bool>>)) as ArrayRef,
                Arc::new(StringArray::from(vec![] as Vec<Option<String>>)) as ArrayRef,
                Arc::new(TimestampMillisecondArray::from(vec![] as Vec<Option<i64>>)) as ArrayRef,
                Arc::new(arrow::array::FixedSizeListArray::from_iter_primitive::<Float32Array, _, _>(
                    vec![] as Vec<Option<Vec<Option<f32>>>>, 384
                )) as ArrayRef,
            ],
        ).context("Failed to create empty batch")
    }

    /// Remember what worked or didn't work
    pub async fn remember(&mut self, pattern: AgentPattern, embedding: Option<Vec<f32>>) -> Result<()> {
        let dataset = self.dataset.as_ref()
            .context("Dataset not initialized")?;

        // Build simple record
        let mut desc_builder = StringBuilder::new();
        let mut context_builder = StringBuilder::new();
        let mut success_builder = arrow::array::BooleanBuilder::new();
        let mut files_builder = StringBuilder::new();
        let mut timestamp_builder = TimestampMillisecondBuilder::new();

        desc_builder.append_value(&pattern.description);
        context_builder.append_value(&pattern.context);
        success_builder.append_value(pattern.success);
        files_builder.append_value(serde_json::to_string(&pattern.files)?);
        timestamp_builder.append_value(pattern.timestamp.timestamp_millis());

        // Handle optional embedding
        let embedding_array = if let Some(emb) = embedding {
            arrow::array::FixedSizeListArray::from_iter_primitive::<Float32Array, _, _>(
                vec![Some(emb.into_iter().map(Some).collect())], 384
            )
        } else {
            arrow::array::FixedSizeListArray::from_iter_primitive::<Float32Array, _, _>(
                vec![None], 384
            )
        };

        let batch = RecordBatch::try_new(
            dataset.schema().clone(),
            vec![
                Arc::new(desc_builder.finish()) as ArrayRef,
                Arc::new(context_builder.finish()) as ArrayRef,
                Arc::new(success_builder.finish()) as ArrayRef,
                Arc::new(files_builder.finish()) as ArrayRef,
                Arc::new(timestamp_builder.finish()) as ArrayRef,
                Arc::new(embedding_array) as ArrayRef,
            ],
        )?;

        // Append to dataset
        let path = self.base_dir.join("patterns.lance");
        let params = WriteParams::default();
        self.dataset = Some(
            Dataset::write(batch, &path, Some(params)).await?
        );

        Ok(())
    }

    /// Find patterns similar to current context
    pub async fn find_similar(&self, text: &str, embedding: &[f32], limit: usize) -> Result<Vec<AgentPattern>> {
        let dataset = self.dataset.as_ref()
            .context("Dataset not initialized")?;

        // Use lance's vector search
        let results = dataset
            .scan()
            .nearest("embedding", embedding, limit)?
            .try_collect::<Vec<_>>()
            .await?;

        let mut patterns = Vec::new();
        for batch in results {
            let desc_array = batch.column_by_name("description")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid description type")?;
            let context_array = batch.column_by_name("context")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid context type")?;
            let success_array = batch.column_by_name("success")?.as_any()
                .downcast_ref::<arrow::array::BooleanArray>().context("Invalid success type")?;
            let files_array = batch.column_by_name("files")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid files type")?;
            let timestamp_array = batch.column_by_name("timestamp")?.as_any()
                .downcast_ref::<TimestampMillisecondArray>().context("Invalid timestamp type")?;

            for i in 0..batch.num_rows() {
                let files: Vec<String> = if !files_array.is_null(i) {
                    serde_json::from_str(files_array.value(i))?
                } else {
                    vec![]
                };

                patterns.push(AgentPattern {
                    description: desc_array.value(i).to_string(),
                    context: context_array.value(i).to_string(),
                    success: success_array.value(i),
                    files,
                    timestamp: DateTime::from_timestamp_millis(timestamp_array.value(i))
                        .unwrap_or_else(|| Utc::now()),
                });
            }
        }

        Ok(patterns)
    }

    /// Get successful patterns for a specific file
    pub async fn get_patterns_for_file(&self, file: &str) -> Result<Vec<AgentPattern>> {
        let dataset = self.dataset.as_ref()
            .context("Dataset not initialized")?;

        // Filter by file and success
        let filter = format!("files LIKE '%{}%' AND success = true", file);
        let results = dataset
            .scan()
            .filter(&filter)?
            .try_collect::<Vec<_>>()
            .await?;

        let mut patterns = Vec::new();
        for batch in results {
            // Similar extraction logic as find_similar
            let desc_array = batch.column_by_name("description")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid description type")?;
            let context_array = batch.column_by_name("context")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid context type")?;
            let files_array = batch.column_by_name("files")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid files type")?;

            for i in 0..batch.num_rows() {
                let files: Vec<String> = if !files_array.is_null(i) {
                    serde_json::from_str(files_array.value(i))?
                } else {
                    vec![]
                };

                patterns.push(AgentPattern {
                    description: desc_array.value(i).to_string(),
                    context: context_array.value(i).to_string(),
                    success: true, // We filtered for success
                    files,
                    timestamp: Utc::now(), // Simplified
                });
            }
        }

        Ok(patterns)
    }

    /// Get recent successful patterns (what's been working lately)
    pub async fn get_recent_successes(&self, limit: usize) -> Result<Vec<AgentPattern>> {
        let dataset = self.dataset.as_ref()
            .context("Dataset not initialized")?;

        // Get successful patterns ordered by recency
        let results = dataset
            .scan()
            .filter("success = true")?
            .limit(limit)?
            .try_collect::<Vec<_>>()
            .await?;

        let mut patterns = Vec::new();
        for batch in results {
            // Extract patterns (similar to above methods)
            let desc_array = batch.column_by_name("description")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid description type")?;
            let context_array = batch.column_by_name("context")?.as_any()
                .downcast_ref::<StringArray>().context("Invalid context type")?;

            for i in 0..batch.num_rows() {
                patterns.push(AgentPattern {
                    description: desc_array.value(i).to_string(),
                    context: context_array.value(i).to_string(),
                    success: true,
                    files: vec![],
                    timestamp: Utc::now(),
                });
            }
        }

        Ok(patterns)
    }
}

/// Integration with AgentController - simple functions the agent can actually call
impl SimpleLanceMemory {
    /// Agent calls this when something works
    pub async fn record_success(&mut self, what: &str, context: &str, files: Vec<String>) -> Result<()> {
        self.remember(AgentPattern {
            description: what.to_string(),
            context: context.to_string(),
            success: true,
            files,
            timestamp: Utc::now(),
        }, None).await
    }

    /// Agent calls this when something fails
    pub async fn record_failure(&mut self, what: &str, context: &str, files: Vec<String>) -> Result<()> {
        self.remember(AgentPattern {
            description: what.to_string(),
            context: context.to_string(),
            success: false,
            files,
            timestamp: Utc::now(),
        }, None).await
    }

    /// Agent calls this to get context for current task
    pub async fn get_relevant_patterns(&self, context: &str, embedding: Option<&[f32]>) -> Result<String> {
        let patterns = if let Some(emb) = embedding {
            self.find_similar(context, emb, 3).await?
        } else {
            self.get_recent_successes(3).await?
        };

        if patterns.is_empty() {
            return Ok(String::new());
        }

        Ok(format!(
            "Similar successful patterns from this project:\n{}",
            patterns.iter()
                .filter(|p| p.success)
                .map(|p| format!("- {}: {}", p.description, p.context))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_simple_memory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut memory = SimpleLanceMemory::new(temp_dir.path()).await.unwrap();

        // Record a success
        memory.record_success(
            "Fixed compilation error",
            "User asked to fix rust error in main.rs",
            vec!["src/main.rs".to_string()]
        ).await.unwrap();

        // Get recent successes
        let successes = memory.get_recent_successes(5).await.unwrap();
        assert!(!successes.is_empty());
        assert_eq!(successes[0].description, "Fixed compilation error");

        // Get patterns for file
        let file_patterns = memory.get_patterns_for_file("main.rs").await.unwrap();
        assert!(!file_patterns.is_empty());
    }
}
