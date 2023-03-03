/*
Normally I'd just use redis directly...
but since we aren't running redis yet and our needs are super basic, we'll add a wrapper
so that we can use an in-memory cache for now, and then swap in hosted redis on prod
should be easy enough to change this later if we want to...
*/

// if we decide to use an in memory cache on production, we'll at least swap in
// an existing npm module that has some more features and expiry implemented for us...
const inMemoryCache = {};

export async function setCache(key: string, val: Record<string, any>) {
  inMemoryCache[key] = val;
}

export async function getCache<T extends Record<string, any>>(
  key: string,
  deleteKey = false,
): Promise<T | undefined> {
  const val = inMemoryCache[key];
  if (val === undefined) return undefined;

  if (deleteKey) delete inMemoryCache[key];
  return val as T;
}
