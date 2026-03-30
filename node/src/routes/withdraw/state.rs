use mugraph_core::{
    error::Error,
    types::{Signature, WithdrawRequest, WithdrawalRecord, WithdrawalStatus},
};
use redb::ReadableTable;

use crate::{
    database::{DEPOSITS, NOTES, WITHDRAWALS},
    routes::Context,
};

pub(super) fn atomic_burn_and_record_pending(
    request: &WithdrawRequest,
    ctx: &Context,
    tx_hash: &str,
) -> Result<(), Error> {
    let network_byte = ctx.config.network_byte();
    let key = crate::tx_ids::parse_withdrawal_key(tx_hash, network_byte)?;

    {
        let read_tx = ctx.database.read()?;
        let withdrawals = read_tx.open_table(WITHDRAWALS)?;
        if let Some(existing) = withdrawals.get(&key)?
            && existing.value().status == WithdrawalStatus::Failed
        {
            let write_tx = ctx.database.write()?;
            {
                let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;
                withdrawals_table.insert(&key, &WithdrawalRecord::pending())?;
            }
            write_tx.commit()?;
            tracing::info!(
                "Reused failed withdrawal state for retry without reburning notes: {}",
                &tx_hash[..std::cmp::min(16, tx_hash.len())]
            );
            return Ok(());
        }
    }

    let write_tx = ctx.database.write()?;
    {
        let mut notes_table = write_tx.open_table(NOTES)?;

        for note in &request.notes {
            let sig_bytes: &[u8; 32] = note.signature.0.as_ref();
            let signature = Signature::from(*sig_bytes);

            if notes_table.get(signature)?.is_some() {
                return Err(Error::AlreadySpent { signature });
            }

            notes_table.insert(signature, true)?;
        }

        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;
        withdrawals_table.insert(&key, &WithdrawalRecord::pending())?;
    }

    write_tx.commit()?;

    tracing::info!(
        "Burned {} notes and recorded pending withdrawal {}",
        request.notes.len(),
        &tx_hash[..std::cmp::min(16, tx_hash.len())]
    );

    Ok(())
}

pub(super) fn mark_withdrawal_failed(
    ctx: &Context,
    tx_hash: &str,
) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;

    {
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;

        let network_byte = ctx.config.network_byte();
        let key = crate::tx_ids::parse_withdrawal_key(tx_hash, network_byte)?;
        let record = WithdrawalRecord::failed();
        withdrawals_table.insert(key, &record)?;
    }

    write_tx.commit()?;
    Ok(())
}

pub(super) fn mark_withdrawal_completed(
    ctx: &Context,
    tx_hash: &str,
    consumed_deposits: &[mugraph_core::types::UtxoRef],
) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;

    {
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;

        let network_byte = ctx.config.network_byte();
        let key = crate::tx_ids::parse_withdrawal_key(tx_hash, network_byte)?;

        let existing = withdrawals_table.get(&key)?.map(|v| v.value());
        let Some(existing) = existing else {
            return Err(Error::InvalidInput {
                reason: "Pending withdrawal not found for completion"
                    .to_string(),
            });
        };

        if existing.status == WithdrawalStatus::Completed {
            return Err(Error::InvalidInput {
                reason: "Withdrawal already completed".to_string(),
            });
        }

        withdrawals_table.insert(key, &WithdrawalRecord::completed())?;

        let mut deposits_table = write_tx.open_table(DEPOSITS)?;
        for utxo_ref in consumed_deposits {
            let existing_record =
                deposits_table.get(utxo_ref)?.map(|v| v.value());
            if let Some(mut record) = existing_record {
                record.spent = true;
                deposits_table.insert(utxo_ref, &record)?;
            }
        }
    }

    write_tx.commit()?;

    tracing::info!("Marked withdrawal {} as completed", tx_hash);

    Ok(())
}
