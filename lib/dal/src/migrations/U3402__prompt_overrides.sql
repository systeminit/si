CREATE TABLE prompt_overrides 
(
    kind                        VARCHAR(255)             NOT NULL PRIMARY KEY,
    prompt_yaml                 TEXT                     NOT NULL
);
