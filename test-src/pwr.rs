fn pwr(base: u32, exponent: u32) -> u32 {
	(0..exponent).into_iter().fold(1, |acc, x| acc * base)
}
