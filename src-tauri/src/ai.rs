use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GradeResult {
    pub score: f64,
    pub confidence: f64,
    pub deductions: Vec<String>,
    pub comment: String,
}

pub async fn call_ai_api(
    image_base64: &str,
    rubric: &str,
    config: &ApiConfig,
    max_score: Option<f64>,
    reference_answer: Option<&str>,
    question_title: Option<&str>,
    reference_answer_image: Option<&str>,
    timeout_secs: u64,
) -> Result<GradeResult, String> {
    let system_role = "你是一名高校阅卷老师，根据评分标准和答案对学生答案评分。请输出JSON格式的结果。";

    let has_ref_image = reference_answer_image.is_some_and(|s| !s.is_empty());
    let max_s = max_score.unwrap_or(100.0);

    let prompt = if has_ref_image {
        format!(
            "题目：{}\n评分标准：{}\n参考答案：{}\n满分：{}\n\n图片1是标准答案，图片2是学生答案。\n请根据评分标准和标准答案（图片1）给学生答案（图片2）评分。\n\n评分规则：\n1. 逐项评分，各项得分之和不超过满分{}\n2. 空白或无关答案给0分\n3. 部分正确酌情给分\n\n请输出JSON：\n{{\"score\":分数,\"confidence\":置信度0~1,\"deductions\":[\"扣分项\"],\"comment\":\"评语\"}}\n\n只输出JSON，不要包含其他文字和markdown标记。",
            question_title.unwrap_or(""),
            rubric,
            reference_answer.unwrap_or("无"),
            max_s,
            max_s
        )
    } else {
        format!(
            "题目：{}\n评分标准：{}\n参考答案：{}\n满分：{}\n\n图片是学生答案，请根据评分标准评分。\n\n评分规则：\n1. 逐项评分，各项得分之和不超过满分{}\n2. 空白或无关答案给0分\n3. 部分正确酌情给分\n\n请输出JSON：\n{{\"score\":分数,\"confidence\":置信度0~1,\"deductions\":[\"扣分项\"],\"comment\":\"评语\"}}\n\n只输出JSON，不要包含其他文字和markdown标记。",
            question_title.unwrap_or(""),
            rubric,
            reference_answer.unwrap_or("无"),
            max_s,
            max_s
        )
    };

    let body = if has_ref_image {
        serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system_role},
                {"role": "user", "content": [
                    {"type": "text", "text": &prompt},
                    {"type": "image_url", "image_url": {"url": reference_answer_image.unwrap_or("")}},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}}
                ]}
            ],
            "max_tokens": 4096,
            "temperature": 0.3
        })
    } else {
        serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system_role},
                {"role": "user", "content": [
                    {"type": "text", "text": &prompt},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}}
                ]}
            ],
            "max_tokens": 4096,
            "temperature": 0.3
        })
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let mut last_error = String::new();

    for attempt in 1..=3 {
        match client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    let error_text = response.text().await.unwrap_or_default();
                    if error_text.contains("does not support image") || error_text.contains("does not support") && error_text.contains("image") {
                        last_error = "当前模型不支持图片输入，请在设置中更换为支持视觉的模型（如 Qwen/Qwen2.5-32B-Instruct、gpt-4o 等）".to_string();
                    } else {
                        last_error = format!("API 返回错误 ({}): {}", status, &error_text);
                    }
                    log::warn!("AI API attempt {}/3 failed: {}", attempt, last_error);
                    if attempt < 3 {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(last_error);
                }

                let body = response.text().await.map_err(|e| e.to_string())?;
                if body.contains("does not support image") || body.contains("does not support") && body.contains("image") {
                    last_error = "当前模型不支持图片输入，请在设置中更换为支持视觉的模型（如 Qwen/Qwen2.5-32B-Instruct、gpt-4o 等）".to_string();
                    log::warn!("AI API body error: {}", last_error);
                    return Err(last_error);
                }
                let json: Value = serde_json::from_str(&body).map_err(|e| format!("解析返回JSON失败: {}", e))?;
                if let Some(err_msg) = json["error"]["message"].as_str() {
                    if err_msg.contains("does not support image") || err_msg.contains("image input") {
                        last_error = "当前模型不支持图片输入，请在设置中更换为支持视觉的模型（如 Qwen/Qwen2.5-32B-Instruct、gpt-4o 等）".to_string();
                        return Err(last_error);
                    }
                }
                let content = json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                if content.is_empty() {
                    last_error = format!("API 返回为空 (尝试 {}/3)", attempt);
                    log::warn!("AI API empty response, attempt {}/3", attempt);
                    if attempt < 3 {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(last_error);
                }
                match try_parse_json(&content) {
                    Some(parsed) => {
                        return Ok(GradeResult {
                            score: parsed["score"].as_f64().unwrap_or(0.0),
                            confidence: parsed["confidence"].as_f64().unwrap_or(0.5),
                            deductions: parsed["deductions"].as_array()
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                            comment: parsed["comment"].as_str().or_else(|| parsed["commit"].as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                    None => {
                        last_error = format!("AI 返回 JSON 解析失败 (尝试 {}/3)", attempt);
                        log::warn!("AI JSON parse failed, attempt {}/3: {}", attempt, &content[..content.len().min(200)]);
                        if attempt < 3 {
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            continue;
                        }
                        return Err(last_error);
                    }
                }
            }
            Err(e) => {
                last_error = format!("API request failed: {}", e);
                log::warn!("AI API attempt {}/3 failed: {}", attempt, last_error);
                if attempt < 3 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                return Err(last_error);
            }
        }
    }

    Err(last_error)
}

fn try_parse_json(text: &str) -> Option<Value> {
    let mut cleaned = text.trim().to_string();
    // remove markdown code fences
    cleaned = cleaned.replace("```json", "").replace("```JSON", "").replace("```", "").trim().to_string();
    // if there's still non-JSON text, extract the first {...} object
    if !cleaned.starts_with('{') && !cleaned.starts_with('[') {
        if let Some(start) = cleaned.find('{') {
            if let Some(end) = cleaned.rfind('}') {
                cleaned = cleaned[start..=end].to_string();
            }
        }
    }
    // attempt direct parse
    if let Ok(v) = serde_json::from_str::<Value>(&cleaned) {
        return Some(v);
    }
    // fix single quotes -> double quotes
    let fixed = cleaned
        .replace('\'', "\"")
        .replace(",\n]", "\n]")
        .replace(",\n}", "\n}")
        .replace(",]", "]")
        .replace(",}", "}");
    if let Ok(v) = serde_json::from_str::<Value>(&fixed) {
        return Some(v);
    }
    None
}
