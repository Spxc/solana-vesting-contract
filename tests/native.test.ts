import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

// Import your program ID
//import { vestingProgramId } from "./src/lib";

describe("Vesting Program", () => {
  let connection: Connection;
  let programId: PublicKey;
  let tokenProgramId: PublicKey;

  // Accounts used in the tests
  let funder: Keypair;
  let vault: Keypair;
  let recipient: Keypair;

  before(async () => {
    // Establish connection to the devnet cluster
    connection = new Connection("https://api.devnet.solana.com", "confirmed");

    // Load your program ID
    programId = new PublicKey("DFGuapfSuXhUpU9V1yNbMUZ76tReRqeY8F4byVTcUWV8");

    // Load SPL Token program ID
    tokenProgramId = TOKEN_PROGRAM_ID;

    // Generate keypairs for testing accounts
    funder = Keypair.generate();
    vault = Keypair.generate();
    recipient = Keypair.generate();
  });

  it("should initialize vesting", async () => {
    // test logic for initializing vesting
  });

  it("should claim vested funds", async () => {
    // test logic for claiming vested funds
  });

  it("should initialize vesting", async () => {
    // test logic for initializing vesting
  });

  it("should not claim vested funds", async () => {
    // test logic for trying to claimvested funds before vested periode is over
  });
});
