import { invoke } from "@tauri-apps/api/core";

export type TauriCommandArgs = Record<string, unknown>;

export async function invokeTauriCommand<TResult>(
  command: string,
  args?: TauriCommandArgs,
): Promise<TResult> {
  return invoke<TResult>(command, args);
}
