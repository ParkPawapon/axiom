import type { ReactNode } from "react";

interface SettingsSectionProps {
  children: ReactNode;
  description: string;
  title: string;
}

export function SettingsSection({ children, description, title }: SettingsSectionProps) {
  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="border-b border-voicebox-border pb-4">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Settings</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          {title}
        </h2>
        <p className="mt-3 text-sm text-voicebox-secondary">{description}</p>
      </div>
      <div className="mt-5">{children}</div>
    </section>
  );
}
