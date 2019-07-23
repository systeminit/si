exports.up = function(knex) {
  return knex.schema.createTable('users', function(table) {
    table.increments();
    table.timestamps();
    table.string('name');
    table
      .string('email')
      .notNullable()
      .unique();
  });
};

exports.down = function(knex) {
  return knex.schema.dropTable('users');
};
