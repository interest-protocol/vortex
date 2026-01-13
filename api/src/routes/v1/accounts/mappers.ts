import { normalizeSuiObjectId } from '@mysten/sui/utils';

import type { AccountDocument } from '@/db/collections/index.ts';
import type { Account } from './types.ts';

export const toAccount = (doc: AccountDocument): Account => ({
    id: doc._id,
    objectId: normalizeSuiObjectId(doc.account_object_id),
    hashedSecret: doc.hashed_secret,
    owner: doc.owner,
    createdAt: doc.created_at,
    txDigest: doc.tx_digest,
    hidden: doc.hidden ?? false,
});
