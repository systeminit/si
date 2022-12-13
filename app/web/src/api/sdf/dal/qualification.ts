export interface QualificationPrototype {
  id: string;
  title: string;
  link?: string;
  description?: string;
  isComponentSpecific: boolean;
}

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
