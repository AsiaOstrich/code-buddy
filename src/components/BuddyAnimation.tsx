import Lottie from "lottie-react";

import idleData from "../animations/idle.json";
import workingData from "../animations/working.json";
import thinkingData from "../animations/thinking.json";
import waitingInputData from "../animations/waiting_input.json";
import waitingConfirmData from "../animations/waiting_confirm.json";
import completedData from "../animations/completed.json";
import errorData from "../animations/error.json";

const ANIMATION_MAP: Record<string, object> = {
  idle: idleData,
  working: workingData,
  thinking: thinkingData,
  waiting_input: waitingInputData,
  waiting_confirm: waitingConfirmData,
  completed: completedData,
  error: errorData,
};

interface BuddyAnimationProps {
  status: string;
}

export function BuddyAnimation({ status }: BuddyAnimationProps) {
  const animationData = ANIMATION_MAP[status] ?? ANIMATION_MAP.idle;

  return <Lottie animationData={animationData} loop />;
}
