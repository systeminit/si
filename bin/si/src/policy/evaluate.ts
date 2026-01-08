/**
 * Policy Evaluation Module - Main orchestrator for policy evaluation
 *
 * This module coordinates the five-stage policy evaluation process:
 * 1. Extract policy structure from markdown
 * 2. Collect source data from System Initiative
 * 3. Evaluate policy compliance using Claude
 * 4. Generate markdown report
 * 5. Upload results (coming soon)
 *
 * @module
 */

import { Context } from "../context.ts";
import { resolveChangeSet } from "../change-set/utils.ts";

/**
 * Options for the policy evaluate command
 */
export interface PolicyEvaluateOptions {
  /** Output directory for evaluation results (defaults to current directory) */
  output?: string;
  /** Change set ID or name (defaults to HEAD) */
  changeSet?: string;
  /** Skip uploading the policy evaluation results */
  noUpload?: boolean;
}

/**
 * Evaluates a policy file through the five-stage process.
 *
 * @param policyFilePath - Path to the policy markdown file
 * @param options - Command options including output directory, change set, and upload flag
 */
export async function callPolicyEvaluate(
  policyFilePath: string,
  options: PolicyEvaluateOptions,
): Promise<void> {
  const ctx = Context.instance();
  const workspaceId = Context.workspaceId();

  try {
    ctx.logger.info("Starting policy evaluation for: {path}", {
      path: policyFilePath,
    });

    // Determine output directory
    const outputDir = options.output || Deno.cwd();
    const baseName = policyFilePath.split("/").pop()?.replace(/\.md$/, "") ||
      "policy";

    // Create output paths
    const extractedPolicyPath = `${outputDir}/${baseName}-extracted.json`;
    const sourceDataPath = `${outputDir}/${baseName}-source-data.json`;
    const evaluationPath = `${outputDir}/${baseName}-evaluation.json`;
    const reportPath = `${outputDir}/${baseName}-report.md`;

    // Read policy file
    ctx.logger.debug("Reading policy file...");
    const policyContent = await Deno.readTextFile(policyFilePath);

    // Stage 1: Extract policy structure
    const { extractPolicy } = await import("./extract_policy.ts");
    const extractedPolicy = await extractPolicy(
      policyContent,
      extractedPolicyPath,
    );
    ctx.logger.info("Policy extracted: {title}", {
      title: extractedPolicy.policyTitle,
    });

    // Resolve change set (use provided change set or default to HEAD)
    const { getHeadChangeSetId } = await import("../cli/helpers.ts");
    const changeSetId = options.changeSet
      ? await resolveChangeSet(workspaceId, options.changeSet)
      : await getHeadChangeSetId();

    // Stage 2: Collect source data
    const { collectSourceData } = await import("./collect_source_data.ts");
    const sourceData = await collectSourceData(
      changeSetId,
      extractedPolicy.sourceDataQueries,
      sourceDataPath,
    );

    // Stage 3: Evaluate policy
    const { evaluatePolicy } = await import("./evaluate_policy.ts");
    const evaluation = await evaluatePolicy(
      extractedPolicy.policyText,
      sourceData,
      workspaceId,
      changeSetId,
      sourceDataPath,
      evaluationPath,
    );

    // Stage 4: Generate report
    const { generateReport } = await import("./generate_report.ts");
    const reportName = await generateReport(
      extractedPolicy,
      sourceData,
      evaluation,
      workspaceId,
      changeSetId,
      reportPath,
    );

    // Stage 5: Upload policy evaluation results
    ctx.logger.info("Stage 5: Uploading policy evaluation results...");
    if (options.noUpload) {
      ctx.logger.info("Skipping upload as per user request");
    } else {
      ctx.logger.info("Uploading coming soon");
    }

    // Display summary
    ctx.logger.info("\nPolicy Evaluation Complete");

    // Use different log levels based on result for visual distinction
    if (evaluation.result === "Fail") {
      ctx.logger.warn("Result: FAIL");
      if (evaluation.failingComponents.length > 0) {
        ctx.logger.warn("Failing components: {count}", {
          count: evaluation.failingComponents.length,
        });
      } else {
        ctx.logger.warn("Reason: {summary}", { summary: evaluation.summary });
      }
    } else {
      ctx.logger.info("Result: PASS");
    }

    ctx.logger.info("Report: {path}", { path: reportName });

    // // Track analytics
    // ctx.analytics.trackEvent("policy evaluate", {
    //   policyTitle: extractedPolicy.policyTitle,
    //   result: evaluation.result,
    //   failingComponentsCount: evaluation.failingComponents.length,
    // });
  } catch (error) {
    ctx.logger.error("Policy evaluation failed: {error}", {
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}
