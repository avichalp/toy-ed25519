use std::i64;

type FieldElem = [i64; 16];

#[derive(Debug, Clone, PartialEq)]
struct FieldElement<T, const SIZE: usize> {
    items: [T; SIZE],
}

impl<T, const SIZE: usize> FieldElement<T, SIZE> {
    fn len(&self) -> usize {
        SIZE
    }
}

impl FieldElement<u8, 32> {
    fn new(items: [u8; 32]) -> Self {
        Self { items }
    }

    fn unpack(&self) -> FieldElement<i64, 16> {
        let mut unpacked = FieldElement { items: [0; 16] };
        for i in 0..16 {
            let byte1 = self.items[2 * i] as i64;
            let byte2 = self.items[(2 * i) + 1] as i64;
            unpacked.items[i] = (byte2 << 8) + byte1;
        }
        unpacked.items[15] = unpacked.items[15] & 0x7fff;
        unpacked
    }
}

impl FieldElement<i64, 16> {
    fn add(&self, other: &Self) -> Self {
        let mut result = Self { items: [0; 16] };
        for i in 0..16 {
            result.items[i] = self.items[i] + other.items[i];
        }
        result
    }

    fn sub(&self, other: &Self) -> Self {
        let mut result = Self { items: [0; 16] };
        for i in 0..16 {
            result.items[i] = self.items[i] - other.items[i];
        }
        result
    }

    fn mul(&self, other: &Self) -> Self {
        // todo: impl a `product` method for [i64; 32]
        let mut product: [i64; 32] = [0; 32];
        for i in 0..16 {
            for j in 0..16 {
                product[i + j] += self.items[i] * other.items[j];
            }
        }
        for i in 0..15 {
            product[i] += 38 * product[i + 16];
        }

        let mut result = Self { items: [0; 16] };
        for i in 0..16 {
            result.items[i] = product[i];
        }
        result.carry();
        result.carry();
        result
    }

    fn inverse(&self) -> Self {
        // println!("mod inversing");
        let mut temp = self.clone();
        // let mut result = self.clone();
        for i in (0..=253).rev() {
            // println!("temp == {:?}, {:?}", temp, i);
            temp = temp.mul(&temp);
            if i != 2 && i != 4 {
                temp = temp.mul(&self);
            }
        }
        temp
    }

    fn swap(&mut self, other: &mut Self, b: i64) {
        let c = !(b - 1);
        for i in 0..16 {
            let t = c & (self.items[i] ^ other.items[i]);
            self.items[i] ^= t;
            other.items[i] ^= t;
        }
    }

    fn carry(&mut self) {
        for i in 0..16 {
            let carry = self.items[i] >> 16; // 1. divide by 2^16
            self.items[i] -= carry << 16; // 2. multiply by 2^16 and subtract
            if i < 15 {
                self.items[i + 1] += carry;
            } else {
                self.items[0] += 38 * carry;
            }
        }
    }

    fn pack(&mut self) -> FieldElement<u8, 32> {
        let mut temp = Self { items: [0; 16] };
        self.carry();
        self.carry();
        self.carry();
        for _ in 0..2 {
            // 0xffed are the least significant 16 bits of 2^255-19
            // except for the first 16 and last 16 bits all the bits are 1
            temp.items[0] = self.items[0] - 0xffed;
            for i in 1..15 {
                temp.items[i] = self.items[i] - 0xffff - ((temp.items[i - 1] >> 16) & 1);
                temp.items[i - 1] &= 0xffff;
            }
            // 0x7fff are the most significant 16 bits of 2^255-19
            temp.items[15] = self.items[15] - 0x7fff - ((temp.items[14] >> 16) & 1);
            let carry = (temp.items[15] >> 16) & 1;
            temp.items[14] &= 0xffff;
            self.swap(&mut temp, 1 - carry);
        }

        let mut result = FieldElement { items: [0; 32] };
        for i in 0..16 {
            result.items[2 * i] = (self.items[i] & 0xff) as u8;
            result.items[(2 * i) + 1] = (self.items[i] >> 8) as u8;
        }

        result
    }
}

// Takes a 32-byte array and unpacks it into a FieldElem
// by combining every two adjacent bytes together by
// multiplying the second byte by 256 (2^8) and adding it to the first byte.
// Forces the MSB (out[15]) to be 0 since these numbers are
// always less than 2^255 (2^255-19, but we allow [2^255-19, 2^255-1]).
// We could have used u16 instead of i64 theorectically, i64 prevents
// any possible overflow/underflow.
pub fn unpack25519(input: &[u8]) -> FieldElem {
    let mut out: FieldElem = [0; 16];
    for i in 0..16 {
        let byte1 = input[2 * i] as i64;
        let byte2 = input[(2 * i) + 1] as i64;
        out[i] = (byte2 << 8) + byte1;
    }
    out[15] = out[15] & 0x7fff;
    out
}

// Inspect the field element by examining each element in the array.
// Each element is shifted right by 16 bits to check if there is a carry.
// If there is a carry, the carry is subtracted from the current element
// and added to the next element. If the current element is the last element,
// the carry is multiplied by 38 (19 * 2) and added to the first element.
pub fn carry25519(elem: &mut FieldElem) {
    for i in 0..16 {
        let carry = elem[i] >> 16; // 1. divide by 2^16
        elem[i] -= carry << 16; // 2. multiply by 2^16 and subtract
        if i < 15 {
            elem[i + 1] += carry;
        } else {
            elem[0] += 38 * carry;
        }
    }
}

pub fn fadd(a: &FieldElem, b: &FieldElem) -> FieldElem {
    let mut out: FieldElem = [0; 16];
    for i in 0..16 {
        out[i] = a[i] + b[i];
    }
    out
}

pub fn fsub(a: &FieldElem, b: &FieldElem) -> FieldElem {
    let mut out: FieldElem = [0; 16];
    for i in 0..16 {
        out[i] = a[i] - b[i];
    }
    out
}

pub fn fmul(a: &FieldElem, b: &FieldElem) -> FieldElem {
    let mut product: [i64; 32] = [0; 32];
    for i in 0..16 {
        for j in 0..16 {
            product[i + j] += a[i] * b[j];
        }
    }
    for i in 0..15 {
        product[i] += 38 * product[i + 16];
    }

    println!("product == {:?}", product);

    let mut result = product[0..16].try_into().unwrap();
    carry25519(&mut result);
    carry25519(&mut result);
    result
}

// To find the inverse of a FieldElem we use Fermat's Little Theorem.
// a^-1 = a^(p-2) mod p, here p = 2^255-19
// we use the fact that a^2^N is same as multiplying a^2 by itself N times.
//
// p - 2 = 2^255 - 21
// => 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeb
// All the bits of p-2 are 1 except for the 2nd and 4th bits.
//
// The loop in the f inverse function counts down from the
// most-significant to the least-significant bit, squaring
// the current value c using the fmul function for each bit,
// and also multipling c with the input value in for each bit that is 1.
// Even though p=2 consists of 255 bits, the loop is able to start
// at bit 253 and save one iteration by initialising c to in instead of 1.
// At the end, c is copied to the output variable out.
pub fn finverse(mut out: FieldElem, iin: FieldElem) {
    let mut result: FieldElem = [0; 16];
    for i in 0..16 {
        result[i] = iin[i];
    }

    for i in 253..=0 {
        result = fmul(&result, &result);
        if i != 2 && i != 4 {
            result = fmul(&result, &iin);
        }
    }

    println!("result == {:?}", result);

    for i in 0..16 {
        out[i] = result[i];
    }
}

// If b is 1 and bits in p and q differ, swap the bits in p and q.
// If b is 0, do nothing. If the bits are the same, do nothing.
pub fn swap25519(p: &mut FieldElem, q: &mut FieldElem, b: i64) {
    let c = !(b - 1);
    for i in 0..16 {
        let t = c & (p[i] ^ q[i]);
        p[i] ^= t;
        q[i] ^= t;
    }
}

pub fn pack25519(input: &mut FieldElem) -> [u8; 32] {
    let mut m: FieldElem = [0; 16];
    carry25519(input);
    carry25519(input);
    carry25519(input);
    for _ in 0..2 {
        // 0xffed are the least significant 16 bits of 2^255-19
        // except for the first 16 and last 16 bits all the bits are 1
        m[0] = input[0] - 0xffed;
        for i in 1..15 {
            m[i] = input[i] - 0xffff - ((m[i - 1] >> 16) & 1);
            m[i - 1] &= 0xffff;
        }
        // 0x7fff are the most significant 16 bits of 2^255-19
        m[15] = input[15] - 0x7fff - ((m[14] >> 16) & 1);
        let carry = (m[15] >> 16) & 1;
        m[14] &= 0xffff;
        swap25519(input, &mut m, 1 - carry);
    }

    let mut out: [u8; 32] = [0; 32];
    println!("pre packed == {:?}", input);
    for i in 0..16 {
        out[2 * i] = (input[i] & 0xff) as u8;
        out[(2 * i) + 1] = (input[i] >> 8) as u8;
    }

    out
}

fn main() {
    let mut fe1 = FieldElement::new([0; 32]);
    fe1.items[0] = 0x02 as u8;
    println!("fe {:?}", fe1);
    let unpacked_fe1 = fe1.unpack();
    println!("unpacked {:?}", unpacked_fe1);

    /* let mut fe2 = FieldElement::new([0; 32]);
       fe2.items[0] = 0x03 as u8;
       println!("fe {:?}", fe2);
       let unpacked_fe2 = fe2.unpack();
       println!("unpacked {:?}", unpacked_fe2);

       let mut fe3 = unpacked_fe1.mul(&unpacked_fe2);
       println!("fe3 {:?}", fe3);
       let packed_fe3 = fe3.pack();
       println!("packed fe3 {:?}", packed_fe3);
    */
    let mut fe4 = unpacked_fe1.inverse();
    println!("fe4 {:?}", fe4);
    let packed_fe4 = fe4.pack();
    println!("packed fe4 {:?}", packed_fe4);

    /* let mut packed_a: [u8; 32] = [0; 32];
       packed_a[0] = 0x02 as u8;

       let mut packed_b: [u8; 32] = [0; 32];
       packed_b[0] = 0x03 as u8;
    */
    // let unpacked_a = unpack25519(&packed_a);
    // let unpacked_b = unpack25519(&packed_b);
    // println!("unpacked a {:?}", unpacked_a);
    // println!("unpacked b {:?}", unpacked_b);
    // let unpacked_c = unpack25519(&packed_c);

    // let mut out = fmul(&unpacked_a, &unpacked_b);
    // let out: [u8; 32] = [0; 32];
    // let unpacked_out = unpack25519(&out);

    // finverse(unpacked_out, unpacked_b);
    // println!("unpacked out (inverse) {:?}", unpacked_out);

    // println!("{:?}", pack25519(&mut out));
    // println!("{:?}", unpack25519(&pack25519(out)));
    // println!("{:?}", pack25519(fadd(&unpacked_b, &unpacked_b)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packunpack() {
        let mut packed = FieldElement { items: [0; 32] };
        packed.items[31] = 0x2a as u8;
        let mut unpacked = packed.unpack();

        let repacked = unpacked.pack();

        assert_eq!(packed, repacked);
    }

    #[test]
    fn addsub() {
        let mut packed_a = FieldElement { items: [0; 32] };
        packed_a.items[31] = 0x15 as u8;

        let mut packed_b = FieldElement { items: [0; 32] };
        packed_b.items[31] = 0x15 as u8;

        let unpacked_a = packed_a.unpack();
        let unpacked_b = packed_b.unpack();
        let unpacked_c = unpacked_a.add(&unpacked_b);

        assert_eq!(unpacked_a, unpacked_c.sub(&unpacked_b));
    }

    #[test]
    fn mul() {
        let mut packed_a = FieldElement { items: [0; 32] };
        packed_a.items[0] = 0x2 as u8;

        let mut packed_b = FieldElement { items: [0; 32] };
        packed_b.items[0] = 0x3 as u8;

        let unpacked_a = packed_a.unpack();
        let unpacked_b = packed_b.unpack();
        let mut unpacked_c = unpacked_a.mul(&unpacked_b);

        let mut expected_packed: FieldElement<u8, 32> = FieldElement { items: [0; 32] };
        expected_packed.items[0] = 0x6;

        assert_eq!(expected_packed, unpacked_c.pack());
    }
}
