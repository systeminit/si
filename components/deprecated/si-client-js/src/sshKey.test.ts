import { SshKey } from "@/ssh-key";

test('SSH Key Create Default', async () => {
  // Create a "default" key - RSA, 2048 bits
  let defaultKey = await SshKey.create();
 
  expect(defaultKey.keyType).toBe("RSA");
});

test('SSH Key Create DSA', async () => {
  let defaultKey = await SshKey.create({ constraints: { keyType: "DSA", keyFormat: "RFC4716" } });
 
  expect(defaultKey.keyType).toBe("DSA");
  expect(defaultKey.bits).toBe("1024");
});

test('SSH Key Create Args', async () => {
  let defaultKey = await SshKey.create({ args: { name: "Opeth" } });
 
  expect(defaultKey.keyType).toBe("RSA");
  expect(defaultKey.name).toBe("Opeth");
  expect(defaultKey.comment).toBe("Opeth");
});

test('SSH Key Create From Key', async () => {
  let defaultKey = await SshKey.create({ args: { name: "Opeth" } });
 
  expect(defaultKey.keyType).toBe("RSA");
  expect(defaultKey.name).toBe("Opeth");
  expect(defaultKey.comment).toBe("Opeth");
});

// SshKey.find({ "keyType": "RSA" }).replace({ constraints: { keyType: "ed25519" } });
    

  // Create a DSA key, with 1024 bits per FIPS
  //let e = await SshKey.create({ constraints: { keyType: "dsa" } });
  //// Create an Ed25519 key, with a fixed length
  //let e = await SshKey.create({ constraints: { keyType: "ed25519" } });
  //// Create an RSA key with more bits and no other defaults?
  //let e = await SshKey.create({ constraints: { keyType: "rsa", keyBits: 1024 }, args: { comment: "foo@bar" }});
  //// You could also say something like this, with a non-standard amount of keyBits
  //let e = await SshKey.create({ constraints: { keyType: "rsa" }, args: { comment: "foo@bar", bits: 2098 }});

  //let e = await SshKey.create({ constraints: { keyType: "rsa" }, args: { publicKey: .., privateKey: .. }});
  //let e = await SshKey.create({ args: { 
  //  comment: "foo@bar", 
  //  bits: 2098,
  //  publicKey: ...,
  //  privateKey: ..., 
  //}});

  //// What would you do here? disassocaite it? delete it? keep it? warn with a list? fuck
  //e.delete();
  //e.setComment("foo");
  //e.changePassphrase("asdfasdfasdfasld;fja;sldkja;slkdj;lkj");
  //e.publicKey;

  //// 
  //Server.find({ name: "poop", }).sshKey.publicKey;
