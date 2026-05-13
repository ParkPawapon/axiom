import type { ReactNode } from "react";

interface PageShellProps {
  title: string;
  description?: string;
  children: ReactNode;
}

export function PageShell({ children, description, title }: PageShellProps) {
  return (
    <section className="grid gap-5">
      <div className="border-b-2 border-voicebox-black pb-4">
        <h1 className="font-display text-4xl uppercase leading-none text-voicebox-black">
          {title}
        </h1>
        {description ? (
          <p className="mt-3 max-w-3xl text-sm text-voicebox-secondary">{description}</p>
        ) : null}
      </div>
      {children}
    </section>
  );
}
