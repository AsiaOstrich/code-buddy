import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BuddyAnimation } from "./components/BuddyAnimation";
import "./App.css";

const STATUSES = [
  { key: "idle", label: "Idle", color: "#999999" },
  { key: "working", label: "Working", color: "#2196F3" },
  { key: "thinking", label: "Thinking", color: "#9C27B0" },
  { key: "waiting_input", label: "Waiting Input", color: "#FFC107" },
  { key: "waiting_confirm", label: "Waiting Confirm", color: "#FF9800" },
  { key: "completed", label: "Completed", color: "#4CAF50" },
  { key: "error", label: "Error", color: "#F44336" },
] as const;

interface SessionInfo {
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

function statusColor(status: string): string {
  return STATUSES.find((s) => s.key === status)?.color ?? "#999";
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s}s`;
}

function App() {
  const [effectiveStatus, setEffectiveStatus] = useState("idle");
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [message, setMessage] = useState("");

  // 啟動時取得 sessions
  useEffect(() => {
    invoke<SessionInfo[]>("get_sessions")
      .then(setSessions)
      .catch((e) => setMessage(`Error: ${e}`));
  }, []);

  // 監聽 state-changed 事件
  useEffect(() => {
    const unlisten = listen<StateChangedPayload>("state-changed", (event) => {
      setEffectiveStatus(event.payload.effective_status);
      setSessions(event.payload.sessions);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // 手動切換 tray icon（Dev Panel 測試用）
  const switchIcon = async (status: string) => {
    try {
      const result = await invoke<string>("switch_tray_icon", { status });
      setEffectiveStatus(status);
      setMessage(result);
    } catch (error) {
      setMessage(`Error: ${error}`);
    }
  };

  return (
    <div className="container">
      <h1>Code Buddy Dev Panel</h1>
      <div style={{ width: 120, height: 120, margin: "0 auto" }}>
        <BuddyAnimation status={effectiveStatus} />
      </div>
      <p className="status-text">
        有效狀態：
        <strong style={{ color: statusColor(effectiveStatus) }}>
          {effectiveStatus}
        </strong>
      </p>

      {/* 即時 Session 列表 */}
      <div className="section">
        <h2>Sessions ({sessions.length})</h2>
        {sessions.length === 0 ? (
          <p className="empty-hint">尚未偵測到 session</p>
        ) : (
          <ul className="session-list">
            {sessions.map((s) => (
              <li key={s.id} className="session-item">
                <span
                  className="session-dot"
                  style={{ backgroundColor: statusColor(s.status) }}
                />
                <div className="session-info">
                  <span className="session-name">{s.project_name}</span>
                  <span className="session-detail">
                    {s.status} · {formatDuration(s.duration_secs)} ·{" "}
                    {s.id.slice(0, 8)}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* 手動測試按鈕（可收合） */}
      <details className="section">
        <summary><h2 style={{ display: "inline" }}>手動測試</h2></summary>
        <div className="button-grid">
          {STATUSES.map(({ key, label, color }) => (
            <button
              key={key}
              onClick={() => switchIcon(key)}
              className={effectiveStatus === key ? "active" : ""}
              style={{ backgroundColor: color, color: "#fff" }}
            >
              {label}
            </button>
          ))}
        </div>
      </details>

      {message && <p className="message">{message}</p>}
    </div>
  );
}

export default App;
