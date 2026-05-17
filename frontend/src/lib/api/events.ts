import type {
  InputDescriptor,
  OutputDescriptor,
  Session,
  SessionStats
} from './types';

export type Event =
  | { type: 'input_updated'; input: InputDescriptor }
  | { type: 'input_removed'; input_id: string }
  | { type: 'output_updated'; output: OutputDescriptor }
  | { type: 'output_removed'; output_id: string }
  | { type: 'session_armed'; session: Session }
  | { type: 'session_started'; session: Session }
  | { type: 'session_stats'; session_id: string; stats: SessionStats }
  | {
      type: 'levels';
      input_id: string;
      level: {
        left: { peak_dbfs: number; rms_dbfs: number };
        right: { peak_dbfs: number; rms_dbfs: number };
      };
    }
  | {
      type: 'session_stopped';
      session_id: string;
      reason: 'user_requested' | 'taken_over' | 'input_lost' | 'output_lost' | 'errored';
    }
  | {
      type: 'notice';
      severity: 'info' | 'warn' | 'error';
      message: string;
    };
