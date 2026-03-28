// @minter:e2e register-user
test("register a new user", async () => {
  const res = await api.post("/register", {
    email: "alice@example.com",
    password: "s3cure-p4ss!",
  });
  expect(res.status).toBe(201);
  expect(res.body.user_id).toBeDefined();
});

// @minter:e2e login-user
test("login with valid credentials", async () => {
  const res = await api.post("/login", {
    email: "alice@example.com",
    password: "s3cure-p4ss!",
  });
  expect(res.body.token).toBeDefined();
  expect(res.body.expires_in).toBeGreaterThan(0);
});

// @minter:e2e register-duplicate-email
test("reject duplicate email registration", async () => {
  await api.post("/register", { email: "alice@example.com", password: "pass" });
  const res = await api.post("/register", { email: "alice@example.com", password: "other" });
  expect(res.status).toBe(409);
});

// @minter:e2e login-wrong-password
test("reject wrong password", async () => {
  const res = await api.post("/login", {
    email: "alice@example.com",
    password: "wrong-password",
  });
  expect(res.status).toBe(401);
});
