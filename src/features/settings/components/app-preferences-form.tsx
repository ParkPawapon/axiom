import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";

export function AppPreferencesForm() {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Application name</span>
        <Input disabled value="AxiomPHP" />
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Theme</span>
        <Select disabled value="voicebox-light">
          <option value="voicebox-light">VoiceBox Light</option>
        </Select>
      </label>
      <label className="grid gap-2 md:col-span-2">
        <span className="text-sm font-bold text-voicebox-black">Configuration persistence</span>
        <Input disabled value="Rust backend app-data directory" />
      </label>
    </div>
  );
}
