use tfhe::{prelude::*, FheBool, FheInt32, FheUint32};

/*
*   IEEE 754 Extraction
*/

pub fn ieee754_extract_sign(input: u32) -> u32 {
    return input >> 31;
}

pub fn ieee754_extract_exponent(input: u32) -> u32 {
    return (input >> 23) - 127;
}

pub fn ieee754_extract_mantissa(input: u32) -> u32 {
    return input & 0x7FFFFF;
}

/*
*   u32
*/

// The original algorithm
// https://stackoverflow.com/questions/20302904/converting-int-to-float-or-float-to-int-using-bitwise-operations-software-float
pub fn u32_to_ieee754(significand: u32) -> Option<u32> {
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

// The modified algorithm
pub fn u32_to_ieee754_2nd(input: u32) -> u32 {
    let mut shifts: u32 = input.clone() & (1 << 23);
    let zero_found: bool = shifts == 0;

    if zero_found {
        shifts = u32_calculate_shifts(input, shifts);
    }

    let exponent = (-(shifts as i32) + 150) as u32;
    return (exponent << 23) | ((input << shifts) & 0x7FFFFF);
}

fn u32_calculate_shifts(input: u32, shifts: u32) -> u32 {
    let mut cloned_shifts: u32 = shifts.clone();
    let mut zero_found: bool = shifts == 0;

    for i in 0..24 {
        let is_zero: bool = ((input.clone() << i as u32) & (1 << 23)) == 0;
        zero_found = zero_found & is_zero;
        let zero_found_number: u32 = zero_found.clone().cast_into();
        cloned_shifts += zero_found_number;
    }

    return cloned_shifts;
}

// The second original algorithm
// https://codegolf.stackexchange.com/questions/252936/convert-an-integer-to-ieee-754-float

/*
*   FheInt32
*/

pub fn fheint32_to_ieee754(input: &FheInt32) -> FheInt32 {
    let mut shifts: FheInt32 = input.clone() & (1 << 23);
    let zero_found: FheBool = shifts.eq(0);

    shifts = zero_found.if_then_else(&fheint32_calculate_shifts(&input, &shifts).cast_into(), &shifts);

    let exponent: FheInt32 = -shifts.clone() + 150;
    let ushifts: FheUint32 = shifts.cast_into();
    return (exponent << 23u32) | ((input.clone() << ushifts) & 0x7FFFFF);
}

fn fheint32_calculate_shifts(input: &FheInt32, shifts: &FheInt32) -> FheInt32 {
    let mut cloned_shifts: FheInt32 = shifts.clone();
    let mut zero_found: FheBool = shifts.eq(0);

    for i in 0..24 {
        let is_zero: FheBool = (((input.clone() << i as u32) & (1 << 23))).eq(0);
        zero_found = zero_found & is_zero;
        let zero_found_number: FheInt32 = zero_found.clone().cast_into();
        cloned_shifts += zero_found_number;
    }

    return cloned_shifts;
}

/*
*   FheUint32
*/

pub fn fheuint32_to_ieee754(significand: &FheUint32) -> FheUint32 {
    let mut shifts: FheUint32 = significand.clone() & (1 << 23);
    let zero_found: FheBool = shifts.eq(0);

    shifts = zero_found.if_then_else(&fheuint32_calculate_shifts(&significand, &shifts), &shifts);

    let exponent: FheUint32 = -shifts.clone() + 150;
    let ushifts: FheUint32 = shifts.cast_into();
    return (exponent << 23u32) | ((significand.clone() << ushifts) & 0x7FFFFF);
}

fn fheuint32_calculate_shifts(significand: &FheUint32, shifts: &FheUint32) -> FheUint32 {
    let mut cloned_shifts: FheUint32 = shifts.clone();
    let mut zero_found: FheUint32 = !shifts.clone();

    for i in 0..23 {
        let is_zero: FheBool = (((significand << i as u32) & (1 << 23))).eq(0);
        zero_found = is_zero.if_then_else(&zero_found, &(!is_zero.clone()).cast_into());
        cloned_shifts += !zero_found.clone();
    }

    return cloned_shifts;
}
