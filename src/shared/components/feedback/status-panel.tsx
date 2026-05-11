interface StatusPanelProps {
  title: string;
  status: string;
}

export function StatusPanel({ title, status }: StatusPanelProps) {
  return (
    <div className="border border-voicebox-border bg-white p-4">
      <p className="font-bold">{title}</p>
      <p className="mt-2 font-mono text-xs uppercase text-voicebox-secondary">{status}</p>
    </div>
  );
}
