import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getCommitments } from './handlers.ts';

export const commitmentsRoutes = new Hono<AppBindings>().get('/', getCommitments);
