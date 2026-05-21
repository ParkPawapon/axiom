import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  AppSettings,
  AppSettingsUpdate,
  AppSettingsUpdateResult,
} from "../types/settings.types";

export function readSettings() {
  return invokeTauriCommand<AppSettings>("read_settings");
}

export function updateSettings(update: AppSettingsUpdate) {
  return invokeTauriCommand<AppSettingsUpdateResult>("update_settings", { update });
}
