import React from "react";
import ReactDOM from "react-dom/client";
import FloatApp from "./FloatApp";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

function FloatRoot() {
  const [status, setStatus] = React.useState("idle");
  const [opacity, setOpacity] = React.useState(0.7);

  React.useEffect(() => {
    // 取得初始狀態
    invoke<string>("get_current_status").then(setStatus).catch(() => {});

    // 監聽狀態變化
    const unlisten = listen<{ effective_status?: string }>(
      "state-changed",
      (event) => {
        if (event.payload.effective_status) {
          setStatus(event.payload.effective_status);
        }
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // 滾輪調整透明度
  const handleWheel = React.useCallback(
    (e: React.WheelEvent) => {
      e.preventDefault();
      const delta = e.deltaY > 0 ? -0.05 : 0.05;
      const newOpacity = Math.max(0.3, Math.min(1.0, opacity + delta));
      setOpacity(newOpacity);
      invoke("set_float_opacity", { opacity: newOpacity }).catch(() => {});
    },
    [opacity],
  );

  // 拖拉：用原生 DOM 事件，確保在 Lottie SVG 上也能拖
  React.useEffect(() => {
    const win = getCurrentWindow();

    const onMouseDown = (e: MouseEvent) => {
      if (e.button === 0) {
        e.preventDefault();
        win.startDragging().then(() => {
          invoke("save_float_position").catch(() => {});
        });
      }
    };

    document.addEventListener("mousedown", onMouseDown);
    return () => document.removeEventListener("mousedown", onMouseDown);
  }, []);

  return (
    <div onWheel={handleWheel}>
      <FloatApp status={status} />
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <FloatRoot />
  </React.StrictMode>,
);
