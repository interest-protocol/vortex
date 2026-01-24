import type { Context } from 'hono';
import { env } from '@/config/env.ts';
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

            if (env.NODE_ENV !== 'production') {
                const details =
                    error instanceof Error
                        ? { message: error.message, stack: error.stack }
                        : { message: String(error) };
                return c.json({ success: false, error: errorMessage, details }, 500);
            }

            return c.json({ success: false, error: errorMessage }, 500);
        }
    };
