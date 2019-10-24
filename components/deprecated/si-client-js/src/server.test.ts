import { Server } from "@/server";

test('Server Create Default', async () => {
  let e = await Server.create();
  expect(e.operatingSystem.platform).toBe("ubuntu");
  expect(e.sshKey.keyType).toBe("RSA");
  expect(e.diskImage.format).toBe("ami");
  expect(e.cpu.cores).toBe(4);
});

test('Server create with constraints and options', async () => {
  let e = await Server.create({
    "name": "Starsky",
    "memoryGIB": 4000,
    "cpu": { "cores": ["gte", 4] },
    "operatingSystem": { "platform": "ubuntu" },
    "sshKey": { "keyType": "DSA" },
    layer: ...
  });
});

//test('Change the SSH Key', async () => {
//  let originalKey = await Server.find({ "name": "Starsky" }).sshKey.privateKey;
//  let e = await Server.find({ "name": "Starsky" }).sshKey.regenerate();
//  expect(originalKey).not.toBe(e.privateKey);
//});

//test('Update all ubuntu servers in AWS', async () => {
//  let e = await OperatingSystem.find({ "platform": "ubuntu", "server.integration.name": "AWS" }).map(|s| s.update());
//  let e = await Server.find({ "integration.name": "AWS", "operatingSystem.platform": "ubuntu" }).map(|s| s.operatingSystem.update());kkk
//  //let e = await Server.create({
//  //  constraints: {
//  //    "cpu.cores": 4,
//  //    "memoryGIB": 4000,
//  //    "operatingSystem.platform": "ubuntu"
//  //  },
//  //  args: {
//  //    "name": "Starsky"
//  //  }
//  //});
//});


