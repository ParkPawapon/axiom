interface EmptyStateProps {
  title: string;
  description: string;
}

export function EmptyState({ title, description }: EmptyStateProps) {
  return (
    <div className="border border-voicebox-border bg-voicebox-surface p-6">
      <h2 className="font-display text-xl uppercase text-voicebox-black">{title}</h2>
      <p className="mt-2 max-w-2xl text-sm text-voicebox-secondary">{description}</p>
    </div>
  );
}
