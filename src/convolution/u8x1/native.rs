use crate::convolution::{optimisations, Coefficients};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8;

#[inline(always)]
pub(crate) fn horiz_convolution(
    src_image: TypedImageView<U8>,
    mut dst_image: TypedImageViewMut<U8>,
    offset: u32,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard::new(values);
    let precision = normalizer_guard.precision();
    let coefficients_chunks = normalizer_guard.normalized_i16_chunks(window_size, &bounds);
    let initial = 1 << (precision - 1);

    let src_rows = src_image.iter_rows(offset);
    let dst_rows = dst_image.iter_rows_mut();
    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (&coeffs_chunk, dst_pixel) in coefficients_chunks.iter().zip(dst_row.iter_mut()) {
            let first_x_src = coeffs_chunk.start as usize;
            let ks = coeffs_chunk.values;

            let mut ss = initial;
            let src_pixels = unsafe { src_row.get_unchecked(first_x_src..) };
            for (&k, &src_pixel) in ks.iter().zip(src_pixels) {
                ss += src_pixel.0 as i32 * (k as i32);
            }
            dst_pixel.0 = unsafe { optimisations::clip8(ss, precision) };
        }
    }
}

#[inline(always)]
pub(crate) fn vert_convolution(
    src_image: TypedImageView<U8>,
    mut dst_image: TypedImageViewMut<U8>,
    coeffs: Coefficients,
) {
    let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

    let normalizer_guard = optimisations::NormalizerGuard::new(values);
    let precision = normalizer_guard.precision();
    let coefficients_chunks = normalizer_guard.normalized_i16_chunks(window_size, &bounds);
    let initial = 1 << (precision - 1);

    let dst_rows = dst_image.iter_rows_mut();
    for (&coeffs_chunk, dst_row) in coefficients_chunks.iter().zip(dst_rows) {
        let first_y_src = coeffs_chunk.start;
        let ks = coeffs_chunk.values;

        for (x_src, dst_pixel) in dst_row.iter_mut().enumerate() {
            let mut ss = initial;
            let src_rows = src_image.iter_rows(first_y_src);
            for (&k, src_row) in ks.iter().zip(src_rows) {
                let src_pixel = unsafe { src_row.get_unchecked(x_src as usize) };
                ss += src_pixel.0 as i32 * (k as i32);
            }
            dst_pixel.0 = unsafe { optimisations::clip8(ss, precision) };
        }
    }
}
