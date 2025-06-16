export const truncateString = (str: string, length: number) => {
  if (str.length <= length) return str;
  else {
    return `${str.trim().substring(0, length).trim()}...`;
  }
};
