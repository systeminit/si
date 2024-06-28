interface DateParts {
  year: string;
  month: string;
  day: string;
  hour: string;
  minute: string;
  second: string;
  dayPeriod: string;
  era: string;
  literal: string;
  timezone: string;
  timeZoneName: string;
  weekday: string;
  unknown: string;
  fractionalSecond: string;
}

export function dateToVersion(date: Date) {
  const p = new Intl.DateTimeFormat("en", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  })
    .formatToParts(date)
    .reduce((acc, part) => {
      acc[part.type] = part.value;
      return acc;
    }, {} as DateParts);

  return `${p.year}${p.month}${p.day}${p.hour}${p.minute}${p.second}`;
}
