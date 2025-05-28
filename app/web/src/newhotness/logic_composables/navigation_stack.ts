import * as _ from "lodash-es";
import { reactive } from "vue";

const LIMIT = 5;

export type params = Record<string, string | string[]>;
export type query = Record<string, string | string[]>;
export interface Breadcrumb {
  url: string;
  params: params;
  query: query;
  name: string;
}

export const breadcrumbs = reactive<Breadcrumb[]>([]);

export const push = (
  url: string,
  name: string,
  params: params,
  query: query,
) => {
  const prev = breadcrumbs[breadcrumbs.length - 1];
  const data = { url, name, params, query };
  if (!prev) breadcrumbs.push(data);
  else if (
    // dont push dupe navs onto the stack
    prev.name !== name ||
    prev.url !== url ||
    !_.isEqual(prev.params, params) ||
    !_.isEqual(prev.query, query)
  )
    breadcrumbs.push(data);
  if (breadcrumbs.length > LIMIT) breadcrumbs.shift();
};

export const prevPage = () => {
  if (breadcrumbs.length === 1) return undefined; // current page is only page on the stack
  return breadcrumbs[breadcrumbs.length - 2];
};

export const reset = () => {
  breadcrumbs.splice(0, Infinity);
};
