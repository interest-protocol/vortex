import type { MiddlewareHandler } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getDb } from '@/db/mongodb.ts';
import { getRedis } from '@/db/redis.ts';
import {
    createPoolsRepository,
    createAccountsRepository,
    createCommitmentsRepository,
} from '@/repositories/index.ts';
import { createAccountsService } from '@/services/accounts.ts';
import { createHealthService } from '@/services/health.ts';
import { createMerkleService } from '@/services/merkle.ts';
import { createRelayerService } from '@/services/relayer.ts';
import { createTransactionsService } from '@/services/transactions.ts';
import { keypair } from '@/services/sui.ts';

export const databaseMiddleware: MiddlewareHandler<AppBindings> = async (c, next) => {
    const db = getDb();
    const redis = getRedis();

    const pools = createPoolsRepository(db);
    const accounts = createAccountsRepository(db);
    const commitments = createCommitmentsRepository(db);

    c.set('pools', pools);
    c.set('accounts', accounts);
    c.set('commitments', commitments);
    c.set('accountsService', createAccountsService(accounts));
    c.set('healthService', createHealthService(db, redis));
    c.set('merkleService', createMerkleService(redis, commitments));
    c.set('relayerService', createRelayerService(keypair));
    c.set('transactionsService', createTransactionsService());

    await next();
};
