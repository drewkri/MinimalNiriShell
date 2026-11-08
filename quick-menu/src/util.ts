import { getPreferenceValues } from "@vicinae/api";
import { exec, type ExecException } from "node:child_process";
import { promisify } from "node:util";
export const execAsync = promisify(exec);

export const openDeepLink = (link: string): void => {
  execAsync(`vicinae ${link}`);
};

interface Preferences {
  weekStart: Number;
  notificationsCommand: string;
  networkSettingsLink: string;
  showBattery: boolean;
}

export const CurrentPreferences = getPreferenceValues<Preferences>();
