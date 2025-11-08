To run a lambda locally:

1. Set up AWS environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_SESSION_TOKEN) 

2. Set up the python environment with the AWS lambda packages, and the Lambda environment variables:

   ```sh
   source ./setup_venv.sh
   source ./setup_lambda_env.sh billing-fire-posthog-events
   ```

3. Run the lambda handler you want, with any arguments you want:

   ```sh
   python -c 'import importlib; importlib.import_module("billing-data-check-errors").lambda_handler({ "SI_DRY_RUN": True })'
   ```
