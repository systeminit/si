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

let cachedAt: Date | null = null;
let releases: Release[] | null = null;
let loading: boolean = false;

router.get("/github/releases", async (ctx) => {
  const seconds = Math.abs(Date.now() - (cachedAt?.getTime() ?? 0));
  if ((seconds > 180 * 1000 || !releases) && !loading) {
    loading = true;

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

      cachedAt = new Date();
      releases = releasesTemp;
    } catch (err) {
      if (releases) {
        // eslint-disable-next-line no-console
        console.error(err);
        return releases;
      }
      throw err;
    } finally {
      loading = false;
    }
  }

  ctx.body = releases ?? [];
});
