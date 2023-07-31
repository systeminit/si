import Axios from "axios";
import { tryCatch } from "../lib/try-catch";
import { ApiError } from "../lib/api-error";
import { router } from ".";

const ghApi = Axios.create({
  baseURL: `https://${process.env.GH_DOMAIN}`,
});

interface Asset {
  id: number;
  contentType: string;
  size: number;
  name: string;
  url: string;
}

interface Release {
  id: number;
  version: string;
  name: string;
  description: string;
  assets: Asset[];
  publishedAt: string;
}

let releaseCachedAt: Date | null = null;
let releases: Release[] | null = null;
let loadingReleases: boolean = false;

const loadReleases = async (): Promise<Release[]> => {
  const seconds = Math.abs(Date.now() - (releaseCachedAt?.getTime() ?? 0));
  if ((seconds > 180 * 1000 || !releases) && !loadingReleases) {
    loadingReleases = true;

    try {
      const data = await tryCatch(async () => {
        const req = await ghApi.get("/repos/systeminit/si/releases", {
          headers: {
            Accept: "application/vnd.github+json",
            Authorization: `Bearer ${process.env.GH_TOKEN}`,
            "X-Github-Api-Version": "2022-11-28",
          },
        });

        return req.data;
      }, (err) => {
        throw new ApiError(
          err.response.statusText,
          "GITHUB_LIST_RELEASES_FAILURE",
          err.response.data.message,
        );
      });

      const releasesTemp: Release[] = [];
      for (const githubRelease of data) {
        const release: Release = {
          id: githubRelease.id,
          version: githubRelease.tag_name,
          name: githubRelease.name,
          description: githubRelease.body,
          publishedAt: githubRelease.published_at,
          assets: [],
        };

        for (const githubAsset of githubRelease.assets) {
          const asset: Asset = {
            id: githubAsset.id,
            contentType: githubAsset.content_type,
            size: githubAsset.size,
            name: githubAsset.name,
            url: githubAsset.browser_download_url,
          };
          release.assets.push(asset);
        }

        releasesTemp.push(release);
      }

      releaseCachedAt = new Date();
      releases = releasesTemp;
    } catch (err) {
      if (releases) {
        // eslint-disable-next-line no-console
        console.error(err);
        return releases;
      }
      throw err;
    } finally {
      loadingReleases = false;
    }
  }

  return releases ?? [];
};

router.get("/github/releases/latest", async (ctx) => {
  const latest = (await loadReleases())[0];
  if (!latest) {
    throw new ApiError(
      "NotFound",
      "GITHUB_LATEST_RELEASE_NOT_FOUND",
      "not found",
    );
  }

  ctx.body = latest;
});

router.get("/github/releases", async (ctx) => {
  ctx.body = await loadReleases();
});

interface GithubTag {
  ref: string;
  node_id: string;
  url: string;
  object: {
    sha: string;
    type: string;
    url: string;
  };
}

interface LatestContainer {
  namespace: string;
  repository: string;
  name: string;
  gitSha: string;
  digest: string;
}

let containersCachedAt: Date | null = null;
let containers: LatestContainer[] | null = null;
let loadingContainers: boolean = false;

router.get("/github/containers/latest", async (ctx) => {
  const seconds = Math.abs(Date.now() - (containersCachedAt?.getTime() ?? 0));
  if ((seconds > 180 * 1000 || !containers) && !loadingContainers) {
    loadingContainers = true;

    try {
      const data: GithubTag[] = await tryCatch(async () => {
        const req = await ghApi.get("/repos/systeminit/si/git/refs/tags", {
          headers: {
            Accept: "application/vnd.github+json",
            Authorization: `Bearer ${process.env.GH_TOKEN}`,
            "X-Github-Api-Version": "2022-11-28",
          },
        });

        return req.data;
      }, (err) => {
        throw new ApiError(
          err.response.statusText,
          "GITHUB_LIST_TAGS_FAILURE",
          err.response.data.message,
        );
      });

      const prefixes = ["sdf", "veritech", "pinga", "council", "module-index", "web", "otelcol", "jaeger", "nats", "postgres"];
      const latestContainers = [];
      for (const tag of data) {
        for (const prefix of prefixes) {
          const start = `refs/tags/${prefix}/sha256-`;
          if (tag.ref.startsWith(start)) {
            const digest = tag.ref.replace(start, "");
            latestContainers.push({
              namespace: "systeminit",
              repository: prefix,
              name: `${prefix}/sha256-${digest}`,
              gitSha: tag.object.sha,
              digest,
            });
            break;
          }
        }
      }

      containersCachedAt = new Date();
      containers = latestContainers;
    } catch (err) {
      if (containers) {
        // eslint-disable-next-line no-console
        console.error(err);
        ctx.body = containers;
        return;
      }
      throw err;
    } finally {
      loadingContainers = false;
    }
  }

  ctx.body = containers ?? [];
});
