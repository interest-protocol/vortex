import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getAccounts, createAccount } from './handlers.ts';

export const accountsRoutes = new Hono<AppBindings>()
    .get('/', getAccounts)
    .post('/', createAccount);
