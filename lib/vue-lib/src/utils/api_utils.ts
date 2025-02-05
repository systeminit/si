// accepting null | undefined just to allow other parts of the codebase flexibility
// throwing if we ever hit that :(
export type URLPattern = Array<
  string | Record<string, string | undefined | null>
>;
export const describePattern = (pattern: URLPattern): [string, string] => {
  const _url: string[] = [];
  const _urlName: string[] = [];
  pattern.forEach((p) => {
    if (typeof p === "string") {
      _url.push(p);
      _urlName.push(p);
    } else {
      const vals = Object.values(p);
      if (!vals[0])
        throw Error(
          `Bad URLPattern ${JSON.stringify(pattern)} with: ${JSON.stringify(p)}
        `,
        );
      else _url.push(vals[0]); // url gets the value
      const keys = Object.keys(p);
      if (keys.length > 0) _urlName.push(`:${keys[0]}`); // name gets the str
    }
  });
  return [_url.join("/"), _urlName.join("/")];
};
