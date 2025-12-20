import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.js';
import { getPools } from './handlers.js';

export const poolsRoutes = new Hono<AppBindings>().get('/', getPools);
