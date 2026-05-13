import { Select } from "../../../shared/components/ui/select";

interface RuntimeSelectorProps {
  value?: string;
}

export function RuntimeSelector({ value = "project-runtime" }: RuntimeSelectorProps) {
  return (
    <label className="grid gap-2">
      <span className="text-sm font-bold text-voicebox-black">Runtime source</span>
      <Select disabled value={value}>
        <option value={value}>Per-project PHP runtime</option>
      </Select>
    </label>
  );
}
