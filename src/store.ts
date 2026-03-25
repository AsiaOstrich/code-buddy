import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SessionInfo {
  id: string;
  agent_type: string;
  status: string;
  project_path: string;
  project_name: string;
  duration_secs: number;
}

interface StateChangedPayload {
  session_id: string;
  status: string;
  effective_status: string;
  sessions: SessionInfo[];
}

const ATTENTION_STATUSES = new Set([
  "waiting_input",
  "waiting_confirm",
  "error",
]);

interface AppState {
  sessions: SessionInfo[];
  effectiveStatus: string;
  focusSessionId: string | null;
  _unlisten: UnlistenFn | null;

  init: () => Promise<void>;
  pinSession: (sessionId: string) => Promise<void>;
  attentionCount: () => number;
}

export const useAppStore = create<AppState>()((set, get) => ({
  sessions: [],
  effectiveStatus: "idle",
  focusSessionId: null,
  _unlisten: null,

  init: async () => {
    // 先建立 listener，再拉取快照，避免遺漏事件
    const unlisten = await listen<StateChangedPayload>(
      "state-changed",
      (event) => {
        set({
          sessions: event.payload.sessions,
          effectiveStatus: event.payload.effective_status,
          focusSessionId: event.payload.session_id,
        });
      },
    );

    const sessions = await invoke<SessionInfo[]>("get_sessions");
    set({ sessions, _unlisten: unlisten });
  },

  pinSession: async (sessionId: string) => {
    await invoke("pin_session", { sessionId });
    set({ focusSessionId: sessionId });
  },

  attentionCount: () => {
    return get().sessions.filter((s) => ATTENTION_STATUSES.has(s.status))
      .length;
  },
}));
