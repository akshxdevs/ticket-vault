use anchor_lang::prelude::*;
use anchor_spl::token::{self,Token,Transfer};
use anchor_lang::solana_program::{program_error::ProgramError};
use anchor_lang::solana_program::hash::{self};
declare_id!("6httayQ9toKM8tKjSg5p4AJG9Z4s7zWPELY2BVNizWkL");

#[program]
pub mod anchor_ticketvault {
    use super::*;
    pub fn initialize_event(
        ctx: Context<Initialize>,
        total_tickets_issued:u32,
        event_details: String,
        ticket_fee:u64,
        amount: u64,
    ) -> Result<()> {
        let event = &mut ctx.accounts.event;

        event.creator = ctx.accounts.creator.key();
        event.amount = amount;
        event.event_details = event_details;
        event.bump = ctx.bumps.event;
        event.event_ticket_available = true;
        event.total_tickets_issued = total_tickets_issued;
        event.total_tickets_sold = 0;
        event.ticket_fee = ticket_fee;
        event.enrolled_pubkeys = vec![];
        event.enrolled_pubkeys_count = 0;
        event.event_start_time = Clock::get()?.unix_timestamp + 3600; // 1 hour from now
        event.seat_no = 0;
        event.ticket_id = [0; 16];
        Ok(())
    }
    pub fn encroll_event(ctx: Context<EnrollEvent>) -> Result<()>{
        let vault = &mut ctx.accounts.vault;
        let event = &mut ctx.accounts.event;
        let ticket = &mut ctx.accounts.ticket;
        
        // Check if event is still available
        require!(event.event_ticket_available, CustomError::AllTicketsSold);
        
        // Check if user is already enrolled
        require!(!event.enrolled_pubkeys.iter().any(|pk| *pk == ctx.accounts.user.key()), CustomError::AlreadyEnrolled);
        
        vault.owner = ctx.accounts.user.key();
        vault.bump = ctx.bumps.vault;

        let now = Clock::get()?.unix_timestamp;
        
        // Check if event has started
        require!(now < event.event_start_time, CustomError::EventAlreadyStarted);

        // Generate seat selection
        let seat_options = vec!["A1", "B12", "103", "C7", "D20"];
        let seed_data = [
            ctx.accounts.user.key().as_ref(),
            &now.to_le_bytes(),
        ].concat();
        let hash = hash::hash(&seed_data);
        let random_index = (hash.to_bytes()[0] as usize) % seat_options.len();
        let rn_seat = seat_options[random_index];

        // Generate ticket ID
        let ticket_id: [u8; 16] = hash.to_bytes()[..16]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        msg!("Generated Ticket ID: {:?}", ticket_id);

        // Check ticket fee
        require!(event.amount > 0, CustomError::AmountNotEqualToTicketFee);
        
        // Transfer tokens to vault from user using user's signature
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer{
              from: ctx.accounts.user_ata.to_account_info(),
              to: ctx.accounts.vault_ata.to_account_info(),
              authority: ctx.accounts.user.to_account_info()  
            },
        ); 
        token::transfer(cpi_ctx, event.amount)?;
        
        // Update event and ticket data
        event.ticket_id = ticket_id;
        let seat_number = match rn_seat {
            "A1" => 1,
            "B12" => 12,
            "103" => 103,
            "C7" => 7,
            "D20" => 20,
            _ => 1,
        };
        event.seat_no = seat_number;
        event.total_tickets_sold += 1;
        event.enrolled_pubkeys.push(ctx.accounts.user.key());
        event.enrolled_pubkeys_count += 1;
        
        // Check if all tickets are sold
        if event.total_tickets_sold >= event.total_tickets_issued {
            event.event_ticket_available = false;
        }
        
        // Update ticket metadata derived from event BEFORE storing details
        event.ticket_type = match event.amount  {
            1_000_000_000 => TicketType::General,
            x if x >= 10_000_000_000 => TicketType::VIP,
            _ => TicketType::Backstage, 
        };
        
        // Update ticket details
        ticket.user = ctx.accounts.user.key();
        ticket.event = event.key();
        ticket.claimed = false;
        ticket.details = TicketDetails{
            ticket_id: event.ticket_id,
            event_details: event.event_details.clone(),
            event_start_time: event.event_start_time,
            seat_no: event.seat_no,
            amount: event.amount,
            enrolled_pubkeys: event.enrolled_pubkeys.clone(),
            ticket_type: event.ticket_type.clone(),
        };
        
        Ok(())
    }
    pub fn claim_ticket (ctx: Context<ClaimTicket>) -> Result<()> {
        let ticket = &mut ctx.accounts.ticket;
        let event = &mut ctx.accounts.event;
        ticket.bump = ctx.bumps.ticket;
        
        // Check if user is enrolled for this event
        if !event.enrolled_pubkeys.iter().any(|pk| *pk == ctx.accounts.user.key()) {
            msg!("User is not enrolled for this event.");
            return Err(CustomError::NotEnrolled.into());
        }
        
        // Mark ticket as claimed
        ticket.claimed = true;
        msg!("User ticket details: {:#?}", ticket.details);
        
        Ok(())
    }
}

#[account]
pub struct Event {
    pub creator: Pubkey,
    pub bump: u8,
    pub event_details: String,
    pub event_ticket_available: bool,
    pub ticket_id: [u8; 16],
    pub total_tickets_issued: u32,
    pub total_tickets_sold: u32,
    pub ticket_type: TicketType,
    pub enrolled_pubkeys: Vec<Pubkey>,
    pub enrolled_pubkeys_count: u32,
    pub event_start_time: i64,
    pub seat_no: u32,
    pub amount: u64,
    pub ticket_fee:u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [b"event",creator.key().as_ref()],
        payer = creator,
        space = 8 + 32 + 1 + 260 + 1 + 16 + 4 + 1 + 3204 + 4 + 8 + 4 + 4,
        bump
    )]
    pub event: Account<'info, Event>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[account]
pub struct Vault{
    pub owner:Pubkey,
    pub bump:u8
}
#[account]
pub struct Ticket{
    pub event: Pubkey,
    pub user: Pubkey,
    pub claimed:bool,
    pub details:TicketDetails,
    pub bump:u8
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct TicketDetails{
    pub event_details: String,
    pub ticket_id: [u8; 16],
    pub ticket_type: TicketType,
    pub enrolled_pubkeys: Vec<Pubkey>,
    pub event_start_time: i64,
    pub seat_no: u32,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct EnrollEvent<'info> {
    #[account(
        init_if_needed,
        seeds = [b"ticket", event.key().as_ref(), user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32 + 32 + 1 + 260 + 16 + 1 + 3204 + 8 + 4 + 8 + 1, // Event + User + Claimed + Details + Bump
    )]
    pub ticket: Account<'info,Ticket>,
    #[account(mut)]
    pub event: Account<'info, Event>,
    #[account(
        init_if_needed,
        seeds = [b"vault", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32 + 1, 
    )]
    pub vault: Account<'info, Vault>,
    ///CHECKS vault token account
    #[account(mut)]
    pub vault_ata: AccountInfo<'info>,
    ///CHECKS user token account
    #[account(mut)]
    pub user_ata: AccountInfo<'info>,
    ///CHECKS creator token account
    #[account(mut)]
    pub creator_ata: AccountInfo<'info>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    ///CHECKS token program account
    pub token_program:Program<'info,Token>
}

#[derive(Accounts)]
pub struct ClaimTicket<'info>{
    #[account(mut)]
    pub event: Account<'info,Event>,
    #[account(
        init_if_needed,
        seeds = [b"ticket",event.key().as_ref(),user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32 + 32 + 1 + 260 + 16 + 1 + 3204 + 8 + 4 + 8 + 1, // Event + User + Claimed + Details + Bump
    )]
    pub ticket:Account<'info,Ticket>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq,Debug)]
pub enum TicketType {   
    General,
    VIP,
    Backstage,
}

#[error_code]
pub enum CustomError {
    #[msg("Event started already started minutes ago")]
    EventAlreadyStarted,
    #[msg("Amount should be equal to ticket fee")]
    AmountNotEqualToTicketFee,
    #[msg("All tickets are sold out")]
    AllTicketsSold,
    #[msg("User is already enrolled for this event.")]
    AlreadyEnrolled,
    #[msg("Account not initialized.")]
    AccountNotInitialized,
    #[msg("User is not enrolled for this event.")]
    NotEnrolled,
}