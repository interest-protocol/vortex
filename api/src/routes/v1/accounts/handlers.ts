import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { logger } from '@/utils/logger.ts';
import { validateBody, validateQuery } from '@/utils/validation.ts';
import { createAccountSchema, getAccountsQuerySchema } from './schema.ts';
import { toAccount } from './mappers.ts';

export const getAccounts = async (c: Context<AppBindings>) => {
    const accountsService = c.get('accountsService');

    const validation = validateQuery(c, getAccountsQuerySchema);
    if (!validation.success) return validation.response;

    const accounts = await accountsService.findByHashedSecret(validation.data.hashed_secret);

    return c.json({ success: true, data: accounts.map(toAccount) });
};

export const createAccount = async (c: Context<AppBindings>) => {
    const accountsService = c.get('accountsService');

    const validation = await validateBody(c, createAccountSchema);
    if (!validation.success) return validation.response;

    try {
        const accountDoc = await accountsService.create(validation.data);
        return c.json({ success: true, data: toAccount(accountDoc) }, 201);
    } catch (error) {
        logger.error({ error }, 'Failed to create account');
        return c.json({ success: false, error: 'Failed to create account' }, 500);
    }
};
