Precondition

- Every workspace has a pub/priv key pair
- Browser downloads pubkey for your workspace
- Private key in the keypair is also stored encrypted in sdf
- Once it's in the system as encrypted data, no one else can pull it out

Flow

> Click on "Add Key" button in the modal

- In the browser the secret is encrypted
- That encrypted secret is sent to the backend
- Double encryption for storage in the database with a separate, per instance,
  secret key

> In action (action func, qual func, whatever..)

- Check if component uses secret
- Search for before/auth function for the component thats affected
- Load in the id of the secret
- Ask for the unecrypted value
  - Decrypt the corresponding private key with the secret key
  - Decrypt the corresponding value with the secret key
  - Decrypt the value again using the decrypted private key
- That corresponding value is passed into the execution of the function

> Rotation support

- Is there, will delve into later..

> Some brute force capabilities available, simple highlight

```typescript
var poop = "poop";
const getVar = requestStorage.getEnv("POOP_VALUE"); // poop

console.log(poop); // redacted
```
