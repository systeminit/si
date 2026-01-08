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
  /** Name for the policy evaluation (required unless --all is used) */
  name?: string;
  /** Evaluate all policy files in a directory */
  all?: boolean;
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
  // Check if path is a file or directory
  let pathStat: Deno.FileInfo;
  try {
    pathStat = await Deno.stat(policyFilePath);
  } catch (_error) {
    throw new Error(`Path does not exist: ${policyFilePath}`);
  }

  const isDirectory = pathStat.isDirectory;
  const isFile = pathStat.isFile;

  // Validate options based on path type and flags
  if (options.all && isDirectory) {
    // --all with directory: evaluate all .md files in the directory
    return await evaluateAllPoliciesInDirectory(policyFilePath, options);
  } else if (options.all && isFile) {
    // --all with file: ignore the flag and proceed with single file evaluation
    if (!options.name || options.name.trim() === "") {
      throw new Error(
        "When evaluating a single file, the --name option is required. The --all flag only applies to directories.",
      );
    }
    return await evaluateSinglePolicy(policyFilePath, options.name, options);
  } else if (!options.all && isDirectory) {
    // No --all with directory: error
    throw new Error(
      `The path "${policyFilePath}" is a directory. Please use the --all flag to evaluate all policy files in the directory.`,
    );
  } else if (!options.all && isFile) {
    // No --all with file: require name
    if (!options.name || options.name.trim() === "") {
      throw new Error(
        "Policy name is required. Please provide a name using the --name option.",
      );
    }
    return await evaluateSinglePolicy(policyFilePath, options.name, options);
  } else {
    throw new Error(`Invalid path: ${policyFilePath}`);
  }
}

/**
 * Evaluates all policy files in a directory
 */
async function evaluateAllPoliciesInDirectory(
  directoryPath: string,
  options: PolicyEvaluateOptions,
): Promise<void> {
  const ctx = Context.instance();

  // Find all .md files in the directory
  const policyFiles: string[] = [];
  for await (const entry of Deno.readDir(directoryPath)) {
    if (entry.isFile && entry.name.endsWith(".md")) {
      policyFiles.push(`${directoryPath}/${entry.name}`);
    }
  }

  if (policyFiles.length === 0) {
    throw new Error(
      `No policy files (.md) found in directory: ${directoryPath}`,
    );
  }

  ctx.logger.info("Found {count} policy file(s) to evaluate", {
    count: policyFiles.length,
  });

  // Evaluate each policy file
  for (const policyFile of policyFiles) {
    const fileName = policyFile.split("/").pop()!;
    const policyName = fileName.replace(/\.md$/, "");

    ctx.logger.info("=== Evaluating policy: {name} ===", { name: policyName });

    try {
      await evaluateSinglePolicy(policyFile, policyName, options);
    } catch (error) {
      ctx.logger.error("Failed to evaluate {file}: {error}", {
        file: fileName,
        error: error instanceof Error ? error.message : String(error),
      });
      // Continue with next file instead of stopping
    }
  }

  ctx.logger.info("=== Completed evaluating all policies ===");
}

/**
 * Creates output directory and returns all output file paths
 */
async function createOutputPaths(
  policyFilePath: string,
  outputFolder?: string,
): Promise<{
  finalOutputPath: string;
  baseName: string;
  extractedPolicyPath: string;
  sourceDataPath: string;
  evaluationPath: string;
  reportPath: string;
}> {
  const ctx = Context.instance();

  const baseName = policyFilePath.split("/").pop()?.replace(/\.md$/, "") ||
    "policy";

  const folderName = outputFolder ||
    new Date().toISOString().split(".")[0] + "Z";
  const finalOutputPath = resolve(Deno.cwd(), folderName);

  await Deno.mkdir(finalOutputPath, { recursive: true });
  ctx.logger.debug("Created output folder: {path}", { path: finalOutputPath });

  return {
    finalOutputPath,
    baseName,
    extractedPolicyPath: `${finalOutputPath}/${baseName}-extracted.json`,
    sourceDataPath: `${finalOutputPath}/${baseName}-source-data.json`,
    evaluationPath: `${finalOutputPath}/${baseName}-evaluation.json`,
    reportPath: `${finalOutputPath}/report.md`,
  };
}

/**
 * Uploads policy evaluation results to the API
 */
async function uploadPolicyResults(
  policyName: string,
  policyContent: string,
  reportPath: string,
  evaluation: { result: string },
  workspaceId: string,
  changeSetId: string,
): Promise<void> {
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();

  ctx.logger.info("Stage 5: Uploading policy evaluation results...");

  const report = await Deno.readTextFile(reportPath);
  const policyReportsApi = new PolicyReportsApi(apiConfig);

  const resp = await policyReportsApi.uploadPolicyReport({
    workspaceId,
    changeSetId,
    uploadPolicyReportV1Request: {
      name: policyName,
      policy: policyContent,
      report: report,
      result: evaluation.result,
    },
  });

  ctx.logger.info("Policy uploaded: {id}", { id: resp.data.id });
}

/**
 * Displays the evaluation summary
 */
function displayEvaluationSummary(
  evaluation: {
    result: string;
    failingComponents: unknown[];
    summary: string;
  },
  reportPath: string,
): void {
  const ctx = Context.instance();

  ctx.logger.info("Policy Evaluation Complete");

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
}

/**
 * Evaluates a single policy file
 */
async function evaluateSinglePolicy(
  policyFilePath: string,
  policyName: string,
  options: PolicyEvaluateOptions,
): Promise<void> {
  const ctx = Context.instance();
  const workspaceId = Context.workspaceId();

  try {
    ctx.logger.info("Starting policy evaluation for: {path}", {
      path: policyFilePath,
    });

    // Setup output paths
    const paths = await createOutputPaths(policyFilePath, options.outputFolder);

    // Read policy file
    ctx.logger.debug("Reading policy file...");
    const policyContent = await Deno.readTextFile(policyFilePath);

    // Stage 1: Extract policy structure
    const { extractPolicy } = await import("./extract_policy.ts");
    const extractedPolicy = await extractPolicy(
      policyContent,
      paths.extractedPolicyPath,
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
      paths.sourceDataPath,
    );

    // Stage 3: Evaluate policy
    const { evaluatePolicy } = await import("./evaluate_policy.ts");
    const evaluation = await evaluatePolicy(
      extractedPolicy.policyText,
      sourceData,
      workspaceId,
      changeSetId,
      paths.sourceDataPath,
      paths.evaluationPath,
    );

    // Stage 4: Generate report
    const { generateReport } = await import("./generate_report.ts");
    await generateReport(
      extractedPolicy,
      sourceData,
      evaluation,
      workspaceId,
      changeSetId,
      paths.reportPath,
    );

    ctx.logger.info("Files organized in: {folder}", {
      folder: paths.finalOutputPath,
    });

    // Stage 5: Upload policy evaluation results
    if (options.noUpload) {
      ctx.logger.info("Skipping upload as per user request");
    } else {
      await uploadPolicyResults(
        policyName,
        policyContent,
        paths.reportPath,
        evaluation,
        workspaceId,
        changeSetId,
      );
    }

    // Display summary
    displayEvaluationSummary(evaluation, paths.reportPath);

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
