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

fn main() {
    println!("{:?}", FieldElem::default());
}
