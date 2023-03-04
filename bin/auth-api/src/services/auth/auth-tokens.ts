import { JwtPayload } from 'jsonwebtoken';
import * as Koa from 'koa';
import { ApiError } from '../../lib/api-error';
import { getCache } from '../../lib/cache';
import { createJWT, verifyJWT } from "../../lib/jwt";
import { tryCatch } from '../../lib/try-catch';
import { getUserById } from '../users.service';

export const SI_COOKIE_NAME = "si-auth";

// TODO: figure out the shape of the JWT and what data we want

// Auth tokens used for communication between the user's browser and this auth api
type AuthTokenData = {
  userId: string;
};

// will figure out what we want to pass in here...
export function createAuthToken(userId: string) {
  const payload: AuthTokenData = {
    userId,
  };
  return createJWT(payload);
}

export async function decodeAuthToken(token: string) {
  return verifyJWT(token) as AuthTokenData & JwtPayload;
}

// Auth tokens used for communication between the user's browser and SDF
// and between that SDF instance and this auth api if necessary
type SdfAuthTokenData = {
  // TODO: probably want to scope this for an instance type/url?
  userId: string;
};

// will figure out what we want to pass in here...
export function createSdfAuthToken(userId: string) {
  const payload: SdfAuthTokenData = {
    userId,
  };
  return createJWT(payload);
}

export async function decodeSdfAuthToken(token: string) {
  return verifyJWT(token) as SdfAuthTokenData & JwtPayload;
}

export async function loadAuthMiddleware(ctx: Koa.Context, next: Koa.Next) {
  const authToken = ctx.cookies.get(SI_COOKIE_NAME);
  if (!authToken) return next();

  const decoded = await tryCatch(() => {
    return decodeAuthToken(authToken);
  }, (_err) => {
    // TODO: check the type of error before handling this way

    // clear the cookie and return an error
    ctx.cookies.set(SI_COOKIE_NAME, null);
    throw new ApiError('Forbidden', 'AuthTokenCorrupt', 'Invalid auth token');
  });

  console.log(decoded);

  // not sure what would cause this?
  if (!decoded) {
    throw new ApiError('Forbidden', 'AuthTokenCorrupt', 'Invalid auth token');
  }

  // TODO: deal with various other errors, logout on all devices, etc...

  ctx.$ ||= ctx.state; // shorter alias for koa's ctx.state

  const { userId } = decoded;

  // VERY TEMPORARY - grab user data from cache while we dont have a db
  const user = await getUserById(userId);
  if (!user) {
    ctx.cookies.set(SI_COOKIE_NAME, null);
    throw new ApiError('Conflict', 'Cannot find user data');
  }

  ctx.$.user = user;

  return next();
}
