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
    id: "0d196d4c-c441-4a28-ad74-550593615c9f",
    name: "AWS",
    description: "Amazon Web Services",
    options: JSON.stringify(aws_options),
    image: "aws.png",
  });
  await knex("integrations").insert({
    id: "bc11cdf9-8ce3-4af3-9df7-a17289026dc6",
    name: "Azure",
    description: "Microsoft Azure",
    options: JSON.stringify(azure_options),
    image: "azure.png",
  });
  await knex("integrations").insert({
    id: "d085f0d8-5d21-42b5-a745-037d4e1c9523",
    name: "Google",
    description: "Google Cloud",
    options: JSON.stringify(google_options),
    image: "google.png",
  });
  await knex("integrations").insert({
    id: "4982d1ee-1391-4ae0-864a-4d55d1f8b0b7",
    name: "GitHub",
    description: "GitHub",
    options: JSON.stringify(github_options),
    image: "github.png",
  });
  await knex("integrations").insert({
    id: "9bfc0c3e-6273-4196-8e74-364761cb1b04",
    name: "Global",
    description: "Global",
    options: JSON.stringify({}),
    image: "global.svg",
  });
};

exports.down = function(knex) {
  return knex("integrations")
    .where("name", "AWS")
    .orWhere("name", "Azure")
    .orWhere("name", "Google")
    .orWhere("name", "GitHub")
    .orWhere("name", "Global")
    .del();
};
