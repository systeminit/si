/**
 * AI agent for System Initiative that automates AWS infrastructure provisioning
 * by interacting with CloudFormation schemas via the AWS Cloud Control API.
 *
 * This module provides functionality for extracting AWS resource types,
 * extracting field values from CloudFormation schemas, editing existing AWS
 * components, and prototyping infrastructure based on natural language requests.
 *
 * @module
 */

import {
  checkPropertyCaseMismatches,
  editComponent,
} from "./src/editComponent.ts";
import { extractTypes } from "./src/extractTypes.ts";
import { prototypeInfrastructure } from "./src/prototypeInfrastructure.ts";
import { extractFields } from "./src/extractFields.ts";

export type {
  ExtractedField,
  ExtractFieldsResponse,
} from "./src/extractFields.ts";

export {
  /**
   * Checks if a component's domain properties have case mismatches compared to CloudFormation schema.
   * AWS CloudFormation is case-sensitive, so this helps prevent deployment errors.
   */
  checkPropertyCaseMismatches,
  /**
   * Edits an existing System Initiative component based on natural language instructions.
   * Processes the request against CloudFormation schemas to update AWS resource properties.
   */
  editComponent,
  /**
   * Extracts fields from CloudFormation schemas based on natural language request.
   * Parses AWS resource schemas to populate field values based on user requirements.
   */
  extractFields,
  /**
   * Extracts relevant CloudFormation resource types based on a natural language request.
   * Identifies appropriate AWS resource types needed for the described infrastructure.
   */
  extractTypes,
  /**
   * Builds a prototype of infrastructure with System Initiative based on natural language description.
   * Creates properly configured AWS resource components with CloudFormation properties.
   */
  prototypeInfrastructure,
};
