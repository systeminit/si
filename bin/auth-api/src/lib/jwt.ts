/*
instructions to generate JWT signing key
- run `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key` # Don't add passphrase
- run `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`
- `cat jwtRS256.key`
- `cat jwtRS256.key.pub`
*/

import fs from "fs";
import JWT from "jsonwebtoken";

// load private and public key from either env var or paths set in config
// keys in the repo are also used by SDF to verify jwt is signed correctly and in tests to create/sign jwts
let _JWT_PRIVATE_KEY = process.env.JWT_PRIVATE_KEY;
if (!_JWT_PRIVATE_KEY && process.env.JWT_PRIVATE_KEY_PATH) {
  // path is relative to .env file
  _JWT_PRIVATE_KEY = fs.readFileSync(`${process.env.JWT_PRIVATE_KEY_PATH}`, 'utf-8');
}
let _JWT_PUBLIC_KEY = process.env.JWT_PUBLIC_KEY;
if (!_JWT_PUBLIC_KEY && process.env.JWT_PUBLIC_KEY_PATH) {
  // path is relative to .env file
  _JWT_PUBLIC_KEY = fs.readFileSync(`${process.env.JWT_PUBLIC_KEY_PATH}`, 'utf-8');
}
if (!_JWT_PRIVATE_KEY) throw new Error('Missing JWT signing private key');
if (!_JWT_PUBLIC_KEY) throw new Error('Missing JWT signing public key');

_JWT_PRIVATE_KEY = _JWT_PRIVATE_KEY.replace(/\\n/g, '\n');
_JWT_PUBLIC_KEY = _JWT_PUBLIC_KEY.replace(/\\n/g, '\n');

export const JWT_PUBLIC_KEY = _JWT_PUBLIC_KEY;

export function createJWT(
  payload: Record<string, any>,
  options?: Omit<JWT.SignOptions, 'algorithm'>,
) {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return JWT.sign(payload, _JWT_PRIVATE_KEY!, { algorithm: "RS256", ...options });
}
export function verifyJWT(token: string) {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return JWT.verify(token, _JWT_PUBLIC_KEY!);
}
