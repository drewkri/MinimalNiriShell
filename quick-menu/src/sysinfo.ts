import { useState, useEffect } from "react";

import { exec, type ExecException } from "node:child_process";
import { promisify } from "node:util";

const execAsync = promisify(exec);
export const currentTime = (): string => {
  const [timeString, setTimeString] = useState<string>("");

  useEffect(() => {
    const formatCurrentTime = (): string => {
      return new Date().toLocaleTimeString("en-US", {
        hour: "2-digit",
        minute: "2-digit",
        hour12: true,
      });
    };

    // Set initial time immediately
    setTimeString(formatCurrentTime());

    // Update every second
    const intervalId = setInterval(() => {
      setTimeString(formatCurrentTime());
    }, 1000);

    return () => clearInterval(intervalId);
  }, []);

  return timeString;
};

export const currentBattery = (): string => {
  const [batString, setBatString] = useState<string>("");

  useEffect(() => {
    const updateBat = (): void => {
      getBattery().then((s) => {
        setBatString(s);
      });
    };

    updateBat();

    // Update every minute
    const intervalId = setInterval(() => {
      updateBat();
    }, 60000);

    return () => clearInterval(intervalId);
  }, []);

  return batString;
};

async function getBattery(): Promise<string> {
  var result;
  // tries bat0 and bat1
  for (var i = 0; i < 2; i++) {
    try {
      result = await execAsync(`cat /sys/class/power_supply/BAT${i}/capacity`);
      break;
    } catch {
      continue;
    }
  }

  if (result) {
    return result.stdout + "%";
  } else {
    return "Unknown";
  }
}
