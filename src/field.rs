#[derive(Debug, Clone)]
pub struct FieldElement<T, const SIZE: usize> {
    items: [T; SIZE],
}

impl<T: Default + Copy, const SIZE: usize> Default for FieldElement<T, SIZE> {
    fn default() -> Self {
        Self {
            items: [T::default(); SIZE],
        }
    }
}

impl FieldElement<u8, 32> {
    pub fn new(items: [u8; 32]) -> Self {
        Self { items }
    }

    // Takes a 32-byte array and unpacks it into a FieldElem
    // by combining every two adjacent bytes together by
    // multiplying the second byte by 256 (2^8) and adding it to the first byte.
    // Forces the MSB (out[15]) to be 0 since these numbers are
    // always less than 2^255 (2^255-19, but we allow [2^255-19, 2^255-1]).
    // We could have used u16 instead of i64 theorectically, i64 prevents
    // any possible overflow/underflow.
    pub fn unpack(&self) -> FieldElement<i64, 16> {
        let mut unpacked = FieldElement::default();
        self.items.chunks(2).enumerate().for_each(|(i, chunk)| {
            unpacked.items[i] = ((chunk[1] as i64) << 8) + chunk[0] as i64;
        });
        unpacked.items[15] = unpacked.items[15] & 0x7fff;
        unpacked
    }
}

impl FieldElement<i64, 16> {
    pub fn add(&mut self, other: &Self) -> &mut Self {
        for i in 0..16 {
            self.items[i] = self.items[i] + other.items[i];
        }

        self
    }

    pub fn sub(&mut self, other: &Self) -> &mut Self {
        for i in 0..16 {
            self.items[i] = self.items[i] - other.items[i];
        }

        self
    }

    pub fn mul(&mut self, other: &Self) -> &mut Self {
        let mut product = [0; 32];
        for i in 0..16 {
            for j in 0..16 {
                product[i + j] += self.items[i] * other.items[j];
            }
        }

        for i in 0..15 {
            product[i] += 38 * product[i + 16];
        }

        for i in 0..16 {
            self.items[i] = product[i];
        }

        self.carry();
        self.carry();
        self
    }

    // To find the inverse of a FieldElem we use Fermat's Little Theorem.
    // a^-1 = a^(p-2) mod p, here p = 2^255-19
    // we use the fact that a^2^N is same as multiplying a^2 by itself N times.
    //
    // p - 2 = 2^255 - 21
    // => 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeb
    // All the bits of p-2 are 1 except for the 2nd and 4th bits.
    //
    // The loop in the inverse function counts down from the
    // most-significant to the least-significant bit, squaring
    // the current value for each bit, and also multipling the
    // result with the input value in for each bit that is 1.
    // Even though p=2 consists of 255 bits, the loop is able to
    // start at bit 253 and save one iteration by initialising
    // the result to in instead of 1.
    pub fn inverse(&mut self) -> &mut Self {
        let initial = self.clone();
        // let mut result = self.clone();
        for i in (0..=253).rev() {
            self.mul(&self.clone());
            if i != 2 && i != 4 {
                self.mul(&initial);
            }
        }

        self
    }

    // If b is 1 and bits in p and q differ, swap the bits in p and q.
    // If b is 0, do nothing. If the bits are the same, do nothing.
    pub fn swap(&mut self, other: &mut Self, b: i64) {
        let c = !(b - 1);
        for i in 0..16 {
            let t = c & (self.items[i] ^ other.items[i]);
            self.items[i] ^= t;
            other.items[i] ^= t;
        }
    }

    // Inspect the field element by examining each element in the array.
    // Each element is shifted right by 16 bits to check if there is a carry.
    // If there is a carry, the carry is subtracted from the current element
    // and added to the next element. If the current element is the last element,
    // the carry is multiplied by 38 (19 * 2) and added to the first element.
    pub fn carry(&mut self) {
        for i in 0..16 {
            // 1. divide by 2^16
            let carry = self.items[i] >> 16;
            // 2. multiply by 2^16 and subtract
            self.items[i] -= carry << 16;
            if i < 15 {
                self.items[i + 1] += carry;
            } else {
                self.items[0] += 38 * carry;
            }
        }
    }

    pub fn pack(&mut self) -> FieldElement<u8, 32> {
        let mut temp = FieldElement::default();
        self.carry();
        self.carry();
        self.carry();
        for _ in 0..2 {
            // except for the first 16 and last 16 bits all the bits are 1
            // 0xffed are the least significant 16 bits of 2^255-19.
            // subtract them from first item of the field element array
            temp.items[0] = self.items[0] - 0xffed;
            for i in 1..15 {
                // subtract 0xffff from intermediate items and also check if there is a carry
                // by checking if i-1th item exceeds 2^16, if it does subtract the carry bits too
                // and adjust the ith item accordingly by taking mod 2^16
                temp.items[i] = self.items[i] - 0xffff - ((temp.items[i - 1] >> 16) & 1);
                temp.items[i - 1] &= 0xffff;
            }
            // 0x7fff are the most significant 16 bits of 2^255-19, subtract them from the last item
            // of the field element array and also check if there is a carry by checking if the
            // 14th item exceeds 2^16, if it does subtract the carry bits too and adjust the 15th
            temp.items[15] = self.items[15] - 0x7fff - ((temp.items[14] >> 16) & 1);
            let carry = (temp.items[15] >> 16) & 1;
            temp.items[14] &= 0xffff;
            self.swap(&mut temp, 1 - carry);
        }

        let mut result = FieldElement::default();
        for i in 0..16 {
            result.items[2 * i] = (self.items[i] & 0xff) as u8;
            result.items[(2 * i) + 1] = (self.items[i] >> 8) as u8;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn packunpack_prop(items in any::<[u8; 32]>(), l in 0u8..128) {
            let mut items = items;
            // force last byte to be less than 128
            // so that the MSB is 0. This is because
            // p = 2^255-19. we only allow numbers
            // in [0,2^255] (see unpack docs)
            items[31] = l;
            let packed = FieldElement { items };
            let mut unpacked = packed.unpack();

            let repacked = unpacked.pack();

            assert_eq!(packed.items, repacked.items);
        }
    }

    proptest! {
        #[test]
        fn addsub_prop(
            a in any::<[u8; 32]>(),
            b in any::<[u8; 32]>(),
            l in 0u8..128,
            m in 0u8..128
        ) {
            let mut a_items = a;
            a_items[31] = l;
            let packed_a = FieldElement { items: a_items };

            let mut b_items = b;
            b_items[31] = m;
            let packed_b = FieldElement { items: b_items };

            let mut unpacked_a = packed_a.unpack();
            let unpacked_b = packed_b.unpack();
            let expected = unpacked_a.clone();

            unpacked_a.add(&unpacked_b);
            unpacked_a.sub(&unpacked_b);

            assert_eq!(unpacked_a.items, expected.items);
        }
    }

    proptest! {
        #[test]
        fn invmul_prop(
            a in any::<[u8; 32]>(),
            l in 0u8..128,
        ) {
            let mut a_items = a;
            // force last byte to be less than 128
            a_items[31] = l;
            let packed_a = FieldElement { items: a_items };
            let mut unpacked_a = packed_a.unpack();

            let a_before_inverse = unpacked_a.clone();
            // b is a inverse
            unpacked_a.inverse();

            // a * a^-1 = 1
            unpacked_a.mul(&a_before_inverse);
            let packed_a = unpacked_a.pack();

            let mut expected = FieldElement::new([0; 32]);
            expected.items[0] = 1 as u8;

            assert_eq!(expected.items, packed_a.items);
        }
    }
}
