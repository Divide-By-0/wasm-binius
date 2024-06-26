// Copyright 2024 Ulvetanna Inc.

pub use crate::arch::{packed_aes_128::*, packed_aes_256::*, packed_aes_512::*};

#[cfg(test)]
mod test_utils {
	/// Test if `mult_func` operation is a valid multiply operation on the given values for
	/// all possible packed fields defined on u128.
	macro_rules! define_multiply_tests {
		($mult_func:path, $constraint:ty) => {
			$crate::packed_binary_field::test_utils::define_check_packed_mul!(
				$mult_func,
				$constraint
			);

			proptest! {
				#[test]
				fn test_mul_packed_16(a_val in any::<u16>(), b_val in any::<u16>()) {
					use $crate::arch::packed_aes_16::*;

					TestMult::<PackedAESBinaryField2x8b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField1x16b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
				}

				#[test]
				fn test_mul_packed_32(a_val in any::<u32>(), b_val in any::<u32>()) {
					use $crate::arch::packed_aes_32::*;

					TestMult::<PackedAESBinaryField4x8b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField2x16b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField1x32b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
				}

				#[test]
				fn test_mul_packed_64(a_val in any::<u64>(), b_val in any::<u64>()) {
					use $crate::arch::packed_aes_64::*;

					TestMult::<PackedAESBinaryField8x8b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField4x16b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField2x32b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField1x64b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
				}

				#[test]
				fn test_mul_packed_128(a_val in any::<u128>(), b_val in any::<u128>()) {
					use $crate::arch::packed_aes_128::*;

					TestMult::<PackedAESBinaryField16x8b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField8x16b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField4x32b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField2x64b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField1x128b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
				}

				#[test]
				fn test_mul_packed_256(a_val in any::<[u128; 2]>(), b_val in any::<[u128; 2]>()) {
					use $crate::arch::packed_aes_256::*;

					TestMult::<PackedAESBinaryField32x8b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField16x16b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField8x32b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField4x64b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField2x128b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
				}

				#[test]
				fn test_mul_packed_512(a_val in any::<[u128; 4]>(), b_val in any::<[u128; 4]>()) {
					use $crate::arch::packed_aes_512::*;

					TestMult::<PackedAESBinaryField64x8b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField32x16b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField16x32b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField8x64b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
					TestMult::<PackedAESBinaryField4x128b>::test_mul(
						a_val.into(),
						b_val.into(),
					);
				}
			}
		};
	}

	/// Test if `square_func` operation is a valid square operation on the given value for
	/// all possible packed fields.
	macro_rules! define_square_tests {
		($square_func:path, $constraint:ident) => {
			$crate::packed_binary_field::test_utils::define_check_packed_square!(
				$square_func,
				$constraint
			);

			proptest! {
				#[test]
				fn test_square_packed_16(a_val in any::<u16>()) {
					use $crate::arch::packed_aes_16::*;

					TestSquare::<PackedAESBinaryField2x8b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField1x16b>::test_square(a_val.into());
				}

				#[test]
				fn test_square_packed_32(a_val in any::<u32>()) {
					use $crate::arch::packed_aes_32::*;

					TestSquare::<PackedAESBinaryField4x8b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField2x16b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField1x32b>::test_square(a_val.into());
				}

				#[test]
				fn test_square_packed_64(a_val in any::<u64>()) {
					use $crate::arch::packed_aes_64::*;

					TestSquare::<PackedAESBinaryField8x8b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField4x16b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField2x32b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField1x64b>::test_square(a_val.into());
				}

				#[test]
				fn test_square_packed_128(a_val in any::<u128>()) {
					use $crate::arch::packed_aes_128::*;

					TestSquare::<PackedAESBinaryField16x8b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField8x16b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField4x32b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField2x64b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField1x128b>::test_square(a_val.into());
				}

				#[test]
				fn test_square_packed_256(a_val in any::<[u128; 2]>()) {
					use $crate::arch::packed_aes_256::*;

					TestSquare::<PackedAESBinaryField32x8b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField16x16b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField8x32b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField4x64b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField2x128b>::test_square(a_val.into());
				}

				#[test]
				fn test_square_packed_512(a_val in any::<[u128; 4]>()) {
					use $crate::arch::packed_aes_512::*;

					TestSquare::<PackedAESBinaryField64x8b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField32x16b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField16x32b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField8x64b>::test_square(a_val.into());
					TestSquare::<PackedAESBinaryField4x128b>::test_square(a_val.into());
				}
			}
		};
	}

	/// Test if `invert_func` operation is a valid invert operation on the given value for
	/// all possible packed fields.
	macro_rules! define_invert_tests {
		($invert_func:path, $constraint:ident) => {
			$crate::packed_binary_field::test_utils::define_check_packed_inverse!(
				$invert_func,
				$constraint
			);

			proptest! {
				#[test]
				fn test_invert_packed_16(a_val in any::<u16>()) {
					use $crate::arch::packed_aes_16::*;

					TestSquare::<PackedAESBinaryField2x8b>::test_invert(a_val.into());
					TestSquare::<PackedAESBinaryField1x16b>::test_invert(a_val.into());
				}

				#[test]
				fn test_invert_packed_32(a_val in any::<u32>()) {
					use $crate::arch::packed_aes_32::*;

					TestSquare::<PackedAESBinaryField4x8b>::test_invert(a_val.into());
					TestSquare::<PackedAESBinaryField2x16b>::test_invert(a_val.into());
					TestSquare::<PackedAESBinaryField1x32b>::test_invert(a_val.into());
				}

				#[test]
				fn test_invert_packed_64(a_val in any::<u64>()) {
					use $crate::arch::packed_aes_64::*;

					TestSquare::<PackedAESBinaryField8x8b>::test_invert(a_val.into());
					TestSquare::<PackedAESBinaryField4x16b>::test_invert(a_val.into());
					TestSquare::<PackedAESBinaryField2x32b>::test_invert(a_val.into());
					TestSquare::<PackedAESBinaryField1x64b>::test_invert(a_val.into());
				}

				#[test]
				fn test_invert_packed_128(a_val in any::<u128>()) {
					use $crate::arch::packed_aes_128::*;

					TestInvert::<PackedAESBinaryField16x8b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField8x16b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField4x32b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField2x64b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField1x128b>::test_invert(a_val.into());
				}

				#[test]
				fn test_invert_packed_256(a_val in any::<[u128; 2]>()) {
					use $crate::arch::packed_aes_256::*;

					TestInvert::<PackedAESBinaryField32x8b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField16x16b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField8x32b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField4x64b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField2x128b>::test_invert(a_val.into());
				}

				#[test]
				fn test_invert_packed_512(a_val in any::<[u128; 4]>()) {
					use $crate::arch::packed_aes_512::*;

					TestInvert::<PackedAESBinaryField64x8b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField32x16b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField16x32b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField8x64b>::test_invert(a_val.into());
					TestInvert::<PackedAESBinaryField4x128b>::test_invert(a_val.into());
				}
			}
		};
	}

	pub(crate) use define_invert_tests;
	pub(crate) use define_multiply_tests;
	pub(crate) use define_square_tests;
}

#[cfg(test)]
mod tests {
	use super::test_utils::{define_invert_tests, define_multiply_tests, define_square_tests};
	use crate::PackedField;
	use proptest::prelude::*;
	use std::ops::Mul;

	define_multiply_tests!(Mul::mul, PackedField);

	define_square_tests!(PackedField::square, PackedField);

	define_invert_tests!(PackedField::invert_or_zero, PackedField);
}
