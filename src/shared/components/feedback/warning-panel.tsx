interface WarningPanelProps {
  message: string;
}

export function WarningPanel({ message }: WarningPanelProps) {
  return <div className="border border-voicebox-warning bg-white p-4 text-sm text-voicebox-warning">{message}</div>;
}
