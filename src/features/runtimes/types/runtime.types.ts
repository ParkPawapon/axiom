import type { DetectedPhpBinary, PhpVersionOption } from "../../projects/types/project.types";

export interface RuntimeValidationResult {
  readonly runtime: PhpVersionOption;
  readonly detectedBinary?: DetectedPhpBinary | null;
  readonly valid: boolean;
  readonly statusMessage: string;
}
