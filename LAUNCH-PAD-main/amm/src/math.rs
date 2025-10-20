use anchor_lang::prelude::*;
use crate::errors::AmmError;

/// Math utilities for CLMM calculations
pub struct MathUtil;

impl MathUtil {
    /// Calculate sqrt price from tick
    pub fn tick_to_sqrt_price_x64(tick: i32) -> Result<u128> {
        if tick < crate::constants::MIN_TICK || tick > crate::constants::MAX_TICK {
            return Err(AmmError::TickOutOfBounds.into());
        }
        
        // Implementation of tick to sqrt price conversion
        // This is a complex calculation involving powers of 1.0001
        let abs_tick = tick.abs() as u32;
        
        let mut ratio = if abs_tick & 0x1 != 0 {
            0xfffcb933bd6fad37aa2d162d1a594001u128
        } else {
            1u128 << 96  // Q96 fixed point: 1.0 in 96-bit fixed point
        };
        
        if abs_tick & 0x2 != 0 {
            ratio = (ratio * 0xfff97272373d413259a46990580e213a) >> 64;
        }
        if abs_tick & 0x4 != 0 {
            ratio = (ratio * 0xfff2e50f5f656932ef12357cf3c7fdcc) >> 64;
        }
        if abs_tick & 0x8 != 0 {
            ratio = (ratio * 0xffe5caca7e10e4e61c3624eaa0941cd0) >> 64;
        }
        if abs_tick & 0x10 != 0 {
            ratio = (ratio * 0xffcb9843d60f6159c9db58835c926644) >> 64;
        }
        if abs_tick & 0x20 != 0 {
            ratio = (ratio * 0xff973b41fa98c081472e6896dfb254c0) >> 64;
        }
        if abs_tick & 0x40 != 0 {
            ratio = (ratio * 0xff2ea16466c96a3843ec78b326b52861) >> 64;
        }
        if abs_tick & 0x80 != 0 {
            ratio = (ratio * 0xfe5dee046a99a2a811c461f1969c3053) >> 64;
        }
        if abs_tick & 0x100 != 0 {
            ratio = (ratio * 0xfcbe86c7900a88aedcffc83b479aa3a4) >> 64;
        }
        if abs_tick & 0x200 != 0 {
            ratio = (ratio * 0xf987a7253ac413176f2b074cf7815e54) >> 64;
        }
        if abs_tick & 0x400 != 0 {
            ratio = (ratio * 0xf3392b0822b70005940c7a398e4b70f3) >> 64;
        }
        if abs_tick & 0x800 != 0 {
            ratio = (ratio * 0xe7159475a2c29b7443b29c7fa6e889d9) >> 64;
        }
        if abs_tick & 0x1000 != 0 {
            ratio = (ratio * 0xd097f3bdfd2022b8845ad8f792aa5825) >> 64;
        }
        if abs_tick & 0x2000 != 0 {
            ratio = (ratio * 0xa9f746462d870fdf8a65dc1f90e061e5) >> 64;
        }
        if abs_tick & 0x4000 != 0 {
            ratio = (ratio * 0x70d869a156d2a1b890bb3df62baf32f7) >> 64;
        }
        if abs_tick & 0x8000 != 0 {
            ratio = (ratio * 0x31be135f97d08fd981231505542fcfa6) >> 64;
        }
        if abs_tick & 0x10000 != 0 {
            ratio = (ratio * 0x9aa508b5b7a84e1c677de54f3e99bc9) >> 64;
        }
        if abs_tick & 0x20000 != 0 {
            ratio = (ratio * 0x5d6af8dedb81196699c329225ee604) >> 64;
        }
        if abs_tick & 0x40000 != 0 {
            ratio = (ratio * 0x2216e584f5fa1ea926041bedfe98) >> 64;
        }
        if abs_tick & 0x80000 != 0 {
            ratio = (ratio * 0x48a170391f7dc42444e8fa2) >> 64;
        }
        
        if tick > 0 {
            ratio = u128::MAX / ratio;
        }
        
        // Convert to x64 format
        let sqrt_price_x64 = if ratio % (1 << 32) == 0 {
            ratio >> 32
        } else {
            (ratio >> 32) + 1
        };
        
        Ok(sqrt_price_x64)
    }
    
    /// Calculate tick from sqrt price
    pub fn sqrt_price_x64_to_tick(sqrt_price_x64: u128) -> Result<i32> {
        if sqrt_price_x64 < crate::constants::MIN_SQRT_PRICE_X64 
            || sqrt_price_x64 > crate::constants::MAX_SQRT_PRICE_X64 {
            return Err(AmmError::InvalidSqrtPrice.into());
        }
        
        // This is a complex calculation that involves logarithms
        // For now, we'll use a simplified approximation
        let sqrt_price = sqrt_price_x64 as f64 / (1u128 << 64) as f64;
        let price = sqrt_price * sqrt_price;
        let tick = (price.ln() / 1.0001f64.ln()).round() as i32;
        
        Ok(tick.clamp(crate::constants::MIN_TICK, crate::constants::MAX_TICK))
    }
    
    /// Calculate liquidity from amounts
    pub fn get_liquidity_from_amounts(
        sqrt_price_current_x64: u128,
        sqrt_price_lower_x64: u128,
        sqrt_price_upper_x64: u128,
        amount0: u64,
        amount1: u64,
    ) -> Result<u128> {
        if sqrt_price_lower_x64 >= sqrt_price_upper_x64 {
            return Err(AmmError::InvalidTickRange.into());
        }
        
        let liquidity = if sqrt_price_current_x64 <= sqrt_price_lower_x64 {
            // All amount0
            Self::get_liquidity_from_amount0(sqrt_price_lower_x64, sqrt_price_upper_x64, amount0)?
        } else if sqrt_price_current_x64 < sqrt_price_upper_x64 {
            // Both amounts
            let liquidity0 = Self::get_liquidity_from_amount0(
                sqrt_price_current_x64, 
                sqrt_price_upper_x64, 
                amount0
            )?;
            let liquidity1 = Self::get_liquidity_from_amount1(
                sqrt_price_lower_x64, 
                sqrt_price_current_x64, 
                amount1
            )?;
            liquidity0.min(liquidity1)
        } else {
            // All amount1
            Self::get_liquidity_from_amount1(sqrt_price_lower_x64, sqrt_price_upper_x64, amount1)?
        };
        
        Ok(liquidity)
    }
    
    /// Calculate liquidity from amount0
    pub fn get_liquidity_from_amount0(
        sqrt_price_a_x64: u128,
        sqrt_price_b_x64: u128,
        amount0: u64,
    ) -> Result<u128> {
        if sqrt_price_a_x64 > sqrt_price_b_x64 {
            return Err(AmmError::InvalidSqrtPrice.into());
        }
        
        let intermediate = sqrt_price_a_x64
            .checked_mul(sqrt_price_b_x64)
            .ok_or(AmmError::Overflow)?;
        
        let liquidity = (amount0 as u128)
            .checked_mul(intermediate)
            .ok_or(AmmError::Overflow)?
            .checked_div(sqrt_price_b_x64 - sqrt_price_a_x64)
            .ok_or(AmmError::DivisionByZero)?;
            
        Ok(liquidity)
    }
    
    /// Calculate liquidity from amount1
    pub fn get_liquidity_from_amount1(
        sqrt_price_a_x64: u128,
        sqrt_price_b_x64: u128,
        amount1: u64,
    ) -> Result<u128> {
        if sqrt_price_a_x64 > sqrt_price_b_x64 {
            return Err(AmmError::InvalidSqrtPrice.into());
        }
        
        let liquidity = (amount1 as u128)
            .checked_mul(crate::constants::Q64)
            .ok_or(AmmError::Overflow)?
            .checked_div(sqrt_price_b_x64 - sqrt_price_a_x64)
            .ok_or(AmmError::DivisionByZero)?;
            
        Ok(liquidity)
    }
    
    /// Calculate amount0 from liquidity
    pub fn get_amount0_from_liquidity(
        sqrt_price_a_x64: u128,
        sqrt_price_b_x64: u128,
        liquidity: u128,
    ) -> Result<u64> {
        if sqrt_price_a_x64 > sqrt_price_b_x64 {
            return Err(AmmError::InvalidSqrtPrice.into());
        }
        
        let amount0 = liquidity
            .checked_mul(sqrt_price_b_x64 - sqrt_price_a_x64)
            .ok_or(AmmError::Overflow)?
            .checked_div(sqrt_price_a_x64)
            .ok_or(AmmError::DivisionByZero)?
            .checked_div(sqrt_price_b_x64)
            .ok_or(AmmError::DivisionByZero)?;
            
        Ok(amount0 as u64)
    }
    
    /// Calculate amount1 from liquidity
    pub fn get_amount1_from_liquidity(
        sqrt_price_a_x64: u128,
        sqrt_price_b_x64: u128,
        liquidity: u128,
    ) -> Result<u64> {
        if sqrt_price_a_x64 > sqrt_price_b_x64 {
            return Err(AmmError::InvalidSqrtPrice.into());
        }
        
        let amount1 = liquidity
            .checked_mul(sqrt_price_b_x64 - sqrt_price_a_x64)
            .ok_or(AmmError::Overflow)?
            .checked_div(crate::constants::Q64)
            .ok_or(AmmError::DivisionByZero)?;
            
        Ok(amount1 as u64)
    }
    
    /// Get next sqrt price from input amount
    pub fn get_next_sqrt_price_from_amount0_rounding_up(
        sqrt_price_x64: u128,
        liquidity: u128,
        amount: u64,
        add: bool,
    ) -> Result<u128> {
        if amount == 0 {
            return Ok(sqrt_price_x64);
        }
        
        let numerator1 = liquidity << 96;
        
        if add {
            let product = amount as u128 * sqrt_price_x64;
            if product / amount as u128 == sqrt_price_x64 {
                let denominator = numerator1 + product;
                if denominator >= numerator1 {
                    return Ok(Self::mul_div_rounding_up(numerator1, sqrt_price_x64, denominator)?);
                }
            }
            
            Ok(Self::div_rounding_up(numerator1, (numerator1 / sqrt_price_x64) + amount as u128)?)
        } else {
            let product = amount as u128 * sqrt_price_x64;
            let denominator = numerator1 - product;
            Ok(Self::mul_div_rounding_up(numerator1, sqrt_price_x64, denominator)?)
        }
    }
    
    /// Multiply and divide with rounding up
    pub fn mul_div_rounding_up(a: u128, b: u128, denominator: u128) -> Result<u128> {
        let result = a
            .checked_mul(b)
            .ok_or(AmmError::Overflow)?
            .checked_div(denominator)
            .ok_or(AmmError::DivisionByZero)?;
        
        let remainder = a
            .checked_mul(b)
            .ok_or(AmmError::Overflow)?
            .checked_rem(denominator)
            .ok_or(AmmError::DivisionByZero)?;
        
        if remainder > 0 {
            Ok(result + 1)
        } else {
            Ok(result)
        }
    }
    
    /// Divide with rounding up
    pub fn div_rounding_up(numerator: u128, denominator: u128) -> Result<u128> {
        let result = numerator
            .checked_div(denominator)
            .ok_or(AmmError::DivisionByZero)?;
        
        let remainder = numerator
            .checked_rem(denominator)
            .ok_or(AmmError::DivisionByZero)?;
        
        if remainder > 0 {
            Ok(result + 1)
        } else {
            Ok(result)
        }
    }
}