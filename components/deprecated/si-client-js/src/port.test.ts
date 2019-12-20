import { Port } from "@/port";

test('Port findComponent', async () => {
  let p = await Port.findComponent({serviceName: "http", protocol: "tcp"});
  expect(p.length).toBe(1);
  expect(p[0].serviceName).toBe("http");
});

test('Port find when nothing matches', async () => {
  let p = await Port.findComponent({serviceName: "chaos"});
  expect(p.length).toBe(0);
});

test('Port find Complex', async () => {
  let p = await Port.findComponent({_join: "OR", terms: [{_join: "AND", serviceName: "http", protocol: "tcp"}, { "integration.name": "poop" } ]});
  expect(p.length).toBe(1);
  let q = await Port.findComponent({_join: "AND", terms: [{_join: "AND", serviceName: "http", protocol: "tcp"}, { "integration.name": "poop" } ]});
  expect(q.length).toBe(0);
});

test('Port entity creation', async () => {
  const p = await Port.create({ constraints: { serviceName: "http", protocol: "tcp"}, args: { name: "Poop Service http" }});
  expect(p.name).toBe("Poop Service http");
  expect(p.serviceName).toBe("http");
  expect(p.protocol).toBe("tcp");
  console.log(p);
});

// You take a list of constraints to apply in selecting a component, and a list of 
// entities that can be used by the resulting component to fulfill your intent=

// A list of contstraints to look up a component, and a list of entities to fulful promises
//
//Server.create({ memoryGIB: { "gte": "8" }}, [SshKey.find({"name": "adminkey"})]);
//OperatingSystem.find({ "platform": "rhel" });
//Server.create();
