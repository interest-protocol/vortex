import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { validateBody, validateQuery } from '@/utils/validation.ts';
import { withErrorHandler } from '@/utils/handler.ts';
import { createAccountSchema, getAccountsQuerySchema } from './schema.ts';
import { toAccount } from './mappers.ts';

const getAccountsHandler = async (c: Context<AppBindings>) => {
    const validation = validateQuery(c, getAccountsQuerySchema);
    if (!validation.success) return validation.response;

    const accountsService = c.get('accountsService');
    const accounts = await accountsService.findByHashedSecret(validation.data.hashed_secret);

    return c.json({ success: true, data: accounts.map(toAccount) });
};

const createAccountHandler = async (c: Context<AppBindings>) => {
    const validation = await validateBody(c, createAccountSchema);
    if (!validation.success) return validation.response;

    const accountsService = c.get('accountsService');
    const accountDoc = await accountsService.create(validation.data);

    return c.json({ success: true, data: toAccount(accountDoc) }, 201);
};

export const getAccounts = withErrorHandler(getAccountsHandler, 'Failed to fetch accounts');

export const createAccount = withErrorHandler(createAccountHandler, 'Failed to create account');
