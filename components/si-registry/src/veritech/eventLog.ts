import WebSocket from "ws";

export enum EventLogLevel {
  Trace = "trace",
  Debug = "debug",
  Info = "info",
  Warn = "warn",
  Error = "error",
  Fatal = "fatal",
}

class FakeIdGenerator {
  fakeId: number;

  constructor() {
    this.fakeId = 0;
  }

  nextId(): number {
    this.fakeId = this.fakeId + 1;
    return this.fakeId;
  }
}

export class Event {
  fakeIdGenerator: FakeIdGenerator;
  ws: WebSocket;

  constructor(ws: WebSocket) {
    this.fakeIdGenerator = new FakeIdGenerator();
    this.ws = ws;
  }

  log(
    level: EventLog["level"],
    message: EventLog["message"],
    payload: EventLog["payload"],
  ): EventLog {
    const eventLog = new EventLog({
      ws: this.ws,
      fakeId: this.fakeIdGenerator.nextId(),
      level,
      message,
      payload,
    });
    eventLog.save();
    return eventLog;
  }
}

export interface EventLogConstructor {
  fakeId: EventLog["fakeId"];
  ws: WebSocket;
  level: EventLog["level"];
  message: EventLog["message"];
  payload: EventLog["payload"];
}

export class EventLog {
  fakeId: number;
  fakeIdGenerator: FakeIdGenerator;
  level: EventLogLevel;
  message: string;
  payload: Record<string, any>;
  ws: WebSocket;

  constructor({ ws, fakeId, level, message, payload }: EventLogConstructor) {
    this.fakeIdGenerator = new FakeIdGenerator();
    this.fakeId = fakeId;
    this.level = level;
    this.message = message;
    this.payload = payload;
    this.ws = ws;
  }

  save(): void {
    this.ws.send(
      JSON.stringify({
        log: {
          fakeId: this.fakeId,
          level: this.level,
          message: this.message,
          payload: this.payload,
        },
      }),
    );
  }

  fatal(): void {
    this.level = EventLogLevel.Fatal;
    this.save();
  }

  output(stream: OutputLine["stream"], line: OutputLine["line"]): void {
    const ol = new OutputLine({
      eventLogId: this.fakeId,
      ws: this.ws,
      fakeId: this.fakeIdGenerator.nextId(),
      stream,
      line,
    });
    ol.save();
  }
}

export interface OutputLineConstructor {
  fakeId: OutputLine["fakeId"];
  eventLogId: EventLog["fakeId"];
  stream: OutputLine["stream"];
  line: OutputLine["line"];
  ws: WebSocket;
}

export class OutputLine {
  fakeId: number;
  eventLogId: number;
  stream: "stdout" | "stderr" | "all";
  line: string;
  ws: WebSocket;

  constructor({ ws, eventLogId, fakeId, stream, line }: OutputLineConstructor) {
    this.fakeId = fakeId;
    this.eventLogId = eventLogId;
    this.ws = ws;
    this.stream = stream;
    this.line = line;
  }

  save(): void {
    this.ws.send(
      JSON.stringify({
        outputLine: {
          fakeId: this.fakeId,
          eventLogId: this.eventLogId,
          stream: this.stream,
          line: this.line,
        },
      }),
    );
  }
}
