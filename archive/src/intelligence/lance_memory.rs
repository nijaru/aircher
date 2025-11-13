use anyhow::{Context, Result};
use arrow::array::{
    Float32Array, StringArray, TimestampMillisecondArray, Float32Builder,
    StringBuilder, TimestampMillisecondBuilder, RecordBatch, FixedSizeListArray,
    ArrayRef, FixedSizeListBuilder, Int32Array, Int32Builder
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use chrono::{DateTime, Utc};
use lance::{Dataset, Error as LanceError, dataset::WriteParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::fs;

/// Intelligence memory using lance-rs for vector storage and analytics
pub struct LanceMemory {
    base_dir: PathBuf,
    patterns_dataset: Option<Dataset>,
    sessions_dataset: Option<Dataset>,
    preferences_dataset: Option<Dataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub pattern_type: String,
    pub description: String,
    pub embedding: Vec<f32>,
    pub success_rate: f32,
    pub usage_count: i32,
    pub files_involved: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub patterns_used: Vec<String>,
    pub success_score: f32,
    pub context: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub key: String,
    pub value: String,
    pub category: String,
    pub updated_at: DateTime<Utc>,
}

impl LanceMemory {
    /// Create new lance-based memory system
    pub async fn new(project_root: &Path) -> Result<Self> {
        let base_dir = project_root.join(".aircher").join("intelligence");
        fs::create_dir_all(&base_dir)?;

        let mut memory = Self {
            base_dir: base_dir.clone(),
            patterns_dataset: None,
            sessions_dataset: None,
            preferences_dataset: None,
        };

        // Initialize datasets
        memory.init_patterns_dataset().await?;
        memory.init_sessions_dataset().await?;
        memory.init_preferences_dataset().await?;

        Ok(memory)
    }

    async fn init_patterns_dataset(&mut self) -> Result<()> {
        let path = self.base_dir.join("patterns.lance");

        if path.exists() {
            // Open existing dataset
            self.patterns_dataset = Some(Dataset::open(&path).await?);
        } else {
            // Create new dataset with schema
            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("pattern_type", DataType::Utf8, false),
                Field::new("description", DataType::Utf8, false),
                Field::new("embedding", DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    384  // Embedding dimension
                ), false),
                Field::new("success_rate", DataType::Float32, false),
                Field::new("usage_count", DataType::Int32, false),
                Field::new("files_involved", DataType::Utf8, true), // JSON array as string
                Field::new("created_at", DataType::Timestamp(TimeUnit::Millisecond, None), false),
                Field::new("last_used", DataType::Timestamp(TimeUnit::Millisecond, None), false),
                Field::new("metadata", DataType::Utf8, true), // JSON as string
            ]));

            // Create empty dataset
            let empty_batch = self.create_empty_patterns_batch(schema.clone())?;
            let params = WriteParams::default();
            self.patterns_dataset = Some(
                Dataset::write(empty_batch, &path, Some(params)).await?
            );
        }

        Ok(())
    }

    async fn init_sessions_dataset(&mut self) -> Result<()> {
        let path = self.base_dir.join("sessions.lance");

        if path.exists() {
            self.sessions_dataset = Some(Dataset::open(&path).await?);
        } else {
            let schema = Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("start_time", DataType::Timestamp(TimeUnit::Millisecond, None), false),
                Field::new("end_time", DataType::Timestamp(TimeUnit::Millisecond, None), true),
                Field::new("patterns_used", DataType::Utf8, true), // JSON array
                Field::new("success_score", DataType::Float32, false),
                Field::new("context", DataType::Utf8, false),
                Field::new("metadata", DataType::Utf8, true), // JSON
            ]));

            let empty_batch = self.create_empty_sessions_batch(schema.clone())?;
            let params = WriteParams::default();
            self.sessions_dataset = Some(
                Dataset::write(empty_batch, &path, Some(params)).await?
            );
        }

        Ok(())
    }

    async fn init_preferences_dataset(&mut self) -> Result<()> {
        let path = self.base_dir.join("preferences.lance");

        if path.exists() {
            self.preferences_dataset = Some(Dataset::open(&path).await?);
        } else {
            let schema = Arc::new(Schema::new(vec![
                Field::new("key", DataType::Utf8, false),
                Field::new("value", DataType::Utf8, false),
                Field::new("category", DataType::Utf8, false),
                Field::new("updated_at", DataType::Timestamp(TimeUnit::Millisecond, None), false),
            ]));

            let empty_batch = self.create_empty_preferences_batch(schema.clone())?;
            let params = WriteParams::default();
            self.preferences_dataset = Some(
                Dataset::write(empty_batch, &path, Some(params)).await?
            );
        }

        Ok(())
    }

    fn create_empty_patterns_batch(&self, schema: Arc<Schema>) -> Result<RecordBatch> {
        let id_array = StringArray::from(vec![] as Vec<Option<String>>);
        let pattern_type_array = StringArray::from(vec![] as Vec<Option<String>>);
        let description_array = StringArray::from(vec![] as Vec<Option<String>>);

        // Create empty embedding array
        let embedding_builder = FixedSizeListBuilder::new(Float32Builder::new(), 384);
        let embedding_array = embedding_builder.finish();

        let success_rate_array = Float32Array::from(vec![] as Vec<Option<f32>>);
        let usage_count_array = Int32Array::from(vec![] as Vec<Option<i32>>);
        let files_array = StringArray::from(vec![] as Vec<Option<String>>);
        let created_at_array = TimestampMillisecondArray::from(vec![] as Vec<Option<i64>>);
        let last_used_array = TimestampMillisecondArray::from(vec![] as Vec<Option<i64>>);
        let metadata_array = StringArray::from(vec![] as Vec<Option<String>>);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(pattern_type_array) as ArrayRef,
                Arc::new(description_array) as ArrayRef,
                Arc::new(embedding_array) as ArrayRef,
                Arc::new(success_rate_array) as ArrayRef,
                Arc::new(usage_count_array) as ArrayRef,
                Arc::new(files_array) as ArrayRef,
                Arc::new(created_at_array) as ArrayRef,
                Arc::new(last_used_array) as ArrayRef,
                Arc::new(metadata_array) as ArrayRef,
            ],
        ).context("Failed to create empty patterns batch")
    }

    fn create_empty_sessions_batch(&self, schema: Arc<Schema>) -> Result<RecordBatch> {
        let id_array = StringArray::from(vec![] as Vec<Option<String>>);
        let start_time_array = TimestampMillisecondArray::from(vec![] as Vec<Option<i64>>);
        let end_time_array = TimestampMillisecondArray::from(vec![] as Vec<Option<i64>>);
        let patterns_array = StringArray::from(vec![] as Vec<Option<String>>);
        let success_array = Float32Array::from(vec![] as Vec<Option<f32>>);
        let context_array = StringArray::from(vec![] as Vec<Option<String>>);
        let metadata_array = StringArray::from(vec![] as Vec<Option<String>>);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(start_time_array) as ArrayRef,
                Arc::new(end_time_array) as ArrayRef,
                Arc::new(patterns_array) as ArrayRef,
                Arc::new(success_array) as ArrayRef,
                Arc::new(context_array) as ArrayRef,
                Arc::new(metadata_array) as ArrayRef,
            ],
        ).context("Failed to create empty sessions batch")
    }

    fn create_empty_preferences_batch(&self, schema: Arc<Schema>) -> Result<RecordBatch> {
        let key_array = StringArray::from(vec![] as Vec<Option<String>>);
        let value_array = StringArray::from(vec![] as Vec<Option<String>>);
        let category_array = StringArray::from(vec![] as Vec<Option<String>>);
        let updated_at_array = TimestampMillisecondArray::from(vec![] as Vec<Option<i64>>);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(key_array) as ArrayRef,
                Arc::new(value_array) as ArrayRef,
                Arc::new(category_array) as ArrayRef,
                Arc::new(updated_at_array) as ArrayRef,
            ],
        ).context("Failed to create empty preferences batch")
    }

    /// Add a new pattern to memory
    pub async fn add_pattern(&mut self, pattern: Pattern) -> Result<()> {
        let dataset = self.patterns_dataset.as_ref()
            .context("Patterns dataset not initialized")?;

        // Build record batch for new pattern
        let mut id_builder = StringBuilder::new();
        let mut type_builder = StringBuilder::new();
        let mut desc_builder = StringBuilder::new();
        let mut embedding_builder = FixedSizeListBuilder::new(Float32Builder::new(), 384);
        let mut success_builder = Float32Builder::new();
        let mut usage_builder = Int32Builder::new();
        let mut files_builder = StringBuilder::new();
        let mut created_builder = TimestampMillisecondBuilder::new();
        let mut last_used_builder = TimestampMillisecondBuilder::new();
        let mut metadata_builder = StringBuilder::new();

        id_builder.append_value(&pattern.id);
        type_builder.append_value(&pattern.pattern_type);
        desc_builder.append_value(&pattern.description);

        // Add embedding
        let values_builder = embedding_builder.values();
        for val in &pattern.embedding {
            values_builder.append_value(*val);
        }
        embedding_builder.append(true);

        success_builder.append_value(pattern.success_rate);
        usage_builder.append_value(pattern.usage_count);
        files_builder.append_value(serde_json::to_string(&pattern.files_involved)?);
        created_builder.append_value(pattern.created_at.timestamp_millis());
        last_used_builder.append_value(pattern.last_used.timestamp_millis());
        metadata_builder.append_value(serde_json::to_string(&pattern.metadata)?);

        let batch = RecordBatch::try_new(
            dataset.schema().clone(),
            vec![
                Arc::new(id_builder.finish()) as ArrayRef,
                Arc::new(type_builder.finish()) as ArrayRef,
                Arc::new(desc_builder.finish()) as ArrayRef,
                Arc::new(embedding_builder.finish()) as ArrayRef,
                Arc::new(success_builder.finish()) as ArrayRef,
                Arc::new(usage_builder.finish()) as ArrayRef,
                Arc::new(files_builder.finish()) as ArrayRef,
                Arc::new(created_builder.finish()) as ArrayRef,
                Arc::new(last_used_builder.finish()) as ArrayRef,
                Arc::new(metadata_builder.finish()) as ArrayRef,
            ],
        )?;

        // Append to dataset
        let path = self.base_dir.join("patterns.lance");
        let params = WriteParams::default();
        self.patterns_dataset = Some(
            Dataset::write(batch, &path, Some(params)).await?
        );

        Ok(())
    }

    /// Search for similar patterns using vector similarity
    pub async fn search_similar_patterns(
        &self,
        embedding: &[f32],
        limit: usize
    ) -> Result<Vec<Pattern>> {
        let dataset = self.patterns_dataset.as_ref()
            .context("Patterns dataset not initialized")?;

        // Use lance's vector search
        let results = dataset
            .scan()
            .nearest("embedding", embedding, limit)?
            .try_collect::<Vec<_>>()
            .await?;

        // Convert results to Pattern structs
        let mut patterns = Vec::new();
        for batch in results {
            let id_array = batch.column_by_name("id")
                .context("Missing id column")?
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Invalid id column type")?;

            let type_array = batch.column_by_name("pattern_type")
                .context("Missing pattern_type column")?
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Invalid pattern_type column type")?;

            let desc_array = batch.column_by_name("description")
                .context("Missing description column")?
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Invalid description column type")?;

            let success_array = batch.column_by_name("success_rate")
                .context("Missing success_rate column")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .context("Invalid success_rate column type")?;

            let usage_array = batch.column_by_name("usage_count")
                .context("Missing usage_count column")?
                .as_any()
                .downcast_ref::<Int32Array>()
                .context("Invalid usage_count column type")?;

            for i in 0..batch.num_rows() {
                patterns.push(Pattern {
                    id: id_array.value(i).to_string(),
                    pattern_type: type_array.value(i).to_string(),
                    description: desc_array.value(i).to_string(),
                    embedding: vec![], // Skip embedding for now
                    success_rate: success_array.value(i),
                    usage_count: usage_array.value(i),
                    files_involved: vec![],
                    created_at: Utc::now(),
                    last_used: Utc::now(),
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(patterns)
    }

    /// Get patterns by type
    pub async fn get_patterns_by_type(&self, pattern_type: &str) -> Result<Vec<Pattern>> {
        let dataset = self.patterns_dataset.as_ref()
            .context("Patterns dataset not initialized")?;

        // Filter by pattern type
        let filter = format!("pattern_type = '{}'", pattern_type);
        let results = dataset
            .scan()
            .filter(&filter)?
            .try_collect::<Vec<_>>()
            .await?;

        // Convert to Pattern structs (similar to search_similar_patterns)
        let mut patterns = Vec::new();
        for batch in results {
            // ... conversion logic similar to above
        }

        Ok(patterns)
    }

    /// Add a new session
    pub async fn add_session(&mut self, session: Session) -> Result<()> {
        let dataset = self.sessions_dataset.as_ref()
            .context("Sessions dataset not initialized")?;

        // Build record batch for new session
        let mut id_builder = StringBuilder::new();
        let mut start_builder = TimestampMillisecondBuilder::new();
        let mut end_builder = TimestampMillisecondBuilder::new();
        let mut patterns_builder = StringBuilder::new();
        let mut success_builder = Float32Builder::new();
        let mut context_builder = StringBuilder::new();
        let mut metadata_builder = StringBuilder::new();

        id_builder.append_value(&session.id);
        start_builder.append_value(session.start_time.timestamp_millis());

        if let Some(end_time) = session.end_time {
            end_builder.append_value(end_time.timestamp_millis());
        } else {
            end_builder.append_null();
        }

        patterns_builder.append_value(serde_json::to_string(&session.patterns_used)?);
        success_builder.append_value(session.success_score);
        context_builder.append_value(&session.context);
        metadata_builder.append_value(serde_json::to_string(&session.metadata)?);

        let batch = RecordBatch::try_new(
            dataset.schema().clone(),
            vec![
                Arc::new(id_builder.finish()) as ArrayRef,
                Arc::new(start_builder.finish()) as ArrayRef,
                Arc::new(end_builder.finish()) as ArrayRef,
                Arc::new(patterns_builder.finish()) as ArrayRef,
                Arc::new(success_builder.finish()) as ArrayRef,
                Arc::new(context_builder.finish()) as ArrayRef,
                Arc::new(metadata_builder.finish()) as ArrayRef,
            ],
        )?;

        // Append to dataset
        let path = self.base_dir.join("sessions.lance");
        let params = WriteParams::default();
        self.sessions_dataset = Some(
            Dataset::write(batch, &path, Some(params)).await?
        );

        Ok(())
    }

    /// Update user preference
    pub async fn set_preference(&mut self, pref: UserPreference) -> Result<()> {
        let dataset = self.preferences_dataset.as_ref()
            .context("Preferences dataset not initialized")?;

        // Build record batch
        let mut key_builder = StringBuilder::new();
        let mut value_builder = StringBuilder::new();
        let mut category_builder = StringBuilder::new();
        let mut updated_builder = TimestampMillisecondBuilder::new();

        key_builder.append_value(&pref.key);
        value_builder.append_value(&pref.value);
        category_builder.append_value(&pref.category);
        updated_builder.append_value(pref.updated_at.timestamp_millis());

        let batch = RecordBatch::try_new(
            dataset.schema().clone(),
            vec![
                Arc::new(key_builder.finish()) as ArrayRef,
                Arc::new(value_builder.finish()) as ArrayRef,
                Arc::new(category_builder.finish()) as ArrayRef,
                Arc::new(updated_builder.finish()) as ArrayRef,
            ],
        )?;

        // Append to dataset
        let path = self.base_dir.join("preferences.lance");
        let params = WriteParams::default();
        self.preferences_dataset = Some(
            Dataset::write(batch, &path, Some(params)).await?
        );

        Ok(())
    }

    /// Get analytics on pattern effectiveness
    pub async fn get_pattern_analytics(&self) -> Result<HashMap<String, f32>> {
        let dataset = self.patterns_dataset.as_ref()
            .context("Patterns dataset not initialized")?;

        // Get average success rate by pattern type
        let results = dataset
            .scan()
            .try_collect::<Vec<_>>()
            .await?;

        let mut analytics = HashMap::new();
        let mut type_totals: HashMap<String, (f32, i32)> = HashMap::new();

        for batch in results {
            let type_array = batch.column_by_name("pattern_type")
                .context("Missing pattern_type column")?
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Invalid pattern_type column type")?;

            let success_array = batch.column_by_name("success_rate")
                .context("Missing success_rate column")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .context("Invalid success_rate column type")?;

            for i in 0..batch.num_rows() {
                let pattern_type = type_array.value(i);
                let success_rate = success_array.value(i);

                let entry = type_totals.entry(pattern_type.to_string())
                    .or_insert((0.0, 0));
                entry.0 += success_rate;
                entry.1 += 1;
            }
        }

        // Calculate averages
        for (pattern_type, (total, count)) in type_totals {
            if count > 0 {
                analytics.insert(pattern_type, total / count as f32);
            }
        }

        Ok(analytics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_lance_memory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let memory = LanceMemory::new(temp_dir.path()).await.unwrap();

        // Check that directories were created
        assert!(temp_dir.path().join(".aircher").join("intelligence").exists());
    }

    #[tokio::test]
    async fn test_pattern_storage_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let mut memory = LanceMemory::new(temp_dir.path()).await.unwrap();

        // Create a test pattern
        let pattern = Pattern {
            id: "test-pattern-1".to_string(),
            pattern_type: "debugging".to_string(),
            description: "Fix compilation errors".to_string(),
            embedding: vec![0.1; 384], // Dummy embedding
            success_rate: 0.85,
            usage_count: 10,
            files_involved: vec!["main.rs".to_string()],
            created_at: Utc::now(),
            last_used: Utc::now(),
            metadata: HashMap::new(),
        };

        // Add pattern
        memory.add_pattern(pattern.clone()).await.unwrap();

        // Search for similar patterns
        let results = memory.search_similar_patterns(&pattern.embedding, 5).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "test-pattern-1");
    }
}
