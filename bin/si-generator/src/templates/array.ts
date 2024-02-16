export const partial = `
.setName("<%= it.prop.name %>")
.setKind("array")
.setEntry(
<%~ include("@renderPropPartial", { prop: it.prop.entry, omitVariable: true }) %>
)
`;
