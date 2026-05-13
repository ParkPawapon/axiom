import { Chip } from "../../../shared/components/ui/chip";

const securityControls = [
  "Thin Tauri command boundary",
  "Backend path validation",
  "PHP process command allowlist",
  "Per-project process guard",
  "No frontend shell construction",
  "Least-privilege Tauri capabilities",
] as const;

export function SecuritySettingsForm() {
  return (
    <div className="grid gap-3">
      {securityControls.map((control) => (
        <div
          className="flex items-center justify-between gap-3 border border-voicebox-border bg-voicebox-surface p-3"
          key={control}
        >
          <span className="text-sm font-bold text-voicebox-black">{control}</span>
          <Chip tone="success">Enabled</Chip>
        </div>
      ))}
      <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        Editable security preferences are intentionally disabled until the backend settings
        repository exposes a typed persistence contract.
      </p>
    </div>
  );
}
