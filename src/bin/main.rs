use std::io;
use num_traits::{zero, Float, Pow};

use tfhe::boolean::client_key;
use tfhe::{prelude::*, ClientKey, FheBool, FheUint};
use tfhe::{ConfigBuilder, generate_keys, FheInt32, FheUint32, set_server_key};

use thesislib::{ieee754::*};

fn uint_to_float(significand: u32) -> Option<u32> {
    // Only support 0 < significand < 1 << 24.
    if significand == 0 || significand >= 1 << 24 {
        return None; // or Err("Invalid input"); or whatever you'd like here.
    }

    let mut shifts: u32 = 0;

    // Align the leading 1 of the significand to the hidden-1
    // position. Count the number of shifts required.
    let mut temp_significand = significand;
    while temp_significand & (1 << 23) == 0 {
        temp_significand <<= 1;
        shifts += 1;
    }

    // The number 1.0 has an exponent of 0, and would need to be
    // shifted left 23 times. The number 2.0, however, has an
    // exponent of 1 and needs to be shifted left only 22 times.
    // Therefore, the exponent should be (23 - shifts). IEEE-754
    // format requires a bias of 127, though, so the exponent field
    // is given by the following expression:
    let exponent = 127 + 23 - shifts;

    // Now merge significand and exponent. Be sure to strip away
    // the hidden 1 in the significand.
    let merged = (exponent << 23) | (temp_significand & 0x7FFFFF);

    // Return the merged value as Option<u32>.
    Some(merged)
}

fn uint_to_float1(significand: u32) -> u32 {
    let mut i = 0;
    let mut shifts = 0;

    let mut temp_significand = significand;
    while temp_significand & (1 << 23) == 0 {
        i += 1;
        println!("i: {}", i);
        temp_significand <<= 1;
        shifts += 1;
    }
    println!("{}", shifts);
    
    let exponent = 127 + 23 - shifts;
    
    let merged = (exponent << 23) | (temp_significand & 0x7FFFFF);
    
    return merged;
}

fn uint_to_float2(significand: u32) -> u32 {
    let mut shifts = significand & (1 << 23);
    let first_is_zero: bool = shifts == 0;
    let mut zero_found: bool = first_is_zero;
    println!("{:032b}", significand);
    println!("{:032b}", (1 << 23));
    println!("{:032b}", shifts);
    println!("shifts: [{:2}] first_is_zero: [{: ^5}] zero_round: [{: ^5}]", shifts, first_is_zero, zero_found);

    // if first_is_zero {
    for i in 0..23 {
        let temp_significand = (significand << i) & (1 << 23);
        println!("{:032b}", temp_significand);
        let is_zero = temp_significand == 0;
        if is_zero {
            shifts += zero_found as u32 & first_is_zero as u32 & 1;
        } else {
            zero_found = is_zero;
            shifts += zero_found as u32;
        }
        println!("i: [{:2}] is_zero: [{: ^5}] zero_round: [{: ^5}] shifts: [{:2}]", i, is_zero, zero_found, shifts);
    }
    // }

    // let exponent = 127 + 23 - shifts;
    // let exponent = (150 - shifts as i32) as u32;
    let exponent: u32 = ((-(shifts as i32)) + 150) as u32;

    let merged = (exponent << 23) | ((significand << shifts) & 0x7FFFFF);
    // println!("output: {:032b}", merged);

    return merged;
}

// fn uint_to_float3(significand: u32) -> u32 {
//     let mut shifts = ((significand & (1 << 23)) != 0) as u32;
//     let mut zero_found = true; // TODO:

//     for i in 1..24 {
//         let is_zero = ((significand << i - 1) & (1 << 23)) == 0;
//         if is_zero {
//             shifts += 1 & zero_found as u32;
//         } else {
//             zero_found = zero_found & !is_zero;
//             shifts += 0;
//         }
//     }

//     // let exponent = 127 + 23 - shifts;
//     // let exponent = (150 - shifts as i32) as u32;
//     let exponent: u32 = ((-(shifts as i32)) + 150) as u32;

//     return (exponent << 23) | ((significand << shifts) & 0x7FFFFF);
// }

fn u32_to_ieee754(significand: u32) -> u32 {
    let mut shifts = significand & (1 << 23);
    let first_is_zero: bool = shifts == 0;
    let mut zero_round: bool = !first_is_zero;
    println!("shifts: [{:2}] first_is_zero: [{: ^5}] zero_round: [{: ^5}]", shifts, first_is_zero, zero_round);

    for i in 0..23 {
        let mut is_zero = ((significand << i) & (1 << 23)) == 0;
        if first_is_zero == false {
            is_zero = first_is_zero;
        }
        
        if is_zero {
            zero_round = zero_round;
        }
        else {
            zero_round = !is_zero;
        }

        if is_zero {
            shifts += !zero_round as u32;
        } else {
            shifts += !zero_round as u32;
        }
        
        // println!("i: [{:2}] is_zero: [{: ^5}] zero_round: [{: ^5}] shifts: [{:2}]", i, is_zero, zero_round, shifts);
    }

    let mut sig = significand.clone();
    let mut con = 0;
    for i in 0..24 {
        println!("i = {}", i);
        let mut is_zero = ((sig.clone() << i) & (1 << 23)) == 0;
        if is_zero {
            con += 1;
        } else {
            break
        }
    }
    println!("{}", con);

    let exponent = (-(shifts as i32) + 150) as u32;

    return (exponent << 23) | ((significand << shifts) & 0x7FFFFF);
}

fn fheuint32_to_ieee754(significand: &FheUint32, client_key: &ClientKey) -> FheUint32 {
    let mut shifts: FheUint32 = significand.clone() & (1 << 23);
    let first_is_zero: FheBool = shifts.eq(0);

    shifts = first_is_zero.if_then_else(&calculate_shifts_uint(&significand, &shifts, &first_is_zero, &client_key), &shifts);

    let exponent: FheUint32 = -shifts.clone() + 150;
    let ushifts: FheUint32 = shifts.cast_into();
    return (exponent << 23u32) | ((significand.clone() << ushifts) & 0x7FFFFF);
}

fn calculate_shifts_uint(significand: &FheUint32, shifts: &FheUint32, flag: &FheBool, client_key: &ClientKey) -> FheUint32 {
    let mut cloned_shifts: FheUint32 = shifts.clone();
    // let mut zero_round: FheUint32 = (!(flag.clone())).cast_into();
    let mut zero_round: FheUint32 = !shifts.clone();
    let decrypted_zero_round: u32 = shifts.decrypt(&client_key);
    println!("{}", decrypted_zero_round);

    for i in 0..23 {
        println!("i = {}", i);
        let is_zero: FheBool = (((significand << i as u32) & (1 << 23))).eq(0);
        zero_round = is_zero.if_then_else(&zero_round, &(!is_zero.clone()).cast_into());
        cloned_shifts += !zero_round.clone();
    }

    return cloned_shifts;
}

fn find_m_recursive(x: u32, m:u32) -> u32 {
    if 2u32.pow(2 * m) <= x {
        find_m_recursive(x, m + 1)
    } else {
        m - 1
    }
}

fn isqrt(x: u32, m :u32) -> u32 {
    let mut a: u32 = 1 << (2 * m);
    let mut b: u32 = 1 << (2 * m);
    let mut c: u32 = a >> 2;
    let mut s: u32 = a + b + c;

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

fn fsqrt64(mut expo: u64, mant: u64, fact: u64) -> u64 {
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

/*
    IEEE 754 Extraction
*/

fn ieee754_extract_sign(input: u32) -> u32 {
    return input >> 31;
}

fn ieee754_extract_exponent(input: u32) -> u32 {
    return (input >> 23) - 127;
}

fn ieee754_extract_mantissa(input: u32) -> u32 {
    return input & 0x7FFFFF | 0x800000;
}

/*
    Main
*/

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("failed to read line");
    let numb: u32 = input.trim().parse().expect("failed to parse");

    // Parsing
    println!("\nParsing:");
    println!("numb: [u32] {: ^20} [x] {:#018x} [b] {:064b}", numb, numb, numb);
    let ieee: u32 = u32_to_ieee754(numb);
    println!("ieee: [u32] {: ^20} [x] {:#018x} [b] {:064b}", ieee, ieee, ieee);
    let ieee2: u32 = uint_to_float1(numb);
    println!("ieee: [u32] {: ^20} [x] {:#018x} [b] {:064b}", ieee2, ieee2, ieee2);
    let sign: u32 = ieee754_extract_sign(ieee);
    println!("sign: [u32] {: ^20} [x] {:#018x} [b] {:064b}", sign, sign, sign);
    let expo: u32 = ieee754_extract_exponent(ieee);
    println!("expo: [u32] {: ^20} [x] {:#018x} [b] {:064b}", expo, expo, expo);
    let mut mant: u32 = ieee754_extract_mantissa(ieee);
    println!("mant: [u32] {: ^20} [x] {:#018x} [b] {:064b}", mant, mant, mant);
    // mant |= 0x800000;
    // println!("mant: [u32] {: ^20} [x] {:#016x} [b] {:064b}", mant, mant, mant);

    // Approximation
    println!("\nApproximation");
    let mut appr: u32 = numb;
    appr += 127 << 23;
    appr >>= 1;
    println!("appr: [u32] {: ^20} [x] {:#018x} [b] {:064b}", appr, appr, appr);

    // Square Root
    println!("\nSquare Root");
    let sqrt: u64 = fsqrt64(expo as u64, mant as u64, 23 as u64);
    println!("sqrt: [u64] {: ^20} [x] {:#018x} [b] {:064b}", sqrt, sqrt, sqrt);

    // extract the floating part as a new significand
    // let ext_sig = ((plaintext & 0x7FFFFF) | 0x800000) as u64;
    // let ext_sig = ((ieee & 0x7FFFFF)) as u64;
    // println!("extra signif: {:064b}", ext_sig);
    // let N: u64 = ext_sig << (23 + 1);
    // println!("N: {}", N);
    // println!("N: {:032b}", N);
    // let found_m = m64(N, 0);
    // println!("found_m: {}", found_m);
    // let rN = isqrt64(N, found_m);
    // println!("nN: {}", rN);
    // let r: u64 = fsqrt64(6, 70, 23);
    // println!("r: {}", r);

    // let config = ConfigBuilder::all_disabled().enable_default_integers().build();
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);

    // let msg1 = 14;
    // let msg2 = 14;

    // let mut ciphertext = FheUint32::encrypt(plaintext, &client_key);
    // let ct1 = FheInt32::encrypt(msg1, &client_key);
    // let ct2 = FheInt32::encrypt(msg2, &client_key);

    // Do operations using the server key! VERY IMPORTANT!
    set_server_key(server_key);

    
    // Encrypted number
    let encrypted_number = FheUint32::encrypt(numb, &client_key);
    let decrypted_number: u32 = encrypted_number.decrypt(&client_key);
    println!("decn: [u32] {: ^20} [x] {:#018x} [b] {:064b}", decrypted_number, decrypted_number, decrypted_number);


    let encrypted_ieee754 = fheuint32_to_ieee754(&encrypted_number, &client_key);
    let decrypted_ieee754:u32 = encrypted_ieee754.decrypt(&client_key);

    println!("decry output: {}", decrypted_ieee754);
    println!("decry output: {:#01x}", decrypted_ieee754);
    println!("decry output: {:032b}", decrypted_ieee754);


    // Homomorphic IEEE 754
    println!("\nHomomorphic IEEE 754:");
    let h_ieee: u32 = u32_to_ieee754(numb);
    println!("ieee: [u32] {: ^20} [x] {:#018x} [b] {:064b}", h_ieee, h_ieee, h_ieee);


    // test signed
    let test: i32 = numb.cast_into();
    let encrypted_number = FheInt32::encrypt(test, &client_key);
    let encrypted_ieee754 = fheint32_to_ieee754(&encrypted_number);
    let decrypted_ieee754:i32 = encrypted_ieee754.decrypt(&client_key);
    println!("decry output: {}", decrypted_ieee754);
    println!("ieee: [u32] {: ^20} [x] {:#018x} [b] {:064b}", decrypted_ieee754, decrypted_ieee754, decrypted_ieee754);

    // ciphertext += 1;
    // let ct3 = ct1.clone() + ct2.clone();
    // let ct4 = -ct3.clone();
    // let ct5 = &ct1 >> 1u32;
    // let cmp = ct3.eq(ct3.clone()) + 1;
    // // cmp.if_then_else(&(ct3.clone() + 1), &(ct4.clone() + 1));

    // let decrypted:u32 = ciphertext.decrypt(&client_key);
    // let msg3:i32 = ct3.decrypt(&client_key);
    // let msg4:i32 = ct4.decrypt(&client_key);
    // let msg5:i32 = ct5.decrypt(&client_key);
    // let decrypted_cmp:i32 = cmp.decrypt(&client_key);
    
    // print!("decrypted: {}\n", decrypted);
    // print!("msg3: {}\n", msg3);
    // print!("msg4: {}\n", msg4);
    // print!("msg5: {}\n", msg5);
    // print!("decrypted_cmp: {}\n", decrypted_cmp);
}