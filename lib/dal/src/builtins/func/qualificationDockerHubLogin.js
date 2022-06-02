async function qualificationDockerHubLogin(component) {
  const { username, password } = component.data.properties.domain.secret.message;

  const request = await fetch("https://hub.docker.com/v2/users/login", {
    method: "POST",
    body: JSON.stringify({ username, password }),
    headers: {'Content-Type': 'application/json'}
  });
  const response = await request.json();
  return {
    qualified: !!response.token,
    message: response.detail ?? (response.message ?? "Docker Hub login succeeded"),
  };
}
