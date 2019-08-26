exports.up = function(knex) {
  return knex.schema
    .createTable("integrations", table => {
      table
        .uuid("id")
        .primary()
        .notNullable()
        .unique();
      table.timestamps();
      table.string("name").notNullable;
      table.string("description");
      table.jsonb("options");
      table.string("image");
    })
    .createTable("integration_instances", table => {
      table
        .uuid("id")
        .primary()
        .notNullable()
        .unique();
      table.timestamps();
      table.string("name").notNullable;
      table.string("description").notNullable;
      table.jsonb("options").notNullable;
      table
        .integer("user_id")
        .references("users.id")
        .onDelete("CASCADE");
      table
        .integer("integration_id")
        .references("integrations.id")
        .onDelete("CASCADE");
    })
    .createTable("integration_instance_workspaces", table => {
      table
        .integer("integration_instance_id")
        .references("integration_instances.id")
        .onDelete("CASCADE");
      table
        .integer("workspace_id")
        .references("workspaces.id")
        .onDelete("CASCADE");
    });
};

exports.down = function(knex) {
  return knex.schema
    .dropTable("integration_instance_workspaces")
    .dropTable("integration_instances")
    .dropTable("integrations");
};
