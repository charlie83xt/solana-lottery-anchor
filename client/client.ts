import BN from "bn.js";
import * as web3 from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey, clusterApiUrl } from '@solana/web3.js';
import { Program } from "@coral-xyz/anchor"
import type { SolanaLottery } from "../target/types/solana_lottery";

// Configure the client to use the local cluster
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.SolanaLottery as anchor.Program<SolanaLottery>;

// Client
console.log("My address:", program.provider.publicKey.toString());
// const balance = await program.provider.connection.getBalance(program.provider.publicKey);
// console.log(`My balance: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

const lotteryPubkey = new PublicKey("66tYyDnMHZLWYdXkR7hsYrxB4wb35FQtqNWhUvDbTHoa");
const PROGRAM_ID = new PublicKey("HPVGDAGGGSV93gZu3vk3uxz3RSWdvZQ6Tc9EgUCLi5TG");

// const lotteryBuffer = new anchor.BN(4).toArrayLike(Buffer, 'le', 8);
// const [lotteryPda] = await PublicKey.findProgramAddress(
//   [Buffer.from("lottery"), lotteryBuffer],
//   PROGRAM_ID
// );
// console.log("Client derived PDA:", lotteryPda.toBase58())
// async getGlobalStatePDA(): Promise<void> {
const [pda] = await PublicKey.findProgramAddressSync(
  [Buffer.from("global_state_v2")],
  PROGRAM_ID
  );
console.log("Global State PDA:", pda.toBase58());
    // return pda;
// }
const fetchedGlobalState = await program.account.fetch(globalState)