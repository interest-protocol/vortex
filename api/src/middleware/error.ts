import type { ErrorHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';

export const errorHandler: ErrorHandler<AppBindings> = (err, c) => {
    const status = 'status' in err && typeof err.status === 'number' ? err.status : 500;

    if (env.NODE_ENV !== 'production') {
        logger.error(
            {
                error: {
                    message: err.message,
                    stack: err.stack,
                    name: err.name,
                },
                status,
                path: c.req.path,
                method: c.req.method,
            },
            `[DEV] Unhandled error: ${err.message}`
        );
    } else {
        logger.error({ err }, 'Unhandled error');
    }

    if (env.NODE_ENV !== 'production') {
        return c.json(
            {
                success: false,
                error: err.message,
                stack: err.stack,
            },
            status as 500
        );
    }

    return c.json(
        {
            success: false,
            error: 'Internal Server Error',
        },
        status as 500
    );
};
