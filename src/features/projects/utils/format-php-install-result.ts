import type {
  PhpRuntimeInstallProvider,
  ProjectPhpInstallResult,
} from "../types/project.types";

const providerLabels: Record<PhpRuntimeInstallProvider, string> = {
  homebrew: "Homebrew",
  scoop: "Scoop",
};

export function formatPhpInstallResult(result: ProjectPhpInstallResult) {
  const diagnostics =
    result.diagnostics.length > 0
      ? result.diagnostics
          .map((diagnostic) => `${diagnostic.level}: ${diagnostic.message}`)
          .join(" ")
      : "No package-manager diagnostics were returned.";

  const rollback = result.rollback
    ? `Rollback ${result.rollback.succeeded ? "succeeded" : "failed"}: ${result.rollback.message}`
    : "Rollback was not required.";

  return [
    result.statusMessage,
    `Provider: ${providerLabels[result.provider]}.`,
    `Package: ${result.packageName}.`,
    `Diagnostics: ${diagnostics}`,
    rollback,
  ].join(" ");
}
