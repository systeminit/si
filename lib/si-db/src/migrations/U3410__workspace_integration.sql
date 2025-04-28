CREATE TABLE workspace_integrations
(
    pk                          ident primary key default ident_create_v1(),
    slack_webhook_url           text NULL,
    workspace_pk                ident NOT NULL
);
CREATE UNIQUE INDEX ON workspace_integrations (pk);
CREATE UNIQUE INDEX ON workspace_integrations (workspace_pk);
CREATE UNIQUE INDEX ON workspace_integrations (slack_webhook_url, workspace_pk);
CREATE INDEX ON workspace_integrations (slack_webhook_url);
CREATE INDEX ON workspace_integrations (workspace_pk);
