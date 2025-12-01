//! Enhanced Pipeline Execution Module
//!
//! Provides realistic pipeline execution functionality for the Tauri application.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<PipelineStep>,
    pub status: PipelineStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, Value>,
}

/// Pipeline step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub config: Value,
    pub timeout_seconds: u64,
}

/// Step types supported by the pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    TextProcessing,
    DataTransform,
    ApiCall,
    Condition,
    Loop,
    Custom(String),
}

/// Pipeline execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    Draft,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Pipeline execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecution {
    pub id: String,
    pub pipeline_id: String,
    pub status: PipelineStatus,
    pub input: Value,
    pub output: Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub step_results: Vec<StepResult>,
    pub execution_time_ms: Option<u64>,
}

/// Result of a single pipeline step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub step_name: String,
    pub status: PipelineStatus,
    pub input: Value,
    pub output: Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

/// Pipeline executor
pub struct PipelineExecutor {
    pipelines: HashMap<String, Pipeline>,
    executions: HashMap<String, PipelineExecution>,
}

impl PipelineExecutor {
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            executions: HashMap::new(),
        }
    }

    /// Initialize the pipeline executor
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Initialize with sample pipelines
        self.initialize_sample_pipelines();
        Ok(())
    }

    /// Initialize sample pipelines for demonstration
    fn initialize_sample_pipelines(&mut self) {
        let sample_pipelines = vec![
            Pipeline {
                id: "pipeline-text-processor".to_string(),
                name: "Text Processor".to_string(),
                description: "Processes text input and applies various transformations".to_string(),
                steps: vec![
                    PipelineStep {
                        id: "step-1".to_string(),
                        name: "Input Validation".to_string(),
                        step_type: StepType::TextProcessing,
                        config: serde_json::json!({
                            "operation": "validate",
                            "required_fields": ["content"]
                        }),
                        timeout_seconds: 30,
                    },
                    PipelineStep {
                        id: "step-2".to_string(),
                        name: "Text Cleanup".to_string(),
                        step_type: StepType::TextProcessing,
                        config: serde_json::json!({
                            "operation": "cleanup",
                            "remove_extra_whitespace": true,
                            "normalize_line_endings": true
                        }),
                        timeout_seconds: 60,
                    },
                    PipelineStep {
                        id: "step-3".to_string(),
                        name: "Format Output".to_string(),
                        step_type: StepType::TextProcessing,
                        config: serde_json::json!({
                            "operation": "format",
                            "output_type": "formatted_text"
                        }),
                        timeout_seconds: 30,
                    },
                ],
                status: PipelineStatus::Ready,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("category".to_string(), Value::String("text-processing".to_string())),
                    ("version".to_string(), Value::String("1.0".to_string())),
                ]),
            },
            Pipeline {
                id: "pipeline-data-transform".to_string(),
                name: "Data Transformer".to_string(),
                description: "Transforms data between different formats".to_string(),
                steps: vec![
                    PipelineStep {
                        id: "step-1".to_string(),
                        name: "Parse Input".to_string(),
                        step_type: StepType::DataTransform,
                        config: serde_json::json!({
                            "operation": "parse",
                            "input_format": "json"
                        }),
                        timeout_seconds: 30,
                    },
                    PipelineStep {
                        id: "step-2".to_string(),
                        name: "Transform Data".to_string(),
                        step_type: StepType::DataTransform,
                        config: serde_json::json!({
                            "operation": "map",
                            "transformations": [
                                {"field": "value", "operation": "multiply", "factor": 2},
                                {"field": "status", "operation": "uppercase"}
                            ]
                        }),
                        timeout_seconds: 60,
                    },
                    PipelineStep {
                        id: "step-3".to_string(),
                        name: "Serialize Output".to_string(),
                        step_type: StepType::DataTransform,
                        config: serde_json::json!({
                            "operation": "serialize",
                            "output_format": "json"
                        }),
                        timeout_seconds: 30,
                    },
                ],
                status: PipelineStatus::Ready,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                metadata: HashMap::from([
                    ("category".to_string(), Value::String("data-transformation".to_string())),
                    ("version".to_string(), Value::String("1.0".to_string())),
                ]),
            },
        ];

        for pipeline in sample_pipelines {
            self.pipelines.insert(pipeline.id.clone(), pipeline);
        }
    }

    /// Create a new pipeline
    pub fn create_pipeline(&mut self, pipeline: Pipeline) -> Result<&Pipeline, String> {
        // Validate pipeline
        if pipeline.name.is_empty() {
            return Err("Pipeline name cannot be empty".to_string());
        }

        if pipeline.steps.is_empty() {
            return Err("Pipeline must have at least one step".to_string());
        }

        // Validate steps
        for (i, step) in pipeline.steps.iter().enumerate() {
            if step.name.is_empty() {
                return Err(format!("Step {} cannot have empty name", i + 1));
            }
            if step.timeout_seconds == 0 {
                return Err(format!("Step {} must have a positive timeout", i + 1));
            }
        }

        self.pipelines.insert(pipeline.id.clone(), pipeline.clone());
        Ok(self.pipelines.get(&pipeline.id.clone()).unwrap())
    }

    /// List all pipelines
    pub fn list_pipelines(&self) -> Vec<&Pipeline> {
        self.pipelines.values().collect()
    }

    /// Get pipeline by ID
    pub fn get_pipeline(&self, pipeline_id: &str) -> Option<&Pipeline> {
        self.pipelines.get(pipeline_id)
    }

    /// Update pipeline
    pub fn update_pipeline(&mut self, pipeline_id: &str, mut pipeline: Pipeline) -> Result<&Pipeline, String> {
        if !self.pipelines.contains_key(pipeline_id) {
            return Err("Pipeline not found".to_string());
        }

        pipeline.id = pipeline_id.to_string();
        pipeline.updated_at = Utc::now();

        self.pipelines.insert(pipeline_id.to_string(), pipeline);
        Ok(self.pipelines.get(pipeline_id).unwrap())
    }

    /// Delete pipeline
    pub fn delete_pipeline(&mut self, pipeline_id: &str) -> Option<Pipeline> {
        self.pipelines.remove(pipeline_id)
    }

    /// Execute a pipeline
    pub async fn execute_pipeline(
        &mut self,
        pipeline_id: &str,
        input: Value,
    ) -> Result<&PipelineExecution, String> {
        let pipeline = self.pipelines.get(pipeline_id)
            .ok_or_else(|| "Pipeline not found".to_string())?
            .clone();

        if pipeline.status != PipelineStatus::Ready {
            return Err("Pipeline is not ready for execution".to_string());
        }

        let execution_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();

        let mut execution = PipelineExecution {
            id: execution_id.clone(),
            pipeline_id: pipeline_id.to_string(),
            status: PipelineStatus::Running,
            input: input.clone(),
            output: Value::Null,
            started_at,
            completed_at: None,
            error_message: None,
            step_results: Vec::new(),
            execution_time_ms: None,
        };

        // Execute each step
        let mut current_data = input;
        let mut total_time = 0u64;

        for step in &pipeline.steps {
            let step_start = Utc::now();

            let step_result = self.execute_step(step, current_data.clone()).await;
            let step_end = Utc::now();
            let step_duration = (step_end - step_start).num_milliseconds() as u64;

            total_time += step_duration;

            let step_result = StepResult {
                step_id: step.id.clone(),
                step_name: step.name.clone(),
                status: step_result.status.clone(),
                input: current_data.clone(),
                output: step_result.output,
                started_at: step_start,
                completed_at: Some(step_end),
                error_message: step_result.error_message,
                execution_time_ms: step_duration,
            };

            current_data = step_result.output.clone();

            // If step failed, stop execution
            if step_result.status == PipelineStatus::Failed {
                execution.status = PipelineStatus::Failed;
                execution.error_message = step_result.error_message;
                execution.completed_at = Some(step_end);
                execution.step_results.push(step_result);
                break;
            }

            execution.step_results.push(step_result);
        }

        // Update execution status
        let completed_at = Utc::now();
        execution.status = if execution.error_message.is_some() {
            PipelineStatus::Failed
        } else {
            PipelineStatus::Completed
        };
        execution.output = current_data;
        execution.completed_at = Some(completed_at);
        execution.execution_time_ms = Some(total_time);

        self.executions.insert(execution_id.clone(), execution);
        Ok(self.executions.get(&execution_id).unwrap())
    }

    /// Execute a single pipeline step
    async fn execute_step(&self, step: &PipelineStep, input: Value) -> StepResult {
        let started_at = Utc::now();

        // Simulate step execution time
        let execution_time = std::time::Duration::from_millis(step.timeout_seconds * 1000);

        // For demo purposes, we'll use a shorter time
        let simulated_time = std::time::Duration::from_millis(
            std::cmp::min(step.timeout_seconds * 100, 1000) // Cap at 1 second for demo
        );
        tokio::time::sleep(simulated_time).await;

        let result = match step.step_type {
            StepType::TextProcessing => self.execute_text_processing_step(step, &input),
            StepType::DataTransform => self.execute_data_transform_step(step, &input),
            StepType::ApiCall => self.execute_api_call_step(step, &input).await,
            StepType::Condition => self.execute_condition_step(step, &input),
            StepType::Loop => self.execute_loop_step(step, &input),
            StepType::Custom(_) => self.execute_custom_step(step, &input),
        };

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: result.status,
            input,
            output: result.output,
            started_at,
            completed_at: Some(Utc::now()),
            error_message: result.error_message,
            execution_time_ms: simulated_time.as_millis() as u64,
        }
    }

    /// Execute text processing step
    fn execute_text_processing_step(&self, step: &PipelineStep, input: &Value) -> StepResult {
        let operation = step.config.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("validate");

        let output = match operation {
            "validate" => {
                if let Some(text) = input.get("content").and_then(|v| v.as_str()) {
                    if !text.is_empty() {
                        serde_json::json!({
                            "valid": true,
                            "content": text,
                            "length": text.len(),
                            "word_count": text.split_whitespace().count()
                        })
                    } else {
                        serde_json::json!({
                            "valid": false,
                            "error": "Content cannot be empty"
                        })
                    }
                } else {
                    serde_json::json!({
                        "valid": false,
                        "error": "Missing 'content' field"
                    })
                }
            },
            "cleanup" => {
                if let Some(text) = input.get("content").and_then(|v| v.as_str()) {
                    let remove_whitespace = step.config.get("remove_extra_whitespace")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    let normalize_line_endings = step.config.get("normalize_line_endings")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    let mut cleaned = text.to_string();
                    if remove_whitespace {
                        cleaned = cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");
                    }
                    if normalize_line_endings {
                        cleaned = cleaned.replace("\r\n", "\n").replace('\r', '\n');
                    }

                    serde_json::json!({
                        "content": cleaned,
                        "original_length": text.len(),
                        "cleaned_length": cleaned.len(),
                        "removed_chars": text.len() - cleaned.len()
                    })
                } else {
                    serde_json::json!({
                        "error": "Missing 'content' field"
                    })
                }
            },
            "format" => {
                let output_type = step.config.get("output_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("plain");

                if let Some(content) = input.get("content").and_then(|v| v.as_str()) {
                    match output_type {
                        "formatted_text" => serde_json::json!({
                            "content": format!("=== Formatted Text ===\n\n{}", content),
                            "format_type": "formatted_text"
                        }),
                        "json" => serde_json::json!({
                            "content": content,
                            "format_type": "json",
                            "wrapped": format!("{{\"text\": \"{}\"}}", content)
                        }),
                        _ => serde_json::json!({
                            "content": content,
                            "format_type": output_type
                        })
                    }
                } else {
                    serde_json::json!({
                        "error": "Missing 'content' field"
                    })
                }
            },
            _ => serde_json::json!({
                "error": format!("Unknown text processing operation: {}", operation)
            })
        };

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: if output.get("error").is_some() {
                PipelineStatus::Failed
            } else {
                PipelineStatus::Completed
            },
            input: input.clone(),
            output: output.clone(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: output.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()),
            execution_time_ms: 0, // Will be set by caller
        }
    }

    /// Execute data transformation step
    fn execute_data_transform_step(&self, step: &PipelineStep, input: &Value) -> StepResult {
        let operation = step.config.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("parse");

        let output = match operation {
            "parse" => {
                let input_format = step.config.get("input_format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json");

                match input_format {
                    "json" => {
                        if input.is_object() || input.is_array() {
                            serde_json::json!({
                                "parsed_data": input,
                                "format": "json",
                                "validation": "passed"
                            })
                        } else {
                            serde_json::json!({
                                "error": "Invalid JSON input",
                                "input_type": "not_object_or_array"
                            })
                        }
                    },
                    _ => serde_json::json!({
                        "error": format!("Unsupported input format: {}", input_format)
                    })
                }
            },
            "serialize" => {
                let output_format = step.config.get("output_format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("json");

                match output_format {
                    "json" => {
                        serde_json::json!({
                            "serialized_data": input,
                            "format": "json"
                        })
                    },
                    "csv" => {
                        if let Some(data) = input.get("data") {
                            // Simple CSV serialization for array of objects
                            if let Some(array) = data.as_array() {
                                let headers = array.get(0)
                                    .and_then(|obj| obj.as_object())
                                    .map(|obj| obj.keys().collect::<Vec<_>>())
                                    .unwrap_or_default();

                                let csv_content = array.iter()
                                    .map(|item| {
                                        if let Some(obj) = item.as_object() {
                                            headers.iter()
                                                .map(|key| {
                                                    obj.get(key)
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("")
                                                })
                                                .collect::<Vec<_>>()
                                                .join(",")
                                        } else {
                                            vec![]
                                        }.join(",")
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");

                                serde_json::json!({
                                    "csv_content": csv_content,
                                    "headers": headers,
                                    "row_count": array.len()
                                })
                            } else {
                                serde_json::json!({
                                    "csv_content": serde_json::to_string(&data).unwrap_or_default(),
                                    "format": "csv"
                                })
                            }
                        } else {
                            serde_json::json!({
                                "csv_content": serde_json::to_string(&input).unwrap_or_default(),
                                "format": "csv"
                            })
                        }
                    },
                    _ => serde_json::json!({
                        "error": format!("Unsupported output format: {}", output_format)
                    })
                }
            },
            "map" => {
                let transformations = step.config.get("transformations")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&serde_json::Value::Array(vec![]));

                let mut result = input.clone();

                if let Some(transform_array) = transformations.as_array() {
                    for transform in transform_array {
                        if let Some(obj) = transform.as_object() {
                            let field = obj.get("field")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let operation = obj.get("operation")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            if !field.is_empty() && input.pointer(&format!("/{}", field)).is_some() {
                                match operation {
                                    "multiply" => {
                                        if let Some(factor) = obj.get("factor").and_then(|v| v.as_f64()) {
                                            if let Some(current) = input.pointer(&format!("/{}", field)).and_then(|v| v.as_f64()) {
                                                let new_value = current * factor;
                                                let _ = result.pointer_mut(&format!("/{}", field));
                                                if let Some(val_ref) = result.pointer_mut(&format!("/{}", field)) {
                                                    *val_ref = serde_json::Value::Number(serde_json::Number::from_f64(new_value).unwrap_or_else(|_| serde_json::Number::from(0)));
                                                }
                                            }
                                        }
                                    },
                                    "add" => {
                                        if let Some(add_value) = obj.get("value") {
                                            if let Some(current) = input.pointer(&format!("/{}", field)).and_then(|v| v.as_f64()) {
                                                if let Some(add_num) = add_value.as_f64() {
                                                    let new_value = current + add_num;
                                                    let _ = result.pointer_mut(&format!("/{}", field));
                                                    if let Some(val_ref) = result.pointer_mut(&format!("/{}", field)) {
                                                        *val_ref = serde_json::Value::Number(serde_json::Number::from_f64(new_value).unwrap_or_else(|_| serde_json::Number::from(0)));
                                                    }
                                                } else if let Some(add_str) = add_value.as_str() {
                                                    if let Some(current_str) = input.pointer(&format!("/{}", field)).and_then(|v| v.as_str()) {
                                                        let new_value = format!("{}{}", current_str, add_str);
                                                        let _ = result.pointer_mut(&format!("/{}", field));
                                                        if let Some(val_ref) = result.pointer_mut(&format!("/{}", field)) {
                                                            *val_ref = serde_json::Value::String(new_value);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    "uppercase" => {
                                        if let Some(current_str) = input.pointer(&format!("/{}", field)).and_then(|v| v.as_str()) {
                                            let uppercase_value = current_str.to_uppercase();
                                            let _ = result.pointer_mut(&format!("/{}", field));
                                            if let Some(val_ref) = result.pointer_mut(&format!("/{}", field)) {
                                                *val_ref = serde_json::Value::String(uppercase_value);
                                            }
                                        }
                                    },
                                    "lowercase" => {
                                        if let Some(current_str) = input.pointer(&format!("/{}", field)).and_then(|v| v.as_str()) {
                                            let lowercase_value = current_str.to_lowercase();
                                            let _ = result.pointer_mut(&format!("/{}", field));
                                            if let Some(val_ref) = result.pointer_mut(&format!("/{}", field)) {
                                                *val_ref = serde_json::Value::String(lowercase_value);
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                serde_json::json!({
                    "transformed_data": result,
                    "transformations_applied": transformations.len(),
                    "original_data": input
                })
            },
            _ => serde_json::json!({
                "error": format!("Unknown data transformation operation: {}", operation)
            })
        };

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: if output.get("error").is_some() {
                PipelineStatus::Failed
            } else {
                PipelineStatus::Completed
            },
            input: input.clone(),
            output: output.clone(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: output.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()),
            execution_time_ms: 0, // Will be set by caller
        }
    }

    /// Execute API call step (mock implementation)
    async fn execute_api_call_step(&self, step: &PipelineStep, input: &Value) -> StepResult {
        let url = step.config.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.example.com/mock");

        let method = step.config.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST");

        // Simulate API call
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let output = serde_json::json!({
            "api_call": {
                "url": url,
                "method": method,
                "status": "success",
                "response_code": 200
            },
            "input_data": input,
            "mock_response": {
                "message": "API call completed successfully",
                "timestamp": Utc::now().to_rfc3339()
            }
        });

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: PipelineStatus::Completed,
            input: input.clone(),
            output,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: None,
            execution_time_ms: 500,
        }
    }

    /// Execute condition step
    fn execute_condition_step(&self, step: &PipelineStep, input: &Value) -> StepResult {
        let condition = step.config.get("condition")
            .and_then(|v| v.as_str())
            .unwrap_or("exists");

        let output = match condition {
            "exists" => {
                let field = step.config.get("field")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let field_exists = field.is_empty() || input.pointer(&format!("/{}", field)).is_some();

                serde_json::json!({
                    "condition": condition,
                    "field": field,
                    "exists": field_exists,
                    "result": field_exists
                })
            },
            "equals" => {
                let field = step.config.get("field")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let expected = step.config.get("value");

                let current_value = input.pointer(&format!("/{}", field));
                let matches = match (current_value, expected) {
                    (Some(current_val), Some(expected_val)) => current_val == expected_val,
                    (None, None) => true,
                    _ => false,
                };

                serde_json::json!({
                    "condition": condition,
                    "field": field,
                    "expected": expected,
                    "current": current_value,
                    "result": matches
                })
            },
            _ => serde_json::json!({
                "error": format!("Unknown condition: {}", condition)
            })
        };

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: PipelineStatus::Completed,
            input: input.clone(),
            output: output.clone(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: output.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()),
            execution_time_ms: 0,
        }
    }

    /// Execute loop step (mock implementation)
    fn execute_loop_step(&self, step: &PipelineStep, input: &Value) -> StepResult {
        let iterations = step.config.get("iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);

        let output = serde_json::json!({
            "loop_execution": {
                "iterations": iterations,
                "input": input,
                "results": (0..iterations).map(|i| serde_json::json!({
                    "iteration": i,
                    "input": input,
                    "output": format!("Iteration {} result", i + 1)
                })).collect::<Vec<_>>()
            }
        });

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: PipelineStatus::Completed,
            input: input.clone(),
            output,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: None,
            execution_time_ms: iterations * 100, // Simulated time
        }
    }

    /// Execute custom step
    fn execute_custom_step(&self, step: &PipelineStep, input: &Value) -> StepResult {
        let output = serde_json::json!({
            "custom_step": {
                "step_type": format!("{:?}", step.step_type),
                "config": step.config,
                "input": input,
                "output": "Custom step execution completed"
            }
        });

        StepResult {
            step_id: step.id.clone(),
            step_name: step.name.clone(),
            status: PipelineStatus::Completed,
            input: input.clone(),
            output,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error_message: None,
            execution_time_ms: 200,
        }
    }

    /// Get pipeline execution by ID
    pub fn get_execution(&self, execution_id: &str) -> Option<&PipelineExecution> {
        self.executions.get(execution_id)
    }

    /// List pipeline executions
    pub fn list_executions(&self, pipeline_id: Option<String>) -> Vec<&PipelineExecution> {
        self.executions
            .values()
            .filter(|exec| {
                pipeline_id
                    .as_ref()
                    .map(|id| id == &exec.pipeline_id)
                    .unwrap_or(true)
            })
            .collect()
    }

    /// Get execution statistics
    pub fn get_execution_statistics(&self) -> HashMap<String, serde_json::Value> {
        let total_executions = self.executions.len();
        let successful = self.executions
            .values()
            .filter(|exec| exec.status == PipelineStatus::Completed)
            .count();
        let failed = self.executions
            .values()
            .filter(|exec| exec.status == PipelineStatus::Failed)
            .count();

        HashMap::from([
            ("total_executions".to_string(), serde_json::Value::Number(total_executions.into())),
            ("successful".to_string(), serde_json::Value::Number(successful.into())),
            ("failed".to_string(), serde_json::Value::Number(failed.into())),
            ("success_rate".to_string(), serde_json::Value::Number(
                if total_executions > 0 {
                    (successful as f64 / total_executions as f64) * 100.0
                } else {
                    0.0
                }
            )),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let mut executor = PipelineExecutor::new();
        let pipelines = executor.list_pipelines();
        assert!(!pipelines.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let mut executor = PipelineExecutor::new();

        let input = serde_json::json!({
            "content": "Hello,   World!"
        });

        let execution = executor.execute_pipeline("pipeline-text-processor", input).await.unwrap();
        assert_eq!(execution.status, PipelineStatus::Completed);
        assert!(!execution.step_results.is_empty());
    }
}