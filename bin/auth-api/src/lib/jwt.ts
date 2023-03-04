/*
instructions to generate JWT signing key
- run `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key` # Don't add passphrase
- run `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`
- `cat jwtRS256.key`
- `cat jwtRS256.key.pub`
*/

import JWT from "jsonwebtoken";

const JWT_PRIVATE_KEY = process.env.JWT_PRIVATE_KEY!;
export const JWT_PUBLIC_KEY = process.env.JWT_PRIVATE_KEY!;

export function createJWT(payload: Record<string, any>) {
  return JWT.sign(payload, JWT_PRIVATE_KEY, { algorithm: "RS256" });
}
export function verifyJWT(token: string) {
  return JWT.verify(token, JWT_PUBLIC_KEY);
}
