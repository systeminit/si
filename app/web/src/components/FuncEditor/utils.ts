export const toOptionValues = <T extends { value: string | number; label: string }>(options: T[], ids: string[]): T[] =>
  options.filter((opt) => (typeof opt.value === "string" ? ids.includes(opt.value) : false));
