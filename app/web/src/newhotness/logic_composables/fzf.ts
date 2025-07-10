import { extendedMatch, Fzf } from "fzf";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const useFzf = (list: any, selector: (item: any) => string) => {
  return new Fzf(list, {
    casing: "case-insensitive",
    match: extendedMatch,
    selector,
  });
};
