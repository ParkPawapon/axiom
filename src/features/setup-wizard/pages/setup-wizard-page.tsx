import { useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { getSetupDiagnostics } from "../api/setup-wizard.commands";
import { DatabaseStep } from "../components/database-step";
import { DomainStep } from "../components/domain-step";
import { PermissionStep } from "../components/permission-step";
import { PhpVersionStep } from "../components/php-version-step";
import { SetupStepper } from "../components/setup-stepper";
import { SetupSummaryStep } from "../components/setup-summary-step";
import { SystemCheckStep } from "../components/system-check-step";
import type { SetupDiagnosticStep, SetupDiagnosticsReport } from "../types/setup-wizard.types";

function StepContent({ step }: { step: SetupDiagnosticStep }) {
  if (step.id === "system") {
    return <SystemCheckStep step={step} />;
  }
  if (step.id === "php") {
    return <PhpVersionStep step={step} />;
  }
  if (step.id === "database") {
    return <DatabaseStep step={step} />;
  }
  if (step.id === "domain") {
    return <DomainStep step={step} />;
  }
  if (step.id === "permission") {
    return <PermissionStep step={step} />;
  }

  return <SetupSummaryStep step={step} />;
}

export function SetupWizardPage() {
  const [activeStepId, setActiveStepId] = useState("system");
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [report, setReport] = useState<SetupDiagnosticsReport>();

  useEffect(() => {
    async function load() {
      setIsLoading(true);
      setErrorMessage(undefined);

      try {
        const nextReport = await getSetupDiagnostics();
        setReport(nextReport);
        setActiveStepId((currentStepId) =>
          nextReport.steps.some((step) => step.id === currentStepId)
            ? currentStepId
            : nextReport.steps[0]?.id ?? "system",
        );
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Setup diagnostics could not be loaded safely."));
      } finally {
        setIsLoading(false);
      }
    }

    void load();
  }, []);

  const activeStep = useMemo(
    () => report?.steps.find((step) => step.id === activeStepId) ?? report?.steps[0],
    [activeStepId, report?.steps],
  );

  return (
    <PageShell
      title="Setup"
      description="Guided readiness checks for a safe local PHP environment."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading ? <LoadingState label="Loading setup diagnostics" /> : null}

      {report && activeStep ? (
        <div className="grid gap-4 lg:grid-cols-[18rem_minmax(0,1fr)]">
          <SetupStepper
            activeStepId={activeStep.id}
            onSelectStep={setActiveStepId}
            steps={report.steps}
          />
          <StepContent step={activeStep} />
        </div>
      ) : null}
    </PageShell>
  );
}
