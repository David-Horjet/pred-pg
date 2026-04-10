pub mod delegation {
    use anchor_lang::prelude::*;
    use crate::state::{Bet, Protocol, Pool};
    use crate::constants::{SEED_BET, SEED_POOL, SEED_PROTOCOL};
    use crate::errors::CustomError;
    use crate::events::{PoolDelegated, PoolUndelegated, BetDelegated, BetUndelegated};
    use ephemeral_rollups_sdk::access_control::instructions::DelegatePermissionCpiBuilder;
    use ephemeral_rollups_sdk::anchor::{delegate, commit};
    use ephemeral_rollups_sdk::cpi::DelegateConfig;
    use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;
    #[instruction(pool_id:u64)]
    pub struct DelegatePool<'info> {
        #[account(mut)]
        pub admin: Signer<'info>,
        #[account(
            seeds = [SEED_PROTOCOL],
            bump,
            constraint = protocol.admin = = admin.key()@CustomError::Unauthorized
        )]
        pub protocol: Account<'info, Protocol>,
        /// CHECK: The buffer account
        #[account(
            mut,
            seeds = [ephemeral_rollups_sdk::pda::DELEGATE_BUFFER_TAG,
            pool.key().as_ref()],
            bump,
            seeds::program = crate::id()
        )]
        pub buffer_pool: AccountInfo<'info>,
        /// CHECK: The delegation record account
        #[account(
            mut,
            seeds = [ephemeral_rollups_sdk::pda::DELEGATION_RECORD_TAG,
            pool.key().as_ref()],
            bump,
            seeds::program = delegation_program.key()
        )]
        pub delegation_record_pool: AccountInfo<'info>,
        /// CHECK: The delegation metadata account
        #[account(
            mut,
            seeds = [ephemeral_rollups_sdk::pda::DELEGATION_METADATA_TAG,
            pool.key().as_ref()],
            bump,
            seeds::program = delegation_program.key()
        )]
        pub delegation_metadata_pool: AccountInfo<'info>,
        /// CHECK: The main pool account.
        #[account(
            mut,
            seeds = [SEED_POOL,
            admin.key().as_ref(),
            &pool_id.to_le_bytes()],
            bump
        )]
        pub pool: AccountInfo<'info>,
        /// CHECK: The MagicBlock Ephemeral Rollup Validator (TEE)
        pub validator: UncheckedAccount<'info>,
        /// CHECK: The owner program of the pda
        #[account(address = crate::id())]
        pub owner_program: AccountInfo<'info>,
        /// CHECK: The delegation program
        #[account(address = ephemeral_rollups_sdk::id())]
        pub delegation_program: AccountInfo<'info>,
        pub system_program: Program<'info, System>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info, DelegatePoolBumps> for DelegatePool<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >],
            __ix_data: &[u8],
            __bumps: &mut DelegatePoolBumps,
            __reallocs: &mut std::collections::BTreeSet<
                anchor_lang::solana_program::pubkey::Pubkey,
            >,
        ) -> anchor_lang::Result<Self> {
            let mut __ix_data = __ix_data;
            struct __Args {
                pool_id: u64,
            }
            impl borsh::ser::BorshSerialize for __Args
            where
                u64: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.pool_id, writer)?;
                    Ok(())
                }
            }
            impl borsh::de::BorshDeserialize for __Args
            where
                u64: borsh::BorshDeserialize,
            {
                fn deserialize_reader<R: borsh::maybestd::io::Read>(
                    reader: &mut R,
                ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                    Ok(Self {
                        pool_id: borsh::BorshDeserialize::deserialize_reader(reader)?,
                    })
                }
            }
            let __Args { pool_id } = __Args::deserialize(&mut __ix_data)
                .map_err(|_| {
                    anchor_lang::error::ErrorCode::InstructionDidNotDeserialize
                })?;
            let admin: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("admin"))?;
            let protocol: anchor_lang::accounts::account::Account<Protocol> = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("protocol"))?;
            let buffer_pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("buffer_pool"))?;
            let delegation_record_pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_record_pool"))?;
            let delegation_metadata_pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_metadata_pool"))?;
            let pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("pool"))?;
            let validator: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("validator"))?;
            let owner_program: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("owner_program"))?;
            let delegation_program: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_program"))?;
            let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("system_program"))?;
            if !AsRef::<AccountInfo>::as_ref(&admin).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("admin"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[SEED_PROTOCOL],
                &__program_id,
            );
            __bumps.protocol = __bump;
            if protocol.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("protocol")
                        .with_pubkeys((protocol.key(), __pda_address)),
                );
            }
            if !(protocol.admin == admin.key()) {
                return Err(
                    anchor_lang::error::Error::from(CustomError::Unauthorized)
                        .with_account_name("protocol"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[ephemeral_rollups_sdk::pda::DELEGATE_BUFFER_TAG, pool.key().as_ref()],
                &crate::id().key(),
            );
            __bumps.buffer_pool = __bump;
            if buffer_pool.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("buffer_pool")
                        .with_pubkeys((buffer_pool.key(), __pda_address)),
                );
            }
            if !&buffer_pool.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("buffer_pool"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[
                    ephemeral_rollups_sdk::pda::DELEGATION_RECORD_TAG,
                    pool.key().as_ref(),
                ],
                &delegation_program.key().key(),
            );
            __bumps.delegation_record_pool = __bump;
            if delegation_record_pool.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("delegation_record_pool")
                        .with_pubkeys((delegation_record_pool.key(), __pda_address)),
                );
            }
            if !&delegation_record_pool.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_record_pool"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[
                    ephemeral_rollups_sdk::pda::DELEGATION_METADATA_TAG,
                    pool.key().as_ref(),
                ],
                &delegation_program.key().key(),
            );
            __bumps.delegation_metadata_pool = __bump;
            if delegation_metadata_pool.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("delegation_metadata_pool")
                        .with_pubkeys((delegation_metadata_pool.key(), __pda_address)),
                );
            }
            if !&delegation_metadata_pool.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_metadata_pool"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[SEED_POOL, admin.key().as_ref(), &pool_id.to_le_bytes()],
                &__program_id,
            );
            __bumps.pool = __bump;
            if pool.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("pool")
                        .with_pubkeys((pool.key(), __pda_address)),
                );
            }
            if !&pool.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("pool"),
                );
            }
            {
                let actual = owner_program.key();
                let expected = crate::id();
                if actual != expected {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAddress,
                            )
                            .with_account_name("owner_program")
                            .with_pubkeys((actual, expected)),
                    );
                }
            }
            {
                let actual = delegation_program.key();
                let expected = ephemeral_rollups_sdk::id();
                if actual != expected {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAddress,
                            )
                            .with_account_name("delegation_program")
                            .with_pubkeys((actual, expected)),
                    );
                }
            }
            Ok(DelegatePool {
                admin,
                protocol,
                buffer_pool,
                delegation_record_pool,
                delegation_metadata_pool,
                pool,
                validator,
                owner_program,
                delegation_program,
                system_program,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for DelegatePool<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.admin.to_account_infos());
            account_infos.extend(self.protocol.to_account_infos());
            account_infos.extend(self.buffer_pool.to_account_infos());
            account_infos.extend(self.delegation_record_pool.to_account_infos());
            account_infos.extend(self.delegation_metadata_pool.to_account_infos());
            account_infos.extend(self.pool.to_account_infos());
            account_infos.extend(self.validator.to_account_infos());
            account_infos.extend(self.owner_program.to_account_infos());
            account_infos.extend(self.delegation_program.to_account_infos());
            account_infos.extend(self.system_program.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for DelegatePool<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.admin.to_account_metas(None));
            account_metas.extend(self.protocol.to_account_metas(None));
            account_metas.extend(self.buffer_pool.to_account_metas(None));
            account_metas.extend(self.delegation_record_pool.to_account_metas(None));
            account_metas.extend(self.delegation_metadata_pool.to_account_metas(None));
            account_metas.extend(self.pool.to_account_metas(None));
            account_metas.extend(self.validator.to_account_metas(None));
            account_metas.extend(self.owner_program.to_account_metas(None));
            account_metas.extend(self.delegation_program.to_account_metas(None));
            account_metas.extend(self.system_program.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for DelegatePool<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::Result<()> {
            anchor_lang::AccountsExit::exit(&self.admin, program_id)
                .map_err(|e| e.with_account_name("admin"))?;
            anchor_lang::AccountsExit::exit(&self.buffer_pool, program_id)
                .map_err(|e| e.with_account_name("buffer_pool"))?;
            anchor_lang::AccountsExit::exit(&self.delegation_record_pool, program_id)
                .map_err(|e| e.with_account_name("delegation_record_pool"))?;
            anchor_lang::AccountsExit::exit(&self.delegation_metadata_pool, program_id)
                .map_err(|e| e.with_account_name("delegation_metadata_pool"))?;
            anchor_lang::AccountsExit::exit(&self.pool, program_id)
                .map_err(|e| e.with_account_name("pool"))?;
            Ok(())
        }
    }
    pub struct DelegatePoolBumps {
        pub protocol: u8,
        pub buffer_pool: u8,
        pub delegation_record_pool: u8,
        pub delegation_metadata_pool: u8,
        pub pool: u8,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DelegatePoolBumps {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "DelegatePoolBumps",
                "protocol",
                &self.protocol,
                "buffer_pool",
                &self.buffer_pool,
                "delegation_record_pool",
                &self.delegation_record_pool,
                "delegation_metadata_pool",
                &self.delegation_metadata_pool,
                "pool",
                &&self.pool,
            )
        }
    }
    impl Default for DelegatePoolBumps {
        fn default() -> Self {
            DelegatePoolBumps {
                protocol: u8::MAX,
                buffer_pool: u8::MAX,
                delegation_record_pool: u8::MAX,
                delegation_metadata_pool: u8::MAX,
                pool: u8::MAX,
            }
        }
    }
    impl<'info> anchor_lang::Bumps for DelegatePool<'info>
    where
        'info: 'info,
    {
        type Bumps = DelegatePoolBumps;
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_delegate_pool {
        use super::*;
        use anchor_lang::prelude::borsh;
        /// Generated client accounts for [`DelegatePool`].
        pub struct DelegatePool {
            pub admin: Pubkey,
            pub protocol: Pubkey,
            pub buffer_pool: Pubkey,
            pub delegation_record_pool: Pubkey,
            pub delegation_metadata_pool: Pubkey,
            pub pool: Pubkey,
            pub validator: Pubkey,
            pub owner_program: Pubkey,
            pub delegation_program: Pubkey,
            pub system_program: Pubkey,
        }
        impl borsh::ser::BorshSerialize for DelegatePool
        where
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.admin, writer)?;
                borsh::BorshSerialize::serialize(&self.protocol, writer)?;
                borsh::BorshSerialize::serialize(&self.buffer_pool, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_record_pool, writer)?;
                borsh::BorshSerialize::serialize(
                    &self.delegation_metadata_pool,
                    writer,
                )?;
                borsh::BorshSerialize::serialize(&self.pool, writer)?;
                borsh::BorshSerialize::serialize(&self.validator, writer)?;
                borsh::BorshSerialize::serialize(&self.owner_program, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_program, writer)?;
                borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for DelegatePool {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.admin,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.protocol,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.buffer_pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_record_pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_metadata_pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.validator,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.owner_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.delegation_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.system_program,
                            false,
                        ),
                    );
                account_metas
            }
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a CPI struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is an
    /// AccountInfo.
    ///
    /// To access the struct in this module, one should use the sibling
    /// [`cpi::accounts`] module (also generated), which re-exports this.
    pub(crate) mod __cpi_client_accounts_delegate_pool {
        use super::*;
        /// Generated CPI struct of the accounts for [`DelegatePool`].
        pub struct DelegatePool<'info> {
            pub admin: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub protocol: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub buffer_pool: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_record_pool: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_metadata_pool: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub pool: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub validator: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub owner_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for DelegatePool<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.admin),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.protocol),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.buffer_pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_record_pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_metadata_pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.validator),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.owner_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.delegation_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.system_program),
                            false,
                        ),
                    );
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for DelegatePool<'info> {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.admin));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.protocol),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.buffer_pool),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_record_pool,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_metadata_pool,
                        ),
                    );
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.pool));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.validator),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.owner_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.system_program,
                        ),
                    );
                account_infos
            }
        }
    }
    impl<'info> DelegatePool<'info> {
        pub fn delegate_pool<'a>(
            &'a self,
            payer: &'a Signer<'info>,
            seeds: &[&[u8]],
            config: ephemeral_rollups_sdk::cpi::DelegateConfig,
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            let del_accounts = ephemeral_rollups_sdk::cpi::DelegateAccounts {
                payer,
                pda: &self.pool.to_account_info(),
                owner_program: &self.owner_program,
                buffer: &self.buffer_pool,
                delegation_record: &self.delegation_record_pool,
                delegation_metadata: &self.delegation_metadata_pool,
                delegation_program: &self.delegation_program,
                system_program: &self.system_program,
            };
            ephemeral_rollups_sdk::cpi::delegate_account(del_accounts, seeds, config)
        }
    }
    pub fn delegate_pool(ctx: Context<DelegatePool>, pool_id: u64) -> Result<()> {
        let admin_key = ctx.accounts.admin.key();
        let admin_bytes = admin_key.as_ref();
        let pool_id_bytes = pool_id.to_le_bytes();
        let seeds = &[SEED_POOL, admin_bytes, &pool_id_bytes];
        let config = DelegateConfig {
            validator: Some(ctx.accounts.validator.key()),
            ..DelegateConfig::default()
        };
        ctx.accounts.delegate_pool(&ctx.accounts.admin, seeds, config)?;
        {
            anchor_lang::solana_program::log::sol_log_data(
                &[
                    &anchor_lang::Event::data(
                        &PoolDelegated {
                            pool_address: ctx.accounts.pool.key(),
                        },
                    ),
                ],
            );
        };
        ::solana_msg::sol_log("Pool account delegated successfully.");
        Ok(())
    }
    pub struct DelegateBetPermission<'info> {
        /// The user who owns the bet — must sign to authorize delegation.
        #[account(mut)]
        pub user: Signer<'info>,
        /// Pays for any accounts the delegation program creates (buffer, record, metadata).
        /// Separated from user so users need zero SOL under the gas-sponsorship model.
        #[account(mut)]
        pub payer: Signer<'info>,
        /// CHECK: Manually validated against the bet's pool_identifier.
        pub pool: AccountInfo<'info>,
        /// CHECK: The user's bet account (The Permissioned Account)
        #[account(mut)]
        pub user_bet: AccountInfo<'info>,
        /// CHECK: The permission account associated with the user_bet.
        #[account(mut)]
        pub permission: UncheckedAccount<'info>,
        /// CHECK: The MagicBlock Permission Program
        pub permission_program: UncheckedAccount<'info>,
        /// CHECK: The MagicBlock Delegation Program
        pub delegation_program: UncheckedAccount<'info>,
        /// CHECK: Delegation buffer (Derived by client or SDK)
        #[account(mut)]
        pub delegation_buffer: UncheckedAccount<'info>,
        /// CHECK: Delegation record (Derived by client or SDK)
        #[account(mut)]
        pub delegation_record: UncheckedAccount<'info>,
        /// CHECK: Delegation metadata (Derived by client or SDK)
        #[account(mut)]
        pub delegation_metadata: UncheckedAccount<'info>,
        /// CHECK: The MagicBlock Ephemeral Rollup Validator (TEE)
        pub validator: UncheckedAccount<'info>,
        pub system_program: Program<'info, System>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info, DelegateBetPermissionBumps>
    for DelegateBetPermission<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >],
            __ix_data: &[u8],
            __bumps: &mut DelegateBetPermissionBumps,
            __reallocs: &mut std::collections::BTreeSet<
                anchor_lang::solana_program::pubkey::Pubkey,
            >,
        ) -> anchor_lang::Result<Self> {
            let user: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("user"))?;
            let payer: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("payer"))?;
            let pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("pool"))?;
            let user_bet: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("user_bet"))?;
            let permission: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("permission"))?;
            let permission_program: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("permission_program"))?;
            let delegation_program: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_program"))?;
            let delegation_buffer: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_buffer"))?;
            let delegation_record: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_record"))?;
            let delegation_metadata: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_metadata"))?;
            let validator: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("validator"))?;
            let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("system_program"))?;
            if !AsRef::<AccountInfo>::as_ref(&user).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("user"),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&payer).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("payer"),
                );
            }
            if !&user_bet.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("user_bet"),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&permission).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("permission"),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&delegation_buffer).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_buffer"),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&delegation_record).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_record"),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&delegation_metadata).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_metadata"),
                );
            }
            Ok(DelegateBetPermission {
                user,
                payer,
                pool,
                user_bet,
                permission,
                permission_program,
                delegation_program,
                delegation_buffer,
                delegation_record,
                delegation_metadata,
                validator,
                system_program,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for DelegateBetPermission<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.user.to_account_infos());
            account_infos.extend(self.payer.to_account_infos());
            account_infos.extend(self.pool.to_account_infos());
            account_infos.extend(self.user_bet.to_account_infos());
            account_infos.extend(self.permission.to_account_infos());
            account_infos.extend(self.permission_program.to_account_infos());
            account_infos.extend(self.delegation_program.to_account_infos());
            account_infos.extend(self.delegation_buffer.to_account_infos());
            account_infos.extend(self.delegation_record.to_account_infos());
            account_infos.extend(self.delegation_metadata.to_account_infos());
            account_infos.extend(self.validator.to_account_infos());
            account_infos.extend(self.system_program.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for DelegateBetPermission<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.user.to_account_metas(None));
            account_metas.extend(self.payer.to_account_metas(None));
            account_metas.extend(self.pool.to_account_metas(None));
            account_metas.extend(self.user_bet.to_account_metas(None));
            account_metas.extend(self.permission.to_account_metas(None));
            account_metas.extend(self.permission_program.to_account_metas(None));
            account_metas.extend(self.delegation_program.to_account_metas(None));
            account_metas.extend(self.delegation_buffer.to_account_metas(None));
            account_metas.extend(self.delegation_record.to_account_metas(None));
            account_metas.extend(self.delegation_metadata.to_account_metas(None));
            account_metas.extend(self.validator.to_account_metas(None));
            account_metas.extend(self.system_program.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for DelegateBetPermission<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::Result<()> {
            anchor_lang::AccountsExit::exit(&self.user, program_id)
                .map_err(|e| e.with_account_name("user"))?;
            anchor_lang::AccountsExit::exit(&self.payer, program_id)
                .map_err(|e| e.with_account_name("payer"))?;
            anchor_lang::AccountsExit::exit(&self.user_bet, program_id)
                .map_err(|e| e.with_account_name("user_bet"))?;
            anchor_lang::AccountsExit::exit(&self.permission, program_id)
                .map_err(|e| e.with_account_name("permission"))?;
            anchor_lang::AccountsExit::exit(&self.delegation_buffer, program_id)
                .map_err(|e| e.with_account_name("delegation_buffer"))?;
            anchor_lang::AccountsExit::exit(&self.delegation_record, program_id)
                .map_err(|e| e.with_account_name("delegation_record"))?;
            anchor_lang::AccountsExit::exit(&self.delegation_metadata, program_id)
                .map_err(|e| e.with_account_name("delegation_metadata"))?;
            Ok(())
        }
    }
    pub struct DelegateBetPermissionBumps {}
    #[automatically_derived]
    impl ::core::fmt::Debug for DelegateBetPermissionBumps {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "DelegateBetPermissionBumps")
        }
    }
    impl Default for DelegateBetPermissionBumps {
        fn default() -> Self {
            DelegateBetPermissionBumps {}
        }
    }
    impl<'info> anchor_lang::Bumps for DelegateBetPermission<'info>
    where
        'info: 'info,
    {
        type Bumps = DelegateBetPermissionBumps;
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_delegate_bet_permission {
        use super::*;
        use anchor_lang::prelude::borsh;
        /// Generated client accounts for [`DelegateBetPermission`].
        pub struct DelegateBetPermission {
            ///The user who owns the bet — must sign to authorize delegation.
            pub user: Pubkey,
            ///Pays for any accounts the delegation program creates (buffer, record, metadata).
            ///Separated from user so users need zero SOL under the gas-sponsorship model.
            pub payer: Pubkey,
            pub pool: Pubkey,
            pub user_bet: Pubkey,
            pub permission: Pubkey,
            pub permission_program: Pubkey,
            pub delegation_program: Pubkey,
            pub delegation_buffer: Pubkey,
            pub delegation_record: Pubkey,
            pub delegation_metadata: Pubkey,
            pub validator: Pubkey,
            pub system_program: Pubkey,
        }
        impl borsh::ser::BorshSerialize for DelegateBetPermission
        where
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.user, writer)?;
                borsh::BorshSerialize::serialize(&self.payer, writer)?;
                borsh::BorshSerialize::serialize(&self.pool, writer)?;
                borsh::BorshSerialize::serialize(&self.user_bet, writer)?;
                borsh::BorshSerialize::serialize(&self.permission, writer)?;
                borsh::BorshSerialize::serialize(&self.permission_program, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_program, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_buffer, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_record, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_metadata, writer)?;
                borsh::BorshSerialize::serialize(&self.validator, writer)?;
                borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for DelegateBetPermission {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.user,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.payer,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.user_bet,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.permission,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.permission_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.delegation_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_buffer,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_record,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_metadata,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.validator,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.system_program,
                            false,
                        ),
                    );
                account_metas
            }
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a CPI struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is an
    /// AccountInfo.
    ///
    /// To access the struct in this module, one should use the sibling
    /// [`cpi::accounts`] module (also generated), which re-exports this.
    pub(crate) mod __cpi_client_accounts_delegate_bet_permission {
        use super::*;
        /// Generated CPI struct of the accounts for [`DelegateBetPermission`].
        pub struct DelegateBetPermission<'info> {
            ///The user who owns the bet — must sign to authorize delegation.
            pub user: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            ///Pays for any accounts the delegation program creates (buffer, record, metadata).
            ///Separated from user so users need zero SOL under the gas-sponsorship model.
            pub payer: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub pool: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub user_bet: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub permission: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub permission_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_buffer: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_record: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_metadata: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub validator: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for DelegateBetPermission<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.user),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.payer),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.user_bet),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.permission),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.permission_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.delegation_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_buffer),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_record),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_metadata),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.validator),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.system_program),
                            false,
                        ),
                    );
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for DelegateBetPermission<'info> {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.user));
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.payer));
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.pool));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.user_bet),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.permission),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.permission_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_buffer,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_record,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_metadata,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.validator),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.system_program,
                        ),
                    );
                account_infos
            }
        }
    }
    pub fn delegate_bet_permission(
        ctx: Context<DelegateBetPermission>,
        _request_id: String,
    ) -> Result<()> {
        let (pool_pubkey, owner, bump) = {
            let user_bet_data = ctx.accounts.user_bet.try_borrow_data()?;
            let mut data_slice: &[u8] = &user_bet_data;
            let bet = Bet::try_deserialize(&mut data_slice)?;
            (bet.pool_pubkey, bet.user_pubkey, bet.bump)
        };
        if !(owner == ctx.accounts.user.key()) {
            return Err(
                anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: CustomError::Unauthorized.name(),
                    error_code_number: CustomError::Unauthorized.into(),
                    error_msg: CustomError::Unauthorized.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/swiv_privacy/src/instructions/delegation.rs",
                            line: 125u32,
                        }),
                    ),
                    compared_values: None,
                }),
            );
        }
        if !(pool_pubkey == ctx.accounts.pool.key()) {
            return Err(
                anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: CustomError::PoolMismatch.name(),
                    error_code_number: CustomError::PoolMismatch.into(),
                    error_msg: CustomError::PoolMismatch.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/swiv_privacy/src/instructions/delegation.rs",
                            line: 126u32,
                        }),
                    ),
                    compared_values: None,
                }),
            );
        }
        let pool_key = ctx.accounts.pool.key();
        let user_key = ctx.accounts.user.key();
        let seeds_for_signing = &[
            SEED_BET,
            pool_key.as_ref(),
            user_key.as_ref(),
            &[bump],
        ];
        let signer_seeds = &[&seeds_for_signing[..]];
        DelegatePermissionCpiBuilder::new(&ctx.accounts.permission_program)
            .payer(&ctx.accounts.payer)
            .authority(&ctx.accounts.user, false)
            .permissioned_account(&ctx.accounts.user_bet, true)
            .permission(&ctx.accounts.permission)
            .system_program(&ctx.accounts.system_program)
            .owner_program(&ctx.accounts.permission_program)
            .delegation_buffer(&ctx.accounts.delegation_buffer)
            .delegation_record(&ctx.accounts.delegation_record)
            .delegation_metadata(&ctx.accounts.delegation_metadata)
            .delegation_program(&ctx.accounts.delegation_program)
            .validator(Some(&ctx.accounts.validator))
            .invoke_signed(signer_seeds)?;
        ::solana_msg::sol_log("Permission account delegated successfully.");
        Ok(())
    }
    pub struct DelegateBet<'info> {
        /// The user who owns the bet — must sign to authorize delegation.
        #[account(mut)]
        pub user: Signer<'info>,
        /// Pays for any accounts the delegation SDK creates internally.
        /// Separated from user so users need zero SOL under the gas-sponsorship model.
        #[account(mut)]
        pub payer: Signer<'info>,
        /// CHECK: Manually validated against the bet's pool_identifier.
        pub pool: AccountInfo<'info>,
        /// CHECK: The buffer account
        #[account(
            mut,
            seeds = [ephemeral_rollups_sdk::pda::DELEGATE_BUFFER_TAG,
            user_bet.key().as_ref()],
            bump,
            seeds::program = crate::id()
        )]
        pub buffer_user_bet: AccountInfo<'info>,
        /// CHECK: The delegation record account
        #[account(
            mut,
            seeds = [ephemeral_rollups_sdk::pda::DELEGATION_RECORD_TAG,
            user_bet.key().as_ref()],
            bump,
            seeds::program = delegation_program.key()
        )]
        pub delegation_record_user_bet: AccountInfo<'info>,
        /// CHECK: The delegation metadata account
        #[account(
            mut,
            seeds = [ephemeral_rollups_sdk::pda::DELEGATION_METADATA_TAG,
            user_bet.key().as_ref()],
            bump,
            seeds::program = delegation_program.key()
        )]
        pub delegation_metadata_user_bet: AccountInfo<'info>,
        /// CHECK: The user's bet account.
        #[account(mut)]
        pub user_bet: AccountInfo<'info>,
        /// CHECK: The MagicBlock Ephemeral Rollup Validator (TEE)
        pub validator: UncheckedAccount<'info>,
        /// CHECK: The owner program of the pda
        #[account(address = crate::id())]
        pub owner_program: AccountInfo<'info>,
        /// CHECK: The delegation program
        #[account(address = ephemeral_rollups_sdk::id())]
        pub delegation_program: AccountInfo<'info>,
        pub system_program: Program<'info, System>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info, DelegateBetBumps> for DelegateBet<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >],
            __ix_data: &[u8],
            __bumps: &mut DelegateBetBumps,
            __reallocs: &mut std::collections::BTreeSet<
                anchor_lang::solana_program::pubkey::Pubkey,
            >,
        ) -> anchor_lang::Result<Self> {
            let user: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("user"))?;
            let payer: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("payer"))?;
            let pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("pool"))?;
            let buffer_user_bet: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("buffer_user_bet"))?;
            let delegation_record_user_bet: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_record_user_bet"))?;
            let delegation_metadata_user_bet: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_metadata_user_bet"))?;
            let user_bet: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("user_bet"))?;
            let validator: UncheckedAccount = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("validator"))?;
            let owner_program: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("owner_program"))?;
            let delegation_program: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("delegation_program"))?;
            let system_program: anchor_lang::accounts::program::Program<System> = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("system_program"))?;
            if !AsRef::<AccountInfo>::as_ref(&user).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("user"),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&payer).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("payer"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[
                    ephemeral_rollups_sdk::pda::DELEGATE_BUFFER_TAG,
                    user_bet.key().as_ref(),
                ],
                &crate::id().key(),
            );
            __bumps.buffer_user_bet = __bump;
            if buffer_user_bet.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("buffer_user_bet")
                        .with_pubkeys((buffer_user_bet.key(), __pda_address)),
                );
            }
            if !&buffer_user_bet.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("buffer_user_bet"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[
                    ephemeral_rollups_sdk::pda::DELEGATION_RECORD_TAG,
                    user_bet.key().as_ref(),
                ],
                &delegation_program.key().key(),
            );
            __bumps.delegation_record_user_bet = __bump;
            if delegation_record_user_bet.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("delegation_record_user_bet")
                        .with_pubkeys((delegation_record_user_bet.key(), __pda_address)),
                );
            }
            if !&delegation_record_user_bet.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_record_user_bet"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[
                    ephemeral_rollups_sdk::pda::DELEGATION_METADATA_TAG,
                    user_bet.key().as_ref(),
                ],
                &delegation_program.key().key(),
            );
            __bumps.delegation_metadata_user_bet = __bump;
            if delegation_metadata_user_bet.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("delegation_metadata_user_bet")
                        .with_pubkeys((
                            delegation_metadata_user_bet.key(),
                            __pda_address,
                        )),
                );
            }
            if !&delegation_metadata_user_bet.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("delegation_metadata_user_bet"),
                );
            }
            if !&user_bet.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("user_bet"),
                );
            }
            {
                let actual = owner_program.key();
                let expected = crate::id();
                if actual != expected {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAddress,
                            )
                            .with_account_name("owner_program")
                            .with_pubkeys((actual, expected)),
                    );
                }
            }
            {
                let actual = delegation_program.key();
                let expected = ephemeral_rollups_sdk::id();
                if actual != expected {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAddress,
                            )
                            .with_account_name("delegation_program")
                            .with_pubkeys((actual, expected)),
                    );
                }
            }
            Ok(DelegateBet {
                user,
                payer,
                pool,
                buffer_user_bet,
                delegation_record_user_bet,
                delegation_metadata_user_bet,
                user_bet,
                validator,
                owner_program,
                delegation_program,
                system_program,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for DelegateBet<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.user.to_account_infos());
            account_infos.extend(self.payer.to_account_infos());
            account_infos.extend(self.pool.to_account_infos());
            account_infos.extend(self.buffer_user_bet.to_account_infos());
            account_infos.extend(self.delegation_record_user_bet.to_account_infos());
            account_infos.extend(self.delegation_metadata_user_bet.to_account_infos());
            account_infos.extend(self.user_bet.to_account_infos());
            account_infos.extend(self.validator.to_account_infos());
            account_infos.extend(self.owner_program.to_account_infos());
            account_infos.extend(self.delegation_program.to_account_infos());
            account_infos.extend(self.system_program.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for DelegateBet<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.user.to_account_metas(None));
            account_metas.extend(self.payer.to_account_metas(None));
            account_metas.extend(self.pool.to_account_metas(None));
            account_metas.extend(self.buffer_user_bet.to_account_metas(None));
            account_metas.extend(self.delegation_record_user_bet.to_account_metas(None));
            account_metas
                .extend(self.delegation_metadata_user_bet.to_account_metas(None));
            account_metas.extend(self.user_bet.to_account_metas(None));
            account_metas.extend(self.validator.to_account_metas(None));
            account_metas.extend(self.owner_program.to_account_metas(None));
            account_metas.extend(self.delegation_program.to_account_metas(None));
            account_metas.extend(self.system_program.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for DelegateBet<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::Result<()> {
            anchor_lang::AccountsExit::exit(&self.user, program_id)
                .map_err(|e| e.with_account_name("user"))?;
            anchor_lang::AccountsExit::exit(&self.payer, program_id)
                .map_err(|e| e.with_account_name("payer"))?;
            anchor_lang::AccountsExit::exit(&self.buffer_user_bet, program_id)
                .map_err(|e| e.with_account_name("buffer_user_bet"))?;
            anchor_lang::AccountsExit::exit(&self.delegation_record_user_bet, program_id)
                .map_err(|e| e.with_account_name("delegation_record_user_bet"))?;
            anchor_lang::AccountsExit::exit(
                    &self.delegation_metadata_user_bet,
                    program_id,
                )
                .map_err(|e| e.with_account_name("delegation_metadata_user_bet"))?;
            anchor_lang::AccountsExit::exit(&self.user_bet, program_id)
                .map_err(|e| e.with_account_name("user_bet"))?;
            Ok(())
        }
    }
    pub struct DelegateBetBumps {
        pub buffer_user_bet: u8,
        pub delegation_record_user_bet: u8,
        pub delegation_metadata_user_bet: u8,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DelegateBetBumps {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "DelegateBetBumps",
                "buffer_user_bet",
                &self.buffer_user_bet,
                "delegation_record_user_bet",
                &self.delegation_record_user_bet,
                "delegation_metadata_user_bet",
                &&self.delegation_metadata_user_bet,
            )
        }
    }
    impl Default for DelegateBetBumps {
        fn default() -> Self {
            DelegateBetBumps {
                buffer_user_bet: u8::MAX,
                delegation_record_user_bet: u8::MAX,
                delegation_metadata_user_bet: u8::MAX,
            }
        }
    }
    impl<'info> anchor_lang::Bumps for DelegateBet<'info>
    where
        'info: 'info,
    {
        type Bumps = DelegateBetBumps;
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_delegate_bet {
        use super::*;
        use anchor_lang::prelude::borsh;
        /// Generated client accounts for [`DelegateBet`].
        pub struct DelegateBet {
            ///The user who owns the bet — must sign to authorize delegation.
            pub user: Pubkey,
            ///Pays for any accounts the delegation SDK creates internally.
            ///Separated from user so users need zero SOL under the gas-sponsorship model.
            pub payer: Pubkey,
            pub pool: Pubkey,
            pub buffer_user_bet: Pubkey,
            pub delegation_record_user_bet: Pubkey,
            pub delegation_metadata_user_bet: Pubkey,
            pub user_bet: Pubkey,
            pub validator: Pubkey,
            pub owner_program: Pubkey,
            pub delegation_program: Pubkey,
            pub system_program: Pubkey,
        }
        impl borsh::ser::BorshSerialize for DelegateBet
        where
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.user, writer)?;
                borsh::BorshSerialize::serialize(&self.payer, writer)?;
                borsh::BorshSerialize::serialize(&self.pool, writer)?;
                borsh::BorshSerialize::serialize(&self.buffer_user_bet, writer)?;
                borsh::BorshSerialize::serialize(
                    &self.delegation_record_user_bet,
                    writer,
                )?;
                borsh::BorshSerialize::serialize(
                    &self.delegation_metadata_user_bet,
                    writer,
                )?;
                borsh::BorshSerialize::serialize(&self.user_bet, writer)?;
                borsh::BorshSerialize::serialize(&self.validator, writer)?;
                borsh::BorshSerialize::serialize(&self.owner_program, writer)?;
                borsh::BorshSerialize::serialize(&self.delegation_program, writer)?;
                borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for DelegateBet {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.user,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.payer,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.buffer_user_bet,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_record_user_bet,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.delegation_metadata_user_bet,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.user_bet,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.validator,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.owner_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.delegation_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.system_program,
                            false,
                        ),
                    );
                account_metas
            }
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a CPI struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is an
    /// AccountInfo.
    ///
    /// To access the struct in this module, one should use the sibling
    /// [`cpi::accounts`] module (also generated), which re-exports this.
    pub(crate) mod __cpi_client_accounts_delegate_bet {
        use super::*;
        /// Generated CPI struct of the accounts for [`DelegateBet`].
        pub struct DelegateBet<'info> {
            ///The user who owns the bet — must sign to authorize delegation.
            pub user: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            ///Pays for any accounts the delegation SDK creates internally.
            ///Separated from user so users need zero SOL under the gas-sponsorship model.
            pub payer: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub pool: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub buffer_user_bet: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_record_user_bet: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_metadata_user_bet: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub user_bet: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub validator: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub owner_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub delegation_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub system_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for DelegateBet<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.user),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.payer),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.buffer_user_bet),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_record_user_bet),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.delegation_metadata_user_bet),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.user_bet),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.validator),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.owner_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.delegation_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.system_program),
                            false,
                        ),
                    );
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for DelegateBet<'info> {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.user));
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.payer));
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.pool));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.buffer_user_bet,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_record_user_bet,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_metadata_user_bet,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.user_bet),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.validator),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.owner_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.delegation_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.system_program,
                        ),
                    );
                account_infos
            }
        }
    }
    impl<'info> DelegateBet<'info> {
        pub fn delegate_user_bet<'a>(
            &'a self,
            payer: &'a Signer<'info>,
            seeds: &[&[u8]],
            config: ephemeral_rollups_sdk::cpi::DelegateConfig,
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            let del_accounts = ephemeral_rollups_sdk::cpi::DelegateAccounts {
                payer,
                pda: &self.user_bet.to_account_info(),
                owner_program: &self.owner_program,
                buffer: &self.buffer_user_bet,
                delegation_record: &self.delegation_record_user_bet,
                delegation_metadata: &self.delegation_metadata_user_bet,
                delegation_program: &self.delegation_program,
                system_program: &self.system_program,
            };
            ephemeral_rollups_sdk::cpi::delegate_account(del_accounts, seeds, config)
        }
    }
    pub fn delegate_bet(ctx: Context<DelegateBet>, request_id: String) -> Result<()> {
        let (pool_pubkey, owner) = {
            let user_bet_data = ctx.accounts.user_bet.try_borrow_data()?;
            let mut data_slice: &[u8] = &user_bet_data;
            let bet = Bet::try_deserialize(&mut data_slice)?;
            (bet.pool_pubkey, bet.user_pubkey)
        };
        if !(owner == ctx.accounts.user.key()) {
            return Err(
                anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: CustomError::Unauthorized.name(),
                    error_code_number: CustomError::Unauthorized.into(),
                    error_msg: CustomError::Unauthorized.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/swiv_privacy/src/instructions/delegation.rs",
                            line: 187u32,
                        }),
                    ),
                    compared_values: None,
                }),
            );
        }
        if !(pool_pubkey == ctx.accounts.pool.key()) {
            return Err(
                anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: CustomError::PoolMismatch.name(),
                    error_code_number: CustomError::PoolMismatch.into(),
                    error_msg: CustomError::PoolMismatch.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/swiv_privacy/src/instructions/delegation.rs",
                            line: 188u32,
                        }),
                    ),
                    compared_values: None,
                }),
            );
        }
        let pool_key = ctx.accounts.pool.key();
        let user_key = ctx.accounts.user.key();
        let seeds_for_sdk = &[SEED_BET, pool_key.as_ref(), user_key.as_ref()];
        let config = DelegateConfig {
            validator: Some(ctx.accounts.validator.key()),
            ..DelegateConfig::default()
        };
        ctx.accounts.delegate_user_bet(&ctx.accounts.payer, seeds_for_sdk, config)?;
        {
            anchor_lang::solana_program::log::sol_log_data(
                &[
                    &anchor_lang::Event::data(
                        &BetDelegated {
                            bet_address: ctx.accounts.user_bet.key(),
                            user: ctx.accounts.user.key(),
                            request_id,
                        },
                    ),
                ],
            );
        };
        ::solana_msg::sol_log("Bet delegated successfully.");
        Ok(())
    }
    pub struct UndelegatePool<'info> {
        #[account(mut)]
        pub admin: Signer<'info>,
        #[account(
            seeds = [SEED_PROTOCOL],
            bump,
            constraint = protocol.admin = = admin.key()@CustomError::Unauthorized
        )]
        pub protocol: Account<'info, Protocol>,
        /// CHECK: The Pool account
        #[account(mut)]
        pub pool: AccountInfo<'info>,
        pub magic_program: Program<'info, ephemeral_rollups_sdk::anchor::MagicProgram>,
        #[account(mut, address = ephemeral_rollups_sdk::consts::MAGIC_CONTEXT_ID)]
        /// CHECK:`
        pub magic_context: AccountInfo<'info>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info, UndelegatePoolBumps>
    for UndelegatePool<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >],
            __ix_data: &[u8],
            __bumps: &mut UndelegatePoolBumps,
            __reallocs: &mut std::collections::BTreeSet<
                anchor_lang::solana_program::pubkey::Pubkey,
            >,
        ) -> anchor_lang::Result<Self> {
            let admin: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("admin"))?;
            let protocol: anchor_lang::accounts::account::Account<Protocol> = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("protocol"))?;
            let pool: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("pool"))?;
            let magic_program: anchor_lang::accounts::program::Program<
                ephemeral_rollups_sdk::anchor::MagicProgram,
            > = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("magic_program"))?;
            let magic_context: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("magic_context"))?;
            if !AsRef::<AccountInfo>::as_ref(&admin).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("admin"),
                );
            }
            let (__pda_address, __bump) = Pubkey::find_program_address(
                &[SEED_PROTOCOL],
                &__program_id,
            );
            __bumps.protocol = __bump;
            if protocol.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("protocol")
                        .with_pubkeys((protocol.key(), __pda_address)),
                );
            }
            if !(protocol.admin == admin.key()) {
                return Err(
                    anchor_lang::error::Error::from(CustomError::Unauthorized)
                        .with_account_name("protocol"),
                );
            }
            if !&pool.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("pool"),
                );
            }
            if !&magic_context.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("magic_context"),
                );
            }
            {
                let actual = magic_context.key();
                let expected = ephemeral_rollups_sdk::consts::MAGIC_CONTEXT_ID;
                if actual != expected {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAddress,
                            )
                            .with_account_name("magic_context")
                            .with_pubkeys((actual, expected)),
                    );
                }
            }
            Ok(UndelegatePool {
                admin,
                protocol,
                pool,
                magic_program,
                magic_context,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for UndelegatePool<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.admin.to_account_infos());
            account_infos.extend(self.protocol.to_account_infos());
            account_infos.extend(self.pool.to_account_infos());
            account_infos.extend(self.magic_program.to_account_infos());
            account_infos.extend(self.magic_context.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for UndelegatePool<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.admin.to_account_metas(None));
            account_metas.extend(self.protocol.to_account_metas(None));
            account_metas.extend(self.pool.to_account_metas(None));
            account_metas.extend(self.magic_program.to_account_metas(None));
            account_metas.extend(self.magic_context.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for UndelegatePool<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::Result<()> {
            anchor_lang::AccountsExit::exit(&self.admin, program_id)
                .map_err(|e| e.with_account_name("admin"))?;
            anchor_lang::AccountsExit::exit(&self.pool, program_id)
                .map_err(|e| e.with_account_name("pool"))?;
            anchor_lang::AccountsExit::exit(&self.magic_context, program_id)
                .map_err(|e| e.with_account_name("magic_context"))?;
            Ok(())
        }
    }
    pub struct UndelegatePoolBumps {
        pub protocol: u8,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UndelegatePoolBumps {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UndelegatePoolBumps",
                "protocol",
                &&self.protocol,
            )
        }
    }
    impl Default for UndelegatePoolBumps {
        fn default() -> Self {
            UndelegatePoolBumps {
                protocol: u8::MAX,
            }
        }
    }
    impl<'info> anchor_lang::Bumps for UndelegatePool<'info>
    where
        'info: 'info,
    {
        type Bumps = UndelegatePoolBumps;
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_undelegate_pool {
        use super::*;
        use anchor_lang::prelude::borsh;
        /// Generated client accounts for [`UndelegatePool`].
        pub struct UndelegatePool {
            pub admin: Pubkey,
            pub protocol: Pubkey,
            pub pool: Pubkey,
            pub magic_program: Pubkey,
            pub magic_context: Pubkey,
        }
        impl borsh::ser::BorshSerialize for UndelegatePool
        where
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.admin, writer)?;
                borsh::BorshSerialize::serialize(&self.protocol, writer)?;
                borsh::BorshSerialize::serialize(&self.pool, writer)?;
                borsh::BorshSerialize::serialize(&self.magic_program, writer)?;
                borsh::BorshSerialize::serialize(&self.magic_context, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for UndelegatePool {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.admin,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.protocol,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.magic_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.magic_context,
                            false,
                        ),
                    );
                account_metas
            }
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a CPI struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is an
    /// AccountInfo.
    ///
    /// To access the struct in this module, one should use the sibling
    /// [`cpi::accounts`] module (also generated), which re-exports this.
    pub(crate) mod __cpi_client_accounts_undelegate_pool {
        use super::*;
        /// Generated CPI struct of the accounts for [`UndelegatePool`].
        pub struct UndelegatePool<'info> {
            pub admin: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub protocol: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub pool: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub magic_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub magic_context: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for UndelegatePool<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.admin),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.protocol),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.magic_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.magic_context),
                            false,
                        ),
                    );
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for UndelegatePool<'info> {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.admin));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(&self.protocol),
                    );
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.pool));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.magic_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.magic_context,
                        ),
                    );
                account_infos
            }
        }
    }
    pub fn undelegate_pool(ctx: Context<UndelegatePool>) -> Result<()> {
        commit_and_undelegate_accounts(
            &ctx.accounts.admin,
            <[_]>::into_vec(::alloc::boxed::box_new([&ctx.accounts.pool])),
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        {
            anchor_lang::solana_program::log::sol_log_data(
                &[
                    &anchor_lang::Event::data(
                        &PoolUndelegated {
                            pool_address: ctx.accounts.pool.key(),
                        },
                    ),
                ],
            );
        };
        Ok(())
    }
    pub struct BatchUndelegateBets<'info> {
        #[account(mut)]
        pub payer: Signer<'info>,
        #[account(
            mut,
            seeds = [SEED_POOL,
            pool.created_by.as_ref(),
            &(pool.pool_id.to_le_bytes())],
            bump = pool.bump
        )]
        pub pool: Account<'info, Pool>,
        pub magic_program: Program<'info, ephemeral_rollups_sdk::anchor::MagicProgram>,
        #[account(mut, address = ephemeral_rollups_sdk::consts::MAGIC_CONTEXT_ID)]
        /// CHECK:`
        pub magic_context: AccountInfo<'info>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info, BatchUndelegateBetsBumps>
    for BatchUndelegateBets<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            __program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            __accounts: &mut &'info [anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >],
            __ix_data: &[u8],
            __bumps: &mut BatchUndelegateBetsBumps,
            __reallocs: &mut std::collections::BTreeSet<
                anchor_lang::solana_program::pubkey::Pubkey,
            >,
        ) -> anchor_lang::Result<Self> {
            let payer: Signer = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("payer"))?;
            let pool: anchor_lang::accounts::account::Account<Pool> = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("pool"))?;
            let magic_program: anchor_lang::accounts::program::Program<
                ephemeral_rollups_sdk::anchor::MagicProgram,
            > = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("magic_program"))?;
            let magic_context: AccountInfo = anchor_lang::Accounts::try_accounts(
                    __program_id,
                    __accounts,
                    __ix_data,
                    __bumps,
                    __reallocs,
                )
                .map_err(|e| e.with_account_name("magic_context"))?;
            if !AsRef::<AccountInfo>::as_ref(&payer).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("payer"),
                );
            }
            let __pda_address = Pubkey::create_program_address(
                    &[
                        SEED_POOL,
                        pool.created_by.as_ref(),
                        &(pool.pool_id.to_le_bytes()),
                        &[pool.bump][..],
                    ],
                    &__program_id,
                )
                .map_err(|_| {
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("pool")
                })?;
            if pool.key() != __pda_address {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintSeeds,
                        )
                        .with_account_name("pool")
                        .with_pubkeys((pool.key(), __pda_address)),
                );
            }
            if !AsRef::<AccountInfo>::as_ref(&pool).is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("pool"),
                );
            }
            if !&magic_context.is_writable {
                return Err(
                    anchor_lang::error::Error::from(
                            anchor_lang::error::ErrorCode::ConstraintMut,
                        )
                        .with_account_name("magic_context"),
                );
            }
            {
                let actual = magic_context.key();
                let expected = ephemeral_rollups_sdk::consts::MAGIC_CONTEXT_ID;
                if actual != expected {
                    return Err(
                        anchor_lang::error::Error::from(
                                anchor_lang::error::ErrorCode::ConstraintAddress,
                            )
                            .with_account_name("magic_context")
                            .with_pubkeys((actual, expected)),
                    );
                }
            }
            Ok(BatchUndelegateBets {
                payer,
                pool,
                magic_program,
                magic_context,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for BatchUndelegateBets<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.payer.to_account_infos());
            account_infos.extend(self.pool.to_account_infos());
            account_infos.extend(self.magic_program.to_account_infos());
            account_infos.extend(self.magic_context.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for BatchUndelegateBets<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.payer.to_account_metas(None));
            account_metas.extend(self.pool.to_account_metas(None));
            account_metas.extend(self.magic_program.to_account_metas(None));
            account_metas.extend(self.magic_context.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for BatchUndelegateBets<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::Result<()> {
            anchor_lang::AccountsExit::exit(&self.payer, program_id)
                .map_err(|e| e.with_account_name("payer"))?;
            anchor_lang::AccountsExit::exit(&self.pool, program_id)
                .map_err(|e| e.with_account_name("pool"))?;
            anchor_lang::AccountsExit::exit(&self.magic_context, program_id)
                .map_err(|e| e.with_account_name("magic_context"))?;
            Ok(())
        }
    }
    pub struct BatchUndelegateBetsBumps {}
    #[automatically_derived]
    impl ::core::fmt::Debug for BatchUndelegateBetsBumps {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "BatchUndelegateBetsBumps")
        }
    }
    impl Default for BatchUndelegateBetsBumps {
        fn default() -> Self {
            BatchUndelegateBetsBumps {}
        }
    }
    impl<'info> anchor_lang::Bumps for BatchUndelegateBets<'info>
    where
        'info: 'info,
    {
        type Bumps = BatchUndelegateBetsBumps;
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_batch_undelegate_bets {
        use super::*;
        use anchor_lang::prelude::borsh;
        /// Generated client accounts for [`BatchUndelegateBets`].
        pub struct BatchUndelegateBets {
            pub payer: Pubkey,
            pub pool: Pubkey,
            pub magic_program: Pubkey,
            pub magic_context: Pubkey,
        }
        impl borsh::ser::BorshSerialize for BatchUndelegateBets
        where
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
            Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.payer, writer)?;
                borsh::BorshSerialize::serialize(&self.pool, writer)?;
                borsh::BorshSerialize::serialize(&self.magic_program, writer)?;
                borsh::BorshSerialize::serialize(&self.magic_context, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for BatchUndelegateBets {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.payer,
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.pool,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            self.magic_program,
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            self.magic_context,
                            false,
                        ),
                    );
                account_metas
            }
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a CPI struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is an
    /// AccountInfo.
    ///
    /// To access the struct in this module, one should use the sibling
    /// [`cpi::accounts`] module (also generated), which re-exports this.
    pub(crate) mod __cpi_client_accounts_batch_undelegate_bets {
        use super::*;
        /// Generated CPI struct of the accounts for [`BatchUndelegateBets`].
        pub struct BatchUndelegateBets<'info> {
            pub payer: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub pool: anchor_lang::solana_program::account_info::AccountInfo<'info>,
            pub magic_program: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
            pub magic_context: anchor_lang::solana_program::account_info::AccountInfo<
                'info,
            >,
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountMetas for BatchUndelegateBets<'info> {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.payer),
                            true,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.pool),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                            anchor_lang::Key::key(&self.magic_program),
                            false,
                        ),
                    );
                account_metas
                    .push(
                        anchor_lang::solana_program::instruction::AccountMeta::new(
                            anchor_lang::Key::key(&self.magic_context),
                            false,
                        ),
                    );
                account_metas
            }
        }
        #[automatically_derived]
        impl<'info> anchor_lang::ToAccountInfos<'info> for BatchUndelegateBets<'info> {
            fn to_account_infos(
                &self,
            ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
                let mut account_infos = ::alloc::vec::Vec::new();
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.payer));
                account_infos
                    .extend(anchor_lang::ToAccountInfos::to_account_infos(&self.pool));
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.magic_program,
                        ),
                    );
                account_infos
                    .extend(
                        anchor_lang::ToAccountInfos::to_account_infos(
                            &self.magic_context,
                        ),
                    );
                account_infos
            }
        }
    }
    pub fn batch_undelegate_bets<'info>(
        ctx: Context<'_, '_, '_, 'info, BatchUndelegateBets<'info>>,
    ) -> Result<()> {
        let pool = &ctx.accounts.pool;
        let clock = Clock::get()?;
        if !(clock.unix_timestamp >= pool.end_time) {
            return Err(
                anchor_lang::error::Error::from(anchor_lang::error::AnchorError {
                    error_name: CustomError::UndelegationTooEarly.name(),
                    error_code_number: CustomError::UndelegationTooEarly.into(),
                    error_msg: CustomError::UndelegationTooEarly.to_string(),
                    error_origin: Some(
                        anchor_lang::error::ErrorOrigin::Source(anchor_lang::error::Source {
                            filename: "programs/swiv_privacy/src/instructions/delegation.rs",
                            line: 271u32,
                        }),
                    ),
                    compared_values: None,
                }),
            );
        }
        let accounts_to_undelegate: Vec<&AccountInfo<'info>> = ctx
            .remaining_accounts
            .iter()
            .collect();
        if accounts_to_undelegate.is_empty() {
            return Ok(());
        }
        commit_and_undelegate_accounts(
            &ctx.accounts.payer,
            accounts_to_undelegate,
            &ctx.accounts.magic_context,
            &ctx.accounts.magic_program,
        )?;
        for acc in ctx.remaining_accounts.iter() {
            {
                anchor_lang::solana_program::log::sol_log_data(
                    &[
                        &anchor_lang::Event::data(
                            &BetUndelegated {
                                bet_address: acc.key(),
                                user: Pubkey::default(),
                                is_batch: true,
                            },
                        ),
                    ],
                );
            };
        }
        ::solana_msg::sol_log(
            &::alloc::__export::must_use({
                ::alloc::fmt::format(
                    format_args!(
                        "Batch Undelegate executed for {0} bets.",
                        ctx.remaining_accounts.len(),
                    ),
                )
            }),
        );
        Ok(())
    }
}
