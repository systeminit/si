/**
 * Stage 1: Policy Extraction
 * Uses Claude agent to extract structured data from policy markdown
 */

import which from "which";
import { query } from '@anthropic-ai/claude-agent-sdk';
import * as fs from 'node:fs';
import * as path from 'node:path';

export interface ExtractedPolicy {
  policyTitle: string;
  policyText: string; // Full policy text including exceptions
  sourceDataQueries: Record<string, string>; // e.g., { "all-aws-components": "schema:AWS*" }
  outputTags: string[];
}


const EXTRACTION_SYSTEM_PROMPT = `You are a policy document parser. Your task is to extract structured information from a compliance policy document written in markdown.

Extract the following information:

1. **Policy Title**: The main title of the document (from the # heading)
2. **Policy Text**: The full policy description including any exceptions (everything under "## Policy" and "### Exceptions" sections)
3. **Source Data Queries**: Parse the YAML block under "## Source Data" -> "### System Initiative" section
   - Extract key-value pairs where the key is the query name and the value is the query string
   - Example: { "all-aws-components": "schema:AWS*", "frenchy-pants": "schema:AWS*" }
4. **Output Tags**: Parse the YAML block under "## Output Tags" section
   - Extract the array of tags from the "tags" field
   - Example: ["foo", "bar"]

Output your findings as a JSON object with this exact structure:
{
  "policyTitle": "string",
  "policyText": "string (full policy text with exceptions)",
  "sourceDataQueries": { "query-name": "query-string", ... },
  "outputTags": ["tag1", "tag2", ...]
}`;

/**
 * Extract structured policy data from markdown using Claude
 */
export async function extractPolicy(policyContent: string, outputPath: string): Promise<ExtractedPolicy> {
  console.log('Stage 1: Extracting policy structure...');

  const pathToClaudeCodeExecutable = await which("claude");

  const cwd = path.dirname(outputPath);
  const queryInstance = query({
    prompt: `Please parse the following compliance policy document and extract the structured information as specified in your system prompt.

Policy Document:
---
${policyContent}
---

Please output your findings as a JSON object and write it to the file: ${outputPath}`,
    options: {
      systemPrompt: EXTRACTION_SYSTEM_PROMPT,
      allowedTools: ['Write'],
      permissionMode: 'bypassPermissions',
      cwd,
      pathToClaudeCodeExecutable,
    }
  });

  // Stream the agent's work
  for await (const message of queryInstance) {

    if (message.type === 'assistant') {
      // Print assistant messages for visibility
      for (const block of message.message.content) {
        if (block.type === 'text') {
          console.log(block.text);
        }
      }
    } else if (message.type === 'result') {
      if (message.subtype === 'success') {
        console.log(`✓ Policy extraction complete`);

        // Read the extracted policy from file
        if (!fs.existsSync(outputPath)) {
          throw new Error(`Extraction failed: output file not found at ${outputPath}`);
        }

        const extractedJson = fs.readFileSync(outputPath, 'utf-8');
        const extracted: ExtractedPolicy = JSON.parse(extractedJson);

        // Validate the structure
        if (!extracted.policyTitle || !extracted.policyText || !extracted.sourceDataQueries) {
          throw new Error('Invalid extraction result: missing required fields');
        }

        return extracted;
      } else {
        throw new Error(`Policy extraction failed: ${message.subtype}`);
      }
    }
  }

  throw new Error('Policy extraction failed: no result received');
}
