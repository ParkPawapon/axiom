import { invoke } from "@tauri-apps/api/core";

export async function invokeTauriCommand<TResult>(
  command: string,
  args?: Record<string, unknown>,
): Promise<TResult> {
  const tauriRuntime = globalThis as typeof globalThis & {
    __TAURI_INTERNALS__?: unknown;
  };

  if (tauriRuntime.__TAURI_INTERNALS__ === undefined) {
    throw new Error("Tauri backend is not available in browser preview.");
  }

  return invoke<TResult>(command, args);
}
