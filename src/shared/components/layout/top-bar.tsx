interface TopBarProps {
  appName: string;
}

export function TopBar({ appName }: TopBarProps) {
  return (
    <header className="flex h-16 items-center justify-between border-b border-voicebox-border bg-white px-6">
      <div>
        <p className="font-display text-2xl uppercase leading-none text-voicebox-black">{appName}</p>
        <p className="mt-1 font-mono text-xs uppercase text-voicebox-secondary">Architecture scaffold only</p>
      </div>
    </header>
  );
}
