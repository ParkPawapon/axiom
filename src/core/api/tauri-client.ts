import { invoke } from "@tauri-apps/api/core";

export async function invokeTauriCommand<TResult>(
  command: string,
  args?: Record<string, unknown>,
): Promise<TResult> {
  return invoke<TResult>(command, args);
}
