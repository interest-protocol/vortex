pragma circom 2.0.0;

include "./transaction.circom";

component main {public [root, publicAmount, extDataHash, inputNullifier, outputCommitment, noOutputs]} = Transaction(26, 2, 2, 18688842432741139442778047327644092677418528270738216181718229581494125774932);