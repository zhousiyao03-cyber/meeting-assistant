import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AudioDevice,
  TranscriptSegment,
  MeetingSummary,
  SpeakingAdvice,
  MeetingTemplate,
  AppConfig,
  ModelStatus,
  LoadedDocument,
  ModelDownloadProgress,
} from "./types";

// Commands
export const listAudioDevices = () =>
  invoke<AudioDevice[]>("list_audio_devices");

export const checkWhisperModel = () =>
  invoke<ModelStatus>("check_whisper_model");

export const downloadWhisperModel = () =>
  invoke<string>("download_whisper_model");

export const startRecording = (micDevice: string, captureDevice: string) =>
  invoke<void>("start_recording", {
    micDevice,
    captureDevice,
  });

export const stopRecording = () => invoke<void>("stop_recording");

export const getTranscript = () =>
  invoke<TranscriptSegment[]>("get_transcript");

export const getTemplates = () =>
  invoke<MeetingTemplate[]>("get_templates");

export const saveTemplate = (template: MeetingTemplate) =>
  invoke<void>("save_template", { template });

export const deleteTemplate = (id: string) =>
  invoke<void>("delete_template", { id });

export const getConfig = () => invoke<AppConfig>("get_config");

export const saveConfig = (config: AppConfig) =>
  invoke<void>("save_config", { config });

export const loadDocument = (path: string) =>
  invoke<LoadedDocument>("load_document", { path });

// Event listeners
export const onNewTranscript = (
  handler: (segment: TranscriptSegment) => void,
): Promise<UnlistenFn> =>
  listen<TranscriptSegment>("new-transcript", (e) => handler(e.payload));

export const onMeetingSummary = (
  handler: (summary: MeetingSummary) => void,
): Promise<UnlistenFn> =>
  listen<MeetingSummary>("meeting-summary", (e) => handler(e.payload));

export const onSpeakingAdvice = (
  handler: (advice: SpeakingAdvice) => void,
): Promise<UnlistenFn> =>
  listen<SpeakingAdvice>("speaking-advice", (e) => handler(e.payload));

export const onModelDownloadProgress = (
  handler: (progress: ModelDownloadProgress) => void,
): Promise<UnlistenFn> =>
  listen<ModelDownloadProgress>("model-download-progress", (e) =>
    handler(e.payload),
  );
