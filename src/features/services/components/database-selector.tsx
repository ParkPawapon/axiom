import { Select } from "../../../shared/components/ui/select";

interface DatabaseSelectorProps {
  value?: string;
}

export function DatabaseSelector({ value = "none" }: DatabaseSelectorProps) {
  return (
    <label className="grid gap-2">
      <span className="text-sm font-bold text-voicebox-black">Database profile</span>
      <Select disabled value={value}>
        <option value={value}>No database profile selected</option>
      </Select>
    </label>
  );
}
