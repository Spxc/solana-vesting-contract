import { Connection, PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo, getAccount } from "@solana/spl-token";
import * as anchor from "@project-serum/anchor";
import { assert } from "chai";

// Configure the client to use the local cluster.
const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);

// Load the IDL and create a program client
const idl = require("./path/to/idl.json"); // Replace with the path to your IDL file
const programId = new PublicKey("YOUR_PROGRAM_ID"); // Replace with your program ID
const program = new anchor.Program(idl, programId, provider);

const wallet = provider.wallet;

describe("Vesting Program Tests", () => {
    let mint: PublicKey;
    let funderTokenAccount: PublicKey;
    let recipientTokenAccount: PublicKey;
    let vestingState: Keypair;
    let vault: PublicKey;
    let recipient: PublicKey;
    const amount = 1000000000; // Amount to be vested (in smallest unit, e.g., lamports for SOL)
    const vestingEnd = Math.floor(Date.now() / 1000) + 60; // Vesting period ends in 1 minute

    before(async () => {
        // Create a new token mint and associated token accounts
        mint = await createMint(connection, wallet.payer, wallet.publicKey, null, 9);
        funderTokenAccount = await createAssociatedTokenAccount(connection, wallet.payer, mint, wallet.publicKey);
        recipient = Keypair.generate().publicKey;
        recipientTokenAccount = await createAssociatedTokenAccount(connection, wallet.payer, mint, recipient);

        // Mint tokens to the funder's token account
        await mintTo(connection, wallet.payer, mint, funderTokenAccount, wallet.publicKey, amount);

        // Create a new vesting state account
        vestingState = Keypair.generate();

        // Get or create the vault account
        vault = await createAssociatedTokenAccount(connection, wallet.payer, mint, vestingState.publicKey);
    });

    it("Initializes vesting", async () => {
        await program.rpc.initVesting(
            new anchor.BN(amount),
            new anchor.BN(vestingEnd),
            {
                accounts: {
                    vestingState: vestingState.publicKey,
                    vault: vault,
                    funder: wallet.publicKey,
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

        const vestingAccount = await program.account.vestingState.fetch(vestingState.publicKey);
        assert.equal(vestingAccount.isInitialized, true, "Vesting state should be initialized");
        assert.equal(vestingAccount.amount.toNumber(), amount, "Amount should be correct");
        assert.equal(vestingAccount.receiver.toString(), recipient.toString(), "Recipient should be correct");
        assert.equal(vestingAccount.funder.toString(), wallet.publicKey.toString(), "Funder should be correct");
    });

    it("Claims vested tokens", async () => {
        // Wait for the vesting period to end
        await new Promise(resolve => setTimeout(resolve, 60000));

        const currentTimestamp = Math.floor(Date.now() / 1000);
        assert.isAtLeast(currentTimestamp, vestingEnd, "Current timestamp should be at least vesting end");

        await program.rpc.claimVesting({
            accounts: {
                vestingState: vestingState.publicKey,
                vault: vault,
                recipient: recipientTokenAccount,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
            },
            signers: [],
        });

        const recipientAccount = await getAccount(connection, recipientTokenAccount);
        assert.equal(recipientAccount.amount.toNumber(), amount, "Recipient should have received the vested tokens");

        const vestingAccount = await program.account.vestingState.fetch(vestingState.publicKey);
        assert.equal(vestingAccount.isInitialized, false, "Vesting state should be uninitialized after claiming");
    });
});
