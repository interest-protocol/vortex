import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getPools } from './handlers.ts';

export const poolsRoutes = new Hono<AppBindings>().get('/', getPools);
