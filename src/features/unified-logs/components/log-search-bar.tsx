import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";
import type { UnifiedLogSeverityFilter } from "../types/unified-log.types";

interface LogSearchBarProps {
  readonly onQueryChange: (query: string) => void;
  readonly onSeverityChange: (severity: UnifiedLogSeverityFilter) => void;
  readonly onTailCountChange: (tailCount: number) => void;
  readonly query: string;
  readonly severity: UnifiedLogSeverityFilter;
  readonly tailCount: number;
}

export function LogSearchBar({
  onQueryChange,
  onSeverityChange,
  onTailCountChange,
  query,
  severity,
  tailCount,
}: LogSearchBarProps) {
  return (
    <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_11rem_10rem]">
      <label className="grid gap-1 text-sm font-semibold text-voicebox-secondary">
        Search
        <Input
          aria-label="Search log lines"
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="Filter sanitized log lines"
          value={query}
        />
      </label>
      <label className="grid gap-1 text-sm font-semibold text-voicebox-secondary">
        Severity
        <Select
          aria-label="Filter log severity"
          onChange={(event) => onSeverityChange(event.target.value as UnifiedLogSeverityFilter)}
          value={severity}
        >
          <option value="all">All</option>
          <option value="error">Error</option>
          <option value="warn">Warning</option>
          <option value="info">Info</option>
          <option value="debug">Debug</option>
        </Select>
      </label>
      <label className="grid gap-1 text-sm font-semibold text-voicebox-secondary">
        Tail
        <Select
          aria-label="Select log tail count"
          onChange={(event) => onTailCountChange(Number(event.target.value))}
          value={tailCount}
        >
          <option value={50}>50 lines</option>
          <option value={100}>100 lines</option>
          <option value={200}>200 lines</option>
          <option value={500}>500 lines</option>
        </Select>
      </label>
    </div>
  );
}
