import { cors } from 'hono/cors';
import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';

export const corsMiddleware: MiddlewareHandler<AppBindings> =
    env.NODE_ENV === 'production'
        ? cors({
              origin: env.CORS_ORIGIN ?? '',
              allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
              allowHeaders: ['Content-Type', 'Authorization'],
              exposeHeaders: ['Content-Length'],
              maxAge: 86400,
              credentials: true,
          })
        : cors();
