import IORedis, { Redis } from 'ioredis';
import { parseRedisUrl } from 'parse-redis-url-simple';

const redisConnectionObj = parseRedisUrl(process.env.REDIS_URL);

export const REDIS_ENABLED = !!process.env.REDIS_URL;

export const redis = new IORedis({
  ...redisConnectionObj[0],
  lazyConnect: true,
}) as ExtendedIORedis;

// add helper to get/set json objects without worrying about JSON serialization
async function setJSON<T extends { [key: string]: any } >(
  this: Redis,
  key: string,
  payload: T,
  options?: { expiresIn?: number },
) {
  let args: string[] = [];
  // seconds to expire
  if (options?.expiresIn) args = ['EX', options.expiresIn.toString()];
  return this.set(key, JSON.stringify(payload), ...args as any);
}
async function getJSON<T extends { [key: string]: any } >(
  this: Redis,
  key: string,
  options?: { delete?: boolean },
): Promise<T | null> {
  const result = await this.get(key);
  if (result === null) return result;
  if (!result) return null; // treat empty strings as null
  if (options?.delete) await this.del(key);
  return JSON.parse(result) as T;
}

export type ExtendedIORedis = Redis & {
  setJSON: typeof setJSON
  getJSON: typeof getJSON,
};
redis.setJSON = setJSON;
redis.getJSON = getJSON;
