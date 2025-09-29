async function main(input: Input): Promise<Output> {
  const tgArn = input.targetGroupArn;
  const type = input.type;
  if (tgArn && type === "forward") {
    return [
      { TargetGroupArn: tgArn },
    ];
  }
  return [];
}
