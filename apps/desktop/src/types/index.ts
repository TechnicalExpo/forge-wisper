export type ProcessingState =
  | "Idle"
  | "Listening"
  | "Stopping"
  | "Transcribing"
  | "Cleaning"
  | "Structuring"
  | "Verifying"
  | "Inserting"
  | "Success"
  | "Cancelled"
  | "Error";

export type FormattingMode = "Raw" | "Clean" | "Structured" | "Smart";
export type ComputeDevice = "cpu" | "gpu";
export type ModelFamily = "whisper" | "parakeet";
export type LanguageCode = "auto" | "en" | "ur" | "es" | "fr" | "de" | "it" | "pt" | "zh" | "ja" | "ko" | "ar" | "hi" | "ru" | "nl" | "tr" | "pl";

export type RetentionPolicy = "Forever" | "Days30" | "Days7" | "Off";

export interface AppSettings {
  provider: string;
  model: string;
  local_model_family: ModelFamily;
  compute_device: ComputeDevice;
  language: LanguageCode;
  microphone: string | null;
  formatting_mode: FormattingMode;
  hotkey: string;
  is_toggle_mode: boolean;
  retention_policy: RetentionPolicy;
  dictionary: Record<string, string>;
  snippets: Record<string, string>;
  theme: "dark" | "light" | "system";
  launch_at_startup?: boolean;
}

export type SettingsPatch = Partial<Omit<AppSettings, "microphone">> & {
  microphone?: string | null;
};

export interface AudioDeviceInfo {
  name: string;
  is_default: boolean;
}

export interface HistoryRecord {
  id: string;
  created_at: string;
  app_name: string | null;
  provider_id: string;
  model_name: string;
  raw_text: string;
  final_text: string;
  duration_ms: number;
  verification_status: string;
}

export interface LocalModelInfo {
  id: string;
  name: string;
  filename: string;
  size_mb: number;
  ram_estimate_mb: number;
  download_url: string;
  is_installed: boolean;
  is_default: boolean;
  family: ModelFamily;
  format: "ggml-bin" | "onnx-directory";
}

export interface HardwareRecommendation {
  logical_cores: number;
  estimated_ram_gb: number;
  recommended_model_id: string;
  reason: string;
}
