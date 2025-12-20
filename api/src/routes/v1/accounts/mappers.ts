import type { Account, AccountDocument } from './types.js';

export function toAccount(doc: AccountDocument): Account {
    return {
        id: doc._id,
        accountObjectId: doc.account_object_id,
        hashedSecret: doc.hashed_secret,
        owner: doc.owner,
        createdAt: doc.created_at,
        txDigest: doc.tx_digest,
    };
}
