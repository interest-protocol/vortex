import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { executeTransaction } from './handlers.ts';

export const transactionsRoutes = new Hono<AppBindings>().post('/', executeTransaction);
