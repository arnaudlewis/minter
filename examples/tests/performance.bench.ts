// @​minter:benchmark #performance#api-response-time
bench("POST /tasks p95 latency", async () => {
  await api.post("/tasks", { title: "Benchmark task" }, { auth: token });
});

// @​minter:benchmark #performance#throughput
bench("sustained request throughput", async () => {
  await loadTest({ rps: 100, duration: "5m" });
});
