import _logger from "../logger.ts";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import { bfsPropTree, ExpandedPropSpec } from "../spec/props.ts";
import { existsSync } from "node:fs";

const logger = _logger.ns("siSpecs").seal();

const SAVE_INTERVAL = 5;
const DOC_START =
  "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/";

// This is a failsafe that checks if we've generated any bad links to AWS docs for props. If we
// have, it removes them from the generated specs so we don't link off to the aether.
export async function removeBadDocLinks(
  specs: ExpandedPkgSpec[],
  docLinkCache: string,
) {
  const linkCache = await loadLinkCache(docLinkCache);

  // Gather the unique doc links
  const foundLinks = {} as Record<
    string,
    { fragment: string; prop: ExpandedPropSpec }[]
  >;
  for (const { schemas: [{ variants: [variant] }] } of specs) {
    // Add the top level link
    if (variant.data.link) {
      foundLinks[variant.data.link] ??= [];
    }

    // Gather prop links
    bfsPropTree([
      variant.domain,
      variant.resourceValue,
      variant.secrets,
      variant.secretDefinition,
    ], (prop) => {
      if (!prop.data.docLink) return;
      const [docLink, fragment] = prop.data.docLink.split("#", 2);
      if (docLink) {
        foundLinks[docLink] ??= [];
        foundLinks[docLink].push({ fragment, prop });
      }
    });
  }

  // Fetch the docs to see if they are there
  let lastSaved = Date.now();
  for (const [docLink, propRefs] of Object.entries(foundLinks)) {
    if (!docLink.startsWith(DOC_START)) {
      throw new Error(`Unexpected doc link ${docLink}`);
    }
    const page = docLink.slice(DOC_START.length);
    if (!(page in linkCache)) {
      logger.info(`Checking doc link: ${docLink} ...`);
      const response = await fetch(docLink, { redirect: "manual" });
      const { status } = response;
      const fragments = propRefs.map(({ fragment }) => fragment);
      if (status === 200) {
        // Search the doc to make sure the fragments link somewhere
        const doc = await response.text();
        if (fragments.length == 0) {
          linkCache[page] = {};
        } else {
          linkCache[page] = {
            missing: fragments.filter((fragment) =>
              !doc.includes(`id="${fragment}"`)
            ),
          };
        }
      } else {
        // It's not 200, so everything is missing
        linkCache[page] = { error: status, missing: fragments };
      }
    }

    // Save periodically in case there are issues
    if ((Date.now() - lastSaved) > SAVE_INTERVAL) {
      await saveLinkCache(linkCache, docLinkCache);
      lastSaved = Date.now();
    }
  }

  // Actually remove the links
  for (const { schemas: [{ variants: [variant] }] } of specs) {
    if (variant.data.link && linkCache[variant.data.link].error) {
      variant.data.link = null;
    }
  }

  for (const [docLink, propRefs] of Object.entries(foundLinks)) {
    const page = docLink.split("/").pop()!;
    for (const { fragment, prop } of propRefs) {
      if (linkCache[page].missing?.includes(fragment)) {
        prop.data.docLink = null;
      }
    }
  }

  await saveLinkCache(linkCache, docLinkCache);

  return specs;
}

type LinkCache = {
  [docLink in string]: { error?: number; missing?: string[] };
};

async function loadLinkCache(docLinkCache: string): Promise<LinkCache> {
  if (!existsSync(docLinkCache)) return {};
  const result = JSON.parse(await Deno.readTextFile(docLinkCache)) as LinkCache;
  if (typeof Object.values(result)[0] !== "object") return {};
  return result as LinkCache;
}

async function saveLinkCache(linkCache: LinkCache, docLinkCache: string) {
  await Deno.writeTextFile(docLinkCache, JSON.stringify(linkCache, null, 2));
}
