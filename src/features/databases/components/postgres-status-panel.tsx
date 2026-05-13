import { StatusPanel } from "../../../shared/components/feedback/status-panel";

export function PostgresStatusPanel() {
  return (
    <StatusPanel title="PostgreSQL Boundary" tone="warning">
      PostgreSQL lifecycle uses backend OS adapters for supported launchd labels and Windows service
      names. Cluster creation, credential policy, and backup flows remain separate.
    </StatusPanel>
  );
}
