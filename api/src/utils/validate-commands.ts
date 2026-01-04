import { normalizeSuiAddress } from '@mysten/sui/utils';
import {
    validateDepositWithAccountCommands,
    validateWithdrawCommands,
    VORTEX_SWAP_PACKAGE_ID,
} from '@interest-protocol/vortex-sdk';
import invariant from 'tiny-invariant';

type MoveCall = {
    package: string;
    module: string;
    function: string;
};

type Command = {
    MoveCall?: MoveCall;
    $kind: string;
};

export type TransactionJson = {
    commands: Command[];
};

const validateSwapCommands = (commands: Command[]): void => {
    const swapCommands = commands.filter(
        (cmd) =>
            cmd.MoveCall?.module === 'vortex_swap' &&
            (cmd.MoveCall.function === 'start_swap' || cmd.MoveCall.function === 'finish_swap')
    );

    invariant(swapCommands.length === 2, 'Expected exactly 2 swap commands');

    const startSwap = swapCommands[0]?.MoveCall;
    const finishSwap = swapCommands[1]?.MoveCall;

    invariant(startSwap, 'Missing start_swap command');
    invariant(finishSwap, 'Missing finish_swap command');

    invariant(
        normalizeSuiAddress(startSwap.package) === normalizeSuiAddress(VORTEX_SWAP_PACKAGE_ID),
        'start_swap: Package mismatch'
    );
    invariant(startSwap.function === 'start_swap', 'Expected start_swap first');

    invariant(
        normalizeSuiAddress(finishSwap.package) === normalizeSuiAddress(VORTEX_SWAP_PACKAGE_ID),
        'finish_swap: Package mismatch'
    );
    invariant(finishSwap.function === 'finish_swap', 'Expected finish_swap second');
};

type Validator = (commands: Command[]) => void;

const validators: Validator[] = [
    validateDepositWithAccountCommands,
    validateWithdrawCommands,
    validateSwapCommands,
];

export const validateTransactionCommands = (commands: Command[]): void => {
    const isValid = validators.some((validate) => {
        try {
            validate(commands);
            return true;
        } catch {
            return false;
        }
    });

    if (!isValid) {
        throw new Error('Invalid transaction commands');
    }
};
