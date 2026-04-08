//! Vision task analysis tool
//!
//! Uses multimodal LLM to analyze screenshots and locate UI elements.

use crate::error::PcControllerError;
use crate::tools::config::{get_config, VisionConfig};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rmcp::model::*;
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Coordinate format for output
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CoordinateFormat {
    Pixel,
    Grid,
}

impl Default for CoordinateFormat {
    fn default() -> Self {
        CoordinateFormat::Pixel
    }
}

/// Alternative target suggestion
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AlternativeTarget {
    /// Target description
    pub description: String,
    /// Bounding box [x, y, width, height]
    pub bounding_box: [i32; 4],
    /// Click point [x, y]
    pub click_point: [i32; 2],
    /// Grid ID if grid coordinate format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_id: Option<String>,
}

/// Arguments for analyze_task
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeTaskArgs {
    /// Screenshot image as base64 PNG bytes
    pub image_base64: String,
    /// Task description (e.g., "Find the username input field")
    pub task: String,
    /// Grid columns (for grid coordinate format)
    #[serde(default)]
    pub grid_cols: Option<u32>,
    /// Grid rows (for grid coordinate format)
    #[serde(default)]
    pub grid_rows: Option<u32>,
    /// Coordinate format: "pixel" or "grid" (default: pixel)
    #[serde(default)]
    pub coordinate_format: Option<String>,
}

/// Result of analyze_task
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeTaskResult {
    /// Analysis explanation from the LLM
    pub analysis: String,
    /// Whether the target was found
    pub found: bool,
    /// Target description (e.g., "Username input field")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_description: Option<String>,
    /// Bounding box [x, y, width, height]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<[i32; 4]>,
    /// Click point [x, y]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_point: Option<[i32; 2]>,
    /// Grid ID (e.g., "B3") if grid coordinate format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_id: Option<String>,
    /// Confidence score 0.0-1.0
    pub confidence: f32,
    /// Alternative targets if primary not found
    #[serde(default)]
    pub alternatives: Vec<AlternativeTarget>,
    /// Error message if analysis failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parse grid ID like "B3" to column and row (0-indexed)
fn parse_grid_id(grid_id: &str) -> Option<(u32, u32)> {
    let grid_id = grid_id.trim().to_uppercase();
    let mut chars = grid_id.chars();
    let col_char = chars.next()?;
    if !col_char.is_ascii_alphabetic() {
        return None;
    }
    let col = (col_char as u32) - (b'A' as u32);
    let rest: String = chars.collect();
    if rest.is_empty() {
        return None;
    }
    let row: u32 = rest.parse().ok()?;
    if row == 0 {
        return None;
    }
    Some((col, row - 1)) // Convert to 0-indexed
}

/// Calculate grid ID from pixel coordinates
fn coords_to_grid_id(x: i32, y: i32, cols: u32, rows: u32, width: u32, height: u32) -> Option<String> {
    if width == 0 || height == 0 {
        return None;
    }
    let cell_width = width as f64 / cols as f64;
    let cell_height = height as f64 / rows as f64;
    let col = (x as f64 / cell_width).floor() as u32;
    let row = (y as f64 / cell_height).floor() as u32;
    if col >= cols || row >= rows {
        return None;
    }
    let col_letter = (b'A' as u32 + col) as u8 as char;
    Some(format!("{}{}", col_letter, row + 1))
}

/// Calculate click point from bounding box
fn bbox_to_click_point(bbox: &[i32; 4]) -> [i32; 2] {
    [bbox[0] + bbox[2] / 2, bbox[1] + bbox[3] / 2]
}

/// Call OpenAI Vision API
async fn call_vision_api(
    config: &VisionConfig,
    image_base64: &str,
    task: &str,
) -> Result<serde_json::Value, PcControllerError> {
    let url = format!("{}/chat/completions", config.base_url);

    let request_body = json!({
        "model": config.model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": format!(
                            "You are a UI element analyzer. Given a screenshot and a task, analyze the image and respond with a JSON object.\n\
                            Task: {}\n\
                            Respond ONLY with valid JSON in this exact format (no markdown, no explanation):\n\
                            {{\n\
                            \"found\": true/false,\n\
                            \"target_description\": \"description of found element\" | null,\n\
                            \"bounding_box\": [x, y, width, height] in pixels | null,\n\
                            \"confidence\": 0.0-1.0,\n\
                            \"analysis\": \"brief explanation of what you found\"\n\
                            }}\n\
                            If multiple similar elements exist, put alternatives in an 'alternatives' array.\n\
                            Calculate bounding boxes as tight fits around the UI elements.",
                            task
                        )
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", image_base64)
                        }
                    }
                ]
            }
        ],
        "max_tokens": 500
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| PcControllerError::PlatformError(format!("Failed to call vision API: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(PcControllerError::PlatformError(format!(
            "Vision API error {}: {}", status, body
        )));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| PcControllerError::PlatformError(format!("Failed to parse vision response: {}", e)))?;

    Ok(json)
}

/// Parse LLM response to extract analysis result
fn parse_vision_response(
    response: serde_json::Value,
    task: &str,
    grid_cols: Option<u32>,
    grid_rows: Option<u32>,
) -> Result<AnalyzeTaskResult, PcControllerError> {
    let content = response
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
        .ok_or_else(|| PcControllerError::PlatformError("Invalid vision API response format".to_string()))?;

    // Parse JSON from the content string (LLM returns JSON as string)
    let parsed: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| PcControllerError::PlatformError(format!("Failed to parse LLM response: {} - content: {}", e, content)))?;

    let found = parsed.get("found").and_then(|v| v.as_bool()).unwrap_or(false);
    let confidence = parsed.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let analysis = parsed.get("analysis").and_then(|v| v.as_str()).unwrap_or("No analysis").to_string();
    let target_description = parsed.get("target_description").and_then(|v| v.as_str()).map(String::from);

    let use_grid = grid_cols.is_some() && grid_rows.is_some();
    let cols = grid_cols.unwrap_or(12);
    let rows = grid_rows.unwrap_or(8);

    let bounding_box = if found {
        parsed.get("bounding_box").and_then(|v| {
            let arr = v.as_array()?;
            if arr.len() != 4 {
                return None;
            }
            Some([
                arr[0].as_i64()? as i32,
                arr[1].as_i64()? as i32,
                arr[2].as_i64()? as i32,
                arr[3].as_i64()? as i32,
            ])
        })
    } else {
        None
    };

    let click_point = bounding_box.map(|bbox| bbox_to_click_point(&bbox));

    let grid_id = if use_grid {
        click_point.and_then(|pt| coords_to_grid_id(pt[0], pt[1], cols, rows, 1920, 1080))
    } else {
        None
    };

    let alternatives: Vec<AlternativeTarget> = parsed
        .get("alternatives")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().filter_map(|alt| {
                let description = alt.get("description")?.as_str()?.to_string();
                let bbox = alt.get("bounding_box")?.as_array()?;
                if bbox.len() != 4 {
                    return None;
                }
                let bounding_box = [
                    bbox[0].as_i64()? as i32,
                    bbox[1].as_i64()? as i32,
                    bbox[2].as_i64()? as i32,
                    bbox[3].as_i64()? as i32,
                ];
                let click_point = bbox_to_click_point(&bounding_box);
                let grid_id = if use_grid {
                    coords_to_grid_id(click_point[0], click_point[1], cols, rows, 1920, 1080)
                } else {
                    None
                };
                Some(AlternativeTarget {
                    description,
                    bounding_box,
                    click_point,
                    grid_id,
                })
            }).collect()
        })
        .unwrap_or_default();

    Ok(AnalyzeTaskResult {
        analysis,
        found,
        target_description,
        bounding_box,
        click_point,
        grid_id,
        confidence,
        alternatives,
        error: None,
    })
}

/// Execute vision analysis task
pub async fn analyze_task(
    args: &AnalyzeTaskArgs,
) -> Result<CallToolResult, PcControllerError> {
    let config = get_config().ok_or_else(|| {
        PcControllerError::PlatformError(
            "Vision config not initialized. Set OPENAI_API_KEY or use --config".to_string()
        )
    })?;

    let use_grid = args.grid_cols.is_some() && args.grid_rows.is_some();
    let cols = args.grid_cols.unwrap_or(12);
    let rows = args.grid_rows.unwrap_or(8);

    // Retry logic with exponential backoff
    let mut last_error = None;
    for attempt in 0..3 {
        match call_vision_api(&config, &args.image_base64, &args.task).await {
            Ok(response) => {
                let mut result = parse_vision_response(
                    response,
                    &args.task,
                    args.grid_cols,
                    args.grid_rows,
                )?;

                // Update grid_id with actual image dimensions if available
                if use_grid {
                    if let Some(ref click_point) = result.click_point {
                        // Default dimensions, caller should use bounding box for accuracy
                        result.grid_id = coords_to_grid_id(
                            click_point[0],
                            click_point[1],
                            cols,
                            rows,
                            1920, // Default, actual capture provides this
                            1080,
                        );
                    }
                }

                return Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                )]));
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < 2 {
                    let delay = (1 << attempt) * 1000;
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        PcControllerError::PlatformError("Vision analysis failed".to_string())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grid_id() {
        assert_eq!(parse_grid_id("A1"), Some((0, 0)));
        assert_eq!(parse_grid_id("B3"), Some((1, 2)));
        assert_eq!(parse_grid_id("L8"), Some((11, 7)));
        assert_eq!(parse_grid_id("a1"), Some((0, 0)));
        assert_eq!(parse_grid_id(""), None);
        assert_eq!(parse_grid_id("AB"), None);
    }

    #[test]
    fn test_coords_to_grid_id() {
        assert_eq!(coords_to_grid_id(0, 0, 12, 8, 1920, 1080), Some("A1".to_string()));
        assert_eq!(coords_to_grid_id(80, 67, 12, 8, 1920, 1080), Some("A1".to_string())); // Cell center
        assert_eq!(coords_to_grid_id(959, 539, 12, 8, 1920, 1080), Some("F4".to_string())); // Just before center
    }

    #[test]
    fn test_bbox_to_click_point() {
        assert_eq!(bbox_to_click_point(&[100, 200, 50, 30]), [125, 215]);
    }
}
