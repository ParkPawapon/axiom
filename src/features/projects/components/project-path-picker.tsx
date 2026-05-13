import { open } from "@tauri-apps/plugin-dialog";

import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";

interface ProjectPathPickerProps {
  disabled?: boolean;
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export function ProjectPathPicker({ disabled, label, onChange, value }: ProjectPathPickerProps) {
  async function handleBrowse() {
    const selectedPath = await open({
      directory: true,
      multiple: false,
      title: "Select PHP project document root",
    });

    if (typeof selectedPath === "string") {
      onChange(selectedPath);
    }
  }

  return (
    <label className="grid gap-2">
      <span className="text-sm font-bold text-voicebox-black">{label}</span>
      <div className="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
        <Input
          disabled={disabled}
          placeholder="/absolute/path/to/public"
          value={value}
          onChange={(event) => onChange(event.currentTarget.value)}
        />
        <Button disabled={disabled} onClick={() => void handleBrowse()} variant="secondary">
          Browse
        </Button>
      </div>
    </label>
  );
}
