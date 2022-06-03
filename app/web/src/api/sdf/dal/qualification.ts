export interface QualificationPrototype {
  id: number;
  title: string;
  link?: string;
  description?: string;
  isComponentSpecific: boolean;
}

export interface Qualification {
  title: string;
  link?: string;
  description?: string;
  result?: QualificationResult;
  output?: Array<QualificationOutputStream>;
  prototypeId?: number; // The validations qualification doesn't need a prototype, but it can't be edited
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
