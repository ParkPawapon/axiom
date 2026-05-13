import { StatusPanel } from "../../../shared/components/feedback/status-panel";

export function PostgresStatusPanel() {
  return (
    <StatusPanel title="PostgreSQL Boundary" tone="warning">
      PostgreSQL management remains read-only in the frontend until backend adapters define safe
      cluster ownership, port policy, credentials, and crash recovery.
    </StatusPanel>
  );
}
