import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";

interface LogFilterBarProps {
  query: string;
  sourceState: string;
  onQueryChange: (query: string) => void;
}

export function LogFilterBar({ onQueryChange, query, sourceState }: LogFilterBarProps) {
  return (
    <div className="grid gap-3 border border-voicebox-border bg-voicebox-surface p-4 md:grid-cols-[minmax(0,1fr)_12rem]">
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Filter text</span>
        <Input
          placeholder="Filter visible log metadata"
          value={query}
          onChange={(event) => onQueryChange(event.currentTarget.value)}
        />
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Process state</span>
        <Select disabled value={sourceState}>
          <option value={sourceState}>{sourceState}</option>
        </Select>
      </label>
    </div>
  );
}
