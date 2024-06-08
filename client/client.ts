import { Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAssociatedTokenAccount, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import * as anchor from "@project-serum/anchor";

// Configure the client to use the local cluster.
const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);

// Load the IDL and create a program client
const idl = require("./path/to/idl.json"); // Replace with the path to your IDL file
const programId = new PublicKey("YOUR_PROGRAM_ID"); // Replace with your program ID
const program = new anchor.Program(idl, programId, provider);

const wallet = provider.wallet;

// Helper function to create and initialize a new vesting account
async function initVesting() {
    const vestingState = Keypair.generate();
    const vault = await getOrCreateAssociatedTokenAccount(
        connection,
        wallet.payer,
        new PublicKey("YOUR_TOKEN_MINT_ADDRESS"), // Replace with your token mint address
        vestingState.publicKey
    );

    const funder = wallet.publicKey;
    const recipient = new PublicKey("RECIPIENT_PUBLIC_KEY"); // Replace with the recipient's public key
    const amount = 1000000000; // Amount to be vested (in smallest unit, e.g., lamports for SOL)
    const vestingEnd = Math.floor(Date.now() / 1000) + 60 * 60 * 24 * 30; // Vesting period ends in 30 days

    await program.rpc.initVesting(
        new anchor.BN(amount),
        new anchor.BN(vestingEnd),
        {
            accounts: {
                vestingState: vestingState.publicKey,
                vault: vault.address,
                funder: funder,
                recipient: recipient,
                tokenProgram: TOKEN_PROGRAM_ID,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                systemProgram: SystemProgram.programId,
            },
            signers: [vestingState],
            instructions: [
                await program.account.vestingState.createInstruction(vestingState),
            ],
        }
    );

    console.log(`Vesting account initialized: ${vestingState.publicKey}`);
}

// Helper function to claim the vested tokens
async function claimVesting(vestingStateKey: PublicKey) {
    const vestingState = await program.account.vestingState.fetch(vestingStateKey);
    const currentTimestamp = Math.floor(Date.now() / 1000);

    if (currentTimestamp >= vestingState.vestingEnd.toNumber()) {
        console.log("Vesting period has ended. Claiming vested tokens...");

        await program.rpc.claimVesting({
            accounts: {
                vestingState: vestingStateKey,
                vault: vestingState.vault,
                recipient: vestingState.receiver,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            },
            signers: [],
        });

        console.log("Vested tokens claimed successfully!");
    } else {
        console.log("Vesting period is still ongoing. Tokens cannot be claimed yet.");
    }
}

// Main function to execute the logic
async function main() {
    await initVesting();

    // Use the public key of the vesting state created during initVesting
    const vestingStateKey = new PublicKey("VESTING_STATE_PUBLIC_KEY"); // Replace with the actual vesting state public key
    await claimVesting(vestingStateKey);
}

main().catch(err => {
    console.error(err);
});
