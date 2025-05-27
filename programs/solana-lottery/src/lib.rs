use anchor_lang::prelude::*;
use anchor_lang::solana_program::{clock::Clock, program::invoke, system_instruction};
use anchor_lang::solana_program::program::invoke_signed;
// use anchor_lang::solana_program::sysvar::recent_blockhashes;
use anchor_lang::solana_program::sysvar::Sysvar;
// use anchor_lang::system_program::{transfer, Transfer};

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("7VD5huPrnENoik7jMZijXnnnVrKayBY3rwk8BLULh5oQ");

#[program]
pub mod solana_lottery {
    use super::*;

    pub fn initialize_global_state(ctx: Context<InitializeGlobalState>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;

        // Derive the treasury PDA - only created once
        let (treasury_key, treasury_bump) = Pubkey::find_program_address(
            &[b"treasury_pda"],
            ctx.program_id,
        );

        // Save the treasury PDA address in GlobalState for future reference
        global_state.treasury_pda = treasury_key;

        // Initialise Treasury PDA as a system account (Good to prevent i from being infunded)
        let treasury_account_info = ctx.accounts.treasury_pda.to_account_info();
        if treasury_account_info.lamports() == 0 {

            let signer_seeds: &[&[u8]] = &[
                b"treasury_pda",
                &[treasury_bump],
            ];

            invoke_signed(
            &system_instruction::create_account(
                &ctx.accounts.authority.key(),
                &treasury_key,
                Rent::get()?.minimum_balance(0),
                0,
                &ctx.accounts.system_program.key(),
            ), 
            &[
                ctx.accounts.authority.to_account_info(),
                treasury_account_info.clone(),
                ctx.accounts.system_program.to_account_info(),
            ], 
            &[signer_seeds],
            )?;
        }

        global_state.authority = ctx.accounts.authority.key();
        global_state.dev_wallet_julian = ctx.accounts.dev_wallet_julian.key();
        global_state.dev_wallet_diego = ctx.accounts.dev_wallet_diego.key();
        global_state.lottery_count = 6;
        global_state.julian_share = 55;
        global_state.diego_share = 45;
        
        msg!("Global Lottery Counter Initialized!");
        Ok(())
    }

    pub fn set_dev_shares(ctx: Context<SetDevShares>, julian_share: u64, diego_share: u64) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;

        require!(
            ctx.accounts.authority.key() == global_state.authority,
            ErrorCode::Unauthorized
        );

        require!(
            julian_share + diego_share == 100,
            ErrorCode::InvalidShareSplit
        );

        global_state.julian_share = julian_share;
        global_state.diego_share = diego_share;

        msg!("✅ Dev shares updated: Julian: {}%, Diego{}%", julian_share, diego_share);

        Ok(())
    }

    pub fn set_dev_wallets(ctx: Context<SetDevWallets>, julian: Pubkey, diego: Pubkey) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;

        require!(
            ctx.accounts.authority.key() == global_state.authority,
            ErrorCode::Unauthorized
        );

        global_state.dev_wallet_julian = julian;
        global_state.dev_wallet_diego = diego;

        msg!("✅ Dev wallets updated.");
    Ok(())

    }

    pub fn provide_randomness(ctx: Context<ProvideRandomness>, random_value: u64) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;

        require!(lottery.external_randomness.is_none(), ErrorCode::RandomnessAlreadyProvided);

        lottery.external_randomness = Some(random_value);

        msg!("External randomness provided: {}", random_value);

        Ok(())
    }


    pub fn initialize_lottery(
        ctx: Context<InitializeLottery>,
        ticket_price: u64,
        max_participants: u8,
        duration: Option<i64>,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        let lottery = &mut ctx.accounts.lottery;
        let authority = &ctx.accounts.authority;
        let prize_vault = &ctx.accounts.prize_vault;
        let treasury_pda = &ctx.accounts.treasury_pda;
        

        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(0); // no data

        let lottery_key = lottery.key();
        let vault_seeds = &[b"prize_vault", lottery_key.as_ref()];
        // Store prize vault PDA in the lottery state
        let (vault_key, bump) = Pubkey::find_program_address(
            vault_seeds, 
            ctx.program_id
        );

        require_keys_eq!(vault_key, prize_vault.key(), ErrorCode::InvalidVaultPda);

        // Create the system-owned vault
        invoke_signed(
            &system_instruction::create_account(
                &authority.key(),
                &prize_vault.key(),
                required_lamports,
                0,
                &ctx.accounts.system_program.key(), // owner
            ),
            &[
                authority.to_account_info(),
                prize_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[&[b"prize_vault", lottery.key().as_ref(), &[bump]]],
        )?;

        let rollover_balance = **treasury_pda.to_account_info().lamports.borrow();
        if rollover_balance > 0 {

            let signer_seeds: &[&[u8]] = &[
                b"treasury_pda",
                &[ctx.bumps.treasury_pda],
            ];

            invoke_signed(
                &system_instruction::transfer(
                    &treasury_pda.key(),
                    &prize_vault.key(),
                    rollover_balance,
                ),
                &[
                    treasury_pda.to_account_info(),
                    prize_vault.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[signer_seeds],
            )?;

            global_state.last_rollover = rollover_balance;
            msg!("Rollover of {} lamports added to new lottery!", rollover_balance);
        }
        // Iniialising lottery state
        lottery.ticket_price = ticket_price;
        lottery.authority = authority.key();
        lottery.max_participants = max_participants;
        lottery.participants = Vec::new();
        msg!(
            "🗓 Setting Lottery end_time: {:?} + {:?}",
            Clock::get()?.unix_timestamp,
            duration
        );
        let now = Clock::get()?.unix_timestamp;
        lottery.end_time = duration.map(|d| now + d);

        lottery.winner = None;
        lottery.prize_claimed = false;

        // Increment the global lottery counter
        global_state.lottery_count += 1;

        
        lottery.prize_vault = vault_key;

        msg!("Lottery Initialized!");
        msg!("Lottery PDA: {}", lottery.key());
        msg!("Prize Vault PDA: {}", prize_vault.key());

        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let buyer = &ctx.accounts.buyer;
        let prize_vault = &ctx.accounts.prize_vault;
        let system_program = &ctx.accounts.system_program;

        // Ensure the lottery is still open
        require!(!lottery.is_ended(), ErrorCode::LotteryEnded);
        require!(
            lottery.participants.len() < lottery.max_participants as usize,
            ErrorCode::LotteryFull
        );
        require!(
            !lottery.participants.contains(&buyer.key()),
            ErrorCode::AlreadyParticipating
        );

        // Transfer SOL from buyer to vault account
        invoke(
            &system_instruction::transfer(&buyer.key(), &prize_vault.key(), lottery.ticket_price),
            &[
                buyer.to_account_info(),
                prize_vault.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;

        // Add participant
        lottery.participants.push(buyer.key());

        if lottery.participants.len() == lottery.max_participants as usize {
            let clock = Clock::get()?;
            let now = clock.unix_timestamp;       
            lottery.participants_full_at = Some(now);
            msg!("✅ Lottery now full at {}", now);
        }

        msg!("Ticket purchased successfully!");
        Ok(())
    }

    pub fn draw_winner(ctx: Context<DrawWinner>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let prize_vault = &ctx.accounts.prize_vault;
        let authority = &ctx.accounts.authority;
        let claim_pda = &mut ctx.accounts.claim_pda;
        let treasury_pda = &ctx.accounts.treasury_pda;
        let system_program = &ctx.accounts.system_program;
        // let clock = Clock::get()?;


        require_keys_eq!(
            lottery.authority,
            authority.key(),
            ErrorCode::Unauthorized
        );

        // ✅ Print all accounts to verify correctness
        msg!("🔍 Lottery Key: {}", lottery.key());
        msg!("🔍 Prize Vault: {}", prize_vault.key());
        msg!("🔍 Claim PDA (Before INIt): {}", claim_pda.key());
        msg!("🔍 Authority: {}", authority.key());

        // Ensure the lottery has reached its max participants
        require!(
            lottery.participants.len() == lottery.max_participants as usize,
            ErrorCode::LotteryFull
        );
        require!(lottery.winner.is_none(), ErrorCode::WinnerAlreadyDrawn);

        // Select a random winner
        let randomness_seed = if let Some(external) = lottery.external_randomness {
            external
        } else {
            Clock::get()?.slot
        };
        

        let index = (randomness_seed % lottery.participants.len() as u64) as usize;
        let winner = lottery.participants[index];

        lottery.winner = Some(winner);
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        lottery.winner_drawn_at = Some(now);

        msg!("🥇 Winner Selected: {}", winner);
        msg!("✅ Winner drawn at {}", now);

        // calculate distribution amounts
        let total_funds = **prize_vault.to_account_info().lamports.borrow();

        require!(total_funds > 0, ErrorCode::InsufficientFunds);

        let rollover_amount = total_funds * 20 / 100;
        let remaining_funds = total_funds - rollover_amount;

        let vault_bump = ctx.bumps.prize_vault;
        let treasury_bump = ctx.bumps.treasury_pda;
        let lottery_key = lottery.key();

        let vault_signer_seeds: &[&[u8]] = &[b"prize_vault", lottery_key.as_ref(), &[vault_bump]];

        // Transfer 20% to Treasury PDA
        invoke_signed(
            &system_instruction::transfer(
                &prize_vault.key(),
                &treasury_pda.key(),
                rollover_amount,
            ),
            &[
                prize_vault.to_account_info(),
                treasury_pda.to_account_info(),
                system_program.to_account_info(),
            ],
            &[vault_signer_seeds],
        )?;

    // Transfer 80% remaining to Claim PDA
        invoke_signed(
            &system_instruction::transfer(
                &prize_vault.key(),
                &claim_pda.key(),
                remaining_funds,
            ),
            &[
                prize_vault.to_account_info(),
                claim_pda.to_account_info(),
                system_program.to_account_info(),
            ],
            &[vault_signer_seeds],
        )?;

        claim_pda.total_funds = lottery.ticket_price * (lottery.participants.len() as u64);
        claim_pda.claimed = Vec::new();

        lottery.prize_claimed = false;
        lottery.external_randomness = None;

        msg!("🎉 All funds moved to claim PDA and Treasury PDA");

        Ok(())
    }

    pub fn claim_funds(ctx: Context<ClaimFunds>) -> Result<()> {
        let claim_pda = &mut ctx.accounts.claim_pda;
        let claimer = &ctx.accounts.claimer;
        let system_program = &ctx.accounts.system_program;
        let lottery = &mut ctx.accounts.lottery;
        let global_state = &ctx.accounts.global_state;

        // let available_funds = lottery.ticket_price * (lottery.participants.len() as u64);
        let available_funds = **claim_pda.to_account_info().lamports.borrow();
        require!(available_funds > 0, ErrorCode::InsufficientFunds);

        // Ensure this user is either the winner or a dev
        let is_winner = lottery.winner == Some(claimer.key());
        // let is_participant = lottery.participants.contains(&claimer.key());
        let is_dev = global_state.dev_wallet_julian.key() == claimer.key()
            || global_state.dev_wallet_diego.key() == claimer.key();

        
        require!(
            is_winner || is_dev,
            ErrorCode::Unauthorized
        );

        let mut claim_amount: u64 = 0;
        
        let already_claimed = claim_pda.claimed.contains(&claimer.key());

        // Prevent double claiming
        require!(
            !already_claimed,
            ErrorCode::AlreadyClaimed
        );

        
        let dev_fee = available_funds * 25 / 100;
        let julian_fee = (dev_fee * global_state.julian_share) / 100;
        let diego_fee = dev_fee - julian_fee;
        let remaining_after_dev_fee = available_funds - dev_fee;
        let winner_prize = remaining_after_dev_fee;
        // let non_winner_count = (lottery.participants.len() - 1) as u64;
        // let participant_share =
        //     (remaining_after_dev_fee - winner_prize) / non_winner_count;
        
        
        let this_dev_share = if claimer.key() == global_state.dev_wallet_julian.key() {
            julian_fee
        } else if claimer.key() == global_state.dev_wallet_diego.key() {
            diego_fee
        } else {
            0
        };

        // Devs get half of dev fee (split across both)
        if is_dev && is_winner {
            claim_amount += winner_prize;
            claim_amount += this_dev_share;
            lottery.prize_claimed = true;
        } else if is_dev {
            claim_amount += this_dev_share;
        } else if is_winner {
            // Winner gets winner prize (but Not participant share)
            claim_amount += winner_prize;
            lottery.prize_claimed = true;
        }

        require!(
            claim_amount <= **claim_pda.to_account_info().lamports.borrow(),
            ErrorCode::InsufficientFunds
        );

        require!(claim_amount > 0, ErrorCode::AlreadyClaimed);

        // Audit record
        claim_pda.claimed.push(claimer.key());

        // msg!("Signer seeds: claimPda, {}, bump {}", lottery.key(), bump);
        msg!("Sending {} lamports from Lottery!", claim_amount);
        msg!("To Claimer: {}", claimer.key());
        // msg!("Expected PDA signer bump: {}", bump);
        
        // Validating that Claimer is a System account
        require_keys_eq!(
            *claimer.owner, system_program.key(),
            ErrorCode::InvalidClaimerAccount
        );
        // Checking sufficient lamports
        require!(
            **claim_pda.to_account_info().lamports.borrow() >= claim_amount,
            ErrorCode::InsufficientFunds
        );

        // Transfer SOL
        claim_pda.sub_lamports(claim_amount)?;
        claimer.add_lamports(claim_amount)?;


        // msg!("Claim PDA {}", claim_pda.key());
        msg!("📃 CLAIM RECEIPT | wallet: {} | amount: {} | dev: {} | winner: {} | lottery: {}", 
        claimer.key(), 
        claim_amount,
        is_dev,
        is_winner,
        ctx.accounts.global_state.lottery_count - 1
        );
        msg!("Remaining claim_pda lamports: {}", **claim_pda.to_account_info().lamports.borrow());
        msg!("Claimer balance before: {}", **claimer.to_account_info().lamports.borrow());
        msg!("Role breakdown: is_dev={}, is_winner={}", is_dev, is_winner);

        Ok(())
    }


    pub fn close_lottery(ctx: Context<CloseLottery>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let authority = &ctx.accounts.authority;
        // let prize_vault = &mut ctx.accounts.prize_vault;

        require!(
            lottery.authority == authority.key(),
            ErrorCode::Unauthorized
        );

        msg!("🔴 Closing the lottery...");

        Ok(())
    }


}

#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(
        init,
        seeds = [b"global_state_v3"],
        bump,
        payer = authority,
        space = 8 + 8 + 32 + 32 + 32 + 8 + 8 + 32 + 8
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"treasury_pda"],
        bump
    )]
    pub treasury_pda: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub dev_wallet_julian: SystemAccount<'info>,
    pub dev_wallet_diego: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetDevShares<'info> {
    #[account(
        mut, 
        seeds = [b"global_state_v3"],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetDevWallets<'info> {
    #[account(
        mut,
        seeds = [b"global_state_v3"],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,


    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ProvideRandomness<'info> {
    #[account(mut, has_one = authority)]
    pub lottery: Account<'info, Lottery>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    #[account(
        mut,
        constraint = global_state.authority == authority.key() @ ErrorCode::Unauthorized,
        seeds = [b"global_state_v3"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        seeds = [b"lottery", &global_state.lottery_count.to_le_bytes()[..]], //[1..]
        bump,
        payer = authority,
        space = 8 + 32 + 8 + 1 + (32 * 100) + 8 + 33 + 1 + 9 + 9 + 9
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        mut,
        seeds = [b"treasury_pda"],
        bump
    )]
    pub treasury_pda: SystemAccount<'info>,
    
    /// CHECK: This is a system account created in the instruction. It is safe because we create it.
    #[account(mut)]
    pub prize_vault: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct GlobalState {
    pub authority: Pubkey,
    pub dev_wallet_julian: Pubkey,
    pub dev_wallet_diego: Pubkey,
    pub lottery_count: u64,
    pub julian_share: u64,
    pub diego_share: u64,
    pub treasury_pda: Pubkey,
    pub last_rollover: u64,
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    /// CHECK: This is a system account created in the instruction. It is safe because we create it.
    #[account(
        mut,
        seeds = [b"prize_vault", lottery.key().as_ref()],
        bump
        )]
    pub prize_vault: AccountInfo<'info>,

    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DrawWinner<'info> {
    #[account(
        mut,
        constraint = lottery.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub lottery: Account<'info, Lottery>,

    /// CHECK: This is a system account created in the instruction. It is safe because we create it.
    #[account(
        mut,
        seeds = [b"prize_vault", lottery.key().as_ref()],
        bump
        )]
    pub prize_vault: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 4 + (32 * 100),
        seeds = [b"claimPda", lottery.key().as_ref()],
        bump        
    )]
    pub claim_pda: Account<'info, ClaimPool>,

    #[account(
        mut,
        seeds = [b"treasury_pda"],
        bump
    )]
    pub treasury_pda: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct ClaimPool {
    pub total_funds: u64,
    pub claimed: Vec<Pubkey>,
}

#[derive(Accounts)]
pub struct ClaimFunds<'info> {
    #[account(
        mut,
        seeds = [b"claimPda", lottery.key().as_ref()],
        bump
    )]
    pub claim_pda: Account<'info, ClaimPool>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(
        seeds = [b"global_state_v3"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub lottery: Account<'info, Lottery>,
    #[account(mut)]
    pub winner: Signer<'info>,
    #[account(mut)]
    pub dev_wallet: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Lottery {
    pub authority: Pubkey,
    pub ticket_price: u64,
    pub max_participants: u8,
    pub participants: Vec<Pubkey>,
    pub end_time: Option<i64>,
    pub winner: Option<Pubkey>,
    pub prize_claimed: bool,
    pub prize_vault: Pubkey,
    pub participants_full_at: Option<i64>,
    pub winner_drawn_at: Option<i64>,
    pub external_randomness: Option<u64>,
}

impl Lottery {
    pub fn is_ended(&self) -> bool {
        if let Some(end_time) = self.end_time {
            Clock::get().unwrap().unix_timestamp >= end_time
        } else {
            false
        }
        
    }
}

#[derive(Accounts)]
pub struct CloseLottery<'info> {
    #[account(
        mut,
        has_one = authority,
        close = authority
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(mut)]
    pub authority: Signer<'info>,
    // pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The lottery has ended")]
    LotteryEnded,
    #[msg("The lottery is full")]
    LotteryFull,
    #[msg("The lottery funds are Insufficient")]
    InsufficientFunds,
    #[msg("The lottery hasn't ended yet")]
    LotteryNotEnded,
    #[msg("The current vault doesn't match the Vault")]
    InvalidVault,
    #[msg("The provided prize vault PDA does not match expected.")]
    InvalidVaultPda,
    #[msg("This account does not match an expected wallet.")]
    InvalidClaimerAccount,
    #[msg("Winner has already been drawn")]
    WinnerAlreadyDrawn,
    #[msg("Winner cannot be drawn at his time")]
    NoRecentBlockhash,
    #[msg("No participant to send Lamports found")]
    MissingParticipant,
    #[msg("No participants in the lottery")]
    NoParticipants,
    #[msg("Already a participant in the lottery")]
    AlreadyParticipating,
    #[msg("No winner has been drawn yet")]
    NoWinnerDrawn,
    #[msg("Prize has already been claimed")]
    AlreadyClaimed,
    #[msg("Only the winner can claim the prize")]
    NotWinner,
    #[msg("Only the authority can execute this lottery action.")]
    Unauthorized,
    #[msg("Randomness already provided")]
    RandomnessAlreadyProvided,
    #[msg("Dev Share percentages must add to 100")]
    InvalidShareSplit,
}
