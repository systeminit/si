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
  const prev = prevPage();
  if (!prev) breadcrumbs.push({ url, name, params, query });
  else if (
    prev.name !== name &&
    prev.url !== url &&
    !_.isEqual(prev.params, params) &&
    !_.isEqual(prev.query, query)
  )
    breadcrumbs.push({ url, name, params, query });
  if (breadcrumbs.length > LIMIT) breadcrumbs.shift();
};

// 0 is always the current page
export const prevPage = () => breadcrumbs[1];
