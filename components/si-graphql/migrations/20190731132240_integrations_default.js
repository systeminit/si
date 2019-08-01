exports.up = async function(knex) {
  let aws_options = {
    fields: [
      {
        field_id: "access_key",
        field_name: "Access Key",
        field_type: "input",
      },
      {
        field_id: "secret_key",
        field_name: "Secret Key",
        field_type: "secret",
      },
    ],
  };

  let azure_options = {
    fields: [
      {
        field_id: "id_token",
        field_name: "ID Token",
        field_type: "input",
      },
    ],
  };

  let google_options = {
    fields: [
      {
        field_id: "google_json",
        field_name: "Google Token",
        field_type: "input",
      },
    ],
  };

  let github_options = {
    fields: [
      {
        field_id: "github_token",
        field_name: "GitHub Token",
        field_type: "input",
      },
    ],
  };

  await knex("integrations").insert({
    name: "AWS",
    description: "Amazon Web Services",
    options: JSON.stringify(aws_options),
    image: "aws.png",
  });
  await knex("integrations").insert({
    name: "Azure",
    description: "Microsoft Azure",
    options: JSON.stringify(azure_options),
    image: "azure.png",
  });
  await knex("integrations").insert({
    name: "Google",
    description: "Google Cloud",
    options: JSON.stringify(google_options),
    image: "google.png",
  });
  await knex("integrations").insert({
    name: "GitHub",
    description: "GitHub",
    options: JSON.stringify(github_options),
    image: "github.png",
  });
};

exports.down = function(knex) {
  return knex("integrations")
    .where("name", "AWS")
    .orWhere("name", "Azure")
    .orWhere("name", "Google")
    .orWhere("name", "GitHub")
    .del();
};
