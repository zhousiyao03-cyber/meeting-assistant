use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct TriggerResult {
    pub triggered: bool,
    pub reason: String,
}

/// Check if any trigger hints from the template match the recent transcript.
pub fn check_hint_triggers(transcript: &str, hints: &[String]) -> TriggerResult {
    for hint in hints {
        if transcript.contains(hint.as_str()) {
            return TriggerResult {
                triggered: true,
                reason: format!("检测到关键短语：\"{}\"", hint),
            };
        }
    }
    TriggerResult {
        triggered: false,
        reason: String::new(),
    }
}

/// Check if there's been silence (very short recent text relative to time window).
pub fn check_silence(recent_text: &str, window_seconds: f64) -> TriggerResult {
    let chars_per_second = recent_text.len() as f64 / window_seconds;
    // If less than ~2 chars/sec in a 10s window, likely a pause
    if chars_per_second < 2.0 && !recent_text.is_empty() {
        return TriggerResult {
            triggered: true,
            reason: "讨论出现停顿，可能在等待回应".into(),
        };
    }
    TriggerResult {
        triggered: false,
        reason: String::new(),
    }
}

/// Check if the transcript ends with a question.
pub fn check_question(transcript: &str) -> TriggerResult {
    let trimmed = transcript.trim();
    if trimmed.ends_with('?') || trimmed.ends_with('？') {
        return TriggerResult {
            triggered: true,
            reason: "有人提出了问题".into(),
        };
    }
    TriggerResult {
        triggered: false,
        reason: String::new(),
    }
}

/// Run all trigger checks. Returns the first match.
pub fn evaluate_triggers(
    recent_text: &str,
    hints: &[String],
    window_seconds: f64,
) -> Option<TriggerResult> {
    let checks = [
        check_hint_triggers(recent_text, hints),
        check_question(recent_text),
        check_silence(recent_text, window_seconds),
    ];

    checks.into_iter().find(|r| r.triggered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hint_trigger() {
        let hints = vec!["大家觉得怎么样".into(), "有没有问题".into()];
        let result = check_hint_triggers("我觉得这个方案不错，大家觉得怎么样", &hints);
        assert!(result.triggered);
    }

    #[test]
    fn test_question_trigger() {
        let result = check_question("这样做性能会不会有问题？");
        assert!(result.triggered);
    }
}
