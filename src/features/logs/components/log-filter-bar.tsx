import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";

interface LogFilterBarProps {
  isLoading: boolean;
  maxLines: number;
  query: string;
  sourceState: string;
  onMaxLinesChange: (maxLines: number) => void;
  onQueryChange: (query: string) => void;
  onRefresh: () => void;
}

const lineLimits = [100, 300, 500, 1000] as const;

export function LogFilterBar({
  isLoading,
  maxLines,
  onMaxLinesChange,
  onQueryChange,
  onRefresh,
  query,
  sourceState,
}: LogFilterBarProps) {
  return (
    <div className="grid gap-3 border border-voicebox-border bg-voicebox-surface p-4 lg:grid-cols-[minmax(0,1fr)_10rem_12rem_auto]">
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Search log text</span>
        <Input
          disabled={isLoading}
          placeholder="error, accepted, 404"
          value={query}
          onChange={(event) => onQueryChange(event.currentTarget.value)}
        />
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Lines</span>
        <Select
          disabled={isLoading}
          value={String(maxLines)}
          onChange={(event) => onMaxLinesChange(Number(event.currentTarget.value))}
        >
          {lineLimits.map((lineLimit) => (
            <option key={lineLimit} value={lineLimit}>
              {lineLimit}
            </option>
          ))}
        </Select>
      </label>
      <label className="grid gap-2">
        <span className="text-sm font-bold text-voicebox-black">Process state</span>
        <Select disabled value={sourceState}>
          <option value={sourceState}>{sourceState}</option>
        </Select>
      </label>
      <div className="flex items-end">
        <Button disabled={isLoading} onClick={onRefresh} variant="secondary">
          {isLoading ? "Refreshing" : "Refresh"}
        </Button>
      </div>
    </div>
  );
}
