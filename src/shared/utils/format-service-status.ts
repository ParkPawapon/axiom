const statusLabels: Record<string, string> = {
  detected: "DETECTED",
  failed: "FAILED",
  notConfigured: "NOT CONFIGURED",
  running: "RUNNING",
  stopped: "STOPPED",
};

export function formatServiceStatus(status: string) {
  return statusLabels[status] ?? status.toUpperCase();
}
