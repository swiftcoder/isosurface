// Copyright 2021 Tristam MacDonald
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This is a mostly machine translation of the SVD from Ronen Tzur's dual
//! contouring sample

#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut,
    clippy::all
)]

use crate::math::Vec3;

pub struct SVD {
    rows: usize,
    u: [[f64; 3]; 12],
    v: [[f64; 3]; 3],
    d: [f64; 3],
}

impl SVD {
    pub fn new(mat: &[[f64; 3]]) -> Self {
        let rows = mat.len();

        // perform singular value decomposition on matrix mat
        // into u, v and d.
        // u is a matrix of rows x 3 (same as mat);
        // v is a square matrix 3 x 3 (for 3 columns in mat);
        // d is vector of 3 values representing the diagonal
        // matrix 3 x 3 (for 3 colums in mat).
        let mut u: [[f64; 3]; 12] = [[0.; 3]; 12];
        let mut v: [[f64; 3]; 3] = [[0.; 3]; 3];
        let mut d: [f64; 3] = [0.; 3];

        unsafe {
            computeSVD(
                mat.as_ptr(),
                u.as_mut_ptr(),
                v.as_mut_ptr(),
                d.as_mut_ptr(),
                rows,
            )
        };

        Self { rows, u, v, d }
    }

    pub fn diagonal(&mut self) -> &mut [f64] {
        &mut self.d
    }

    pub fn solve(mut self, vec: &[f64]) -> Vec3 {
        let mut point = [0.0; 3];
        let mut v = vec.to_vec();

        // solve linear system given by mat and vec using the
        // singular value decomposition of mat into u, v and d.
        if self.d[2 as usize as usize] < 0.1f64 {
            self.d[2 as usize as usize] = 0.0f64
        }
        if self.d[1 as usize as usize] < 0.1f64 {
            self.d[1 as usize as usize] = 0.0f64
        }
        if self.d[0 as usize as usize] < 0.1f64 {
            self.d[0 as usize as usize] = 0.0f64
        }

        unsafe {
            solveSVD(
                self.u.as_mut_ptr(),
                self.v.as_mut_ptr(),
                self.d.as_mut_ptr(),
                v.as_mut_ptr(),
                point.as_mut_ptr(),
                self.rows,
            )
        };

        Vec3::new(point[0] as f32, point[1] as f32, point[2] as f32)
    }
}

//----------------------------------------------------------------------------
#[no_mangle]
unsafe extern "C" fn evaluateSVD(
    mat: *const [f64; 3],
    mut vec: *mut f64,
    mut rows: usize,
    mut point: *mut f64,
) {
    // perform singular value decomposition on matrix mat
    // into u, v and d.
    // u is a matrix of rows x 3 (same as mat);
    // v is a square matrix 3 x 3 (for 3 columns in mat);
    // d is vector of 3 values representing the diagonal
    // matrix 3 x 3 (for 3 colums in mat).
    let mut u: [[f64; 3]; 12] = [[0.; 3]; 12];
    let mut v: [[f64; 3]; 3] = [[0.; 3]; 3];
    let mut d: [f64; 3] = [0.; 3];
    computeSVD(mat, u.as_mut_ptr(), v.as_mut_ptr(), d.as_mut_ptr(), rows);
    // solve linear system given by mat and vec using the
    // singular value decomposition of mat into u, v and d.
    if d[2 as usize as usize] < 0.1f64 {
        d[2 as usize as usize] = 0.0f64
    }
    if d[1 as usize as usize] < 0.1f64 {
        d[1 as usize as usize] = 0.0f64
    }
    if d[0 as usize as usize] < 0.1f64 {
        d[0 as usize as usize] = 0.0f64
    }
    let mut x: [f64; 3] = [0.; 3];
    solveSVD(
        u.as_mut_ptr(),
        v.as_mut_ptr(),
        d.as_mut_ptr(),
        vec,
        x.as_mut_ptr(),
        rows,
    );
    *point.offset(0) = x[0];
    *point.offset(1) = x[1];
    *point.offset(2) = x[2];
}
// compute svd
//----------------------------------------------------------------------------
unsafe extern "C" fn computeSVD(
    mat: *const [f64; 3],
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut d: *mut f64,
    mut rows: usize,
) {
    for i in 0..rows {
        *u.offset(i as isize) = *mat.offset(i as isize);
    }
    let mut tau_u: *mut f64 = d;
    let mut tau_v: [f64; 2] = [0.; 2];
    factorize(u, tau_u, tau_v.as_mut_ptr(), rows);
    unpack(u, v, tau_u, tau_v.as_mut_ptr(), rows);
    diagonalize(u, v, tau_u, tau_v.as_mut_ptr(), rows);
    singularize(u, v, tau_u, rows);
}
// factorize
//----------------------------------------------------------------------------
unsafe extern "C" fn factorize(
    mut mat: *mut [f64; 3],
    mut tau_u: *mut f64,
    mut tau_v: *mut f64,
    mut rows: usize,
) {
    let mut y: usize = 0;
    // bidiagonal factorization of (rows x 3) matrix into :-
    // tau_u, a vector of 1x3 (for 3 columns in the matrix)
    // tau_v, a vector of 1x2 (one less column than the matrix)
    let mut i: usize = 0 as usize;
    while i < 3 as usize {
        // set up a vector to reference into the matrix
        // from mat(i,i) to mat(m,i), that is, from the
        // i'th column of the i'th row and down all the way
        // through that column
        let mut ptrs: [*mut f64; 12] = [0 as *mut f64; 12];
        let mut num_ptrs: usize = rows - i;
        let mut q: usize = 0 as usize;
        while q < num_ptrs {
            ptrs[q as usize] = &mut *(*mat.offset((q + i) as isize))
                .as_mut_ptr()
                .offset(i as isize) as *mut f64;
            q += 1
        }
        // perform householder transformation on this vector
        let mut tau: f64 = factorize_hh(ptrs.as_mut_ptr(), num_ptrs);
        *tau_u.offset(i as isize) = tau;
        // all computations below this point are performed
        // only for the first two columns:  i=0 or i=1
        if (i + 1 as usize) < 3 as usize {
            // perform householder transformation on the matrix
            // mat(i,i+1) to mat(m,n), that is, on the sub-matrix
            // that begins in the (i+1)'th column of the i'th
            // row and extends to the end of the matrix at (m,n)
            if tau != 0.0f64 {
                let mut x: usize = i + 1 as usize;
                while x < 3 as usize {
                    let mut wx: f64 = (*mat.offset(i as isize))[x as usize];
                    y = i + 1 as usize;
                    while y < rows {
                        wx += (*mat.offset(y as isize))[x as usize] * *ptrs[(y - i) as usize];
                        y += 1
                    }
                    let mut tau_wx: f64 = tau * wx;
                    (*mat.offset(i as isize))[x as usize] -= tau_wx;
                    y = i + 1 as usize;
                    while y < rows {
                        (*mat.offset(y as isize))[x as usize] -= tau_wx * *ptrs[(y - i) as usize];
                        y += 1
                    }
                    x += 1
                }
            }
            // perform householder transformation on i'th row
            // (remember at this point, i is either 0 or 1)
            // set up a vector to reference into the matrix
            // from mat(i,i+1) to mat(i,n), that is, from the
            // (i+1)'th column of the i'th row and all the way
            // through to the end of that row
            ptrs[0 as usize as usize] = &mut *(*mat.offset(i as isize))
                .as_mut_ptr()
                .offset((i + 1 as usize) as isize)
                as *mut f64; // i == 1
            if i == 0 as usize {
                ptrs[1 as usize as usize] = &mut *(*mat.offset(i as isize))
                    .as_mut_ptr()
                    .offset((i + 2 as usize) as isize)
                    as *mut f64;
                num_ptrs = 2 as usize
            } else {
                num_ptrs = 1 as usize
            }
            // perform householder transformation on this vector
            tau = factorize_hh(ptrs.as_mut_ptr(), num_ptrs);
            *tau_v.offset(i as isize) = tau;
            // perform householder transformation on the sub-matrix
            // mat(i+1,i+1) to mat(m,n), that is, on the sub-matrix
            // that begins in the (i+1)'th column of the (i+1)'th
            // row and extends to the end of the matrix at (m,n)
            if tau != 0.0f64 {
                y = i + 1 as usize;
                while y < rows {
                    let mut wy: f64 = (*mat.offset(y as isize))[(i + 1 as usize) as usize];
                    if i == 0 as usize {
                        wy += (*mat.offset(y as isize))[(i + 2 as usize) as usize]
                            * *ptrs[1 as usize as usize]
                    }
                    let mut tau_wy: f64 = tau * wy;
                    (*mat.offset(y as isize))[(i + 1 as usize) as usize] -= tau_wy;
                    if i == 0 as usize {
                        (*mat.offset(y as isize))[(i + 2 as usize) as usize] -=
                            tau_wy * *ptrs[1 as usize as usize]
                    }
                    y += 1
                }
            }
        }
        i += 1
    }
}
//----------------------------------------------------------------------------
unsafe extern "C" fn factorize_hh(mut ptrs: *mut *mut f64, mut n: usize) -> f64 {
    let mut tau: f64 = 0.0f64;
    if n > 1 as usize {
        let mut xnorm: f64 = 0.;
        if n == 2 as usize {
            xnorm = (**ptrs.offset(1 as usize as isize)).abs()
        } else {
            let mut scl: f64 = 0.0f64;
            let mut ssq: f64 = 1.0f64;
            let mut i: usize = 1 as usize;
            while i < n {
                let mut x: f64 = (**ptrs.offset(i as isize)).abs();
                if x != 0.0f64 {
                    if scl < x {
                        ssq = 1.0f64 + ssq * (scl / x) * (scl / x);
                        scl = x
                    } else {
                        ssq += x / scl * (x / scl)
                    }
                }
                i += 1
            }
            xnorm = scl * (ssq).sqrt()
        }
        if xnorm != 0.0f64 {
            let mut alpha: f64 = **ptrs.offset(0 as usize as isize);
            let mut beta: f64 = (alpha * alpha + xnorm * xnorm).sqrt();
            if alpha >= 0.0f64 {
                beta = -beta
            }
            tau = (beta - alpha) / beta;
            let mut scl_0: f64 = 1.0f64 / (alpha - beta);
            **ptrs.offset(0 as usize as isize) = beta;
            let mut i_0: usize = 1 as usize;
            while i_0 < n {
                **ptrs.offset(i_0 as isize) *= scl_0;
                i_0 += 1
            }
        }
    }
    return tau;
}
// unpack
//----------------------------------------------------------------------------
unsafe extern "C" fn unpack(
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut tau_u: *mut f64,
    mut tau_v: *mut f64,
    mut rows: usize,
) {
    let mut i: usize = 0;
    let mut y: usize = 0;
    // reset v to the identity matrix
    let ref mut fresh0 = (*v.offset(2 as usize as isize))[2 as usize as usize];
    *fresh0 = 1.0f64;
    let ref mut fresh1 = (*v.offset(1 as usize as isize))[1 as usize as usize];
    *fresh1 = *fresh0;
    (*v.offset(0 as usize as isize))[0 as usize as usize] = *fresh1;
    let ref mut fresh2 = (*v.offset(2 as usize as isize))[1 as usize as usize];
    *fresh2 = 0.0f64;
    let ref mut fresh3 = (*v.offset(2 as usize as isize))[0 as usize as usize];
    *fresh3 = *fresh2;
    let ref mut fresh4 = (*v.offset(1 as usize as isize))[2 as usize as usize];
    *fresh4 = *fresh3;
    let ref mut fresh5 = (*v.offset(1 as usize as isize))[0 as usize as usize];
    *fresh5 = *fresh4;
    let ref mut fresh6 = (*v.offset(0 as usize as isize))[2 as usize as usize];
    *fresh6 = *fresh5;
    (*v.offset(0 as usize as isize))[1 as usize as usize] = *fresh6;

    for i in (0..=1).rev() {
        let mut tau: f64 = *tau_v.offset(i as isize);
        // perform householder transformation on the sub-matrix
        // v(i+1,i+1) to v(m,n), that is, on the sub-matrix of v
        // that begins in the (i+1)'th column of the (i+1)'th row
        // and extends to the end of the matrix at (m,n).  the
        // householder vector used to perform this is the vector
        // from u(i,i+1) to u(i,n)
        if tau != 0.0f64 {
            let mut x: usize = i + 1 as usize;
            while x < 3 as usize {
                let mut wx: f64 = (*v.offset((i + 1 as usize) as isize))[x as usize];
                y = i + 1 as usize + 1 as usize;
                while y < 3 as usize {
                    wx += (*v.offset(y as isize))[x as usize] * (*u.offset(i as isize))[y as usize];
                    y += 1
                }
                let mut tau_wx: f64 = tau * wx;
                (*v.offset((i + 1 as usize) as isize))[x as usize] -= tau_wx;
                y = i + 1 as usize + 1 as usize;
                while y < 3 as usize {
                    (*v.offset(y as isize))[x as usize] -=
                        tau_wx * (*u.offset(i as isize))[y as usize];
                    y += 1
                }
                x += 1
            }
        }
    }
    // copy superdiagonal of u into tau_v
    i = 0 as usize;
    while i < 2 as usize {
        *tau_v.offset(i as isize) = (*u.offset(i as isize))[(i + 1 as usize) as usize];
        i += 1
    }
    // below, same idea for u:  householder transformations
    // and the superdiagonal copy
    for i in (0..=2).rev() {
        // copy superdiagonal of u into tau_u
        let mut tau_0: f64 = *tau_u.offset(i as isize);
        *tau_u.offset(i as isize) = (*u.offset(i as isize))[i as usize];
        // perform householder transformation on the sub-matrix
        // u(i,i) to u(m,n), that is, on the sub-matrix of u that
        // begins in the i'th column of the i'th row and extends
        // to the end of the matrix at (m,n).  the householder
        // vector used to perform this is the i'th column of u,
        // that is, u(0,i) to u(m,i)
        if tau_0 == 0.0f64 {
            (*u.offset(i as isize))[i as usize] = 1.0f64;
            if i < 2 as usize {
                (*u.offset(i as isize))[2 as usize as usize] = 0.0f64;
                if i < 1 as usize {
                    (*u.offset(i as isize))[1 as usize as usize] = 0.0f64
                }
            }
            y = i + 1 as usize;
            while y < rows {
                (*u.offset(y as isize))[i as usize] = 0.0f64;
                y += 1
            }
        } else {
            let mut x_0: usize = i + 1 as usize;
            while x_0 < 3 as usize {
                let mut wx_0: f64 = 0.0f64;
                y = i + 1 as usize;
                while y < rows {
                    wx_0 +=
                        (*u.offset(y as isize))[x_0 as usize] * (*u.offset(y as isize))[i as usize];
                    y += 1
                }
                let mut tau_wx_0: f64 = tau_0 * wx_0;
                (*u.offset(i as isize))[x_0 as usize] = -tau_wx_0;
                y = i + 1 as usize;
                while y < rows {
                    (*u.offset(y as isize))[x_0 as usize] -=
                        tau_wx_0 * (*u.offset(y as isize))[i as usize];
                    y += 1
                }
                x_0 += 1
            }
            y = i + 1 as usize;
            while y < rows {
                (*u.offset(y as isize))[i as usize] = (*u.offset(y as isize))[i as usize] * -tau_0;
                y += 1
            }
            (*u.offset(i as isize))[i as usize] = 1.0f64 - tau_0
        }
    }
}
// diagonalize
//----------------------------------------------------------------------------
unsafe extern "C" fn diagonalize(
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut tau_u: *mut f64,
    mut tau_v: *mut f64,
    mut rows: usize,
) {
    let mut i: usize = 0;
    let mut j: usize = 0;
    chop(tau_u, tau_v, 3 as usize);
    // progressively reduce the matrices into diagonal form
    let mut b: usize = 3 as usize - 1 as usize;
    while b > 0 as usize {
        if *tau_v.offset((b - 1 as usize) as isize) == 0.0f64 {
            b -= 1
        } else {
            let mut a: usize = b - 1 as usize;
            while a > 0 as usize && *tau_v.offset((a - 1 as usize) as isize) != 0.0f64 {
                a -= 1
            }
            let mut n: usize = b - a + 1 as usize;
            let mut u1: [[f64; 3]; 12] = [[0.; 3]; 12];
            let mut v1: [[f64; 3]; 3] = [[0.; 3]; 3];
            j = a;
            while j <= b {
                i = 0 as usize;
                while i < rows {
                    u1[i as usize][(j - a) as usize] = (*u.offset(i as isize))[j as usize];
                    i += 1
                }
                i = 0 as usize;
                while i < 3 as usize {
                    v1[i as usize][(j - a) as usize] = (*v.offset(i as isize))[j as usize];
                    i += 1
                }
                j += 1
            }
            qrstep(
                u1.as_mut_ptr(),
                v1.as_mut_ptr(),
                &mut *tau_u.offset(a as isize),
                &mut *tau_v.offset(a as isize),
                rows,
                n,
            );
            j = a;
            while j <= b {
                i = 0 as usize;
                while i < rows {
                    (*u.offset(i as isize))[j as usize] = u1[i as usize][(j - a) as usize];
                    i += 1
                }
                i = 0 as usize;
                while i < 3 as usize {
                    (*v.offset(i as isize))[j as usize] = v1[i as usize][(j - a) as usize];
                    i += 1
                }
                j += 1
            }
            chop(
                &mut *tau_u.offset(a as isize),
                &mut *tau_v.offset(a as isize),
                n,
            );
        }
    }
}
//----------------------------------------------------------------------------
unsafe extern "C" fn chop(mut a: *mut f64, mut b: *mut f64, mut n: usize) {
    let mut ai: f64 = *a.offset(0 as usize as isize);
    let mut i: usize = 0 as usize;
    while i < n - 1 as usize {
        let mut bi: f64 = *b.offset(i as isize);
        let mut ai1: f64 = *a.offset((i + 1 as usize) as isize);
        if bi.abs() < 1e-5f64 * (ai.abs() + ai1.abs()) {
            *b.offset(i as isize) = 0.0f64
        }
        ai = ai1;
        i += 1
    }
}
//----------------------------------------------------------------------------
unsafe extern "C" fn qrstep(
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut tau_u: *mut f64,
    mut tau_v: *mut f64,
    mut rows: usize,
    mut cols: usize,
) {
    let mut i: usize = 0;
    if cols == 2 as usize {
        qrstep_cols2(u, v, tau_u, tau_v, rows);
        return;
    }
    if cols == 1 as usize {
        let mut bomb: *mut std::os::raw::c_char = 0 as *mut std::os::raw::c_char;
        *bomb = 0 as usize as std::os::raw::c_char
    }
    // handle zeros on the diagonal or at its end
    i = 0 as usize;
    while i < cols - 1 as usize {
        if *tau_u.offset(i as isize) == 0.0f64 {
            qrstep_middle(u, tau_u, tau_v, rows, cols, i);
            return;
        }
        i += 1
    }
    if *tau_u.offset((cols - 1 as usize) as isize) == 0.0f64 {
        qrstep_end(v, tau_u, tau_v);
        return;
    }
    // perform qr reduction on the diagonal and off-diagonal
    let mut mu: f64 = qrstep_eigenvalue(tau_u, tau_v);
    let mut y: f64 = *tau_u.offset(0 as usize as isize) * *tau_u.offset(0 as usize as isize) - mu;
    let mut z: f64 = *tau_u.offset(0 as usize as isize) * *tau_v.offset(0 as usize as isize);
    let mut ak: f64 = 0.0f64;
    let mut bk: f64 = 0.0f64;
    let mut zk: f64 = 0.;
    let mut ap: f64 = *tau_u.offset(0 as usize as isize);
    let mut bp: f64 = *tau_v.offset(0 as usize as isize);
    let mut aq: f64 = *tau_u.offset(1 as usize as isize);
    let mut k: usize = 0 as usize;
    while k < cols - 1 as usize {
        let mut c: f64 = 0.;
        let mut s: f64 = 0.;
        // perform Givens rotation on V
        computeGivens(y, z, &mut c, &mut s);
        i = 0 as usize;
        while i < 3 as usize {
            let mut vip: f64 = (*v.offset(i as isize))[k as usize];
            let mut viq: f64 = (*v.offset(i as isize))[(k + 1 as usize) as usize];
            (*v.offset(i as isize))[k as usize] = vip * c - viq * s;
            (*v.offset(i as isize))[(k + 1 as usize) as usize] = vip * s + viq * c;
            i += 1
        }
        // perform Givens rotation on B
        let mut bk1: f64 = bk * c - z * s;
        let mut ap1: f64 = ap * c - bp * s;
        let mut bp1: f64 = ap * s + bp * c;
        let mut zp1: f64 = aq * -s;
        let mut aq1: f64 = aq * c;
        if k > 0 as usize {
            *tau_v.offset((k - 1 as usize) as isize) = bk1
        }
        ak = ap1;
        bk = bp1;
        zk = zp1;
        ap = aq1;
        if k < cols - 2 as usize {
            bp = *tau_v.offset((k + 1 as usize) as isize)
        } else {
            bp = 0.0f64
        }
        y = ak;
        z = zk;
        // perform Givens rotation on U
        computeGivens(y, z, &mut c, &mut s);
        i = 0 as usize;
        while i < rows {
            let mut uip: f64 = (*u.offset(i as isize))[k as usize];
            let mut uiq: f64 = (*u.offset(i as isize))[(k + 1 as usize) as usize];
            (*u.offset(i as isize))[k as usize] = uip * c - uiq * s;
            (*u.offset(i as isize))[(k + 1 as usize) as usize] = uip * s + uiq * c;
            i += 1
        }
        // perform Givens rotation on B
        let mut ak1: f64 = ak * c - zk * s;
        bk1 = bk * c - ap * s;
        let mut zk1: f64 = bp * -s;
        ap1 = bk * s + ap * c;
        bp1 = bp * c;
        *tau_u.offset(k as isize) = ak1;
        ak = ak1;
        bk = bk1;
        zk = zk1;
        ap = ap1;
        bp = bp1;
        if k < cols - 2 as usize {
            aq = *tau_u.offset((k + 2 as usize) as isize)
        } else {
            aq = 0.0f64
        }
        y = bk;
        z = zk;
        k += 1
    }
    *tau_v.offset((cols - 2 as usize) as isize) = bk;
    *tau_u.offset((cols - 1 as usize) as isize) = ap;
}
//----------------------------------------------------------------------------
unsafe extern "C" fn qrstep_middle(
    mut u: *mut [f64; 3],
    mut tau_u: *mut f64,
    mut tau_v: *mut f64,
    mut rows: usize,
    mut cols: usize,
    mut col: usize,
) {
    let mut x: f64 = *tau_v.offset(col as isize);
    let mut y: f64 = *tau_u.offset((col + 1 as usize) as isize);
    let mut j: usize = col;
    while j < cols - 1 as usize {
        let mut c: f64 = 0.;
        let mut s: f64 = 0.;
        // perform Givens rotation on U
        computeGivens(y, -x, &mut c, &mut s);
        let mut i: usize = 0 as usize;
        while i < rows {
            let mut uip: f64 = (*u.offset(i as isize))[col as usize];
            let mut uiq: f64 = (*u.offset(i as isize))[(j + 1 as usize) as usize];
            (*u.offset(i as isize))[col as usize] = uip * c - uiq * s;
            (*u.offset(i as isize))[(j + 1 as usize) as usize] = uip * s + uiq * c;
            i += 1
        }
        // perform transposed Givens rotation on B
        *tau_u.offset((j + 1 as usize) as isize) = x * s + y * c;
        if j == col {
            *tau_v.offset(j as isize) = x * c - y * s
        }
        if j < cols - 2 as usize {
            let mut z: f64 = *tau_v.offset((j + 1 as usize) as isize);
            *tau_v.offset((j + 1 as usize) as isize) *= c;
            x = z * -s;
            y = *tau_u.offset((j + 2 as usize) as isize)
        }
        j += 1
    }
}
//----------------------------------------------------------------------------
unsafe extern "C" fn qrstep_end(mut v: *mut [f64; 3], mut tau_u: *mut f64, mut tau_v: *mut f64) {
    let mut x: f64 = *tau_u.offset(1 as usize as isize);
    let mut y: f64 = *tau_v.offset(1 as usize as isize);
    for k in (0..=1).rev() {
        let mut c: f64 = 0.;
        let mut s: f64 = 0.;
        // perform Givens rotation on V
        computeGivens(x, y, &mut c, &mut s);
        let mut i: usize = 0 as usize;
        while i < 3 as usize {
            let mut vip: f64 = (*v.offset(i as isize))[k as usize];
            let mut viq: f64 = (*v.offset(i as isize))[2 as usize as usize];
            (*v.offset(i as isize))[k as usize] = vip * c - viq * s;
            (*v.offset(i as isize))[2 as usize as usize] = vip * s + viq * c;
            i += 1
        }
        // perform Givens rotation on B
        *tau_u.offset(k as isize) = x * c - y * s;
        if k == 1 as usize {
            *tau_v.offset(k as isize) = x * s + y * c
        }
        if k > 0 as usize {
            let mut z: f64 = *tau_v.offset((k - 1 as usize) as isize);
            *tau_v.offset((k - 1 as usize) as isize) *= c;
            x = *tau_u.offset((k - 1 as usize) as isize);
            y = z * s
        }
    }
}
//----------------------------------------------------------------------------
unsafe extern "C" fn qrstep_eigenvalue(mut tau_u: *mut f64, mut tau_v: *mut f64) -> f64 {
    let mut ta: f64 = *tau_u.offset(1 as usize as isize) * *tau_u.offset(1 as usize as isize)
        + *tau_v.offset(0 as usize as isize) * *tau_v.offset(0 as usize as isize);
    let mut tb: f64 = *tau_u.offset(2 as usize as isize) * *tau_u.offset(2 as usize as isize)
        + *tau_v.offset(1 as usize as isize) * *tau_v.offset(1 as usize as isize);
    let mut tab: f64 = *tau_u.offset(1 as usize as isize) * *tau_v.offset(1 as usize as isize);
    let mut dt: f64 = (ta - tb) / 2.0f64;
    let mut mu: f64 = 0.;
    if dt >= 0.0f64 {
        mu = tb - tab * tab / (dt + (dt * dt + tab * tab).sqrt())
    } else {
        mu = tb + tab * tab / ((dt * dt + tab * tab).sqrt() - dt)
    }
    return mu;
}
//----------------------------------------------------------------------------
unsafe extern "C" fn qrstep_cols2(
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut tau_u: *mut f64,
    mut tau_v: *mut f64,
    mut rows: usize,
) {
    let mut i: usize = 0;
    let mut tmp: f64 = 0.;
    // eliminate off-diagonal element in [ 0  tau_v0 ]
    //                                   [ 0  tau_u1 ]
    // to make [ tau_u[0]  0 ]
    //         [ 0         0 ]
    if *tau_u.offset(0 as usize as isize) == 0.0f64 {
        let mut c: f64 = 0.;
        let mut s: f64 = 0.;
        // perform transposed Givens rotation on B
        // multiplied by X = [ 0 1 ]
        //                   [ 1 0 ]
        computeGivens(
            *tau_v.offset(0 as usize as isize),
            *tau_u.offset(1 as usize as isize),
            &mut c,
            &mut s,
        );
        *tau_u.offset(0 as usize as isize) =
            *tau_v.offset(0 as usize as isize) * c - *tau_u.offset(1 as usize as isize) * s;
        *tau_v.offset(0 as usize as isize) =
            *tau_v.offset(0 as usize as isize) * s + *tau_u.offset(1 as usize as isize) * c;
        *tau_u.offset(1 as usize as isize) = 0.0f64;
        // perform Givens rotation on U
        i = 0 as usize;
        while i < rows {
            let mut uip: f64 = (*u.offset(i as isize))[0 as usize as usize];
            let mut uiq: f64 = (*u.offset(i as isize))[1 as usize as usize];
            (*u.offset(i as isize))[0 as usize as usize] = uip * c - uiq * s;
            (*u.offset(i as isize))[1 as usize as usize] = uip * s + uiq * c;
            i += 1
        }
        // multiply V by X, effectively swapping first two columns
        i = 0 as usize;
        while i < 3 as usize {
            tmp = (*v.offset(i as isize))[0 as usize as usize];
            (*v.offset(i as isize))[0 as usize as usize] =
                (*v.offset(i as isize))[1 as usize as usize];
            (*v.offset(i as isize))[1 as usize as usize] = tmp;
            i += 1
        }
    } else if *tau_u.offset(1 as usize as isize) == 0.0f64 {
        let mut c_0: f64 = 0.;
        let mut s_0: f64 = 0.;
        // eliminate off-diagonal element in [ tau_u0  tau_v0 ]
        //                                   [ 0       0      ]
        // perform Givens rotation on B
        computeGivens(
            *tau_u.offset(0 as usize as isize),
            *tau_v.offset(0 as usize as isize),
            &mut c_0,
            &mut s_0,
        );
        *tau_u.offset(0 as usize as isize) =
            *tau_u.offset(0 as usize as isize) * c_0 - *tau_v.offset(0 as usize as isize) * s_0;
        *tau_v.offset(0 as usize as isize) = 0.0f64;
        // perform Givens rotation on V
        i = 0 as usize;
        while i < 3 as usize {
            let mut vip: f64 = (*v.offset(i as isize))[0 as usize as usize];
            let mut viq: f64 = (*v.offset(i as isize))[1 as usize as usize];
            (*v.offset(i as isize))[0 as usize as usize] = vip * c_0 - viq * s_0;
            (*v.offset(i as isize))[1 as usize as usize] = vip * s_0 + viq * c_0;
            i += 1
        }
    } else {
        // make colums orthogonal,
        let mut c_1: f64 = 0.;
        let mut s_1: f64 = 0.;
        // perform Schur rotation on B
        computeSchur(
            *tau_u.offset(0 as usize as isize),
            *tau_v.offset(0 as usize as isize),
            *tau_u.offset(1 as usize as isize),
            &mut c_1,
            &mut s_1,
        );
        let mut a11: f64 =
            *tau_u.offset(0 as usize as isize) * c_1 - *tau_v.offset(0 as usize as isize) * s_1;
        let mut a21: f64 = -*tau_u.offset(1 as usize as isize) * s_1;
        let mut a12: f64 =
            *tau_u.offset(0 as usize as isize) * s_1 + *tau_v.offset(0 as usize as isize) * c_1;
        let mut a22: f64 = *tau_u.offset(1 as usize as isize) * c_1;
        // perform Schur rotation on V
        i = 0 as usize;
        while i < 3 as usize {
            let mut vip_0: f64 = (*v.offset(i as isize))[0 as usize as usize];
            let mut viq_0: f64 = (*v.offset(i as isize))[1 as usize as usize];
            (*v.offset(i as isize))[0 as usize as usize] = vip_0 * c_1 - viq_0 * s_1;
            (*v.offset(i as isize))[1 as usize as usize] = vip_0 * s_1 + viq_0 * c_1;
            i += 1
        }
        // eliminate off diagonal elements
        if a11 * a11 + a21 * a21 < a12 * a12 + a22 * a22 {
            // multiply B by X
            tmp = a11;
            a11 = a12;
            a12 = tmp;
            tmp = a21;
            a21 = a22;
            a22 = tmp;
            // multiply V by X, effectively swapping first
            // two columns
            i = 0 as usize;
            while i < 3 as usize {
                tmp = (*v.offset(i as isize))[0 as usize as usize];
                (*v.offset(i as isize))[0 as usize as usize] =
                    (*v.offset(i as isize))[1 as usize as usize];
                (*v.offset(i as isize))[1 as usize as usize] = tmp;
                i += 1
            }
        }
        // perform transposed Givens rotation on B
        computeGivens(a11, a21, &mut c_1, &mut s_1);
        *tau_u.offset(0 as usize as isize) = a11 * c_1 - a21 * s_1;
        *tau_v.offset(0 as usize as isize) = a12 * c_1 - a22 * s_1;
        *tau_u.offset(1 as usize as isize) = a12 * s_1 + a22 * c_1;
        // perform Givens rotation on U
        i = 0 as usize;
        while i < rows {
            let mut uip_0: f64 = (*u.offset(i as isize))[0 as usize as usize];
            let mut uiq_0: f64 = (*u.offset(i as isize))[1 as usize as usize];
            (*u.offset(i as isize))[0 as usize as usize] = uip_0 * c_1 - uiq_0 * s_1;
            (*u.offset(i as isize))[1 as usize as usize] = uip_0 * s_1 + uiq_0 * c_1;
            i += 1
        }
    };
}
//----------------------------------------------------------------------------
unsafe extern "C" fn computeGivens(mut a: f64, mut b: f64, mut c: *mut f64, mut s: *mut f64) {
    if b == 0.0f64 {
        *c = 1.0f64;
        *s = 0.0f64
    } else if b.abs() > a.abs() {
        let mut t: f64 = -a / b;
        let mut s1: f64 = 1.0f64 / (1 as usize as f64 + t * t).sqrt();
        *s = s1;
        *c = s1 * t
    } else {
        let mut t_0: f64 = -b / a;
        let mut c1: f64 = 1.0f64 / (1 as usize as f64 + t_0 * t_0).sqrt();
        *c = c1;
        *s = c1 * t_0
    };
}
//----------------------------------------------------------------------------
unsafe extern "C" fn computeSchur(
    mut a1: f64,
    mut a2: f64,
    mut a3: f64,
    mut c: *mut f64,
    mut s: *mut f64,
) {
    let mut apq: f64 = a1 * a2 * 2.0f64;
    if apq == 0.0f64 {
        *c = 1.0f64;
        *s = 0.0f64
    } else {
        let mut t: f64 = 0.;
        let mut tau: f64 = (a2 * a2 + (a3 + a1) * (a3 - a1)) / apq;
        if tau >= 0.0f64 {
            t = 1.0f64 / (tau + (1.0f64 + tau * tau).sqrt())
        } else {
            t = -1.0f64 / ((1.0f64 + tau * tau).sqrt() - tau)
        }
        *c = 1.0f64 / (1.0f64 + t * t).sqrt();
        *s = t * *c
    };
}
// singularize
//----------------------------------------------------------------------------
unsafe extern "C" fn singularize(
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut d: *mut f64,
    mut rows: usize,
) {
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut y: usize = 0;
    // make singularize values positive
    j = 0 as usize;
    while j < 3 as usize {
        if *d.offset(j as isize) < 0.0f64 {
            i = 0 as usize;
            while i < 3 as usize {
                (*v.offset(i as isize))[j as usize] = -(*v.offset(i as isize))[j as usize];
                i += 1
            }
            *d.offset(j as isize) = -*d.offset(j as isize)
        }
        j += 1
    }
    // sort singular values in decreasing order
    i = 0 as usize;
    while i < 3 as usize {
        let mut d_max: f64 = *d.offset(i as isize);
        let mut i_max: usize = i;
        j = i + 1 as usize;
        while j < 3 as usize {
            if *d.offset(j as isize) > d_max {
                d_max = *d.offset(j as isize);
                i_max = j
            }
            j += 1
        }
        if i_max != i {
            // swap eigenvalues
            let mut tmp: f64 = *d.offset(i as isize);
            *d.offset(i as isize) = *d.offset(i_max as isize);
            *d.offset(i_max as isize) = tmp;
            // swap eigenvectors
            y = 0 as usize;
            while y < rows {
                tmp = (*u.offset(y as isize))[i as usize];
                (*u.offset(y as isize))[i as usize] = (*u.offset(y as isize))[i_max as usize];
                (*u.offset(y as isize))[i_max as usize] = tmp;
                y += 1
            }
            y = 0 as usize;
            while y < 3 as usize {
                tmp = (*v.offset(y as isize))[i as usize];
                (*v.offset(y as isize))[i as usize] = (*v.offset(y as isize))[i_max as usize];
                (*v.offset(y as isize))[i_max as usize] = tmp;
                y += 1
            }
        }
        i += 1
    }
}
// solve svd
//----------------------------------------------------------------------------
unsafe extern "C" fn solveSVD(
    mut u: *mut [f64; 3],
    mut v: *mut [f64; 3],
    mut d: *mut f64,
    mut b: *mut f64,
    mut x: *mut f64,
    mut rows: usize,
) {
    let mut i: usize = 0;
    let mut j: usize = 0;
    // compute vector w = U^T * b
    let mut w: [f64; 3] = [0.; 3];

    i = 0 as usize;
    while i < rows {
        if *b.offset(i as isize) != 0.0f64 {
            j = 0 as usize;
            while j < 3 as usize {
                w[j as usize] += *b.offset(i as isize) * (*u.offset(i as isize))[j as usize];
                j += 1
            }
        }
        i += 1
    }
    // introduce non-zero singular values in d into w
    i = 0 as usize;
    while i < 3 as usize {
        if *d.offset(i as isize) != 0.0f64 {
            w[i as usize] /= *d.offset(i as isize)
        }
        i += 1
    }
    // compute result vector x = V * w
    i = 0 as usize;
    while i < 3 as usize {
        let mut tmp: f64 = 0.0f64;
        j = 0 as usize;
        while j < 3 as usize {
            tmp += w[j as usize] * (*v.offset(i as isize))[j as usize];
            j += 1
        }
        *x.offset(i as isize) = tmp;
        i += 1
    }
}
