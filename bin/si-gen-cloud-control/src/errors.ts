export class CommandFailed extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CommandFailed";
  }
}

export class ServiceMissing extends Error {
  constructor(serviceName: string) {
    super(
      `Attempt to find schema for service ${serviceName}, but it does not exist`,
    );
    this.name = "SchemaMissing";
  }
}
