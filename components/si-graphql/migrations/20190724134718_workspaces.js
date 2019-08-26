exports.up = function(knex) {
  return knex.schema
    .createTable("workspaces", table => {
      table
        .uuid("id")
        .primary()
        .notNullable()
        .unique();
      table.timestamps();
      table.string("name").notNullable;
      table.string("description");
      table.integer("creator_id").references("users.id");
    })
    .createTable("users_workspaces", table => {
      table
        .integer("user_id")
        .references("users.id")
        .onDelete("CASCADE");
      table
        .integer("workspace_id")
        .references("workspaces.id")
        .onDelete("CASCADE");
    });
};

exports.down = function(knex) {
  return knex.schema.dropTable("workspaces").dropTable("users_workspaces");
};
