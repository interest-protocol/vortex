import { Hono } from "hono";
import type { AppBindings } from "../types/index.js";
import { healthRoutes } from "./health.js";

export const routes = new Hono<AppBindings>()
  .route("/health", healthRoutes);
