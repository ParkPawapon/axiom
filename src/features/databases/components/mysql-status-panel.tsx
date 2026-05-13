import { StatusPanel } from "../../../shared/components/feedback/status-panel";

export function MysqlStatusPanel() {
  return (
    <StatusPanel title="MySQL Boundary" tone="warning">
      MySQL configuration and storage lifecycle are intentionally still backend-gated. The frontend
      only displays service inventory until credential, data directory, and process isolation rules
      are implemented.
    </StatusPanel>
  );
}
