// Adapted from the raycast quick calendar extension.
// https://www.raycast.com/fuksman/calendar
import {
  Action,
  ActionPanel,
  Color,
  Detail,
  openExtensionPreferences,
  showToast,
  getPreferenceValues,
  Toast,
} from "@vicinae/api";
import { useState, useEffect } from "react";
import { Calendar } from "calendar";
import { CurrentPreferences } from "./util";

const days = [
  ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"],
  ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"],
];

const weekStart = Number(CurrentPreferences.weekStart);
const currentMonth = new Date(
  new Date().getFullYear(),
  new Date().getMonth(),
  1,
);

export function getCalendar() {
  const [calendar, setCalendar] = useState("");
  const [header, setHeader] = useState("");
  const [date, setDate] = useState(currentMonth);

  useEffect(() => {
    // const cal = new Calendar(weekStart);
    const cal = new Calendar(0);
    const m = cal.monthDates(date.getFullYear(), date.getMonth());
    const today = new Date().toDateString();
    const header = date.toLocaleString("en", {
      month: "long",
      year: "numeric",
    });
    const daysArray = days[weekStart];

    const table = m
      .map((week) => {
        let row = "|";

        row += week
          .map((day) => {
            const dayString =
              day.getMonth() === date.getMonth()
                ? day.getDate().toString()
                : " ";
            const todayMarker =
              day.toDateString() === today && dayString !== " " ? "**• " : " ";
            return `${todayMarker}${dayString}${todayMarker !== " " ? "**" : ""} |`;
          })
          .join("");

        return `${row}\n`;
      })
      .join("");

    const weeksHeader = "|";
    const daysHeader = daysArray.map((day) => `**${day}**`).join(" |");
    const separator = `${"|"}${daysArray.map(() => ":---:").join(" |")}`;

    setHeader(header);
    setCalendar(
      `# ${header}\n${weeksHeader}${daysHeader} |\n${separator} |\n${table}`,
    );
  }, [date]);

  return calendar;
}
