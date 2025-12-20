import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.js';
import { getAccounts, createAccount } from './handlers.js';

export const accountsRoutes = new Hono<AppBindings>()
    .get('/', getAccounts)
    .post('/', createAccount);
