use parity_scale_codec::{Compact, Decode, Encode, Output};
use parity_scale_codec_derive::{Decode as DecodeDerive, Encode as EncodeDerive};
use std::vec::Vec;

/// A trivial and fast shuffle used by tests.
pub fn shuffle<T>(slice: &mut [T], seed: u64) {
	let mut r = seed as usize;
	for i in (1..slice.len()).rev() {
		let j = r % (i + 1);
		slice.swap(i, j);
		r = r.wrapping_mul(6364793005) + 1;
	}
}

trait Shuffle {
	fn shuffle(self) -> Self;
}

impl<T> Shuffle for Vec<T> {
	fn shuffle(mut self) -> Self {
		let seed = self.len() as u64;
		shuffle(&mut self[..], seed);
		self
	}
}

macro_rules! seq {
    ( $( $x:expr ),* ) => {
        ( $( $x ),* )
    };
}

macro_rules! myvec {
    // Case 1: Create a vector from a list of elements
    ($($elem:expr),* $(,)?) => {
        {
            let mut vec = Vec::new();
            $(vec.push($elem);)*
            vec
        }
    };
    // Case 2: Create a vector by repeating an element
    ($elem:expr; $count:expr) => {
        {
            let mut vec = Vec::with_capacity($count);
            vec.extend(std::iter::repeat($elem).take($count));
            vec
        }
    };
    ($start:expr => $end:expr) => {
        {
            let mut v = Vec::new();
            for i in $start..$end {
                v.push(i);
            }
			v
        }
    };
}

#[derive(EncodeDerive, DecodeDerive, std::fmt::Debug, PartialEq)]
enum TestEnum<T> {
	Dummy,
	Foo(T),
	Bar([T; 8]),
}

fn process<T: Encode + Decode + std::fmt::Debug + PartialEq>(v: T) {
	println!("-------------------------------");
	println!("[{}]", std::any::type_name::<T>());
	// dbg!(&v);
	let b = v.encode();
	println!("{}", hex::encode(&b));
	let d = T::decode(&mut &b[..]).unwrap();
	assert_eq!(v, d);
}

#[test]
fn gen_vectors() {
	// Sequences in different flavors
	// NOTE: In the end everything can be once of these three types:
	// - A primitive integer
	// - A non-uniform fixed length sequence (aka tuple / struct)
	// - A uniform fixed length sequence (aka an array)
	// - A uniform variable length sequence (aka a vector)
	// - An "choice" (aka an enum)

	// Non-uniform fixed length sequence

	process(seq!(0xf1_u8, 0x1234_u16, 0xFF00cc11_u32, 0x1231092319023131_u64));

	#[rustfmt::skip]
	process(seq!{
		0xf1_u8,
		seq!{
			seq!{
				0x1234_u16,
				0xFF00cc11_u32
			},
			seq!{
				0x1231092319023131_u64,
				seq! {
					0x32_u8
				},
				3_i32
			}
		}
	});

	// Uniform fixed length sequences

	process([0_u8; 0]);
	process([(3_u8, 0x3122_u16), (8, 0x3321), (9, 0x9973)]);
	process(TryInto::<[u8; 16]>::try_into(myvec![0_u8 => 16].shuffle()).unwrap());

	// Uniform variable length sequences

	process(myvec![1_u16, 2, 3]);
	process(myvec!(0_u16 => 127));
	process(myvec!(0_u8 => 200));

	// Enumerations

	process(TestEnum::<u8>::Dummy);
	process(TestEnum::Foo(42_u8));
	process(TestEnum::Bar([1_u8, 2, 3, 4, 5, 6, 7, 8]));

	// Optional entries

	process(Option::<u16>::None);
	process(Some(42_u8));

	#[rustfmt::skip]
	process(
		myvec!(0 => 15).shuffle().iter().map(|&i|
			if i % 3 == 0 {
				Option::None
			} else {
				Option::Some(myvec![0_u8 => i as u8].shuffle())
			},
		).collect::<Vec<_>>()
	);

	#[rustfmt::skip]
	process(seq! {
		(Option::Some(0x1234_u16), 42_u8),
		myvec!(0 => 15).shuffle().iter().map(|&i|
			(
				i as u8,
				if i % 3 == 0 {
					Option::None
				} else {
					Option::Some(seq!(i % 5 as u16, myvec![0_u8 => i as u8].shuffle()))
				},
	 		)
		).collect::<Vec<_>>()
	});

	// A mix of the above

	#[rustfmt::skip]
	process(
		myvec!(0 => 10).shuffle().iter().map(|&i|
			seq!{
				i as u16,
				seq! {
					2 * i as u64,
					Some(3 * i as u8)
				}
			}
		)
		.collect::<Vec<_>>()
	);

	#[rustfmt::skip]
	process(seq! {
		3_u8,
		seq! {
			0x5242_u16,
			0x3312_u16
		},
		myvec!(0_u16 => 12).shuffle(),
		myvec!(0_u8 => 30).shuffle().iter().map(|&i| seq!(i as u8, i as u32)).collect::<Vec<_>>()
	});

	// Some compact values

	process(Compact(0_u32));
	process(Compact(127_u32));
	process(Compact(128_u32));
	process(Compact(1023_u32));
	process(Compact(0x1000_u32));
	process(Compact(0x3fff_u32));
	process(Compact(0x4000_u32));
	process(Compact(0xfff1_u32));
	process(Compact(0x1fffff_u32));
	process(Compact(0x200000_u32));
	process(Compact(0xfff1ff_u32));
	process(Compact(0xffffffffff_u64));
	process(Compact(0xab1c50bbc19a_u64));

	// Bit string
	println!("------------------------");
	println!("Fixed-length bit-strings");
	let bits = vec![0].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_fl(&bits)));
	let bits = vec![0, 0, 0].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_fl(&bits)));
	let bits = vec![1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_fl(&bits)));
	let bits = vec![1, 1, 0, 1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_fl(&bits)));
	let bits = vec![0, 0, 1, 1, 0, 0, 1, 1, 0, 1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_fl(&bits)));

	println!("------------------------");
	println!("Variable-length bit-strings");
	let bits = vec![0].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_vl(&bits)));
	let bits = vec![0, 0, 0].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_vl(&bits)));
	let bits = vec![1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_vl(&bits)));
	let bits = vec![1, 0, 1, 1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_vl(&bits)));
	let bits = vec![1, 0, 1, 1, 0, 1, 1, 1, 0, 1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_vl(&bits)));
	let bits = vec![0, 1, 0, 1, 1, 1].iter().map(|&b| b != 0).collect::<Vec<_>>();
	println!("{}", hex::encode(bit_string_encode_vl(&bits)));
}

fn bit_string_encode_vl(bit_string: &[bool]) -> Vec<u8> {
	let mut buf = Compact(bit_string.len() as u64).encode();
	bit_string.chunks(8).for_each(|chunk| {
		let o = chunk.iter().enumerate().fold(0, |octet, (i, &bit)| octet | ((bit as u8) << i));
		buf.push_byte(o);
	});
	buf
}

fn bit_string_encode_fl(bit_string: &[bool]) -> Vec<u8> {
	let mut buf = Vec::with_capacity((bit_string.len() + 7) / 8);
	bit_string.chunks(8).for_each(|chunk| {
		let o = chunk.iter().enumerate().fold(0, |octet, (i, &bit)| octet | ((bit as u8) << i));
		buf.push_byte(o);
	});
	buf
}
