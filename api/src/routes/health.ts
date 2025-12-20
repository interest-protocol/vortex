import { Hono } from "hono";
import type { AppBindings } from "../types/index.js";

export const healthRoutes = new Hono<AppBindings>().get("/", async (c) => {
  const db = c.get("db");
  const redis = c.get("redis");

  const [mongoStatus, redisStatus] = await Promise.all([
    db
      .command({ ping: 1 })
      .then(() => "healthy" as const)
      .catch(() => "unhealthy" as const),
    redis
      .ping()
      .then(() => "healthy" as const)
      .catch(() => "unhealthy" as const),
  ]);

  const isHealthy = mongoStatus === "healthy" && redisStatus === "healthy";

  return c.json(
    {
      status: isHealthy ? "healthy" : "degraded",
      services: {
        mongodb: mongoStatus,
        redis: redisStatus,
      },
      timestamp: new Date().toISOString(),
    },
    isHealthy ? 200 : 503
  );
});
