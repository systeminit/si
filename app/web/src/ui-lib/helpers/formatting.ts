import _ from "lodash";
import { parseISO } from "date-fns";

import TimeAgo from "javascript-time-ago";
import TimeAgoEnLocale from "javascript-time-ago/locale/en";

export type TimestampSize = "mini" | "normal" | "long" | "extended";

TimeAgo.addLocale(TimeAgoEnLocale);

const timeAgo = new TimeAgo("en-US");

const formatters = {
  // TODO: decide one which format(s) we want to support
  // but probably keep it simple (dont support all) to keep things more consistent
  timeAgo(rawDate: string | Date | undefined | null) {
    if (!rawDate) return "?";
    const date = _.isDate(rawDate) ? rawDate : parseISO(rawDate);
    return timeAgo.format(date);
  },
};

export default formatters;

// const dateStr = computed(() => {
//   const d = props.date;
//   if (props.size === "mini") {
//     if (props.relative) {

//     }
//     return format(d, "M/d/y");
//   } else if (props.size === "extended") {
//     if (props.relative) {
//       return `${formatDistanceToNow(d)} ago`;
//     }
//     return `${format(d, "eeee MMMM do, y")} at ${format(d, "h:mm:ss a")}`;
//   } else if (props.size === "long") {
//     if (props.relative) {
//       return `${formatDistanceToNow(d)} ago`;
//     }
//     return `${format(d, "M/d/y")} at ${format(d, "h:mm:ss a")}`;
//   }
//   if (props.relative) {
//     return timeAgo.format(d);
//   }
//   return format(d, "MMMM d, y");
// });
