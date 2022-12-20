export interface Qualification {
  title: string;
  link?: string;
  description?: string;
  result?: QualificationResult | null; // FIXME(victor) Results returning null could be a backend bug
  output?: Array<QualificationOutputStream>;
  prototypeId?: string; // The validations qualification doesn't need a prototype, but it can't be edited
}

export interface QualificationResult {
  title?: string;
  link?: string;
  status: "success" | "warning" | "failure" | "unknown";
  sub_checks: Array<{
    status: "success" | "warning" | "failure" | "unknown";
    description: string;
  }>;
}

export interface QualificationOutputStream {
  line: string;
  stream: string;
  level: string;
}
