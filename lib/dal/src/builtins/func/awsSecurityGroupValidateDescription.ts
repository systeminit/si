async function validate(value: Input): Promise<Output> {
  const validSecurityGroupDescriptionRe =
    /^[a-zA-Z0-9. _\-:\/()#,@\[\]+=&;{}!$*]{1,256}$/;

  if ((value?.length ?? "") < 1) {
    return {
      valid: false,
      message: "Security Group description cannot be empty.",
    };
  }

  return {
    valid: validSecurityGroupDescriptionRe.test(value),
    message:
      "Security Group description must be no more than 256 characters and can only contain the following characters: a-zA-Z0-9. _-:/()#,@[]+=&;{}!$*",
  };
}
