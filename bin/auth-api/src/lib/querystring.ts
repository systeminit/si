import Url from "url";

export function getQueryString(obj: Record<string, any>) {
  return new Url.URLSearchParams(obj).toString();
}
