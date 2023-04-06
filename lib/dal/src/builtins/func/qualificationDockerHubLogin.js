async function qualificationDockerHubLogin(input) {
  if (!input.domain.secret) {
    return {
      result: "failure",
      message: "no credential available"
    }
  }

  const { username, password } = input.domain.secret.message;

  const request = await fetch("https://hub.docker.com/v2/users/login", {
    method: "POST",
    body: JSON.stringify({ username, password }),
    headers: { 'Content-Type': 'application/json' }
  });
  const response = await request.json();
  return {
    result: !!response.token ? "success" : "failure",
    message: response.detail ?? (response.message ?? "docker hub login succeeded"),
  };
}
