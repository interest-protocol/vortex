pragma circom 2.0.0;

include "./transaction.circom";

component main {public [root, publicAmount, extDataHash, inputNullifier, outputCommitment]} = Transaction(31, 2, 2);