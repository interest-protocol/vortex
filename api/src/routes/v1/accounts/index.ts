import { Hono } from 'hono';
import { Transaction } from '@mysten/sui/transactions';
import { VORTEX_PACKAGE_ID } from '@interest-protocol/vortex-sdk';
import type { AppBindings } from '@/types/index.js';
import { ACCOUNTS_COLLECTION, type AccountDocument } from '@/db/collections/index.js';
import { sponsorAndExecuteTransaction } from '@/services/sui.js';
import { validateBody } from '@/utils/validation.js';
import { createAccountSchema } from './schema.js';
import { toAccount } from './mappers.js';

export const accountsRoutes = new Hono<AppBindings>().post('/', async (c) => {
    const db = c.get('db');
    const body = await c.req.json<{ owner?: string; hashedSecret?: string }>();

    const validation = validateBody(c, createAccountSchema, body);
    if (!validation.success) {
        return validation.response;
    }

    const { owner, hashedSecret } = validation.data;

    const tx = new Transaction();

    const account = tx.moveCall({
        target: `${VORTEX_PACKAGE_ID}::vortex_account::new`,
        arguments: [tx.pure.u256(hashedSecret)],
    });

    tx.moveCall({
        target: `${VORTEX_PACKAGE_ID}::vortex_account::share`,
        arguments: [account],
    });

    const txResult = await sponsorAndExecuteTransaction(tx);

    const createdAccount = txResult.objectChanges?.find(
        (change) =>
            change.type === 'created' && change.objectType.includes('vortex_account::VortexAccount')
    );

    if (createdAccount?.type !== 'created') {
        return c.json(
            {
                success: false,
                error: 'Failed to find created account object',
            },
            500
        );
    }

    const accountDoc: AccountDocument = {
        _id: createdAccount.objectId,
        account_object_id: createdAccount.objectId,
        hashed_secret: hashedSecret,
        owner,
        created_at: new Date(),
        tx_digest: txResult.digest,
    };

    await db.collection<AccountDocument>(ACCOUNTS_COLLECTION).insertOne(accountDoc);

    return c.json(
        {
            success: true,
            data: toAccount(accountDoc),
        },
        201
    );
});
