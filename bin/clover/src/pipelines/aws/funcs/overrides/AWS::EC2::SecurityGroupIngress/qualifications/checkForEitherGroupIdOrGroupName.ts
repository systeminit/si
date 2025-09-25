async function main(component: Input): Promise<Output> {
  const props = component.domain || {};
  const groupId = props.GroupId;
  const groupName = props.GroupName;

  if (groupId && groupName) {
    return {
      result: "failure",
      message: "You can only provide one of GroupId or GroupName, not both.",
    };
  }

  if (!groupId && !groupName) {
    return {
      result: "failure",
      message: "You must provide either GroupId or GroupName.",
    };
  }

  return {
    result: "success",
    message: "Component qualified",
  };
}
