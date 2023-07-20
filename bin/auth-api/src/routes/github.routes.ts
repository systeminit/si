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

router.get("/github/releases", async (ctx) => {
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

  const releases: Release[] = [];
  for (const release of data) {
    const assets: Asset[] = [];
    for (const asset of release.assets) {
      assets.push({
        id: asset.id,
        contentType: asset.content_type,
        size: asset.size,
        name: asset.name,
        url: asset.browser_download_url,
      });
    }

    releases.push({
      id: release.id,
      version: release.tag_name,
      name: release.name,
      description: release.body,
      assets,
      publishedAt: release.published_at,
    });
  }

  ctx.body = releases;
});
