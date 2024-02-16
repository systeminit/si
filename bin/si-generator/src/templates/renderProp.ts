export const partial = `
<% if (it.omitVariable == true) { %>
new PropBuilder()
<% } else { %>
const <%= it.prop.variableName %> = new PropBuilder()
<% } %>
<% if (it.prop.kind == "string") { %>
<%~ include("@stringPartial", { prop: it.prop }) %>
<% } %>
<% if (it.prop.kind == "number") { %>
<%~ include("@numberPartial", { prop: it.prop }) %>
<% } %>
<% if (it.prop.kind == "boolean") { %>
<%~ include("@booleanPartial", { prop: it.prop }) %>
<% } %>
<% if (it.prop.kind == "object") { %>
<%~ include("@objectPartial", { prop: it.prop }) %>
<% } %>
<% if (it.prop.kind == "array") { %>
<%~ include("@arrayPartial", { prop: it.prop }) %>
<% } %>
<% if (it.prop.kind == "map") { %>
<%~ include("@mapPartial", { prop: it.prop }) %>
<% } %>
<% if (it.omitVariable == true) { %>
.build()
<% } else { %>
.build();
asset.addProp(<%= it.prop.variableName %>);
<% } %>
`;
