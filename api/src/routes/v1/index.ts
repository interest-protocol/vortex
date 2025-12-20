import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.js';
import { poolsRoutes } from './pools/index.js';

export const v1Routes = new Hono<AppBindings>().route('/pools', poolsRoutes);
