export interface Qualification {
  title: string;
  link?: string;
  description?: string;
  result?: QualificationResult;
  output?: Array<QualificationOutputStream>;
}

export interface QualificationResult {
  title?: string;
  link?: string;
  success: boolean;
  sub_checks: Array<{
    status: "Success" | "Failure" | "Unknown";
    description: string;
  }>;
}

export interface QualificationOutputStream {
  line: string;
  stream: string;
  level: string;
}
