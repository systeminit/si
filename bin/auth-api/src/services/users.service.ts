import { getCache, setCache } from "../lib/cache";

export type UserId = string;

// this will become a model when we implement db
export type User = {
  id: string;
  firstName: string;
  lastName: string;
  email?: string;
  emailVerified: boolean;
  pictureUrl?: string;
  auth0Details?: any;
};

export async function getUserById(id: UserId) {
  return await getCache<User>(`user:${id}`);
}

export async function createUser(user: User) {
  return await setCache(`user:${user.id}`, user);
}
export async function updateUser(user: User) {
  return await setCache(`user:${user.id}`, user);
}
