// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! TODO: Module-level documentation.

#![allow(non_camel_case_types)]

macro_rules! define_type {
    ( $name:ident, $builtin:ty, $test_c_metrics:ident, $get_c_align_fn:ident,
      $get_c_size_fn:ident, $doc:expr ) =>
    {
        #[doc = $doc]
        pub type $name = $builtin;

        define_metrics_tests!($name, $test_c_metrics, $get_c_align_fn,
                              $get_c_size_fn);
    }
}

macro_rules! define_metrics_tests {
    ( $name:ident, $test_c_metrics:ident, $get_c_align_fn:ident,
      $get_c_size_fn:ident ) =>
    {
        define_metrics_tests!($name, $test_c_metrics, $get_c_align_fn,
                              $get_c_size_fn, 1);
    };

    ( $name:ident, $test_c_metrics:ident, $get_c_align_fn:ident,
      $get_c_size_fn:ident, $expected_align_factor:expr ) =>
    {
        #[cfg(test)]
        extern {
            // We can't use `size_t` because we need to test that our
            // definition of `size_t` is correct using this code! We use `u16`
            // because even 8-bit and 16-bit microcontrollers have no trouble
            // with it, and because `u16` is always as smaller or smaller than
            // `usize`.
            fn $get_c_align_fn() -> u16;
            fn $get_c_size_fn() -> u16;
        }

        #[cfg(test)]
        #[test]
        #[allow(unsafe_code)]
        fn $test_c_metrics() {
            use std::mem;

            let c_align = unsafe { $get_c_align_fn() };
            let c_size = unsafe { $get_c_size_fn() };

            // XXX: Remove these assertions and these uses of `as` when Rust
            // supports implicit coercion of `u16` to `usize`.
            assert!(mem::size_of_val(&c_align) <= mem::size_of::<usize>());
            assert!(mem::size_of_val(&c_size) <= mem::size_of::<usize>());

            // Rust uses 4 for the alignment of `i64` and `u64`. On Linux x86,
            // GCC 5 uses 8 but earlier versions use 4 and so does Clang.
            let rust_align =
                if $expected_align_factor != 1 &&
                   mem::align_of::<$name>() != c_align as usize {
                   mem::align_of::<$name>() * $expected_align_factor
                } else {
                    mem::align_of::<$name>()
                };

            assert_eq!((rust_align, mem::size_of::<$name>()),
                       (c_align as usize, c_size as usize));
        }
    }
}

define_type!(int, i32, test_int_metrics, ring_int_align, ring_int_size,
             "The C `int` type. Equivalent to `libc::int`.");

define_type!(
  size_t, usize, test_size_t_metrics, ring_size_t_align, ring_size_t_size,
  "The C `size_t` type from `<stdint.h>`.

  ISO C's `size_t` is defined to be the type of the result of the
  `sizeof` operator and the type of the size parameter to `malloc`. That
  is, C's `size_t` is only required to hold the size of the largest object
  that can be allocated. In particular, it is legal for a C implementation
  to have a maximum object size smaller than the entire address space. For
  example, a C implementation may have an maximum object size of 2^32
  bytes with a 64-bit address space, and typedef `size_t` as `uint32_t` so
  that `sizeof(size_t) == 4` and `sizeof(void*) == 8`.

  Rust's `usize`, on the other hand, is defined to always be the same size
  as a pointer. This means that it is possible, in theory, to have a platform
  where `usize` can represent values that `size_t` cannot represent. However,
  on the vast majority of systems, `usize` and `size_t` are represented the
  same way. If it were required to explicitly cast `usize` to `size_t` on
  common platforms, then many programmers would habitually write expressions
  such as `my_slice.len() as libc::size_t` expecting this to always work and
  be safe. But such a cast is *not* safe on the uncommon platforms where
  `mem::sizeof(libc::size_t) < mem::size_t(usize)`. Consequently, to reduce
  the chances of programmers becoming habituated to such casts that would be
  unsafe on unusual platforms, we have adopted the following convention:

  * On common platforms where C's `size_t` is the same size as `usize`,
    `ring::c::size_t` must be a type alias of `usize`.

  * On uncommon platforms where C's `size_t` is not the same size as `usize`,
    `ring::c::size_t` must be a type alias for a type other than `usize`.

  * Code that was written without consideration for the uncommon platforms
    should not do any explicit casting between `size_t` and `usize`. Such
    code will fail to compile on the uncommon platforms; this is better than
    executing with unsafe truncations.

  * Code that was written with full consideration of the uncommon platforms
    should have explicit casts using `num::cast` or other methods that avoid
    unintended truncation. Such code will then work on all platforms.");

define_metrics_tests!(i8, test_i8_metrics, ring_int8_t_align, ring_int8_t_size);
define_metrics_tests!(u8, test_u8_metrics, ring_uint8_t_align,
                      ring_uint8_t_size);

define_metrics_tests!(i16, test_i16_metrics, ring_int16_t_align,
                      ring_int16_t_size);
define_metrics_tests!(u16, test_u16_metrics, ring_uint16_t_align,
                      ring_uint16_t_size);

define_metrics_tests!(i32, test_i32_metrics, ring_int32_t_align,
                      ring_int32_t_size);
define_metrics_tests!(u32, test_u32_metrics, ring_uint32_t_align,
                      ring_uint32_t_size);

#[cfg(all(test,
          not(all(target_arch = "x86",
                  any(target_os = "linux",
                      target_os = "macos",
                      target_os = "ios")))))]
const SIXTY_FOUR_BIT_ALIGNMENT_FACTOR: usize = 1;

#[cfg(all(test, target_arch = "x86",
                  any(target_os = "linux",
                      target_os = "macos",
                      target_os = "ios")))]
const SIXTY_FOUR_BIT_ALIGNMENT_FACTOR: usize = 2;

define_metrics_tests!(i64, test_i64_metrics, ring_int64_t_align,
                      ring_int64_t_size, SIXTY_FOUR_BIT_ALIGNMENT_FACTOR);
define_metrics_tests!(u64, test_u64_metrics, ring_uint64_t_align,
                      ring_uint64_t_size, SIXTY_FOUR_BIT_ALIGNMENT_FACTOR);
