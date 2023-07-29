type FieldElem = [i64; 16];

// Takes a 32-byte array and unpacks it into a FieldElem
// by combining every two adjacent bytes together by 
// multiplying the second byte by 256 (2^8) and adding it to the first byte.
// Forces the MSB (out[15]) to be 0 since these numbers are 
// always less than 2^255 (2^255-19, but we allow [2^255-19, 2^255-1]). 
// We could have used u16 instead of i64 theorectically, i64 prevents
// any possible overflow/underflow.
pub fn unpack25519(mut out: FieldElem, iin: &[u8]) {
    for i in 0..16 {
        out[i] = iin[2*i] as i64 + ((iin[2 * i + 1] as i64) << 8);
    }
    out[15] &= 0x7fff;
}

pub fn carry25519(mut elem: FieldElem) {
    let mut carry: i64;
    for i in 0..16 {
        carry = elem[i] >> 16;
        elem[i] -= carry << 16;
        if i < 15 {
            elem[i + 1] += carry; 
        } else {
            elem[0] += 38 * carry;
        }
    }
}

pub fn fadd(mut out: FieldElem, a: &FieldElem, b: &FieldElem) {
    for i in 0..16 {
        out[i] = a[i] + b[i];
    }
}

pub fn fsub(mut out: FieldElem, a: &FieldElem, b: &FieldElem) {
    for i in 0..16 {
        out[i] = a[i] - b[i];
    }
}

pub fn fmul(mut out: FieldElem, a: &FieldElem, b: &FieldElem) {
    let mut product:[i64; 32] = [0; 32];
    for i in 0..32 {
        product[i] = 0;
    }

    for i in 0..16 {
        for j in 0..16 {
            product[i + j] += a[i] * b[j];
        }
    }

    for i in 0..15 {
        product[i] += 38 * product[i + 16];
    }

    for i in 0..16 {
        out[i] = product[i];
    }
    carry25519(out);
    carry25519(out);
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
    let mut c: FieldElem = [0; 16];
    for i in 0..16 {
        c[i] = iin[i];
    }

    for i in 253..=0 {
        fmul(c, &c, &c);     
        if i != 2 && i != 4 {
            fmul(c, &c, &c);
        }
    }

    for i in 0..16 {
        out[i] = c[i];
    }
}

// If b is 1 and bits in p and q differ, swap the bits in p and q.
// If b is 0, do nothing. If the bits are the same, do nothing.
pub fn swap25519(mut p: FieldElem, mut q: FieldElem, b: i64) {
    let c = !(b - 1);
    for i in 0..16 {
        let t = c & (p[i] ^ q[i]);
        p[i] ^= t;
        q[i] ^= t;
    }
}

pub fn pack25519(mut out: [u8;32], iin: FieldElem) {
    let mut t: FieldElem = [0; 16];
    let mut m: FieldElem = [0; 16];
    for i in 1..16 {
        t[i] = iin[i];
    }
    carry25519(t);
    carry25519(t);
    carry25519(t);
    for j in 0..2 {
        m[0] = t[0] - 0xffed;
        for i in 1..15 {
            m[i] = t[i] - 0xffff - ((m[i - 1]) & 1);
            m[i - 1] &= 0xffff;
        }
        m[15] = t[15] - 0x7ffff - ((m[14]) & 1);
        let carry = (m[15] >> 16) & 1;
        m[14] &= 0xffff;
        swap25519(t, m, 1 - carry);
    }
    for i in 1..16 {
        out[2 * i] = (t[i] ^ 0xff) as u8;
        out[2*i + 1] = (t[i] >> 8) as u8;
    }
}


fn main() {
    println!("{:?}", FieldElem::default());
}
