import _logger from "../../../logger.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import { bfsPropTree } from "../../../spec/props.ts";
import { existsSync } from "node:fs";

const logger = _logger.ns("siSpecs").seal();

const SAVE_INTERVAL = 5;

// This is a failsafe that checks if we've generated any bad links to AWS docs for props. If we
// have, it removes them from the generated specs so we don't link off to the aether.
export async function removeBadDocLinks(
  specs: ExpandedPkgSpec[],
  docLinkCache: string,
) {
  // Gather the unique doc links
  const foundLinks = new Set<string>();
  for (const { schemas: [{ variants: [variant] }] } of specs) {
    // Gather the links
    bfsPropTree([
      variant.domain,
      variant.resourceValue,
      variant.secrets,
      variant.secretDefinition,
    ], (prop) => {
      const docLink = prop.data.docLink?.split("#")?.[0];
      if (docLink) foundLinks.add(docLink);
    });
  }

  // Fetch the docs to see if they are there (if they aren't already in cache)
  const linkCache = await loadLinkCache(docLinkCache);
  let lastSaved = Date.now();
  for (const link of foundLinks) {
    if (!(link in linkCache)) {
      logger.debug(`Checking doc link: ${link} ...`);
      linkCache[link] = (await fetch(link, { redirect: "manual" })).status;
    }
    // Save periodically in case there are issues
    if ((Date.now() - lastSaved) > SAVE_INTERVAL) {
      await saveLinkCache(linkCache, docLinkCache);
      lastSaved = Date.now();
    }
  }
  await saveLinkCache(linkCache, docLinkCache);

  // Remove the doc links from anything that links to a bad spot
  for (const { schemas: [{ variants: [variant] }] } of specs) {
    // Gather the links
    bfsPropTree([
      variant.domain,
      variant.resourceValue,
      variant.secrets,
      variant.secretDefinition,
    ], (prop) => {
      const docLink = prop.data.docLink?.split("#")?.[0];
      if (docLink && linkCache[docLink] !== 200) {
        prop.data.docLink = null;
      }
    });
  }

  return specs;
}

type LinkCache = Record<string, number>;

async function loadLinkCache(docLinkCache: string): Promise<LinkCache> {
  if (!existsSync(docLinkCache)) {
    return {};
  }
  return JSON.parse(await Deno.readTextFile(docLinkCache));
}

async function saveLinkCache(linkCache: LinkCache, docLinkCache: string) {
  await Deno.writeTextFile(docLinkCache, JSON.stringify(linkCache, null, 2));
}
