import type { ErrorHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';

export const errorHandler: ErrorHandler<AppBindings> = (err, c) => {
    logger.error({ err }, 'Unhandled error');

    const status = 'status' in err && typeof err.status === 'number' ? err.status : 500;
    const message = env.NODE_ENV === 'production' ? 'Internal Server Error' : err.message;

    return c.json(
        {
            success: false,
            error: message,
        },
        status as 500
    );
};
