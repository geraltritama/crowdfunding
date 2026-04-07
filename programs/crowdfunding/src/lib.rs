use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    clock::Clock,
    program::invoke_signed,
    rent::Rent,
    system_instruction,
};

declare_id!("F3k9VhRhE9JwkgRNryjkzu7CL3z8epdLc3hHxe2Bg5gu");

#[program]
pub mod crowdfunding {
    use super::*;

    /// Create Campaign
    pub fn create_campaign(
        ctx: Context<CreateCampaign>,
        goal: u64,
        deadline: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        if deadline <= current_time {
            return Err(CrowdError::InvalidDeadline.into());
        }

        let campaign = &mut ctx.accounts.campaign;
        campaign.creator = ctx.accounts.creator.key();
        campaign.goal = goal;
        campaign.raised = 0;
        campaign.deadline = deadline;
        campaign.claimed = false;

        // Create the vault PDA as a system-owned account with 0 data.
        // Anchor v1.0.0 can't `init` a `SystemAccount`, so we do it manually.
        let campaign_key = campaign.key();
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", campaign_key.as_ref()],
            ctx.program_id,
        );
        require_keys_eq!(vault_pda, ctx.accounts.vault.key(), CrowdError::InvalidVault);

        // Only create if it doesn't exist yet.
        if ctx.accounts.vault.lamports() == 0 {
            let rent = Rent::get()?;
            let lamports = rent.minimum_balance(0);

            let ix = system_instruction::create_account(
                &ctx.accounts.creator.key(),
                &vault_pda,
                lamports,
                0,
                &System::id(),
            );
            let seeds: &[&[u8]] = &[b"vault", campaign_key.as_ref(), &[vault_bump]];
            invoke_signed(
                &ix,
                &[
                    ctx.accounts.creator.to_account_info(),
                    ctx.accounts.vault.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[seeds],
            )?;
        }

        msg!("Campaign created: goal={}, deadline={}", goal, deadline);

        Ok(())
    }

    /// Contribute
    pub fn contribute(ctx: Context<Contribute>, amount: u64) -> Result<()> {
        if amount == 0 {
            return Err(CrowdError::ZeroAmount.into());
        }

        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        let campaign = &mut ctx.accounts.campaign;
        if current_time >= campaign.deadline {
            return Err(CrowdError::CampaignEnded.into());
        }

        let contributor = &ctx.accounts.contributor;
        let vault = &ctx.accounts.vault;
        let system_program = &ctx.accounts.system_program;

        // Transfer SOL from contributor to vault PDA
        let ix = system_instruction::transfer(
            &contributor.key(),
            &vault.key(),
            amount,
        );
        invoke_signed(
            &ix,
            &[
                contributor.to_account_info(),
                vault.to_account_info(),
                system_program.to_account_info(),
            ],
            &[],
        )?;

        // Update campaign totals (checked math)
        campaign.raised = campaign
            .raised
            .checked_add(amount)
            .ok_or(CrowdError::MathOverflow)?;

        msg!(
            "Contributed: {} lamports, total={}",
            amount,
            campaign.raised
        );

        Ok(())
    }

    /// Withdraw (creator claims funds if campaign succeeded)
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        let campaign = &mut ctx.accounts.campaign;
        let creator = &ctx.accounts.creator;
        let vault = &mut ctx.accounts.vault;
        let system_program = &ctx.accounts.system_program;

        if campaign.claimed {
            return Err(CrowdError::AlreadyClaimed.into());
        }

        if current_time < campaign.deadline {
            return Err(CrowdError::TooEarly.into());
        }

        if campaign.raised < campaign.goal {
            return Err(CrowdError::GoalNotReached.into());
        }

        let amount = vault.lamports();
        if amount == 0 {
            return Err(CrowdError::NothingToWithdraw.into());
        }

        // Transfer all lamports from vault PDA to creator
        let vault_key = vault.key();
        let campaign_key = campaign.key();
        let (expected_vault, vault_bump) =
            Pubkey::find_program_address(&[b"vault", campaign_key.as_ref()], ctx.program_id);
        require_keys_eq!(expected_vault, vault_key, CrowdError::InvalidVault);
        let seeds: &[&[u8]] = &[b"vault", campaign_key.as_ref(), &[vault_bump]];

        // System transfer from vault to creator using invoke_signed
        let ix = system_instruction::transfer(&vault_key, &creator.key(), amount);
        invoke_signed(
            &ix,
            &[
                vault.to_account_info(),
                creator.to_account_info(),
                system_program.to_account_info(),
            ],
            &[seeds],
        )?;

        campaign.claimed = true;

        msg!("Withdrawn: {} lamports", amount);

        Ok(())
    }

    /// Refund (donor gets money back if campaign failed)
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        let campaign = &mut ctx.accounts.campaign;
        let contributor = &ctx.accounts.contributor;
        let vault = &mut ctx.accounts.vault;
        let system_program = &ctx.accounts.system_program;

        if campaign.claimed {
            return Err(CrowdError::AlreadyClaimed.into());
        }

        if current_time < campaign.deadline {
            return Err(CrowdError::TooEarly.into());
        }

        if campaign.raised >= campaign.goal {
            return Err(CrowdError::GoalReachedNoRefund.into());
        }

        let amount = vault.lamports();
        if amount == 0 {
            return Err(CrowdError::NothingToRefund.into());
        }

        // Transfer all vault lamports back to the caller (refund path).
        let vault_key = vault.key();
        let campaign_key = campaign.key();
        let (expected_vault, vault_bump) =
            Pubkey::find_program_address(&[b"vault", campaign_key.as_ref()], ctx.program_id);
        require_keys_eq!(expected_vault, vault_key, CrowdError::InvalidVault);
        let seeds: &[&[u8]] = &[b"vault", campaign_key.as_ref(), &[vault_bump]];

        let ix = system_instruction::transfer(&vault_key, &contributor.key(), amount);
        invoke_signed(
            &ix,
            &[
                vault.to_account_info(),
                contributor.to_account_info(),
                system_program.to_account_info(),
            ],
            &[seeds],
        )?;

        // Update state
        campaign.raised = 0;
        campaign.claimed = true;

        msg!("Refunded: {} lamports", amount);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateCampaign<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Campaign account (client picks address, no PDA needed)
    #[account(
        init,
        payer = creator,
        space = 8 + Campaign::LEN
    )]
    pub campaign: Account<'info, Campaign>,

    /// Vault PDA (holds SOL). Created manually in handler.
    /// CHECK: PDA checked against seeds in handler.
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Contribute<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    /// Vault PDA derived from campaign
    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    /// CHECK: PDA checked by seeds/bump.
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(mut, has_one = creator)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    /// CHECK: PDA checked by seeds/bump.
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

    #[account(mut)]
    pub campaign: Account<'info, Campaign>,

    #[account(
        mut,
        seeds = [b"vault", campaign.key().as_ref()],
        bump
    )]
    /// CHECK: PDA checked by seeds/bump.
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Campaign {
    pub creator: Pubkey, // Who created this
    pub goal: u64,       // Target amount
    pub raised: u64,     // Current amount
    pub deadline: i64,   // When it ends
    pub claimed: bool,   // Already withdrawn?
}

impl Campaign {
    pub const LEN: usize =
        32 + // creator
        8 +  // goal
        8 +  // raised
        8 +  // deadline
        1;   // claimed
}

#[error_code]
pub enum CrowdError {
    #[msg("Deadline must be in the future")]
    InvalidDeadline,
    #[msg("Campaign has already ended")]
    CampaignEnded,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Campaign already claimed")]
    AlreadyClaimed,
    #[msg("Too early to withdraw or refund, deadline not reached")]
    TooEarly,
    #[msg("Campaign goal not reached")]
    GoalNotReached,
    #[msg("Nothing to withdraw")]
    NothingToWithdraw,
    #[msg("Goal reached, refunds not allowed")]
    GoalReachedNoRefund,
    #[msg("Nothing to refund")]
    NothingToRefund,
    #[msg("Invalid vault PDA provided")]
    InvalidVault,
}