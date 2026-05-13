interface EmptyStateProps {
  title: string;
  description?: string;
}

export function EmptyState({ description, title }: EmptyStateProps) {
  return (
    <section className="border border-voicebox-border bg-white p-5">
      <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">{title}</h2>
      {description ? <p className="mt-3 text-sm text-voicebox-secondary">{description}</p> : null}
    </section>
  );
}
