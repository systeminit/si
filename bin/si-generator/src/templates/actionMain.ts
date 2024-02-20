export const partial = `
async function main(component: Input): Promise < Output > {
  const cliArguments = {};
<% for (const input of it.inputs) { %>
  _.set(cliArguments, '<%= input.toSet %>', _.get(component, '<%= input.readFrom %>'));
<% } %>

  const child = await siExec.waitUntilEnd("aws", [
    "<%= it.awsService %>",
    "<%= it.awsCommand %>",
    "--region",
    _.get(component, 'properties.domain.extra.Region', ''),
    "--cli-input-json",
    JSON.stringify(cliArguments),
  ]);

  const payload = _.get(component, 'properties.resource.payload');
  if (child.exitCode !== 0) {
    if (payload) {
      return {
        status: "error",
        payload,
        message: \`Action error; exit code \${child.exitCode}.\\n\\nSTDOUT:\\n\\n\${child.stdout}\\n\\nSTDERR:\\n\\n\${child.stderr}\`,
      }
    } else {
      return {
        status: "error",
        message: \`Action error; exit code \${child.exitCode}.\\n\\nSTDOUT:\\n\\n\${child.stdout}\\n\\nSTDERR:\\n\\n\${child.stderr}\`,
      }
    }
  }

  return {
    payload,
    status: "ok"
  }
}
`
