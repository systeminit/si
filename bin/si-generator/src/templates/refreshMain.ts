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

  if (child.exitCode !== 0) {
    const payload = _.get(component, 'properties.resource.payload');
    if (payload) {
      return {
        status: "error",
        payload,
        message: \`Refresh error; exit code \${child.exitCode}.\\n\\nSTDOUT:\\n\\n\${child.stdout}\\n\\nSTDERR:\\n\\n\${child.stderr}\`,
      }
    } else {
      return {
        status: "error",
        message: \`Refresh error; exit code \${child.exitCode}.\\n\\nSTDOUT:\\n\\n\${child.stdout}\\n\\nSTDERR:\\n\\n\${child.stderr}\`,
      }
    }
  }

  const response = JSON.parse(child.stdout);
  const resource = {};
<% for (const output of it.outputs) { %>
<% if (output.toSet) { %>
  _.set(resource, <%= output.toSet %>, _.get(response, '<%= output.readFrom %>');
<% } else { %>
  _.merge(resource, _.get(response, '<%= output.readFrom %>'));
<% } %>
<% } %>
  if (!resource) {
    return {
      status: "error",
      message: \`Resource not found in payload.\\n\\nResponse:\\n\\n\${child.stdout}\`
    }
  }
  return {
    payload: resource,
    status: "ok"
  }
}
`
