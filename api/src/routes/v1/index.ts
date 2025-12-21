import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { accountsRoutes } from './accounts/index.ts';
import { commitmentsRoutes } from './commitments/index.ts';
import { merkleRoutes } from './merkle/index.ts';
import { poolsRoutes } from './pools/index.ts';
import { transactionsRoutes } from './transactions/index.ts';

export const v1Routes = new Hono<AppBindings>()
    .route('/accounts', accountsRoutes)
    .route('/commitments', commitmentsRoutes)
    .route('/merkle', merkleRoutes)
    .route('/pools', poolsRoutes)
    .route('/transactions', transactionsRoutes);
