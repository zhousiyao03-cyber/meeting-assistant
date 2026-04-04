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

export interface MeetingTemplate {
  id: string;
  name: string;
  description: string;
  system_prompt: string;
  trigger_hints: string[];
  advice_style: string;
  enabled: boolean;
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
  template_name: string;
  started_at: string;
  duration_secs: number;
  transcript: string;
  summary: string;
  advices_json: string;
}
