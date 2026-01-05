/**
 * Stage 3: Policy Evaluation
 * Uses Claude agent to evaluate policy compliance against source data
 */

import { query } from '@anthropic-ai/claude-agent-sdk';
import * as fs from 'node:fs';
import * as path from 'node:path';
import type { SourceDataCollection } from './collect_source_data.ts';

export interface FailingComponent {
  componentId: string;
  schema: string;
  name: string;
  description: string;
  deeplink: string;
}

export interface RelevantColumn {
  attributePath: string;
  displayName: string;
}

export interface SourceDataMetadata {
  [queryName: string]: {
    relevantColumns: RelevantColumn[];
    reasoning: string;
  };
}

export interface EvaluationResult {
  result: 'Pass' | 'Fail';
  summary: string;
  failingComponents: FailingComponent[];
  sourceDataMetadata: SourceDataMetadata;
}


const EVALUATION_SYSTEM_PROMPT = `You are an infrastructure compliance evaluator. Your task is to evaluate whether infrastructure components comply with a given policy.

You will be provided with:
1. A compliance policy (with exceptions if applicable)
2. Source data containing infrastructure components organized by query name

Your job is to:
1. Review the policy and understand its requirements and exceptions
2. Examine each component in the source data
3. Determine if each component passes or fails the policy requirements
4. Apply any exceptions mentioned in the policy
5. Generate deep links to failing components using the format: https://app.systeminit.com/n/{workspaceId}/{changeSetId}/h/{componentId}/c

Output your evaluation as a JSON object with this exact structure:
{
  "result": "Pass" | "Fail",
  "summary": "Brief summary of the evaluation (2-3 sentences)",
  "failingComponents": [
    {
      "componentId": "component-id",
      "schema": "AWS::...",
      "name": "component-name",
      "description": "Why this component failed the policy check",
      "deeplink": "https://app.systeminit.com/n/{workspaceId}/{changeSetId}/h/{componentId}/c"
    }
  ],
  "sourceDataMetadata": {
    "query-name": {
      "relevantColumns": [
        {
          "attributePath": "/path/to/attribute",
          "displayName": "Column Name"
        }
      ],
      "reasoning": "Explanation of why these columns are relevant to the policy evaluation"
    }
  }
}

Notes:
- If ALL components pass (or no components exist), set result to "Pass" and failingComponents to []
- If ANY component fails, set result to "Fail" and include ALL failing components
- Be specific in the description field about why each component failed
- Apply exceptions carefully - if a component matches an exception, it should not be in failingComponents
- For sourceDataMetadata: identify which component attributes are most relevant to evaluating this policy
  - Include 2-5 relevant columns per query (the most important attributes for policy compliance)
  - Use clear, human-readable displayNames
  - Provide reasoning explaining why you selected those specific attributes`;

/**
 * Evaluate policy compliance using Claude
 */
export async function evaluatePolicy(
  policyText: string,
  sourceData: SourceDataCollection,
  workspaceId: string,
  changeSetId: string,
  sourceDataPath: string,
  outputPath: string
): Promise<EvaluationResult> {
  console.log('\nStage 3: Evaluating policy compliance...');

  // Count total components
  const totalComponents = Object.values(sourceData).reduce((sum, components) => sum + components.length, 0);
  console.log(`  Evaluating ${totalComponents} component(s)`);

  // If no source data, fail immediately
  if (totalComponents === 0) {
    console.log('  No components found - policy fails');
    const result: EvaluationResult = {
      result: 'Fail',
      summary: 'No infrastructure components found to evaluate against the policy. The policy cannot pass without any components to check.',
      failingComponents: [],
      sourceDataMetadata: {}
    };

    // Write the result to file
    fs.writeFileSync(outputPath, JSON.stringify(result, null, 2), 'utf-8');
    return result;
  }

  const queryInstance = query({
    prompt: `Please evaluate the infrastructure against the compliance policy.

Workspace ID: ${workspaceId}
Change Set ID: ${changeSetId}

Policy:
---
${policyText}
---

The source data is available in the file: ${sourceDataPath}
Please read this file to see all the components organized by query name.

After evaluating each component against the policy, write your findings as a JSON object to: ${outputPath}`,
    options: {
      systemPrompt: EVALUATION_SYSTEM_PROMPT,
      allowedTools: ['Write', 'Read'],
      permissionMode: 'bypassPermissions',
      cwd: path.dirname(outputPath)
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
        console.log(`✓ Policy evaluation complete`);

        // Read the evaluation result from file
        if (!fs.existsSync(outputPath)) {
          throw new Error(`Evaluation failed: output file not found at ${outputPath}`);
        }

        const evaluationJson = fs.readFileSync(outputPath, 'utf-8');
        const evaluation: EvaluationResult = JSON.parse(evaluationJson);

        // Validate the structure
        if (!evaluation.result || !evaluation.summary || !Array.isArray(evaluation.failingComponents)) {
          throw new Error('Invalid evaluation result: missing required fields');
        }

        // Ensure sourceDataMetadata exists
        if (!evaluation.sourceDataMetadata) {
          evaluation.sourceDataMetadata = {};
        }

        console.log(`  Result: ${evaluation.result}`);
        console.log(`  Failing components: ${evaluation.failingComponents.length}`);

        return evaluation;
      } else {
        throw new Error(`Policy evaluation failed: ${message.subtype}`);
      }
    }
  }

  throw new Error('Policy evaluation failed: no result received');
}
