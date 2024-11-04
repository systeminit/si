-- s3 schema from Glue
CREATE EXTERNAL SCHEMA spectrum_schema
FROM DATA CATALOG
DATABASE 'si-prod-data'
IAM_ROLE 'arn:aws:iam::$REDSHIFT_ACCOUNT:role/si-prod-redshift,arn:aws:iam::$S3_ACCOUNT:role/si-prod-glue-data-bucket';
