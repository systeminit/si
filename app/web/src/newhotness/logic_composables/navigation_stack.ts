import { reactive } from "vue";

const LIMIT = 5;

export interface Breadcrumb {
  url: string;
  params: Record<string, string | string[]>;
  name: string;
}

export const breadcrumbs = reactive<Breadcrumb[]>([]);

export const push = (
  url: string,
  name: string,
  params: Record<string, string | string[]>,
) => {
  breadcrumbs.push({ url, name, params });
  if (breadcrumbs.length > LIMIT) breadcrumbs.shift();
};

// 0 is always the current page
export const prevPage = () => breadcrumbs[1];
