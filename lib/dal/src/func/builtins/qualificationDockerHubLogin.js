async function qualificationDockerHubLogin(component) {
  // This feels a little verbose
  const { username, password } = component.data.properties.secret.message;

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
