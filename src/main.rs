type FieldElem = [i64; 16];

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

fn main() {
    println!("{:?}", FieldElem::default());
}
