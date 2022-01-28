export interface Qualification {
  name: string;
  title?: string;
  link?: string;
  description?: string;
  result?: QualificationResult;
}

export interface QualificationResult {
  title?: string;
  link?: string;
  sub_checks?: Array<{
    status: "Success" | "Failure" | "Unknown";
    description: string;
  }>;
  output?: Array<string>;
  success: boolean;
}

export interface QualificationError {
  message: string;
}
