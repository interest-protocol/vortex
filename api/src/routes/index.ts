import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.js';
import { healthRoutes } from './health.js';
import { v1Routes } from './v1/index.js';

export const routes = new Hono<AppBindings>().route('/health', healthRoutes).route('/v1', v1Routes);
