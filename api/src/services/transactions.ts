import { Transaction } from '@mysten/sui/transactions';
import { fromHex } from '@mysten/sui/utils';
import { keypair, sponsorAndExecuteTransaction } from '@/services/sui.ts';
import { validateTransactionCommands, type TransactionJson } from '@/utils/validate-commands.ts';

export type TransactionsService = {
    execute: (txBytes: string) => Promise<string>;
};

export const createTransactionsService = (): TransactionsService => ({
    execute: async (txBytes) => {
        const rebuiltTransaction = Transaction.from(fromHex(txBytes));

        const transactionJson = JSON.parse(await rebuiltTransaction.toJSON()) as TransactionJson;
        validateTransactionCommands(transactionJson.commands);

        rebuiltTransaction.setSender(keypair.toSuiAddress());

        const result = await sponsorAndExecuteTransaction(rebuiltTransaction);
        return result.digest;
    },
});
