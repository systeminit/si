// validators/funcValidators.ts
export type ValidationIssue = { message: string };

export function validateFunctionCode(
  functionType: "qualification" | "codegen" | "management" | "action",
  code?: string,
  actionKind?: "Create" | "Destroy" | "Refresh" | "Update" | "Manual",
): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  if (!code) return issues; // nothing to validate (you'll use defaults below)

  // Very light, robust checks that won't require executing code:
  const hasMain = /(?:async\s+)?function\s+main\s*\(/.test(code);
  if (!hasMain) {
    issues.push({ message: "Missing `function main(...)` entrypoint." });
    return issues;
  }

  // Parameter heuristics by type (cheap, resilient):
  const paramSig = code.match(/function\s+main\s*\(([^)]*)\)/)?.[1] ?? "";
  const hasInputParam = /\b(component|thisComponent|.*: ?Input)\b/.test(paramSig);

  switch (functionType) {
    case "qualification": {
      if (!hasInputParam) issues.push({ message: "Qualification main(component: Input) missing or malformed." });
      if (!/\bresult\b/.test(code) || !/\bmessage\b/.test(code)) {
        issues.push({ message: "Qualification must eventually return { result, message }." });
      }
      break;
    }
    case "codegen": {
      if (!hasInputParam) issues.push({ message: "Codegen main(component: Input) missing or malformed." });
      if (!/\bformat\b/.test(code) || !/\bcode\b/.test(code)) {
        issues.push({ message: "Codegen must return { format, code }." });
      }
      break;
    }
    case "management": {
      // Typically needs { thisComponent, components } in Input and returns Output
      if (!/\{[^}]*thisComponent[^}]*\}/s.test(paramSig)) {
        issues.push({ message: "Management main({ thisComponent, ... }: Input) is recommended." });
      }
      break;
    }
    case "action": {
      if (!hasInputParam) issues.push({ message: "Action main(component: Input) missing or malformed." });

      if (actionKind === "Create") {
        // Create actions should use create/post operations, not get/describe
        if (/\b(get-resource|describe-)\w+/i.test(code)) {
          issues.push({ message: "Create action appears to use get/describe operations instead of create operations." });
        }
        // Create should return resourceId on success
        if (!/\bresourceId\b/.test(code)) {
          issues.push({ message: "Create action should return { resourceId, status } on success." });
        }
      } else if (actionKind === "Destroy") {
        // Destroy actions should use delete operations, not create/get
        if (/\b(create-resource)\b/i.test(code)) {
          issues.push({ message: "Destroy action appears to use create operations instead of delete operations." });
        }
        // Destroy should return null payload on success
        if (!/payload:\s*null/.test(code)) {
          issues.push({ message: "Destroy action should return { payload: null, status } on success." });
        }
      } else if (actionKind === "Refresh") {
        // Refresh should use get/describe operations
        if (/\b(create-resource|delete-resource|update-resource)\b/i.test(code)) {
          issues.push({ message: "Refresh action should only read/describe resources, not modify them." });
        }
        // Refresh should return payload
        if (!/\bpayload\b/.test(code)) {
          issues.push({ message: "Refresh action should return { payload, status }." });
        }
      } else if (actionKind === "Update") {
        // Update should use update/patch operations
        if (/\bcreate-resource\b/i.test(code)) {
          issues.push({ message: "Update action appears to use create operations instead of update operations." });
        }
        // Update should check for existing resource
        if (!/resource.*payload/.test(code)) {
          issues.push({ message: "Update action should verify resource exists before updating." });
        }
      }
      // No specific validations for Manual functions since they can be very diverse!
      break;
    }
  }

  return issues;
}
