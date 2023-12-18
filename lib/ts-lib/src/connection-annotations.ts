import * as _ from "lodash-es";

export function parseConnectionAnnotation(annotation: string): string[] {
  let token = annotation;
  const typeArray = [];

  do {
    const [_, newAnnotation, tail] = token.match(/^([\w ]+)(?:<(.+)>)?$/) ?? [];

    if (!newAnnotation) {
      throw new Error(`Couldn't parse connection annotation "${annotation}"`);
    }

    typeArray.push(newAnnotation.toLowerCase().trim());

    if (!tail) break;

    token = tail;
  } while (token != null);

  return typeArray;
}

export function connectionAnnotationFitsReference(targetCa: string[], referenceCa: string[]) {
  // a fitting target annotation is either the same as the reference one or a supertype thereof

  return targetCa.length >= referenceCa.length
    && _.isEqual(targetCa.slice(-referenceCa.length), referenceCa);
}
