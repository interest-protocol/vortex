import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { validateBody } from '@/utils/validation.ts';
import { withErrorHandler } from '@/utils/handler.ts';
import { executeTransactionSchema } from './schema.ts';

const executeTransactionHandler = async (c: Context<AppBindings>) => {
    const validation = await validateBody(c, executeTransactionSchema);
    if (!validation.success) return validation.response;

    const transactionsService = c.get('transactionsService');
    const digest = await transactionsService.execute(validation.data.txBytes);

    return c.json({ success: true, data: { digest } }, 201);
};

export const executeTransaction = withErrorHandler(
    executeTransactionHandler,
    'Failed to execute transaction'
);
