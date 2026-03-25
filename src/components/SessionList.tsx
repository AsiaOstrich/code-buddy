import type { SessionInfo } from "../store";

const ATTENTION_STATUSES = new Set([
  "waiting_input",
  "waiting_confirm",
  "error",
]);

const STATUS_PRIORITY: Record<string, number> = {
  waiting_input: 6,
  waiting_confirm: 6,
  error: 5,
  working: 4,
  thinking: 3,
  completed: 2,
  idle: 1,
};

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return s > 0 ? `${m}m ${s}s` : `${m}m`;
}

function sortByPriority(sessions: SessionInfo[]): SessionInfo[] {
  return [...sessions].sort(
    (a, b) =>
      (STATUS_PRIORITY[b.status] ?? 0) - (STATUS_PRIORITY[a.status] ?? 0),
  );
}

interface SessionListProps {
  sessions: SessionInfo[];
  focusSessionId: string | null;
  onPin: (sessionId: string) => void;
}

export function SessionList({
  sessions,
  focusSessionId,
  onPin,
}: SessionListProps) {
  if (sessions.length === 0) {
    return <p>沒有活躍的 session</p>;
  }

  const sorted = sortByPriority(sessions);

  return (
    <ul>
      {sorted.map((session) => (
        <li
          key={session.id}
          data-active={session.id === focusSessionId ? "true" : undefined}
          onClick={() => onPin(session.id)}
          style={{ cursor: "pointer" }}
        >
          <span>{session.project_name}</span>
          <span>
            {session.status} · {formatDuration(session.duration_secs)}
          </span>
        </li>
      ))}
    </ul>
  );
}
