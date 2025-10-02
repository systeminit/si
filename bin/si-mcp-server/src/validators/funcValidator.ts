// validators/funcValidators.ts
export type ValidationIssue = { message: string };

export function validateFunctionCode(
  functionType: "qualification" | "codegen" | "management" | "action",
  code: string | undefined,
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
      break;
    }
  }

  return issues;
}
