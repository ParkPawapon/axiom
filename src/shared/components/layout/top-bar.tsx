interface TopBarProps {
  activeRouteLabel: string;
  appName: string;
  tagline: string;
}

export function TopBar({ activeRouteLabel, appName, tagline }: TopBarProps) {
  return (
    <header className="flex min-h-16 items-center justify-between gap-4 border-b border-voicebox-border bg-white px-4 py-3 md:px-6">
      <div>
        <p className="font-display text-2xl uppercase leading-none text-voicebox-black">
          {appName}
        </p>
        <p className="mt-1 font-mono text-xs uppercase text-voicebox-secondary">{tagline}</p>
      </div>
      <div className="hidden border-l border-voicebox-border pl-4 text-right md:block">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Current Workspace</p>
        <p className="mt-1 text-sm font-bold text-voicebox-black">{activeRouteLabel}</p>
      </div>
    </header>
  );
}
