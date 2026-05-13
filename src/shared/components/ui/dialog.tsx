import type { ReactNode } from "react";

import { Button } from "./button";

interface DialogProps {
  title: string;
  children: ReactNode;
  confirmLabel?: string;
  isOpen: boolean;
  onClose: () => void;
  onConfirm?: () => void;
}

export function Dialog({
  children,
  confirmLabel = "Confirm",
  isOpen,
  onClose,
  onConfirm,
  title,
}: DialogProps) {
  if (!isOpen) {
    return null;
  }

  return (
    <div
      aria-modal="true"
      className="fixed inset-0 z-50 grid place-items-center bg-voicebox-black/30 p-4"
      role="dialog"
    >
      <section className="w-full max-w-lg border-2 border-voicebox-black bg-white p-5">
        <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">
          {title}
        </h2>
        <div className="mt-4 text-sm leading-relaxed text-voicebox-secondary">{children}</div>
        <div className="mt-5 flex flex-wrap justify-end gap-2">
          <Button onClick={onClose} variant="secondary">
            Cancel
          </Button>
          {onConfirm ? <Button onClick={onConfirm}>{confirmLabel}</Button> : null}
        </div>
      </section>
    </div>
  );
}
