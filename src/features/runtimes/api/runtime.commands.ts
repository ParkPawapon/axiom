import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type { PhpVersionOption } from "../../projects/types/project.types";
import type { RuntimeValidationResult } from "../types/runtime.types";

export function detectPhpRuntimes() {
  return invokeTauriCommand<PhpVersionOption[]>("detect_php_runtimes");
}

export function validateRuntime(phpVersion: string) {
  return invokeTauriCommand<RuntimeValidationResult>("validate_runtime", { phpVersion });
}
