import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { apiKeyMiddleware } from '@/middleware/index.ts';
import { getAccounts, createAccount, hideAccounts } from './handlers.ts';

export const accountsRoutes = new Hono<AppBindings>()
    .get('/', getAccounts)
    .post('/', createAccount)
    .post('/hide', apiKeyMiddleware, hideAccounts);
