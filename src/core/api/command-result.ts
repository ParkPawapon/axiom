import type { CommandError } from "./command-error";

export type CommandResult<TData> =
  | {
      ok: true;
      data: TData;
    }
  | {
      ok: false;
      error: CommandError;
    };
