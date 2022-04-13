import contentfulUpstream from "contentful";

export const useContentful = async (config: any) => {
  let contentful = contentfulUpstream;
  if (!contentful) {
    contentful = await import("contentful");
  }
  console.log("config", { p: config.public, contentful });
  const c = contentful.createClient({
    space: config.public.space,
    accessToken: config.public.accessToken,
  });
  console.log("contentful", { c, config });
  return c;
};
