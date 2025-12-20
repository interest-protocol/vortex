import type { ErrorHandler } from 'hono';
import type { AppBindings } from '@/types/index.js';
import { env } from '@/config/env.js';
import { logger } from '@/utils/logger.js';

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
