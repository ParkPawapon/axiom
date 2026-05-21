import { Chip } from "../../../shared/components/ui/chip";
import type { AppSettings } from "../types/settings.types";

const securityControls = [
  "Thin Tauri command boundary",
  "Backend path validation",
  "PHP process command allowlist",
  "Per-project process guard",
  "No frontend shell construction",
  "Least-privilege Tauri capabilities",
] as const;

interface SecuritySettingsFormProps {
  settings?: AppSettings;
}

export function SecuritySettingsForm({ settings }: SecuritySettingsFormProps) {
  return (
    <div className="grid gap-3">
      <div className="flex items-center justify-between gap-3 border border-voicebox-border bg-voicebox-surface p-3">
        <span className="text-sm font-bold text-voicebox-black">Audit log persistence</span>
        <Chip tone={settings?.auditLogEnabled === false ? "warning" : "success"}>
          {settings?.auditLogEnabled === false ? "Disabled" : "Enabled"}
        </Chip>
      </div>
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
        Security preferences are loaded from the typed backend settings repository. Editing remains
        guarded by validated update commands.
      </p>
    </div>
  );
}
