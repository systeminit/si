/*
Normally I'd just use redis directly...
but since we aren't running redis yet and our needs are super basic, we'll add a wrapper
so that we can use an in-memory cache for now, and then swap in hosted redis on prod
should be easy enough to change this later if we want to...
*/

// for now we'll say everything being cached should be an object... can allow raw strings/etc later if necessary
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
  inMemoryCache[key] = val;

  // obviously this is dumb and incomplete...
  // will eventually swap in something full featured or just use redis
  if (options?.expiresIn) {
    expireTimeouts[key] = setTimeout(async () => {
      await deleteCacheKey(key);
    }, options.expiresIn * 1000);
  }
}

export async function getCache<T extends CacheableInfo>(
  key: string,
  deleteKey = false,
): Promise<T | undefined> {
  const val = inMemoryCache[key];
  if (val === undefined) return undefined;

  if (deleteKey) {
    await deleteCacheKey(key);
  }
  return val as T;
}

export async function deleteCacheKey(key: string) {
  delete inMemoryCache[key];
  if (expireTimeouts[key]) {
    clearTimeout(expireTimeouts[key]);
  }
}
