export const partial = `
.setName("<%= it.prop.name %>")
.setKind("object")
<% for (const child of it.prop.children) { %>
.addChild(
<%~ include("@renderPropPartial", { prop: child, omitVariable: true }) %>
)
<% } %>
`;
