export interface AudioDevice {
  id: string;
  name: string;
}

export interface TranscriptSegment {
  timestamp: string;
  text: string;
  offset_secs: number;
  speaker: "me" | "other" | "mixed";
}

export interface TranscriptDelta {
  text: string;
  speaker: string;
  offset_secs: number;
  is_final: boolean;
}

export interface MeetingSummary {
  points: string[];
  current_topic: string;
}

export interface SpeakingAdvice {
  reason: string;
  suggestion: string;
  angle: string;
  timestamp: number;
}

export interface TriggerConfig {
  on_ask_opinion: boolean;
  on_question_to_user?: boolean;
  on_domain_topic: boolean;
  on_decision_point: boolean;
  on_discussion_stuck: boolean;
  custom_keywords: string[];
  domain_keywords: string[];
}

export interface MeetingTemplate {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  trigger_hints: string[];
  advice_style: string;
  enabled: boolean;
  language?: string;
  role_persona?: string;
  mimic_style?: string;
  expertise_context?: string;
  stealth_default?: boolean;
  advice_cooldown_seconds?: number;
  trigger_config?: TriggerConfig;
}

export interface LlmConfig {
  base_url: string;
  api_key: string;
  model: string;
}

export interface AudioConfig {
  mic_device: string;
  capture_device: string;
  noise_reduction: boolean;
}

export interface ByoConfig {
  active: boolean;
  openai_api_key: string;
  anthropic_api_key: string;
  anthropic_model: string;
}

export interface AppConfig {
  llm: LlmConfig;
  audio: AudioConfig;
  language_preference: string;
  analysis_mode: string;
  byo: ByoConfig;
  openai_asr_api_key: string;
  openai_asr_model: string;
}

export interface ModelStatus {
  downloaded: boolean;
  path: string | null;
}

export interface LoadedDocument {
  filename: string;
  content: string;
  format: string;
}

export interface ModelDownloadProgress {
  downloaded: number;
  total: number;
}

export interface MeetingRecord {
  id: string;
  title: string;
  template_name: string;
  started_at: string;
  duration_secs: number;
  transcript: string;
  summary: string;
  action_items: string;
  advices_json: string;
}

export interface MeetingMinutes {
  title: string;
  key_points: string[];
  action_items: string[];
  decisions: string[];
}

export interface BackendError {
  source: string;
  message: string;
}

// === New for Confide ===

export type Tier = "free" | "pro" | "ultra";

export interface UserPlan {
  tier: Tier;
  monthly_quota_seconds: number;
  used_this_month_seconds: number;
  overage_rate_per_min_cents: number;
  resume_rag_enabled: boolean;
  resume_credits_remaining: number;
  byo_active: boolean;
  auto_topup_enabled: boolean;
  history_persistence_days: number;
  renews_at: number | null;
  cancelled_at: number | null;
}

export interface ScreenRecordingPermissionStatus {
  status: "granted" | "denied" | "not-determined";
  macos_version_ok: boolean;
}
