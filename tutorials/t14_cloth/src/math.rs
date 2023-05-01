// TODO: replace all of this

#[allow(dead_code, non_snake_case)]
pub fn vecSetZero(a: &mut [f32], anr: usize) {
    let ar = anr * 3;
    a[ar] = 0.0;
    a[ar + 1] = 0.0;
    a[ar + 2] = 0.0;
}

#[allow(non_snake_case)]
pub fn vecScale(a: &mut Vec<f32>, anr: usize, scale: f32) {
    let ar = anr * 3;
    a[ar] *= scale;
    a[ar + 1] *= scale;
    a[ar + 2] *= scale;
}

#[allow(non_snake_case)]
pub fn vecCopy(a: &mut [f32], anr: usize, b: &[f32], bnr: usize) {
    let ar = anr * 3;
    let br = bnr * 3;
    a[ar] = b[br];
    a[ar + 1] = b[br + 1];
    a[ar + 2] = b[br + 2];
}

#[allow(non_snake_case)]
pub fn vecAdd(a: &mut [f32], anr: usize, b: &[f32], bnr: usize, scale: f32) {
    let ar = anr * 3;
    let br = bnr * 3;
    a[ar] += b[br] * scale;
    a[ar + 1] += b[br + 1] * scale;
    a[ar + 2] += b[br + 2] * scale;
}

#[allow(non_snake_case)]
pub fn vecSetDiff(
    dst: &mut [f32],
    dnr: usize,
    a: &[f32],
    anr: usize,
    b: &[f32],
    bnr: usize,
    scale: f32,
) {
    let dr = dnr * 3;
    let ar = anr * 3;
    let br = bnr * 3;
    dst[dr] = (a[ar] - b[br]) * scale;
    dst[dr + 1] = (a[ar + 1] - b[br + 1]) * scale;
    dst[dr + 2] = (a[ar + 2] - b[br + 2]) * scale;
}

#[allow(non_snake_case)]
pub fn vecLengthSquared(a: &[f32], anr: usize) -> f32 {
    let ar = anr * 3;
    let a0 = a[ar];
    let a1 = a[ar + 1];
    let a2 = a[ar + 2];
    return a0 * a0 + a1 * a1 + a2 * a2;
}

#[allow(non_snake_case)]
pub fn vecDistSquared(a: &[f32], anr: usize, b: &[f32], bnr: usize) -> f32 {
    let ar = anr * 3;
    let br = bnr * 3;
    let a0 = a[ar] - b[br];
    let a1 = a[ar + 1] - b[br + 1];
    let a2 = a[ar + 2] - b[br + 2];
    return a0 * a0 + a1 * a1 + a2 * a2;
}

#[allow(non_snake_case)]
pub fn vecDot(a: &[f32], anr: usize, b: &[f32], bnr: usize) -> f32 {
    let ar = anr * 3;
    let br = bnr * 3;
    return a[ar] * b[br] + a[ar + 1] * b[br + 1] + a[ar + 2] * b[br + 2];
}

#[allow(non_snake_case)]
pub fn vecSetCross(a: &mut [f32], anr: usize, b: &[f32], bnr: usize, c: &[f32], cnr: usize) {
    let ar = anr * 3;
    let br = bnr * 3;
    let cr = cnr * 3;
    a[ar] = b[br + 1] * c[cr + 2] - b[br + 2] * c[cr + 1];
    a[ar + 1] = b[br + 2] * c[cr + 0] - b[br + 0] * c[cr + 2];
    a[ar + 2] = b[br + 0] * c[cr + 1] - b[br + 1] * c[cr + 0];
}

#[allow(non_snake_case)]
pub fn matGetDeterminant(a: &[f32; 9]) -> f32 {
    let a11 = a[0];
    let a12 = a[3];
    let a13 = a[6];

    let a21 = a[1];
    let a22 = a[4];
    let a23 = a[7];

    let a31 = a[2];
    let a32 = a[5];
    let a33 = a[8];
    a11*a22*a33 + a12*a23*a31 + a13*a21*a32 - a13*a22*a31 - a12*a21*a33 - a11*a23*a32
}

#[allow(non_snake_case)]
pub fn matSetMult(A: &[f32; 9], a: &mut [f32], anr: usize) {
    // note: use to pass b: &[f32], bnr: usize, but was the same values as a and anr, can cant borrow mut and immutable at the same time
    let bnr = anr * 3;  //bnr *= 3;     
    let bx = a[bnr];
    let by = a[bnr + 1];
    let bz = a[bnr + 2];
    vecSetZero(a, anr);
    vecAdd(a,anr, A,0, bx);
    vecAdd(a,anr, A,1, by);
    vecAdd(a,anr, A,2, bz);
}

#[allow(non_snake_case)]
pub fn matSetInverse(a: &mut [f32; 9]) {
    let det = matGetDeterminant(a);
    if det == 0.0 {
        for i in 0..9 {
            a[i] = 0.0; // this was 'anr + i', think it was a bug and just defaulted to 0
            return;
        }
    }
    let invDet = 1.0 / det;
    let a11 = a[0];
    let a12 = a[3]; 
    let a13 = a[6];
    
    let a21 = a[1];
    let a22 = a[4];
    let a23 = a[7];

    let a31 = a[2];
    let a32 = a[5];
    let a33 = a[8];

    a[0] =  (a22 * a33 - a23 * a32) * invDet; 
    a[3] = -(a12 * a33 - a13 * a32) * invDet;
    a[6] =  (a12 * a23 - a13 * a22) * invDet;
    a[1] = -(a21 * a33 - a23 * a31) * invDet;
    a[4] =  (a11 * a33 - a13 * a31) * invDet;
    a[7] = -(a11 * a23 - a13 * a21) * invDet;
    a[2] =  (a21 * a32 - a22 * a31) * invDet;
    a[5] = -(a11 * a32 - a12 * a31) * invDet;
    a[8] =  (a11 * a22 - a12 * a21) * invDet;
}