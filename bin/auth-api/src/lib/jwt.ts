/*
instructions to generate JWT signing key
 run `ssh-keygen -t ecdsa -b 256 -m PEM -f jwtES256.key`
- run `openssl ec -in jwtES256.key -pubout -outform PEM -out jwtES256.key.pub`
- `cat jwtES256.key`
- `cat jwtES256.key.pub`

For RS256: (deprecated)
- run `ssh-keygen -t rsa -b 4096 -m PEM -f jwtRS256.key` # Don't add passphrase
- run `openssl rsa -in jwtRS256.key -pubout -outform PEM -out jwtRS256.key.pub`
- `cat jwtRS256.key`
- `cat jwtRS256.key.pub`
*/

import fs from "fs";
import JWT from "jsonwebtoken";

const DEFAULT_ALGO = "RS256";

type Algo = "RS256" | "ES256";

const jwtAlgo = (algo?: string): Algo => {
  switch (algo) {
    case "RS256":
    case "ES256":
      return algo;
    default:
      return DEFAULT_ALGO;
  }
};

const keyEnvPaths = {
  primary: {
    private: "JWT_PRIVATE_KEY",
    privatePath: "JWT_PRIVATE_KEY_PATH",
    public: "JWT_PUBLIC_KEY",
    publicPath: "JWT_PUBLIC_KEY_PATH",
    algo: "JWT_ALGO",
  },
  secondary: {
    private: "JWT_2ND_PRIVATE_KEY",
    privatePath: "JWT_2ND_PRIVATE_KEY_PATH",
    public: "JWT_2ND_PUBLIC_KEY",
    publicPath: "JWT_2ND_PUBLIC_KEY_PATH",
    algo: "JWT_2ND_ALGO",
  },
};

// load private and public keys from either env var or paths set in config keys
// in the repo are also used by SDF to verify jwt is signed correctly and in
// tests to create/sign jwts

const prepareKeys = (which: "primary" | "secondary"): { privKey?: string, pubKey?: string, algo: Algo } => {
  const privateLiteral = process.env[keyEnvPaths[which].private];
  const privatePath = process.env[keyEnvPaths[which].privatePath];

  let privKey = privateLiteral ?? (privatePath ? fs.readFileSync(privatePath, 'utf-8') : undefined);
  if (privKey) {
    privKey = privKey.replace(/\\n/g, '\n');
  }

  const publicLiteral = process.env[keyEnvPaths[which].public];
  const publicPath = process.env[keyEnvPaths[which].publicPath];

  let pubKey = publicLiteral ?? (publicPath ? fs.readFileSync(publicPath, 'utf-8') : undefined);
  if (pubKey) {
    pubKey = pubKey.replace(/\\n/g, '\n');
  }

  const algo = jwtAlgo(process.env[keyEnvPaths[which].algo]);

  return {
    privKey,
    pubKey,
    algo,
  };
};

const { privKey: primaryPrivKey, pubKey: primaryPubKey, algo } = prepareKeys("primary");
const { pubKey: secondaryPubKey } = prepareKeys("secondary");

if (!primaryPrivKey) throw new Error('Missing JWT signing private key');
if (!primaryPubKey) throw new Error('Missing JWT signing public key');

export const JWT_PUBLIC_KEY = primaryPubKey;
export const JWT_2ND_PUBLIC_KEY = secondaryPubKey;

export function createJWT(
  payload: Record<string, any>,
  options?: Omit<JWT.SignOptions, 'algorithm'>,
) {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return JWT.sign(payload, primaryPrivKey!, { algorithm: algo, ...options });
}
export function verifyJWT(token: string) {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  try {
    return JWT.verify(token, primaryPubKey!);
  } catch (err) {
    if (secondaryPubKey) {
      return JWT.verify(token, secondaryPubKey);
    } else {
      throw err;
    }
  }
}
