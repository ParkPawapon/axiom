interface LoadingStateProps {
  label: string;
}

export function LoadingState({ label }: LoadingStateProps) {
  return <div className="border border-voicebox-border bg-white p-4 font-mono text-xs uppercase text-voicebox-secondary">{label}</div>;
}
