import * as _ from "lodash-es";
import { format, formatDistanceToNow, parseISO } from "date-fns";
import TimeAgo from "javascript-time-ago";
import en from "javascript-time-ago/locale/en";

export type TimestampSize = "mini" | "normal" | "long" | "extended";

TimeAgo.addLocale(en);
const timeAgo = new TimeAgo("en-US");

export function dateString(
  date: string | Date,
  size: TimestampSize,
  relative = false,
  showTimeIfToday = false,
) {
  let d: Date;
  if (_.isString(date)) {
    d = parseISO(date);
  } else {
    d = date;
  }

  if (
    !relative &&
    showTimeIfToday &&
    d.toDateString() === new Date().toDateString()
  ) {
    if (size === "long" || size === "extended") {
      return `Today at ${format(d, "h:mm:ss a")}`;
    }
    return format(d, "h:mm:ss a");
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
    return `${format(d, "eeee MMMM do, y")} at ${format(d, "h:mm:ss a")}`;
  } else if (size === "long") {
    if (relative) {
      return `${formatDistanceToNow(d)} ago`;
    }
    return `${format(d, "M/d/y")} at ${format(d, "h:mm:ss a")}`;
  }
  if (relative) {
    return timeAgo.format(d);
  }
  return format(d, "MMMM d, y");
}
