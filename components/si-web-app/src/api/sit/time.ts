export function timeDelta(t1: string, t2: string): string {
  const d1 = new Date(t1);
  const d2 = new Date(t2);
  const timeDelta = Math.abs(d2.getTime() - d1.getTime());
  const seconds = timeDelta / 1000;
  return `${seconds} sec`;
}
