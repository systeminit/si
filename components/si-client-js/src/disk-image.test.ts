import { DiskImage } from "@/disk-image";

test('Disk Image Create', async () => {
  let d = await DiskImage.create({ constraints: { "format": "ami" } });
  expect(d.name).toBe("AWS us-west-1 Ubuntu 18.04.03 hvm-instance");
  expect(d.format).toBe("ami");
});

