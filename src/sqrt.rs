use std::ptr::null;

use num_traits::{Pow, ToPrimitive};
use tfhe::core_crypto::commons::traits::CastInto;
use tfhe::prelude::{FheDecrypt, FheEq, FheOrd, IfThenElse};
use tfhe::{ClientKey, FheBool, FheInt64, FheUint64};

use crate::ieee754::ieee754_extract_sign;

const IEEE754_MANTISSA_SIZE: u64 = 23;

/*
*   FheInt64
*/

pub fn isqrt_homo(x: &FheUint64) -> FheInt64 {
    let common_one: FheUint64 = (x.clone() | 0x8000000000000000) >> 63u64;
    let mut a: FheUint64 = common_one.clone() << (2u64 * IEEE754_MANTISSA_SIZE);
    let mut b: FheUint64 = a.clone();
    let mut c: FheUint64 = a.clone() >> 2u64;
    let mut s: FheUint64 = a.clone() + b.clone() + c.clone();
    let mut can_continue: FheBool = common_one.clone().eq(1u64);

    for k in 1..= IEEE754_MANTISSA_SIZE {
        let is_eq: FheBool = x.eq(&s);
        let is_gt: FheBool = x.gt(&s);
        
        b = can_continue.if_then_else(
            &(b.clone() >> 1u64),
            &b.clone()
        );

        b = (is_eq.clone() & can_continue.clone()).if_then_else(
            &((b.clone() + c.clone()) >> (IEEE754_MANTISSA_SIZE - k)),
            &b.clone()
        );
        can_continue = (is_eq.clone() & can_continue.clone() & !is_eq.clone()) | (!is_eq.clone() & can_continue.clone());
        
        a = (is_gt.clone() & can_continue.clone()).if_then_else(
            &s.clone(),
            &a.clone()
        );
        b = (is_gt.clone() & can_continue.clone()).if_then_else(
            &(b.clone() + c.clone()),
            &b.clone()
        );

        c = can_continue.if_then_else(
            &(c.clone() >> 2u64),
            &c.clone()
        );
        s = can_continue.if_then_else(
            &(a.clone() + b.clone() + c.clone()),
            &s.clone()
        );
    }

    return b.cast_into();
}


// pub fn fsqrt_homo(ieee754: FheInt64) -> FheInt64 {
//     // Extract IEEE 754 Parts
//     // let sign: FheInt64 = ieee754.clone() >> 31u64; // TODO: is this necessary?
//     let exponent: FheInt64 = (ieee754.clone() >> IEEE754_MANTISSA_SIZE) - 127;
//     let mantissa: FheInt64 = ieee754.clone() & 0x7FFFFF;

//     // Calculate stuff
//     let common_one: FheInt64 = (ieee754.clone() | 0x80000000) >> 31u64;
//     let hidden_bit: FheInt64 = common_one.clone() << IEEE754_MANTISSA_SIZE;
//     let normalized_mantissa: FheInt64 = mantissa.clone() | hidden_bit.clone();
//     let big_n: FheInt64 = (exponent.clone() % 2).eq(&common_one).if_then_else(
//         &(normalized_mantissa.clone() << (IEEE754_MANTISSA_SIZE + 1)),
//         &(normalized_mantissa.clone() << IEEE754_MANTISSA_SIZE)
//     );
//     let new_exponent: FheInt64 = (exponent.clone() % 2).eq(&common_one).if_then_else(
//         &(exponent.clone() - 1),
//         &exponent.clone()
//     );
//     let first_root: u64 = isqrt_homo(&big_n);
//     let base_exponent: FheInt64 = (127 + (new_exponent.clone() >> 1u64)) << IEEE754_MANTISSA_SIZE;
//     let root: FheInt64 = base_exponent | first_root as i64;

//     return root;
// }

/*
*   u32
*/

pub fn find_m_recursive(x: u32, m:u32) -> u32 {
    if 2u32.pow(2 * m) <= x {
        find_m_recursive(x, m + 1)
    } else {
        m - 1
    }
}

pub fn isqrt(x: u32, m :u32) -> u32 {
    let mut a: u32 = 1 << (2 * m);
    let mut b: u32 = 1 << (2 * m);
    let mut c: u32 = a >> 2;
    let mut s: u32 = a + b + c;

    for k in 1..m {
        b >>= 1;

        if s == x {
            b += c;
            return b >> (m - k)
        }

        if s < x {
            a = s;
            b += c;
        }

        c >>= 2;
        s = a + b + c;
    }
    return b
}

pub fn fsqrt(m: u64, mut e: u64, mut f: u64) -> u64{
    // f = f << 32;
    println!("mant: [u32] {: ^20} [x] {:#018x} [b] {:064b}", f, f, f);

    let mut big_n: u64;

    if e % 2 == 0 {
        big_n = (f << m) << 1;
    } else {
        big_n = (f << (m + 1)) << 1;
        e = e - 1;
    }
    println!("bign: [u64] {: ^20} [x] {:#018x} [b] {:064b}", big_n, big_n, big_n);
    println!("expo: [u64] {: ^20} [x] {:#018x} [b] {:064b}", e, e, e);

    big_n = 158329674399744;
    let mut sqrt_big_n = isqrt64(big_n, m);
    println!("bbbn: [u64] {: ^20} [x] {:#018x} [b] {:064b}", sqrt_big_n, sqrt_big_n, sqrt_big_n);

    return sqrt_big_n << ((e - (m * 2)) >> 1);
}

/*
*   u64
*/

fn m64(x: u64, m:u64) -> u64 {
    if (2u64).pow(2 * m as u32) <= x {
        m64(x, (m + 1) as u64)
    } else {
        m - 1
    }
}

fn isqrt64(x: u64, m :u64) -> u64 {
    let mut a: u64 = 1 << (2 * m);
    let mut b: u64 = 1 << (2 * m);
    let mut c: u64 = a >> 2;
    let mut s: u64 = a + b + c;

    for k in 1..(m + 1) {
        b >>= 1;
        if s == x {
            b += c;
            return b >> (m - k)
        }

        if s < x {
            a = s;
            b += c;
        }

        c >>= 2;
        s = a + b + c;
    }
    return b
}

/*
*   f64 u64
*/

pub fn fsqrt64(mut expo: u64, mant: u64, fact: u64) -> u64 {
    println!("expo: [u64] {: ^20} [x] {:#018x} [b] {:064b}", expo, expo, expo);
    println!("mant: [u64] {: ^20} [x] {:#018x} [b] {:064b}", mant, mant, mant);
    println!("fact: [u64] {: ^20} [x] {:#018x} [b] {:064b}", fact, fact, fact);

    let mut bign: u64 = 0;

    if expo % 2 == 0 {
        bign = mant << (2 * fact);
    } else {
        bign = mant << (2* fact + 1);
        expo -= 1;
    }

    println!("bign: [u64] {: ^20} [x] {:#018x} [b] {:064b}", bign, bign, bign);
    println!("expo: [u64] {: ^20} [x] {:#018x} [b] {:064b}", expo, expo, expo);

    let sqrt: u64 = isqrt64(bign, fact);
    println!("sqrt: [u64] {: ^20} [x] {:#018x} [b] {:064b}", sqrt, sqrt, sqrt);

    let nexp: u64 = (expo >> 1) - fact;
    println!("nexp: [u64] {: ^20} [x] {:#018x} [b] {:064b}", nexp, nexp, nexp);
    if nexp > 0 {
        return sqrt << nexp;
    } else {
        return sqrt >> nexp;
    }
}
