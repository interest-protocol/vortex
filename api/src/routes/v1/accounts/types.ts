export type { AccountDocument } from '@/db/collections/index.js';

export type Account = {
    id: string;
    accountObjectId: string;
    hashedSecret: string;
    owner: string;
    createdAt: Date;
    txDigest: string;
};
