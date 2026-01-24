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
            if (env.NODE_ENV !== 'production') {
                const errorDetails =
                    error instanceof Error
                        ? { message: error.message, stack: error.stack, name: error.name }
                        : { message: String(error) };
                logger.error(
                    { error: errorDetails, context: errorMessage },
                    `[DEV] ${errorMessage}`
                );
            } else {
                logger.error({ error }, errorMessage);
            }

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
