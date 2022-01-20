export interface Qualification {
  name: string;
  title?: string;
  link?: string;
  description?: string;
  result?: QualificationResult;
}

export interface QualificationResult {
  errors: Array<QualificationError>;
  success: boolean;
}

export interface QualificationError {
  message: string;
}
