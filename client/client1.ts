import BN from "bn.js";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from '@solana/web3.js';
import type { SolanaLottery } from "../target/types/solana_lottery";
import * as dotenv from "dotenv";

dotenv.config();

const PROGRAM_ID = new PublicKey("7VD5huPrnENoik7jMZijXnnnVrKayBY3rwk8BLULh5oQ");

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.SolanaLottery as anchor.Program<SolanaLottery>;

const [pda] = await PublicKey.findProgramAddressSync(
    [Buffer.from("global_state_v3")],
    PROGRAM_ID
);
const globalState = await program.account.globalState.fetch(pda);
const lotteryCount = globalState.lotteryCount.toNumber() - 1;

// Deriving lottery PDA in seed format
const lotteryCountBuffer = new BN(lotteryCount).toArrayLike(Buffer, 'le', 8);
const [currentLotteryPDA] = await PublicKey.findProgramAddressSync(
    [Buffer.from("lottery"), lotteryCountBuffer],
    PROGRAM_ID
);

console.log("Current Lottery PDA:", currentLotteryPDA.toBase58());

const EXISTING_LOTTERY_PUBKEY = currentLotteryPDA;
const participant = anchor.web3.Keypair.generate();

(async () => {
    console.log("Trying to buy a ticket in the existing lottery...");

    // Airdrop to new paricipant
    const signature = await program.provider.connection.requestAirdrop(
        participant.publicKey,
        1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await program.provider.connection.confirmTransaction(signature);

    // Try buying ticket
    await program.methods
    .buyTicket()
    .accounts({
        lottery: EXISTING_LOTTERY_PUBKEY,
        buyer: participant.publicKey,
        systemProgram: SystemProgram.programId,
    })
    .signers([participant])
    .rpc();

    console.log("✅ Ticket bought for existing lottery!");

    // Check updated paricipants
    const lotteryAccount = await program.account.lottery.fetch(EXISTING_LOTTERY_PUBKEY);
    console.log("Updated participants:", lotteryAccount.participants);
})();