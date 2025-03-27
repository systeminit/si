/**
 * AI agent for System Initiative that automates AWS infrastructure provisioning
 * by interacting with CloudFormation schemas via the AWS Cloud Control API.
 *
 * This module provides functionality for extracting field values from CloudFormation
 * schemas and editing existing AWS components based on natural language requests.
 *
 * @module
 */

import {
  checkPropertyCaseMismatches,
  editComponent,
  proposeEdits,
  extractionResponseToMarkdown,
} from "./src/editComponent.ts";
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
   * Proposes edits for a component based on natural language instructions.
   * Extracts field suggestions but doesn't actually perform the component update.
   */
  proposeEdits,
  /**
   * Extracts fields from CloudFormation schemas based on natural language request.
   * Parses AWS resource schemas to populate field values based on user requirements.
   */
  extractFields,
  /**
   * Translates an extract fields response to a Markdown document for easy
   * reading.
   */
  extractionResponseToMarkdown,
};
