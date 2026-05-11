interface TopBarProps {
  appName: string;
}

export function TopBar({ appName }: TopBarProps) {
  return (
    <header className="flex h-16 items-center justify-between bg-white px-6">
      <div>
        <p className="font-display text-2xl uppercase leading-none">{appName}</p>
      </div>
      <p className="font-mono text-xs uppercase text-voicebox-secondary">
        Architecture scaffold only
      </p>
    </header>
  );
}
