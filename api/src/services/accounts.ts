import { Transaction } from '@mysten/sui/transactions';
import { VORTEX_PACKAGE_ID } from '@interest-protocol/vortex-sdk';
import type { AccountDocument } from '@/db/collections/index.ts';
import type {
    AccountsRepository,
    FindAccountsParams,
    HideAccountsParams,
} from '@/repositories/index.ts';
import { sponsorAndExecuteTransaction } from './sui.ts';

export type CreateAccountParams = {
    owner: string;
    hashedSecret: string;
};

export type AccountsService = {
    findByHashedSecret: (params: FindAccountsParams) => Promise<AccountDocument[]>;
    create: (params: CreateAccountParams) => Promise<AccountDocument>;
    hideMany: (params: HideAccountsParams) => Promise<number>;
};

export const createAccountsService = (repository: AccountsRepository): AccountsService => ({
    findByHashedSecret: (params) => repository.findByHashedSecret(params),

    hideMany: (params) => repository.hideMany(params),

    create: async ({ owner, hashedSecret }) => {
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
                change.type === 'created' &&
                change.objectType.includes('vortex_account::VortexAccount')
        );

        if (createdAccount?.type !== 'created') {
            throw new Error('Failed to find created account object');
        }

        const accountDoc: AccountDocument = {
            _id: createdAccount.objectId,
            account_object_id: createdAccount.objectId,
            hashed_secret: hashedSecret,
            owner,
            created_at: new Date(),
            tx_digest: txResult.digest,
        };

        await repository.insert(accountDoc);

        return accountDoc;
    },
});
