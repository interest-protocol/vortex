import type { Account, AccountDocument } from './types.ts';

export const toAccount = (doc: AccountDocument): Account => ({
    id: doc._id,
    accountObjectId: doc.account_object_id,
    hashedSecret: doc.hashed_secret,
    owner: doc.owner,
    createdAt: doc.created_at,
    txDigest: doc.tx_digest,
});
