/**
 * Policy Evaluation Module - Main orchestrator for policy evaluation
 *
 * This module coordinates the policy evaluation process:
 * 1. Extract policy structure from markdown
 * 2. Collect source data from System Initiative
 * 3. Evaluate policy compliance using Claude
 * 4. Generate markdown report
 * 5. Upload results (coming soon)
 *
 * All files are written directly to the output folder (timestamped or custom-named)
 * to avoid unnecessary file operations.
 *
 * @module
 */

import { PolicyReportsApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import { resolveChangeSet } from "../change-set/utils.ts";
import { resolve } from "@std/path";

/**
 * Options for the policy evaluate command
 */
export interface PolicyEvaluateOptions {
  /** Name for the policy evaluation (required) */
  name: string;
  /** Folder name to organize results (defaults to timestamp) */
  outputFolder?: string;
  /** Change set ID or name (defaults to HEAD) */
  changeSet?: string;
  /** Skip uploading the policy evaluation results */
  noUpload?: boolean;
}

/**
 * Evaluates a policy file through the policy evaluation process.
 *
 * @param policyFilePath - Path to the policy markdown file
 * @param options - Command options including output folder, change set, and upload flag
 */
export async function callPolicyEvaluate(
  policyFilePath: string,
  options: PolicyEvaluateOptions,
): Promise<void> {
  // Validate name before doing anything else
  if (!options.name || options.name.trim() === "") {
    throw new Error("Policy name is required. Please provide a name using the --name option.");
  }

  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

  try {
    ctx.logger.info("Starting policy evaluation for: {path}", {
      path: policyFilePath,
    });

    // Determine output directory and folder name
    const baseName = policyFilePath.split("/").pop()?.replace(/\.md$/, "") ||
      "policy";

    // Determine final output path:
    // - If outputFolder is specified, use it in the current directory
    // - Otherwise, create a timestamp folder in the current directory
    // Always resolve to absolute paths to avoid working directory issues
    const folderName = options.outputFolder ||
      new Date().toISOString().split(".")[0] + "Z";
    const finalOutputPath = resolve(Deno.cwd(), folderName);

    // Create output folder upfront
    await Deno.mkdir(finalOutputPath, { recursive: true });
    ctx.logger.debug("Created output folder: {path}", {
      path: finalOutputPath,
    });

    // Create output paths - write directly to final folder
    const extractedPolicyPath = `${finalOutputPath}/${baseName}-extracted.json`;
    const sourceDataPath = `${finalOutputPath}/${baseName}-source-data.json`;
    const evaluationPath = `${finalOutputPath}/${baseName}-evaluation.json`;
    const reportPath = `${finalOutputPath}/report.md`;

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
    await generateReport(
      extractedPolicy,
      sourceData,
      evaluation,
      workspaceId,
      changeSetId,
      reportPath,
    );

    // All files have been written to the output folder
    ctx.logger.info("Files organized in: {folder}", {
      folder: finalOutputPath,
    });

    // Stage 5: Upload policy evaluation results
    ctx.logger.info("Stage 5: Uploading policy evaluation results...");
    if (options.noUpload) {
      ctx.logger.info("Skipping upload as per user request");
    } else {
      ctx.logger.info("Uploading coming soon");

      // Read the generated report
      const report = await Deno.readTextFile(reportPath);

      const policyReportsApi = new PolicyReportsApi(apiConfig);

      const resp = await policyReportsApi.uploadPolicyReport({
        workspaceId,
        changeSetId,
        uploadPolicyReportV1Request: {
          name: options.name,
          policy: policyContent,
          report: report,
          result: evaluation.result,
        },
      });

      const data = { id: resp.data.id };

      console.log(`Policy Uploaded: ${data.id}`);
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

    ctx.logger.info("Report: {path}", { path: reportPath });

    ctx.analytics.trackEvent("policy evaluate", {
      result: evaluation.result,
    });
  } catch (error) {
    ctx.logger.error("Policy evaluation failed: {error}", {
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}
