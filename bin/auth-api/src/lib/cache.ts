/*
small wrapper around redis so taht we can use in-memory cache when running locally
if/when we add a redis component anything else, we can delete this and just use it directly
*/
/* eslint-disable no-console */

// for now we'll say everything being cached should be an object... can allow raw strings/etc later if necessary
import { redis, REDIS_ENABLED } from "./redis";

type CacheableInfo = Record<string, any>;

// if we decide to use an in memory cache on production, we'll at least swap in
// an existing npm module that has some more features and expiry implemented for us...
const inMemoryCache: Record<string, CacheableInfo> = {};
const expireTimeouts: Record<string, NodeJS.Timeout> = {};

export async function setCache(
  /** cache key */
  key: string,
  /** value (object) to store */
  val: CacheableInfo,
  /** additional options */
  options?: {
    /** expire data from cache after delay (seconds) */
    expiresIn?: number;
  },
) {
  const start = Date.now();
  try {
    if (REDIS_ENABLED) {
      await redis.setJSON(key, val, options);
    } else {
      inMemoryCache[key] = val;

      // obviously this is dumb and incomplete... but is only used for local dev
      if (options?.expiresIn) {
        expireTimeouts[key] = setTimeout(async () => {
          await deleteCacheKey(key);
        }, options.expiresIn * 1000);
      }
    }
    const duration_ms = Date.now() - start;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "info",
      type: "redis",
      operation: "set",
      key,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
    }));
  } catch (error) {
    const duration_ms = Date.now() - start;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "error",
      type: "redis",
      operation: "set",
      key,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
      error: error instanceof Error ? error.message : String(error),
    }));
    throw error;
  }
}

export async function getCache<T extends CacheableInfo>(
  key: string,
  deleteKey = false,
): Promise<T | undefined> {
  const start = Date.now();
  try {
    let result: T | undefined;
    let hit = false;

    if (REDIS_ENABLED) {
      const obj = await redis.getJSON(key, { delete: deleteKey });
      if (obj) {
        result = obj as unknown as T;
        hit = true;
      }
    } else {
      const val = inMemoryCache[key];
      if (val !== undefined) {
        hit = true;
        if (deleteKey) {
          await deleteCacheKey(key);
        }
        result = val as T;
      }
    }

    const duration_ms = Date.now() - start;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "info",
      type: "redis",
      operation: "get",
      key,
      hit,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
    }));

    return result;
  } catch (error) {
    const duration_ms = Date.now() - start;
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level: "error",
      type: "redis",
      operation: "get",
      key,
      duration_ms,
      ...(duration_ms > 5000 && { slowCall: true }),
      error: error instanceof Error ? error.message : String(error),
    }));
    throw error;
  }
}

export async function deleteCacheKey(key: string) {
  if (REDIS_ENABLED) {
    await redis.del(key);
  } else {
    delete inMemoryCache[key];
    if (expireTimeouts[key]) {
      clearTimeout(expireTimeouts[key]);
    }
  }
}

export function cleanupInMemoryCache() {
  if (REDIS_ENABLED) return;

  Object.keys(expireTimeouts).forEach((key) => {
    clearTimeout(expireTimeouts[key]);
    delete expireTimeouts[key];
  });

  Object.keys(inMemoryCache).forEach((key) => {
    delete inMemoryCache[key];
  });
}
