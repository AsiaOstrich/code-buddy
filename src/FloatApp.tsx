import Lottie from "lottie-react";

import idleData from "./animations/idle.json";
import workingData from "./animations/working.json";
import thinkingData from "./animations/thinking.json";
import waitingInputData from "./animations/waiting_input.json";
import waitingConfirmData from "./animations/waiting_confirm.json";
import completedData from "./animations/completed.json";
import errorData from "./animations/error.json";

const ANIMATION_MAP: Record<string, object> = {
  idle: idleData,
  working: workingData,
  thinking: thinkingData,
  waiting_input: waitingInputData,
  waiting_confirm: waitingConfirmData,
  completed: completedData,
  error: errorData,
};

const STATUS_COLORS: Record<string, string> = {
  idle: "rgb(160, 160, 160)",
  working: "rgb(30, 120, 255)",
  thinking: "rgb(160, 100, 255)",
  waiting_input: "rgb(255, 180, 0)",
  waiting_confirm: "rgb(255, 220, 0)",
  completed: "rgb(50, 200, 80)",
  error: "rgb(240, 50, 50)",
};

interface FloatAppProps {
  status: string;
}

export default function FloatApp({ status }: FloatAppProps) {
  const animationData = ANIMATION_MAP[status] ?? ANIMATION_MAP.idle;
  const borderColor = STATUS_COLORS[status] ?? STATUS_COLORS.idle;

  return (
    <div
      data-testid="float-container"
      style={{
        width: "80px",
        height: "80px",
        borderRadius: "50%",
        overflow: "hidden",
        border: `3px solid ${borderColor}`,
        cursor: "grab",
        background: "transparent",
      }}
    >
      <Lottie animationData={animationData} loop />
    </div>
  );
}
