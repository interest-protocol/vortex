import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { healthRoutes } from './health.ts';
import { v1Routes } from './v1/index.ts';

export const routes = new Hono<AppBindings>().route('/health', healthRoutes).route('/v1', v1Routes);
