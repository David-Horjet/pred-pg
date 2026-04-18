use crate::errors::CustomError;
use anchor_lang::prelude::*;

pub const MATH_PRECISION: u128 = 1_000_000; 

pub fn calculate_accuracy_score(
    prediction: u64,
    result: u64,
    max_accuracy_buffer: u64,
) -> Result<u64> {
    if result == 0 {
        return Ok(0);
    }

    let diff = if prediction > result {
        prediction - result
    } else {
        result - prediction
    };

    if max_accuracy_buffer > 0 && diff >= max_accuracy_buffer {
        return Ok(0);
    }

    let error_scaled = (diff as u128)
        .checked_mul(MATH_PRECISION)
        .ok_or(CustomError::MathOverflow)?
        .checked_div(result as u128)
        .ok_or(CustomError::MathOverflow)?;

    // AccuracyScore = 1 / (1 + 10 * Error)
    // Scaled to precision: (PRECISION^2) / (PRECISION + 10 * error_scaled)
    let denominator = MATH_PRECISION
        .checked_add(
            error_scaled
                .checked_mul(10)
                .ok_or(CustomError::MathOverflow)?,
        )
        .ok_or(CustomError::MathOverflow)?;

    let score = (MATH_PRECISION * MATH_PRECISION)
        .checked_div(denominator)
        .ok_or(CustomError::MathOverflow)?;

    Ok(score as u64)
}

pub fn calculate_time_bonus(
    start_time: i64,
    cutoff_time: i64,
    entry_time: i64,
) -> Result<u64> {
    if entry_time >= cutoff_time {
        return Ok(MATH_PRECISION as u64);
    }

    let total_duration = (cutoff_time - start_time) as u128;
    let time_from_start = (cutoff_time - entry_time) as u128;

    if total_duration == 0 {
        return Ok(MATH_PRECISION as u64);
    }

    // T = (cutoff_time - entry_time) / (cutoff_time - start_time)
    let t_scaled = time_from_start
        .checked_mul(MATH_PRECISION)
        .ok_or(CustomError::MathOverflow)?
        .checked_div(total_duration)
        .ok_or(CustomError::MathOverflow)?;

    // T^2
    let t_sq_scaled = t_scaled
        .checked_mul(t_scaled)
        .ok_or(CustomError::MathOverflow)?
        .checked_div(MATH_PRECISION)
        .ok_or(CustomError::MathOverflow)?;

    // TimeWeight = 1 + (1.5 * T^2)
    // Scaled: MATH_PRECISION + (1.5 * t_sq_scaled) 
    let bonus = t_sq_scaled
        .checked_mul(15)
        .ok_or(CustomError::MathOverflow)?
        .checked_div(10)
        .ok_or(CustomError::MathOverflow)?;

    let factor = MATH_PRECISION
        .checked_add(bonus)
        .ok_or(CustomError::MathOverflow)?;

    Ok(factor as u64)
}

pub fn calculate_conviction_bonus(update_count: u32) -> u64 {
    if update_count == 0 {
        1_500_000
    } else {
        1_000_000
    }
}

pub fn calculate_weight(
    stake: u64,
    accuracy_score_scaled: u64,
    time_bonus_scaled: u64,
    conviction_scaled: u64,
) -> Result<u128> {
    
    let stake_u128 = stake as u128;
    
    let raw_product = stake_u128
        .checked_mul(accuracy_score_scaled as u128).unwrap()
        .checked_mul(time_bonus_scaled as u128).unwrap()
        .checked_mul(conviction_scaled as u128).unwrap();

    let final_weight = raw_product
        .checked_div(MATH_PRECISION).unwrap()
        .checked_div(MATH_PRECISION).unwrap()
        .checked_div(MATH_PRECISION).unwrap();

    Ok(final_weight)
}