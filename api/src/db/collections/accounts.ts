export const ACCOUNTS_COLLECTION = 'accounts';

export type AccountDocument = {
    _id: string;
    account_object_id: string;
    hashed_secret: string;
    owner: string;
    created_at: Date;
    tx_digest: string;
};
