use crate::core::Method;
use crate::core::{PeriodType, ValueType, Window};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [Weighed Moving Average](https://en.wikipedia.org/wiki/Moving_average#Weighted_moving_average) of specified `length` for timeseries of type [`ValueType`].
///
/// # Parameters
///
/// Has a single parameter `length`: [`PeriodType`]
///
/// `length` should be > 0
///
/// # Input type
///
/// Input type is [`ValueType`]
///
/// # Output type
///
/// Output type is [`ValueType`]
///
/// # Examples
///
/// ```
/// use yata::prelude::*;
/// use yata::methods::WMA;
///
/// // WMA of length=3
/// let mut wma = WMA::new(3, 3.0);
///
/// wma.next(3.0);
/// wma.next(6.0);
///
/// assert_eq!(wma.next(9.0), 7.0);
/// assert_eq!(wma.next(12.0), 10.0);
/// ```
///
/// # Perfomance
///
/// O(1)
///
/// # See also
///
/// [Volume Weighted Moving Average](crate::methods::VWMA) for computing weighted moving average with custom weights over every value
///
/// [`ValueType`]: crate::core::ValueType
/// [`PeriodType`]: crate::core::PeriodType
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WMA {
	sum: ValueType,
	invert_sum: ValueType,
	float_length: ValueType,
	total: ValueType,
	numerator: ValueType,
	window: Window<ValueType>,
}

impl Method for WMA {
	type Params = PeriodType;
	type Input = ValueType;
	type Output = Self::Input;

	fn new(length: Self::Params, value: Self::Input) -> Self {
		debug_assert!(length > 0, "WMA: length should be > 0");

		let length2 = length as usize;
		let sum = ((length2 * (length2 + 1)) / 2) as ValueType;
		let float_length = length as ValueType;
		Self {
			sum: sum,
			invert_sum: sum.recip(),
			float_length,
			total: value * float_length,
			numerator: value * sum,
			window: Window::new(length, value),
		}
	}

	#[inline]
	fn next(&mut self, value: Self::Input) -> Self::Output {
		let prev_value = self.window.push(value);

		self.numerator += self.float_length.mul_add(value, -self.total);
		self.total += value - prev_value;

		self.numerator * self.invert_sum
	}
}

#[cfg(test)]
mod tests {
	#![allow(unused_imports)]
	use super::{Method, WMA as TestingMethod};
	use crate::core::ValueType;
	use crate::helpers::RandomCandles;
	use crate::methods::Conv;

	#[allow(dead_code)]
	const SIGMA: ValueType = 1e-5;

	#[test]
	fn test_wma_const() {
		use super::*;
		use crate::core::{Candle, Method};
		use crate::methods::tests::test_const;

		for i in 1..30 {
			let input = (i as ValueType + 56.0) / 16.3251;
			let mut method = TestingMethod::new(i, input);

			let output = method.next(input);
			test_const(&mut method, input, output);
		}
	}

	#[test]
	fn test_wma1() {
		let mut candles = RandomCandles::default();

		let mut ma = TestingMethod::new(1, candles.first().close);

		candles.take(100).for_each(|x| {
			assert!((x.close - ma.next(x.close)).abs() < SIGMA);
		});
	}

	#[test]
	fn test_wma() {
		let candles = RandomCandles::default();

		let src: Vec<ValueType> = candles.take(100).map(|x| x.close).collect();

		(1..20).for_each(|ma_length| {
			let mut ma = TestingMethod::new(ma_length, src[0]);
			let mut conv = Conv::new((1..=ma_length).map(|x| x as ValueType).collect(), src[0]);
			let ma_length = ma_length as usize;

			let div = (1..=ma_length).sum::<usize>() as ValueType;
			src.iter().enumerate().for_each(|(i, &x)| {
				let value = ma.next(x);
				let value2 = (0..ma_length).fold(0.0, |s, v| {
					let j = i.saturating_sub(v);
					s + src[j] * (ma_length - v) as ValueType
				}) / div;
				let value3 = conv.next(x);

				assert!((value2 - value).abs() < SIGMA);
				assert!((value3 - value).abs() < SIGMA);
			});
		});
	}
}
