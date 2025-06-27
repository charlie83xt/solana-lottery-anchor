import BN from "bn.js";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from '@solana/web3.js';
import type { SolanaLottery } from "../target/types/solana_lottery";
import * as dotenv from "dotenv";
// import { Provider } from "@project-serum/anchor";

dotenv.config();

// const PROGRAM_ID = new PublicKey("7VD5huPrnENoik7jMZijXnnnVrKayBY3rwk8BLULh5oQ");
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.SolanaLottery as anchor.Program<SolanaLottery>;

(async () => {
    // Derive the global state PDA
    const [globalStatePDA] = await PublicKey.findProgramAddressSync(
        [Buffer.from("global_state_v3")],
        program.programId
    );

    // Derive the treasury PDA
    const [treasuryPDA] = await PublicKey.findProgramAddressSync(
        [Buffer.from("treasury_pda")],
        program.programId
    );

    // Dev wallets here
    const devWalletJulian = new PublicKey("CaUVjLTBAhgppw6vz9jenc2VcAf7sJ6YvY1u5UL6FV72")
    const devWalletDiego = new PublicKey("JxKgegCWHUdkkgfeuhmHoKYp4skWQKEBQbCoxCkKu43")

    // Call the on-chain method to initialise the global state
    const tx = await program.methods
    .initializeGlobalState()
    .accounts({
        globalState: globalStatePDA,
        treasuryPDA,
        authority: provider.wallet.publicKey,
        devWalletJulian,
        devWalletDiego,
        systemProgram: SystemProgram.programId,
    })
    .rpc();

    console.log("✅ GlobalState initialised, TX:", tx);
    console.log("✅ GlobalState PDA:", globalStatePDA.toBase58());
})();
