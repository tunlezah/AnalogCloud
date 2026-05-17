// Mirror of `shared/src/*.rs`. Keep in sync with the Rust types.

export type SampleFormat = 'f32' | 's16' | 's24' | 's32';

export interface AudioFormat {
  sample_rate: number;
  channels: number;
  format: SampleFormat;
}

export type InputKind =
  | 'bluetooth_turntable'
  | 'bluetooth_cassette'
  | 'bluetooth_generic'
  | 'line_in'
  | 'usb'
  | 'virtual';

export type InputTransport =
  | 'bluetooth_a2dp'
  | 'bluetooth_hfp'
  | 'alsa'
  | 'pipewire_node'
  | 'virtual';

export interface InputDescriptor {
  id: string;
  name: string;
  kind: InputKind;
  transport: InputTransport;
  codec: string | null;
  native_format: AudioFormat | null;
  connected: boolean;
  selected: boolean;
  last_level_dbfs: number | null;
}

export type OutputKind = 'browser' | 'local' | 'chromecast' | 'air_play';

export type OutputTransport =
  | 'web_rtc_opus'
  | 'pipewire_sink'
  | 'opus_http'
  | 'aac_http'
  | 'raop';

export type OutputState =
  | 'idle'
  | 'negotiating'
  | 'streaming'
  | 'reconnecting'
  | 'errored';

export interface OutputDescriptor {
  id: string;
  name: string;
  kind: OutputKind;
  transport: OutputTransport;
  state: OutputState;
  latency_ms_estimate: number | null;
  active: boolean;
  reconnectable: boolean;
}

export type SessionState =
  | 'idle'
  | 'armed'
  | 'playing'
  | 'reconnecting'
  | 'stopped'
  | 'errored';

export interface SessionStats {
  latency_ms: number;
  buffer_ms: number;
  codec: string | null;
  bitrate_kbps: number | null;
  reconnects: number;
  cpu_usage_pct: number | null;
}

export interface Session {
  session_id: string;
  input_id: string;
  output_kind: OutputKind;
  output_device: string;
  transport: OutputTransport;
  started_at: string;
  controller_clients: string[];
  state: SessionState;
  stats: SessionStats;
}

export interface StartSessionRequest {
  input_id: string;
  output_kind: OutputKind;
  output_device: string;
}

export type AudioMode = 'reference' | 'compatible';

export type EqPreset =
  | 'flat'
  | 'vinyl-warmth'
  | 'bass-focus'
  | 'speech'
  | 'tape'
  | 'night'
  | 'custom';

export interface EqualizerSettings {
  enabled: boolean;
  preset: EqPreset;
  bands_db: number[];
}

export type EnrichmentMode = 'off' | 'immediate' | 'delayed' | 'manual';

export interface EnrichmentSettings {
  mode: EnrichmentMode;
  delayed_wait_seconds: number;
}

export interface HassSettings {
  enabled: boolean;
  mqtt_host: string | null;
  mqtt_port: number | null;
  mqtt_username: string | null;
  mqtt_topic_prefix: string | null;
}

export interface Settings {
  audio_mode: AudioMode;
  equalizer: EqualizerSettings;
  enrichment: EnrichmentSettings;
  hass: HassSettings;
  theme: string;
}

export const EQ_BANDS_HZ = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000] as const;
