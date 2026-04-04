use serde::Serialize;
use crate::advisor::templates::TriggerConfig;

#[derive(Clone, Debug, Serialize)]
pub struct TriggerResult {
    pub triggered: bool,
    pub reason: String,
}

/// Run all trigger checks in priority order. Returns the first match.
pub fn evaluate_triggers(
    recent_text: &str,
    trigger_config: &TriggerConfig,
    window_seconds: f64,
) -> Option<TriggerResult> {
    let mut checks = Vec::new();

    if trigger_config.on_ask_opinion {
        checks.push(check_asking_for_opinion(recent_text));
    }
    if trigger_config.on_domain_topic && !trigger_config.domain_keywords.is_empty() {
        checks.push(check_domain_topic(recent_text, &trigger_config.domain_keywords));
    }
    if !trigger_config.custom_keywords.is_empty() {
        checks.push(check_hint_triggers(recent_text, &trigger_config.custom_keywords));
    }
    if trigger_config.on_decision_point {
        checks.push(check_decision_point(recent_text));
    }
    if trigger_config.on_discussion_stuck {
        checks.push(check_discussion_stuck(recent_text, window_seconds));
    }

    checks.into_iter().find(|r| r.triggered)
}

/// Detect when someone is asking for group/your opinion.
/// e.g. "大家觉得怎么样" "前端这边怎么看" "你们有什么想法" "谁有意见"
fn check_asking_for_opinion(transcript: &str) -> TriggerResult {
    let opinion_patterns = [
        // Asking the group
        "大家觉得", "大家怎么看", "大家有什么想法", "大家意见",
        "你们觉得", "你们怎么看", "你们有什么想法",
        "谁有想法", "谁有意见", "还有其他意见",
        "有没有补充", "有什么建议", "有什么看法",
        // Asking frontend specifically
        "前端这边", "前端怎么看", "前端有什么",
        "前端的意见", "前端同学",
    ];

    for pat in &opinion_patterns {
        if transcript.contains(pat) {
            return TriggerResult {
                triggered: true,
                reason: format!("有人在征求意见：\"{}\"", pat),
            };
        }
    }

    // Check if the last sentence is a question directed at the group
    // (ends with ？and contains group-addressing words)
    if let Some(last_sentence) = extract_last_sentence(transcript) {
        if last_sentence.ends_with('？') || last_sentence.ends_with('?') {
            let group_words = ["大家", "你们", "各位", "谁", "哪位", "前端"];
            if group_words.iter().any(|w| last_sentence.contains(w)) {
                return TriggerResult {
                    triggered: true,
                    reason: format!("有人向团队提问：\"{}\"", truncate_str(&last_sentence, 30)),
                };
            }
        }
    }

    TriggerResult { triggered: false, reason: String::new() }
}

/// Detect when discussion touches domain-specific topics.
fn check_domain_topic(transcript: &str, domain_keywords: &[String]) -> TriggerResult {
    // Only trigger if domain keywords appear in recent text (last ~200 chars)
    // to avoid triggering on old mentions
    let recent_tail = if transcript.len() > 200 {
        &transcript[transcript.len() - 200..]
    } else {
        transcript
    };

    let matched: Vec<&String> = domain_keywords
        .iter()
        .filter(|kw| recent_tail.contains(kw.as_str()))
        .collect();

    if !matched.is_empty() {
        return TriggerResult {
            triggered: true,
            reason: format!("讨论涉及专业领域：{}", matched.iter().take(3).map(|s| s.as_str()).collect::<Vec<_>>().join("、")),
        };
    }

    TriggerResult { triggered: false, reason: String::new() }
}

/// Check custom trigger hints from the meeting template.
fn check_hint_triggers(transcript: &str, hints: &[String]) -> TriggerResult {
    // Only check against recent tail to avoid re-triggering on old text
    let recent_tail = if transcript.len() > 200 {
        &transcript[transcript.len() - 200..]
    } else {
        transcript
    };

    for hint in hints {
        if recent_tail.contains(hint.as_str()) {
            return TriggerResult {
                triggered: true,
                reason: format!("检测到关键话题：\"{}\"", hint),
            };
        }
    }
    TriggerResult { triggered: false, reason: String::new() }
}

/// Detect decision points: disagreement, trade-offs, or need for conclusion.
fn check_decision_point(transcript: &str) -> TriggerResult {
    let recent_tail = if transcript.len() > 300 {
        &transcript[transcript.len() - 300..]
    } else {
        transcript
    };

    let disagreement_patterns = [
        // Disagreement / debate
        "我觉得不", "我不太同意", "不太行", "有问题吧",
        "但是我觉得", "换个思路", "另一个方案",
        "A方案", "B方案", "方案一", "方案二",
        // Need for decision
        "到底用哪", "选哪个", "怎么决定", "定一下",
        "拍个板", "做个决定", "需要确认",
        // Trade-off discussion
        "优缺点", "利弊", "trade-off", "权衡",
    ];

    for pat in &disagreement_patterns {
        if recent_tail.contains(pat) {
            return TriggerResult {
                triggered: true,
                reason: format!("出现技术决策点：\"{}\"", pat),
            };
        }
    }

    TriggerResult { triggered: false, reason: String::new() }
}

/// Check if discussion is stuck or going off-track.
fn check_discussion_stuck(recent_text: &str, window_seconds: f64) -> TriggerResult {
    // Low speech rate = possible stuck
    let chars_per_second = recent_text.chars().count() as f64 / window_seconds;
    if chars_per_second < 1.5 && !recent_text.is_empty() && recent_text.chars().count() > 5 {
        return TriggerResult {
            triggered: true,
            reason: "讨论出现停顿，可以主动推进".into(),
        };
    }

    // Detect off-track: repeated topic without conclusion
    let off_track_patterns = [
        "刚才说到哪了", "跑题了", "回到正题", "我们本来在讨论",
        "先不说这个", "这个先放一下",
    ];
    for pat in &off_track_patterns {
        if recent_text.contains(pat) {
            return TriggerResult {
                triggered: true,
                reason: "讨论跑偏，可以帮忙拉回主题".into(),
            };
        }
    }

    TriggerResult { triggered: false, reason: String::new() }
}

/// Extract the last sentence from transcript text.
fn extract_last_sentence(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Split by common sentence endings
    let delimiters = ['。', '！', '？', '?', '!', '\n'];
    let sentences: Vec<&str> = trimmed.split(|c| delimiters.contains(&c))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Get the last complete sentence, or the trailing partial
    if let Some(last) = sentences.last() {
        // Re-append the delimiter if it was a question mark
        if trimmed.ends_with('？') || trimmed.ends_with('?') {
            Some(format!("{}？", last))
        } else {
            Some(last.to_string())
        }
    } else {
        None
    }
}

/// Truncate a string to max_chars, appending "..." if truncated.
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect::<String>() + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advisor::templates::TriggerConfig;

    fn default_config() -> TriggerConfig {
        TriggerConfig::default()
    }

    #[test]
    fn test_asking_opinion() {
        let result = check_asking_for_opinion("这个方案大家觉得怎么样");
        assert!(result.triggered);
        assert!(result.reason.contains("征求意见"));
    }

    #[test]
    fn test_asking_frontend() {
        let result = check_asking_for_opinion("前端这边有什么想法吗？");
        assert!(result.triggered);
    }

    #[test]
    fn test_domain_topic() {
        let keywords = vec!["组件".into(), "渲染".into(), "CSS".into()];
        let result = check_domain_topic("这个组件的渲染性能有点问题", &keywords);
        assert!(result.triggered);
        assert!(result.reason.contains("专业领域"));
    }

    #[test]
    fn test_decision_point() {
        let result = check_decision_point("A方案成本低但是扩展性差，B方案相反");
        assert!(result.triggered);
    }

    #[test]
    fn test_random_question_no_trigger() {
        // A random question that's NOT directed at the group should NOT trigger
        let result = check_asking_for_opinion("这怎么变成银河了？是银河，不是银河。");
        assert!(!result.triggered);
    }

    #[test]
    fn test_hint_trigger() {
        let hints = vec!["有什么阻塞".into()];
        let result = check_hint_triggers("最近有什么阻塞吗", &hints);
        assert!(result.triggered);
    }

    #[test]
    fn test_stuck() {
        let result = check_discussion_stuck("嗯，那个，就是说...", 10.0);
        assert!(result.triggered);
    }

    #[test]
    fn test_evaluate_triggers_with_config() {
        let config = default_config();
        let result = evaluate_triggers("这个方案大家觉得怎么样", &config, 10.0);
        assert!(result.is_some());
    }

    #[test]
    fn test_evaluate_triggers_domain_disabled() {
        let config = TriggerConfig {
            on_ask_opinion: false,
            on_domain_topic: false,
            on_decision_point: false,
            on_discussion_stuck: false,
            custom_keywords: vec![],
            domain_keywords: vec!["前端".into()],
        };
        // Nothing should trigger since all flags are off
        let result = evaluate_triggers("前端组件渲染大家觉得怎么样", &config, 10.0);
        assert!(result.is_none());
    }
}
