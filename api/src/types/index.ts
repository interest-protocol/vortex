import type { Context } from "hono";
import type { Db } from "mongodb";
import type { Redis } from "ioredis";

export interface AppBindings {
  Variables: {
    db: Db;
    redis: Redis;
  };
}

export type AppContext = Context<AppBindings>;
