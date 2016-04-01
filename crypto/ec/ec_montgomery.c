/* Originally written by Bodo Moeller and Nils Larsch for the OpenSSL project.
 * ====================================================================
 * Copyright (c) 1998-2005 The OpenSSL Project.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in
 *    the documentation and/or other materials provided with the
 *    distribution.
 *
 * 3. All advertising materials mentioning features or use of this
 *    software must display the following acknowledgment:
 *    "This product includes software developed by the OpenSSL Project
 *    for use in the OpenSSL Toolkit. (http://www.openssl.org/)"
 *
 * 4. The names "OpenSSL Toolkit" and "OpenSSL Project" must not be used to
 *    endorse or promote products derived from this software without
 *    prior written permission. For written permission, please contact
 *    openssl-core@openssl.org.
 *
 * 5. Products derived from this software may not be called "OpenSSL"
 *    nor may "OpenSSL" appear in their names without prior written
 *    permission of the OpenSSL Project.
 *
 * 6. Redistributions of any form whatsoever must retain the following
 *    acknowledgment:
 *    "This product includes software developed by the OpenSSL Project
 *    for use in the OpenSSL Toolkit (http://www.openssl.org/)"
 *
 * THIS SOFTWARE IS PROVIDED BY THE OpenSSL PROJECT ``AS IS'' AND ANY
 * EXPRESSED OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE OpenSSL PROJECT OR
 * ITS CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
 * NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
 * OF THE POSSIBILITY OF SUCH DAMAGE.
 * ====================================================================
 *
 * This product includes cryptographic software written by Eric Young
 * (eay@cryptsoft.com).  This product includes software written by Tim
 * Hudson (tjh@cryptsoft.com).
 *
 */
/* ====================================================================
 * Copyright 2002 Sun Microsystems, Inc. ALL RIGHTS RESERVED.
 *
 * Portions of the attached software ("Contribution") are developed by
 * SUN MICROSYSTEMS, INC., and are contributed to the OpenSSL project.
 *
 * The Contribution is licensed pursuant to the OpenSSL open source
 * license provided above.
 *
 * The elliptic curve binary polynomial software is originally written by
 * Sheueling Chang Shantz and Douglas Stebila of Sun Microsystems
 * Laboratories. */

#include <openssl/ec.h>

#include <openssl/bn.h>
#include <openssl/err.h>
#include <openssl/mem.h>

#include "internal.h"


int ec_GFp_mont_field_mul(const EC_GROUP *group, BIGNUM *r, const BIGNUM *a,
                          const BIGNUM *b, BN_CTX *ctx) {
  return BN_mod_mul_montgomery(r, a, b, &group->mont, ctx);
}

int ec_GFp_mont_field_sqr(const EC_GROUP *group, BIGNUM *r, const BIGNUM *a,
                          BN_CTX *ctx) {
  return BN_mod_mul_montgomery(r, a, a, &group->mont, ctx);
}

int ec_GFp_mont_field_encode(const EC_GROUP *group, BIGNUM *r, const BIGNUM *a,
                             BN_CTX *ctx) {
  return BN_to_montgomery(r, a, &group->mont, ctx);
}

int ec_GFp_mont_field_decode(const EC_GROUP *group, BIGNUM *r, const BIGNUM *a,
                             BN_CTX *ctx) {
  return BN_from_montgomery(r, a, &group->mont, ctx);
}

static int ec_GFp_mont_point_get_affine_coordinates(const EC_GROUP *group,
                                                    const EC_POINT *point,
                                                    BIGNUM *x, BIGNUM *y,
                                                    BN_CTX *ctx) {
  if (EC_POINT_is_at_infinity(group, point)) {
    OPENSSL_PUT_ERROR(EC, EC_R_POINT_AT_INFINITY);
    return 0;
  }

  BN_CTX *new_ctx = NULL;
  if (ctx == NULL) {
    ctx = new_ctx = BN_CTX_new();
    if (ctx == NULL) {
      return 0;
    }
  }

  int ret = 0;

  BN_CTX_start(ctx);

  if (BN_cmp(&point->Z, &group->one) == 0) {
    /* |point| is already affine. */
    if (x != NULL && !group->meth->field_decode(group, x, &point->X, ctx)) {
      goto err;
    }
    if (y != NULL && !group->meth->field_decode(group, y, &point->Y, ctx)) {
      goto err;
    }
  } else {
    /* transform  (X, Y, Z)  into  (x, y) := (X/Z^2, Y/Z^3) */

    BIGNUM *Z = BN_CTX_get(ctx);
    BIGNUM *Z_1 = BN_CTX_get(ctx);
    BIGNUM *Z_2 = BN_CTX_get(ctx);
    BIGNUM *Z_3 = BN_CTX_get(ctx);
    if (Z == NULL ||
        Z_1 == NULL ||
        Z_2 == NULL ||
        Z_3 == NULL) {
      goto err;
    }

    /* This formulation only uses |field_decode|, never |field_encode| since it
     * should |field_decode| should be as or more efficient as |field_encode|,
     * at least in theory. */

    if (!group->meth->field_decode(group, Z, &point->Z, ctx) ||
        !group->meth->field_decode(group, Z, Z, ctx) ||
        !BN_mod_exp_mont_consttime(Z_1, Z, &group->field_minus_2,
                                   &group->field, ctx, &group->mont)) {
      OPENSSL_PUT_ERROR(EC, ERR_R_BN_LIB);
      goto err;
    }

    if (!group->meth->field_sqr(group, Z_2, Z_1, ctx) ||
        !group->meth->field_decode(group, Z_2, Z_2, ctx)) {
      goto err;
    }

    if (x != NULL) {
      if (!group->meth->field_mul(group, x, &point->X, Z_2, ctx)) {
        goto err;
      }
    }

    if (y != NULL) {
      if (!group->meth->field_mul(group, Z_3, Z_2, Z_1, ctx) ||
          !group->meth->field_mul(group, y, &point->Y, Z_3, ctx)) {
        goto err;
      }
    }
  }

  ret = 1;

err:
  BN_CTX_end(ctx);
  BN_CTX_free(new_ctx);
  return ret;
}

const EC_METHOD EC_GFp_mont_method = {
  ec_GFp_mont_point_get_affine_coordinates,
  ec_wNAF_mul_private /* XXX: Not constant time. */,
  ec_wNAF_mul_public,
  ec_GFp_mont_field_mul,
  ec_GFp_mont_field_sqr,
  ec_GFp_mont_field_encode,
  ec_GFp_mont_field_decode,
};
