import { StatusPanel } from "../../../shared/components/feedback/status-panel";

export function MysqlStatusPanel() {
  return (
    <StatusPanel title="MySQL Boundary" tone="warning">
      MySQL lifecycle uses a backend allowlist for Homebrew launchd labels on macOS and known
      Windows service names. Credentials and data directory provisioning remain separate.
    </StatusPanel>
  );
}
