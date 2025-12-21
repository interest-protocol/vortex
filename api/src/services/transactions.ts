import { Transaction } from '@mysten/sui/transactions';
import { fromHex } from '@mysten/sui/utils';
import {
    validateDepositWithAccountCommands,
    validateWithdrawCommands,
} from '@interest-protocol/vortex-sdk';
import { keypair, sponsorAndExecuteTransaction } from '@/services/sui.ts';

type TransactionJson = {
    commands: object[];
};

const validateTransactionCommands = (commands: object[]): void => {
    try {
        validateDepositWithAccountCommands(commands);
    } catch {
        validateWithdrawCommands(commands);
    }
};

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
