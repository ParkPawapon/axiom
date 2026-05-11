export function redactSecretLikeValue(value: string): string {
  return value.length > 0 ? "[redacted]" : "";
}
