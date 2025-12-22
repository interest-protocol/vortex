export type Account = {
    id: string;
    objectId: string;
    hashedSecret: string;
    owner: string;
    createdAt: Date;
    txDigest: string;
};
