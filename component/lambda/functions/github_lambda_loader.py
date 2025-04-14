# From https://github.com/systeminit/si/blob/main/component/lambda/functions/github_lambda_loader.py
import importlib, urllib.request, urllib.parse, hashlib, pathlib, os, sys, tempfile, logging

logging.getLogger().setLevel("INFO")

# The name of the file containing the lambda handler (defaults to <lambda_function_name>.py)
SI_IMPORT_LAMBDA_HANDLER = os.environ.get('SI_IMPORT_LAMBDA_HANDLER', f"{os.environ['AWS_LAMBDA_FUNCTION_NAME']}.py")
# Where to download the Python files from (e.g. https://raw.githubusercontent.com/systeminit/si/refs/heads/main/component/lambda/functions/)
SI_LAMBDA_FUNCTIONS_URL = os.environ['SI_LAMBDA_FUNCTIONS_URL']
# The modules to import (defaults to all of them if not specified)
SI_IMPORT_PYTHON        = os.environ.get('SI_IMPORT_MODULES', "si_lambda.py si_types.py si_redshift.py si_lago_api.py si_auth_api.py si_posthog_api.py")

# Create temporary directory to download modules
with tempfile.TemporaryDirectory('github_lambda_loader') as DOWNLOADED_MODULE_DIR:
    sys.path.append(DOWNLOADED_MODULE_DIR)

    # Download referenced modules
    def download_module_file(relative_path: str):
        url = urllib.parse.urljoin(SI_LAMBDA_FUNCTIONS_URL, relative_path)
        file = pathlib.Path(DOWNLOADED_MODULE_DIR, relative_path)
        logging.debug(f"Retrieving {url} to {file} ...")
        urllib.request.urlretrieve(url, file)
        logging.info(f"{file}: SHA256 = {hashlib.sha256(file.read_bytes()).hexdigest()}, size: {file.stat().st_size} bytes")
        return file

    for relative_path in SI_IMPORT_PYTHON.split():
        download_module_file(relative_path)
    lambda_handler_module_file = download_module_file(SI_IMPORT_LAMBDA_HANDLER)

    # Import lambda_handler
    logging.info(f"Importing {lambda_handler_module_file}.lambda_handler ...")
    lambda_handler = importlib.import_module(lambda_handler_module_file.stem).lambda_handler
