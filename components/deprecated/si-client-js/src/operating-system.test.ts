import { OperatingSystem } from "@/operating-system";

test('Operating System Create Default', async () => {
  // Create a "default" operating system - Ubuntu 18.04
  let d = await OperatingSystem.create();
 
  expect(d.platform).toBe("ubuntu");
  expect(d.platformVersion).toBe("18.04");
});

//test('SSH Key Create DSA', async () => {
//  let defaultKey = await SshKey.create({ constraints: { keyType: "DSA", keyFormat: "RFC4716" } });
// 
//  expect(defaultKey.keyType).toBe("DSA");
//  expect(defaultKey.bits).toBe("1024");
//});
//
//test('SSH Key Create Args', async () => {
//  let defaultKey = await SshKey.create({ args: { name: "Opeth" } });
// 
//  expect(defaultKey.keyType).toBe("RSA");
//  expect(defaultKey.name).toBe("Opeth");
//  expect(defaultKey.comment).toBe("Opeth");
//});
//
//test('SSH Key Create From Key', async () => {
//  let defaultKey = await SshKey.create({ args: { name: "Opeth" } });
// 
//  expect(defaultKey.keyType).toBe("RSA");
//  expect(defaultKey.name).toBe("Opeth");
//  expect(defaultKey.comment).toBe("Opeth");
//});
