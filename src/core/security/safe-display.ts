export function safeDisplay(value: unknown): string {
  return typeof value === "string" ? value : "";
}
