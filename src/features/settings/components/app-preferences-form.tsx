import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";
import type { AppSettings } from "../types/settings.types";

interface AppPreferencesFormProps {
  settings?: AppSettings;
}

export function AppPreferencesForm({ settings }: AppPreferencesFormProps) {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Application name</span>
        <Input disabled value={settings?.appName ?? "AxiomPHP"} />
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Theme</span>
        <Select disabled value={settings?.theme ?? "voiceboxLight"}>
          <option value="voiceboxLight">VoiceBox Light</option>
        </Select>
      </label>
      <label className="grid gap-2 md:col-span-2">
        <span className="text-sm font-bold text-voicebox-black">Configuration persistence</span>
        <Input disabled value="Rust backend app-data directory" />
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Default PHP port</span>
        <Input disabled value={String(settings?.defaultPhpPort ?? 8080)} />
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Docker context</span>
        <Input disabled value={settings?.dockerContext ?? "Default Docker context"} />
      </label>
    </div>
  );
}
