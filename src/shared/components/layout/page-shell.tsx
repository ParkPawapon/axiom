import type { ReactNode } from "react";

interface PageShellProps {
  title: string;
  description: string;
  children?: ReactNode;
}

export function PageShell({ title, description, children }: PageShellProps) {
  return (
    <section className="mx-auto flex max-w-6xl flex-col gap-6">
      <div className="border-b-2 border-voicebox-black pb-4">
        <h1 className="font-display text-4xl uppercase leading-none">{title}</h1>
        <p className="mt-3 max-w-3xl text-sm text-voicebox-secondary">{description}</p>
      </div>
      {children}
    </section>
  );
}
