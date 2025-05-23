import PQueue from "p-queue";

export const mjolnirQueue = new PQueue({ concurrency: 10, autoStart: true });
