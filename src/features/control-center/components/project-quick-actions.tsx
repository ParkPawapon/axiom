import { openPath, openUrl } from "@tauri-apps/plugin-opener";

import { Button } from "../../../shared/components/ui/button";
import type { QuickActionKind, QuickActionSummary } from "../types/control-center.types";

interface ProjectQuickActionsProps {
  actions: QuickActionSummary[];
  isBusy: boolean;
  onProjectAction: (kind: QuickActionKind) => Promise<void>;
  onNavigateLogs: () => void;
}

export function ProjectQuickActions({
  actions,
  isBusy,
  onProjectAction,
  onNavigateLogs,
}: ProjectQuickActionsProps) {
  async function runAction(action: QuickActionSummary) {
    if (!action.enabled) {
      return;
    }

    if (action.kind === "openBrowser" || action.kind === "openPhpMyAdmin") {
      if (action.target) {
        await openUrl(action.target);
      }
      return;
    }

    if (action.kind === "openFolder") {
      if (action.target) {
        await openPath(action.target);
      }
      return;
    }

    if (action.kind === "openLogs") {
      onNavigateLogs();
      return;
    }

    await onProjectAction(action.kind);
  }

  return (
    <section className="border-2 border-voicebox-black bg-white p-4">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">
          Quick Actions
        </h2>
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Safe commands only</p>
      </div>
      <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
        {actions.map((action) => (
          <div className="grid gap-1" key={action.id}>
            <Button
              disabled={!action.enabled || isBusy}
              onClick={() => void runAction(action)}
              variant={action.kind === "startProject" ? "primary" : "secondary"}
            >
              {action.label}
            </Button>
            {!action.enabled && action.disabledReason ? (
              <p className="min-h-8 text-xs leading-snug text-voicebox-secondary">
                {action.disabledReason}
              </p>
            ) : (
              <p className="min-h-8 text-xs leading-snug text-voicebox-tertiary">Ready</p>
            )}
          </div>
        ))}
      </div>
    </section>
  );
}
