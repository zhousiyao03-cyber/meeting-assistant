export interface AudioDevice {
  id: string;
  name: string;
}

export interface TranscriptSegment {
  timestamp: string;
  text: string;
  offset_secs: number;
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
  role_persona: string;
  mimic_style: string;
  expertise_context: string;
  trigger_config: TriggerConfig;
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

export interface AppConfig {
  llm: LlmConfig;
  audio: AudioConfig;
  language_preference: string;
  analysis_mode: string;
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
