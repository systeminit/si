ALTER TABLE audit_logs
ADD COLUMN authentication_method JSONB;
-- Set authentication_method = 'Jwt(Web)' for rows where user_id is not null
UPDATE audit_logs
SET authentication_method = '{"method":"Jwt","role":"Web"}'
WHERE user_id IS NOT NULL;
-- Set authentication_method = 'System' for rows where user_id is null
UPDATE audit_logs
SET authentication_method = '{"method":"System"}'
WHERE user_id IS NULL;
