import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
// for interacting with the vesting program
import * as anchor from "@project-serum/anchor";
// for connecting to the Solana cluster
import { Connection } from "@solana/web3.js";

// client
console.log("My address:", pg.wallet.publicKey.toString());

// fetch the balance of the client's address
const balance = await pg.connection.getBalance(pg.wallet.publicKey);
console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

// fetch the vesting state for the client's address
const vestingStateKey = "<VESTING_STATE_PUBLIC_KEY>"; // replace with the public key of the client's vesting state account
const vestingState = await program.account.vestingState.fetch(vestingStateKey);
console.log("Vesting state:", vestingState);

// check if the vesting period has ended
const currentTimestamp = Math.floor(Date.now() / 1000);
if (currentTimestamp >= vestingState.vestingEnd) {
  console.log("Vesting period has ended. Claiming vested tokens...");

  // claim the vested tokens
  await program.rpc.claimVesting({
    accounts: {
      vestingState: vestingStateKey,
      vault: vestingState.vault,
      recipient: vestingState.receiver,
      tokenProgram: TOKEN_PROGRAM_ID,
      clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
    },
    signers: [], // the signer should be the vault keypair if required
  });

  console.log("Vested tokens claimed successfully!");
} else {
  console.log("Vesting period is still ongoing. Tokens cannot be claimed yet.");
}
