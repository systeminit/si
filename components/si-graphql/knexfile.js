// Update with your config settings.

module.exports = {
  development: {
    client: "sqlite3",
    useNullAsDefault: true,
    connection: {
      filename: "./db/dev.sqlite3",
    },
  },
  production: {
    client: "sqlite3",
    useNullAsDefault: true,
    connection: {
      filename: "/hab/svc/si-graphql/data/db/prod.sqlite3",
    },
  },

  // staging: {
  //   client: 'postgresql',
  //   connection: {
  //     database: 'my_db',
  //     user: 'username',
  //     password: 'password',
  //   },
  //   pool: {
  //     min: 2,
  //     max: 10,
  //   },
  //   migrations: {
  //     tableName: 'knex_migrations',
  //   },
  // },

  // production: {
  //   client: 'postgresql',
  //   connection: {
  //     database: 'my_db',
  //     user: 'username',
  //     password: 'password',
  //   },
  //   pool: {
  //     min: 2,
  //     max: 10,
  //   },
  //   migrations: {
  //     tableName: 'knex_migrations',
  //   },
  // },
};
