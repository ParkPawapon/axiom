export type Result<TData, TError> =
  | {
      ok: true;
      data: TData;
    }
  | {
      ok: false;
      error: TError;
    };
