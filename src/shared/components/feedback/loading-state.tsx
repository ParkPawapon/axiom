interface LoadingStateProps {
  label: string;
}

export function LoadingState({ label }: LoadingStateProps) {
  return <p className="font-mono text-xs uppercase text-voicebox-secondary">{label}</p>;
}
