export type Account = {
    id: string;
    accountObjectId: string;
    hashedSecret: string;
    owner: string;
    createdAt: Date;
    txDigest: string;
};
