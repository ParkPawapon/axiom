export function SafeDefaultsPanel() {
  return (
    <section className="border border-voicebox-border bg-white p-4">
      <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">
        Safe Defaults
      </h2>
      <div className="mt-4 grid gap-2 text-sm text-voicebox-secondary">
        <p>Advanced Docker trust, KMS, scheduler, and registry metadata stay in detail views.</p>
        <p>Daily controls show only ready, missing, running, stopped, or safely blocked states.</p>
        <p>Secrets never cross into frontend logs or diagnostics.</p>
      </div>
    </section>
  );
}
