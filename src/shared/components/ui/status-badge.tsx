export type StatusTone = "neutral" | "success" | "warning" | "error";

interface StatusBadgeProps {
  label: string;
  tone?: StatusTone;
}

const toneClassNames: Record<StatusTone, string> = {
  neutral: "border-voicebox-black text-voicebox-black",
  success: "border-voicebox-success text-voicebox-success",
  warning: "border-voicebox-warning text-voicebox-warning",
  error: "border-voicebox-red text-voicebox-red",
};

export function StatusBadge({ label, tone = "neutral" }: StatusBadgeProps) {
  return (
    <span className={`inline-flex border px-2 py-1 font-mono text-xs ${toneClassNames[tone]}`}>
      {label}
    </span>
  );
}
