import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getMerklePathHandler } from './handlers.ts';

export const merkleRoutes = new Hono<AppBindings>().post('/path', getMerklePathHandler);
