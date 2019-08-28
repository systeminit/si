import { OperatingSystem } from "./operating-system";
import { DiskImage } from "./disk-image";
import { Cpu } from "./cpu";
import { Server } from "./server";

// All relationships have to be defined here. This is because Javascript
// modules are resolved at import time - which means if you do anything
// with a value other than initialize it, what you wind up with is
// "undefined" for the value.
//
// So rather than the kind of nice, clean fun times I had writing these
// from within my initialization in the modules themselves, all of them
// just wind up in here - to keep things nice and orderly, and avoid
// a whole raft of nasty undefined bugs.
//
// You're welcome.

DiskImage.hasOne({
  from: "operatingSystemId",
  to: {
    field: "operatingSystem",
    model: OperatingSystem,
  },
});

OperatingSystem.hasMany({
  from: {
    __typename: "DiskImageComponent",
    field: "operatingSystemId",
  },
  to: {
    field: "diskImages",
    model: DiskImage,
  },
});

Server.hasOne({ from: "cpuId", to: { field: "cpu", model: Cpu } });
