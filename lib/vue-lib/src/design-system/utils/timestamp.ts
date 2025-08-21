import * as _ from "lodash-es";
import { format, formatDistanceToNow, parseISO } from "date-fns";
import TimeAgo from "javascript-time-ago";
import en from "javascript-time-ago/locale/en";

// Size Formats Without Relative
// mini - M/D/YYYY
// normal - Month DD, YYYY
// today - TODAY or M/D/YYYY at TimeWithoutSeconds
// long - M/D/YYYY at TimeWithSeconds
// extended - Weekday Month DayWithSuffix, YYYY at TimeWithSeconds
export type TimestampSize = "mini" | "normal" | "long" | "extended";

TimeAgo.addLocale(en);
const timeAgo = new TimeAgo("en-US");

export function dateString(
  date: string | Date,
  size: TimestampSize,
  relative = false,
  relativeShorthand = false,
  showTimeIfToday = false,
  dateClasses = "",
  timeClasses = "",
) {
  const dateClassesSpan = dateClasses ? `<span class="${dateClasses}">` : "";
  const dateClassesCloseSpan = dateClasses ? "</span>" : "";
  const timeClassesSpan = dateClasses ? `<span class="${timeClasses}">` : "";
  const timeClassesCloseSpan = timeClasses ? "</span>" : "";
  let d: Date;
  if (_.isString(date)) {
    d = parseISO(date);
  } else {
    d = date;
  }

  if (
    !relative &&
    !relativeShorthand &&
    showTimeIfToday &&
    d.toDateString() === new Date().toDateString()
  ) {
    if (size === "long" || size === "extended") {
      return `${dateClassesSpan}Today${dateClassesCloseSpan} ${timeClassesSpan}at ${format(
        d,
        "h:mm:ss a",
      )}${timeClassesCloseSpan}`;
    }
    return `${timeClassesSpan}${format(d, "h:mm:ss a")}${timeClassesCloseSpan}`;
  }

  if (size === "mini") {
    if (relative) {
      return timeAgo.format(d, "mini-minute-now");
    }
    return format(d, "M/d/y");
  } else if (size === "extended") {
    if (relative) {
      return `${formatDistanceToNow(d)} ago`;
    }
    return `${dateClassesSpan}${format(
      d,
      "eeee MMMM do, y",
    )}${dateClassesCloseSpan} ${timeClassesSpan}at ${format(
      d,
      "h:mm:ss a",
    )}${timeClassesCloseSpan}`;
  } else if (size === "long") {
    if (relative) {
      return `${formatDistanceToNow(d)} ago`;
    }
    return `${dateClassesSpan}${format(
      d,
      "M/d/y",
    )}${dateClassesCloseSpan} ${timeClassesSpan}at ${format(
      d,
      "h:mm:ss a",
    )}${timeClassesCloseSpan}`;
  }

  if (relative) {
    return timeAgo.format(d);
  } else if (relativeShorthand) {
    return timeAgo.format(d, "twitter-first-minute");
  }

  return `${dateClassesSpan}${format(d, "MMMM d, y")}${dateClassesCloseSpan}`;
}

export function durationString(ms: number): string {
  if (ms < 0) ms = -ms;
  const time = {
    day: Math.floor(ms / 86400000),
    hour: Math.floor(ms / 3600000) % 24,
    minute: Math.floor(ms / 60000) % 60,
    second: Math.floor(ms / 1000) % 60,
    millisecond: Math.floor(ms) % 1000,
  };
  return Object.entries(time)
    .filter((val) => val[1] !== 0)
    .map(([key, val]) => `${val} ${key}${val !== 1 ? "s" : ""}`)
    .join(", ");
}
