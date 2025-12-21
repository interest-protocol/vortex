import type { Db } from 'mongodb';
import type { Redis } from 'ioredis';
import { nodeClient } from './sui.ts';

export type HealthStatus = 'healthy' | 'unhealthy';

export type HealthCheckResult = {
    mongodb: HealthStatus;
    redis: HealthStatus;
    sui: HealthStatus;
};

export type HealthService = {
    check: () => Promise<HealthCheckResult>;
};

export const createHealthService = (db: Db, redis: Redis): HealthService => ({
    check: async () => {
        const [mongodb, redisStatus, sui] = await Promise.all([
            db
                .command({ ping: 1 })
                .then((): HealthStatus => 'healthy')
                .catch((): HealthStatus => 'unhealthy'),
            redis
                .ping()
                .then((): HealthStatus => 'healthy')
                .catch((): HealthStatus => 'unhealthy'),
            nodeClient
                .getCurrentEpoch()
                .then((): HealthStatus => 'healthy')
                .catch((): HealthStatus => 'unhealthy'),
        ]);

        return { mongodb, redis: redisStatus, sui };
    },
});
