-- Hides v0 key pair and ec2 instances as v1 will be created
UPDATE schema_variants SET ui_hidden = TRUE WHERE id IN (SELECT default_schema_variant_id FROM schemas WHERE name = 'Key Pair');
UPDATE schema_variants SET ui_hidden = TRUE WHERE id IN (SELECT default_schema_variant_id FROM schemas WHERE name = 'EC2 Instance');
