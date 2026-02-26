import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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

function App() {
  const [current, setCurrent] = useState("idle");
  const [message, setMessage] = useState("");

  const switchIcon = async (status: string) => {
    try {
      const result = await invoke<string>("switch_tray_icon", { status });
      setCurrent(status);
      setMessage(result);
    } catch (error) {
      setMessage(`Error: ${error}`);
    }
  };

  return (
    <div className="container">
      <h1>Code Buddy Dev Panel</h1>
      <p className="status-text">
        目前狀態：<strong>{current}</strong>
      </p>
      <div className="button-grid">
        {STATUSES.map(({ key, label, color }) => (
          <button
            key={key}
            onClick={() => switchIcon(key)}
            className={current === key ? "active" : ""}
            style={{ backgroundColor: color, color: "#fff" }}
          >
            {label}
          </button>
        ))}
      </div>
      {message && <p className="message">{message}</p>}
    </div>
  );
}

export default App;
