use tfhe::{prelude::*, FheBool, FheInt32, FheUint32};

pub fn fheint32_to_ieee754(input: &FheInt32) -> FheInt32 {
    let mut shifts: FheInt32 = input.clone() & (1 << 23);
    let first_is_zero: FheBool = shifts.eq(0);

    shifts = first_is_zero.if_then_else(&calculate_shifts(&input, &shifts).cast_into(), &shifts);

    let exponent: FheInt32 = -shifts.clone() + 150;
    let ushifts: FheUint32 = shifts.cast_into();
    return (exponent << 23u32) | ((input.clone() << ushifts) & 0x7FFFFF);
}

pub fn calculate_shifts(input: &FheInt32, shifts: &FheInt32) -> FheInt32 {
    let mut cloned_shifts: FheInt32 = shifts.clone();
    let mut zero_round: FheBool = shifts.eq(0);

    for i in 0..24 {
        let is_zero: FheBool = (((input.clone() << i as u32) & (1 << 23))).eq(0);
        zero_round = zero_round & is_zero;
        let zero_found_number: FheInt32 = zero_round.clone().cast_into();
        cloned_shifts += zero_found_number;
    }

    return cloned_shifts;
}