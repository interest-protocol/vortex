import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { logger } from './logger.ts';

type Handler = (c: Context<AppBindings>) => Promise<Response>;

export const withErrorHandler =
    (handler: Handler, errorMessage: string): Handler =>
    async (c) => {
        try {
            return await handler(c);
        } catch (error) {
            logger.error({ error }, errorMessage);
            return c.json({ success: false, error: errorMessage }, 500);
        }
    };
