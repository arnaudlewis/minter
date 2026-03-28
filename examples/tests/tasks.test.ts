// @minter:e2e create-task
test("create a new task", async () => {
  const res = await api.post("/tasks", { title: "Buy groceries" }, { auth: token });
  expect(res.status).toBe(201);
  expect(res.body.title).toBe("Buy groceries");
  expect(res.body.completed).toBe(false);
});

// @minter:e2e list-tasks
test("list user tasks", async () => {
  const res = await api.get("/tasks", { auth: token });
  expect(res.body.length).toBeGreaterThanOrEqual(1);
});

// @minter:e2e complete-task
test("mark task as completed", async () => {
  const res = await api.patch("/tasks/task-1/complete", {}, { auth: token });
  expect(res.body.completed).toBe(true);
});

// @minter:e2e create-task-unauthenticated
test("reject task creation without auth", async () => {
  const res = await api.post("/tasks", { title: "Buy groceries" });
  expect(res.status).toBe(401);
});

// @minter:e2e complete-nonexistent-task
test("return 404 for nonexistent task", async () => {
  const res = await api.patch("/tasks/nonexistent/complete", {}, { auth: token });
  expect(res.status).toBe(404);
});
