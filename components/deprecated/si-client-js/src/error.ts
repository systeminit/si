//class InvalidArgumentError extends Error {
//    constructor(message?: string) {
//        super(message);
//        // see: typescriptlang.org/docs/handbook/release-notes/typescript-2-2.html
//        this.name = InvalidArgumentError.name; // stack traces display correctly now 
//    }
//}

export class LookupError extends Error {
  constructor(message?: string) {
    super(message); 
    Object.setPrototypeOf(this, new.target.prototype);
    this.name = LookupError.name; 
  }
}

