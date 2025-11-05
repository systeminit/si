To run a lambda locally:

1. Set up the python environment with the AWS lambda packages:

   ```sh
   source ./setup_venv.sh
   ```

2. Go to the Lambda in si-shared-prod and look up env vars

   - Copy LAMBDA_REDSHIFT_ACCESS to your environment
   - Copy LAGO_API_TOKEN_ARN to your environment
   - Copy BILLING_USER_WORKSPACE_ID to your environment
   - Copy BILLING_USER_PASSWORD_ARN to your environment

3. Set up AWS environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_SESSION_TOKEN) 

4. Add this to the bottom of the script you want to run:

   ```python
   lambda_handler()
   ```

5. Run the lambda handler you want:

   ```sh
   python billing-data-check-errors.py
   ```
