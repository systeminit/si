/**
 * Error classes for the CloudFormation database module.
 * 
 * This module defines custom error types used throughout the CloudFormation
 * database implementation.
 * 
 * @module errors
 */

/**
 * Error thrown when a command fails to execute properly
 */
export class CommandFailed extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CommandFailed";
  }
}

/**
 * Error thrown when attempting to access a CloudFormation service that doesn't exist
 */
export class ServiceMissing extends Error {
  constructor(serviceName: string) {
    super(
      `Attempt to find schema for service ${serviceName}, but it does not exist`,
    );
    this.name = "SchemaMissing";
  }
}
