import { Cpu } from "@/cpu";

test('Cpu findComponent', async () => {
  let c = await Cpu.findComponent({name: "Intel Xeon 8175"});
  expect(c.length).toBe(1);
  expect(c[0].manufacturer).toBe("Intel");
});

test('CPU find when nothing matches', async () => {
  let c = await Cpu.findComponent({serviceName: "chaos"});
  expect(c.length).toBe(0);
});

test('CPU entity creation', async () => {
  const c = await Cpu.create({ constraints: { "name": "Intel Xeon 8175" }});
  expect(c.name).toBe("Intel Xeon 8175");
});

