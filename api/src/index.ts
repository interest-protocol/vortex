import { Hono } from 'hono';
import { logger as honoLogger } from 'hono/logger';
import { env } from './config/env.js';
import { connectMongoDB, disconnectMongoDB } from './db/mongodb.js';
import { connectRedis, disconnectRedis } from './db/redis.js';
import { corsMiddleware, databaseMiddleware, errorHandler } from './middleware/index.js';
import { routes } from './routes/index.js';
import type { AppBindings } from './types/index.js';
import { logger } from './utils/logger.js';

const app = new Hono<AppBindings>();

app.use(honoLogger());
app.use(corsMiddleware);
app.use(databaseMiddleware);
app.onError(errorHandler);

app.get('/', (c) =>
    c.json({
        success: true,
        data: {
            name: 'Vortex API',
            version: '1.0.0',
        },
    })
);

app.route('/api', routes);

async function main() {
    await connectMongoDB();
    connectRedis();

    logger.info({ host: env.HOST, port: env.PORT }, 'Server started');

    return {
        port: env.PORT,
        hostname: env.HOST,
        fetch: app.fetch,
    };
}

async function shutdown() {
    logger.info('Shutting down...');
    await disconnectMongoDB();
    await disconnectRedis();
    process.exit(0);
}

process.on('SIGINT', () => void shutdown());
process.on('SIGTERM', () => void shutdown());

export default await main();
