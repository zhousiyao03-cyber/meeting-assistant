import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AudioDevice,
  TranscriptSegment,
  TranscriptDelta,
  MeetingSummary,
  SpeakingAdvice,
  MeetingTemplate,
  AppConfig,
  ModelStatus,
  LoadedDocument,
  ModelDownloadProgress,
  MeetingRecord,
  MeetingMinutes,
  BackendError,
  UserPlan,
  ScreenRecordingPermissionStatus,
} from "./types";

// === Commands ===

export const listAudioDevices = () =>
  invoke<AudioDevice[]>("list_audio_devices");

export const checkWhisperModel = () =>
  invoke<ModelStatus>("check_whisper_model");

export const downloadWhisperModel = () =>
  invoke<string>("download_whisper_model");

export const startRecording = (micDevice: string) =>
  invoke<void>("start_recording", { micDevice });

export const stopRecording = () => invoke<void>("stop_recording");

export const getTranscript = () =>
  invoke<TranscriptSegment[]>("get_transcript");

export const getTemplates = () => invoke<MeetingTemplate[]>("get_templates");

export const getTemplatesForLocale = (locale: string) =>
  invoke<MeetingTemplate[]>("get_templates_for_locale", { locale });

export const saveTemplate = (template: MeetingTemplate) =>
  invoke<void>("save_template", { template });

export const deleteTemplate = (id: string) =>
  invoke<void>("delete_template", { id });

export const getConfig = () => invoke<AppConfig>("get_config");

export const saveConfig = (config: AppConfig) =>
  invoke<void>("save_app_config", { config });

export const loadDocument = (path: string) =>
  invoke<LoadedDocument>("load_document", { path });

export const loadReferenceDoc = (path: string) =>
  invoke<string>("load_reference_doc", { path });

export const clearReferenceDoc = () => invoke<void>("clear_reference_doc");

export const getRecordingStatus = () =>
  invoke<{ is_recording: boolean; is_paused: boolean; elapsed_secs: number }>(
    "get_recording_status",
  );

export const pauseRecording = () => invoke<void>("pause_recording");
export const resumeRecording = () => invoke<void>("resume_recording");

export const saveMeeting = (record: MeetingRecord) =>
  invoke<void>("save_meeting", { record });

export const listMeetings = () => invoke<MeetingRecord[]>("list_meetings");

export const setActiveTemplate = (id: string) =>
  invoke<void>("set_active_template", { id });

export const setMeetingContextNote = (note: string) =>
  invoke<void>("set_meeting_context_note", { note });

export const getMeetingContextNote = () =>
  invoke<string>("get_meeting_context_note");

export const setActiveLocale = (locale: string) =>
  invoke<void>("set_active_locale", { locale });

export const generateMeetingMinutes = (transcript: string, summary: string) =>
  invoke<MeetingMinutes>("generate_meeting_minutes", { transcript, summary });

export const deleteMeeting = (id: string) =>
  invoke<void>("delete_meeting", { id });

// Permission
export const checkScreenRecordingPermission = () =>
  invoke<ScreenRecordingPermissionStatus>("check_screen_recording_permission");

export const openScreenRecordingSettings = () =>
  invoke<void>("open_screen_recording_settings");

// Stealth
export const setStealthMode = (on: boolean) =>
  invoke<void>("set_stealth_mode", { on });

export const isStealthOn = () => invoke<boolean>("is_stealth_on");

// License
export const getUserPlan = () => invoke<UserPlan>("get_user_plan");
export const setLicenseKey = (key: string) =>
  invoke<UserPlan>("set_license_key", { key });
export const clearLicenseKey = () => invoke<void>("clear_license_key");

// === Event listeners ===

export const onNewTranscript = (
  handler: (segment: TranscriptSegment) => void,
): Promise<UnlistenFn> =>
  listen<TranscriptSegment>("new-transcript", (e) => handler(e.payload));

export const onTranscriptDelta = (
  handler: (delta: TranscriptDelta) => void,
): Promise<UnlistenFn> =>
  listen<TranscriptDelta>("transcript-delta", (e) => handler(e.payload));

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

export const onBackendError = (
  handler: (error: BackendError) => void,
): Promise<UnlistenFn> =>
  listen<BackendError>("backend-error", (e) => handler(e.payload));

export const onStealthChanged = (
  handler: (on: boolean) => void,
): Promise<UnlistenFn> =>
  listen<boolean>("stealth-changed", (e) => handler(e.payload));

export const onPanicStop = (handler: () => void): Promise<UnlistenFn> =>
  listen("panic-stop", () => handler());

export const onOpacityStep = (
  handler: (delta: number) => void,
): Promise<UnlistenFn> =>
  listen<number>("opacity-step", (e) => handler(e.payload));

export const onMenuNewMeeting = (
  handler: (kind: "interview" | "general") => void,
): Promise<UnlistenFn> =>
  listen<string>("menu-new-meeting", (e) =>
    handler(e.payload as "interview" | "general"),
  );

export const onPlanUpdated = (
  handler: (plan: UserPlan) => void,
): Promise<UnlistenFn> =>
  listen<UserPlan>("plan-updated", (e) => handler(e.payload));

export const onQuotaLow = (
  handler: (remainingSecs: number) => void,
): Promise<UnlistenFn> =>
  listen<number>("quota-low", (e) => handler(e.payload));

export const onQuotaExhausted = (handler: () => void): Promise<UnlistenFn> =>
  listen("quota-exhausted", () => handler());
