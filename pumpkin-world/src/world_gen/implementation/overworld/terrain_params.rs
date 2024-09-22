// From da java

use std::sync::Arc;

use crate::world_gen::noise::density::spline::{
    FloatAmplifier, Spline, SplineBuilder, SplineValue,
};
use crate::world_gen::noise::density::{peaks_valleys_noise, DensityFunction};
use crate::world_gen::noise::lerp_32;

#[inline]
fn get_offset_value(f: f32, g: f32, h: f32) -> f32 {
    let k = 1f32 - (1f32 - g) * 0.5f32;
    let l = 0.5f32 * (1f32 - g);

    let m = (f + 1.17f32) * 0.46082947f32;
    let n = m * k - l;

    if f < h {
        n.max(-0.2222f32)
    } else {
        n.max(0f32)
    }
}

#[inline]
fn meth_42045(f: f32) -> f32 {
    let k = 1f32 - (1f32 - f) * 0.5f32;
    let l = 0.5f32 * (1f32 - f);

    l / (0.46082947f32 * k) - 1.17f32
}

#[inline]
fn meth_42047(f: f32, g: f32, h: f32, i: f32) -> f32 {
    (g - f) / (i - h)
}

fn meth_42050(
    function: Arc<DensityFunction>,
    f: f32,
    bl: bool,
    amplifier: FloatAmplifier,
) -> Spline {
    let mut builder = SplineBuilder::new(function, amplifier);

    let i = get_offset_value(-1f32, f, -0.7f32);
    let k = get_offset_value(1f32, f, -0.7f32);

    let l = meth_42045(f);

    let builder = if -0.65f32 < l && l < 1f32 {
        let n = get_offset_value(-0.65f32, f, -0.7f32);
        let p = get_offset_value(-0.75f32, f, -0.7f32);
        let q = meth_42047(i, p, -1f32, -0.75f32);
        let builder = builder
            .add_value(-1f32, i, q)
            .add_value(-0.75f32, p, 0f32)
            .add_value(-0.65f32, n, 0f32);

        let r = get_offset_value(l, f, -0.7f32);
        let s = meth_42047(r, k, l, 1f32);
        builder
            .add_value(l - 0.01f32, r, 0f32)
            .add_value(l, r, s)
            .add_value(1f32, k, s)
    } else {
        let n = meth_42047(i, k, -1f32, 1f32);
        let builder = if bl {
            builder
                .add_value(-1f32, 0.2f32.max(i), 0f32)
                .add_value(0f32, lerp_32(0.5f32, i, k), n)
        } else {
            builder.add_value(-1f32, i, n)
        };

        builder.add_value(1f32, k, n)
    };

    builder.build()
}

#[allow(clippy::too_many_arguments)]
fn meth_42048(
    ridges: Arc<DensityFunction>,
    continental: f32,
    f: f32,
    g: f32,
    h: f32,
    i: f32,
    j: f32,
    amplifier: FloatAmplifier,
) -> Spline {
    let k = j.max(0.5f32 * (f - continental));
    let l = 5f32 * (g - f);
    SplineBuilder::new(ridges, amplifier)
        .add_value(-1f32, continental, 0f32)
        .add_value(-0.4f32, f, k.min(l))
        .add_value(0f32, g, l)
        .add_value(0.4f32, h, 2f32 * (h - g))
        .add_value(1f32, i, 0.7f32 * (i - h))
        .build()
}

fn method_42054<'a>(
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    ridges_folded: Arc<DensityFunction<'a>>,
    f: f32,
    bl: bool,
    amplifier: FloatAmplifier,
) -> Spline<'a> {
    let spline = SplineBuilder::new(ridges.clone(), amplifier.clone())
        .add_value(-0.2f32, 6.3f32, 0f32)
        .add_value(0.2f32, f, 0f32)
        .build();

    let mut builder = SplineBuilder::new(erosion, amplifier.clone());
    let builder = builder
        .add_spline(-0.6f32, SplineValue::Spline(spline.clone()), 0f32)
        .add_spline(
            -0.5f32,
            SplineValue::Spline(
                SplineBuilder::new(ridges.clone(), amplifier.clone())
                    .add_value(-0.05f32, 6.3f32, 0f32)
                    .add_value(0.05f32, 2.67f32, 0f32)
                    .build(),
            ),
            0f32,
        )
        .add_spline(-0.35f32, SplineValue::Spline(spline.clone()), 0f32)
        .add_spline(-0.25f32, SplineValue::Spline(spline.clone()), 0f32)
        .add_spline(
            -0.1f32,
            SplineValue::Spline(
                SplineBuilder::new(ridges.clone(), amplifier.clone())
                    .add_value(-0.05f32, 2.67, 0f32)
                    .add_value(0.05f32, 6.3f32, 0f32)
                    .build(),
            ),
            0f32,
        )
        .add_spline(0.03f32, SplineValue::Spline(spline.clone()), 0f32);

    let builder = if bl {
        let spline2 = SplineBuilder::new(ridges.clone(), amplifier.clone())
            .add_value(0f32, f, 0f32)
            .add_value(0.1f32, 0.625f32, 0f32)
            .build();
        let spline3 = SplineBuilder::new(ridges_folded.clone(), amplifier.clone())
            .add_value(-0.9f32, f, 0f32)
            .add_spline(-0.69f32, SplineValue::Spline(spline2), 0f32)
            .build();

        builder
            .add_value(0.35f32, f, 0f32)
            .add_spline(0.45f32, SplineValue::Spline(spline3.clone()), 0f32)
            .add_spline(0.55f32, SplineValue::Spline(spline3.clone()), 0f32)
            .add_value(0.62f32, f, 0f32)
    } else {
        let spline2 = SplineBuilder::new(ridges_folded.clone(), amplifier.clone())
            .add_spline(-0.7f32, SplineValue::Spline(spline.clone()), 0f32)
            .add_value(-0.15f32, 1.37f32, 0f32)
            .build();

        let spline3 = SplineBuilder::new(ridges_folded.clone(), amplifier.clone())
            .add_spline(0.45f32, SplineValue::Spline(spline.clone()), 0f32)
            .add_value(0.7f32, 1.56f32, 0f32)
            .build();

        builder
            .add_spline(0.05f32, SplineValue::Spline(spline3.clone()), 0f32)
            .add_spline(0.4f32, SplineValue::Spline(spline3.clone()), 0f32)
            .add_spline(0.45f32, SplineValue::Spline(spline2.clone()), 0f32)
            .add_spline(0.55f32, SplineValue::Spline(spline2.clone()), 0f32)
            .add_value(0.56f32, f, 0f32)
    };

    builder.build()
}

fn method_42052<'a>(
    ridges: Arc<DensityFunction<'a>>,
    ridges_folded: Arc<DensityFunction<'a>>,
    f: f32,
    g: f32,
    amplifier: FloatAmplifier,
) -> Spline<'a> {
    let h = peaks_valleys_noise(0.4f32);
    let i = peaks_valleys_noise(0.56666666f32);
    let j = (h + i) / 2f32;

    let mut builder = SplineBuilder::new(ridges_folded, amplifier.clone());

    let builder = builder.add_value(h, 0f32, 0f32);
    let builder = if g > 0f32 {
        builder.add_spline(
            j,
            SplineValue::Spline(method_42049(ridges.clone(), f, amplifier.clone())),
            0f32,
        )
    } else {
        builder.add_value(j, 0f32, 0f32)
    };

    let builder = if f > 0f32 {
        builder.add_spline(
            1f32,
            SplineValue::Spline(method_42049(ridges.clone(), f, amplifier.clone())),
            0f32,
        )
    } else {
        builder.add_value(1f32, 0f32, 0f32)
    };

    builder.build()
}

fn method_42049<'a>(
    ridges: Arc<DensityFunction<'a>>,
    f: f32,
    amplifier: FloatAmplifier,
) -> Spline<'a> {
    let g = 0.63f32 * f;
    let h = 0.3f32 * f;
    SplineBuilder::new(ridges, amplifier)
        .add_value(-0.01, g, 0f32)
        .add_value(0.01f32, h, 0f32)
        .build()
}

fn method_42053<'a>(
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    ridges_folded: Arc<DensityFunction<'a>>,
    f: f32,
    g: f32,
    h: f32,
    i: f32,
    amplifier: FloatAmplifier,
) -> Spline<'a> {
    let spline = method_42052(
        ridges.clone(),
        ridges_folded.clone(),
        f,
        h,
        amplifier.clone(),
    );
    let spline2 = method_42052(
        ridges.clone(),
        ridges_folded.clone(),
        g,
        i,
        amplifier.clone(),
    );

    SplineBuilder::new(erosion, amplifier)
        .add_spline(-1f32, SplineValue::Spline(spline), 0f32)
        .add_spline(-0.78, SplineValue::Spline(spline2.clone()), 0f32)
        .add_spline(-05775f32, SplineValue::Spline(spline2), 0f32)
        .add_value(-0.375f32, 0f32, 0f32)
        .build()
}

#[allow(clippy::too_many_arguments)]
fn create_continental_offset_spline<'a>(
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    continental: f32,
    f: f32,
    g: f32,
    h: f32,
    i: f32,
    j: f32,
    bl: bool,
    bl2: bool,
    amplifier: FloatAmplifier,
) -> Spline<'a> {
    let spline = meth_42050(
        ridges.clone(),
        lerp_32(h, 0.6f32, 1.5f32),
        bl2,
        amplifier.clone(),
    );
    let spline2 = meth_42050(
        ridges.clone(),
        lerp_32(h, 0.6f32, 1f32),
        bl2,
        amplifier.clone(),
    );
    let spline3 = meth_42050(ridges.clone(), h, bl2, amplifier.clone());
    let spline4 = meth_42048(
        ridges.clone(),
        continental - 0.15f32,
        0.5f32 * h,
        lerp_32(0.5f32, 0.5f32, 0.5f32) * h,
        0.5f32 * h,
        0.6f32 * h,
        0.5f32,
        amplifier.clone(),
    );

    let spline5 = meth_42048(
        ridges.clone(),
        continental,
        i * h,
        f * h,
        0.5f32 * h,
        0.6f32 * h,
        0.5f32,
        amplifier.clone(),
    );
    let spline6 = meth_42048(
        ridges.clone(),
        continental,
        i,
        i,
        f,
        g,
        0.5f32,
        amplifier.clone(),
    );
    let spline7 = meth_42048(
        ridges.clone(),
        continental,
        i,
        i,
        f,
        g,
        0.5f32,
        amplifier.clone(),
    );

    let spline8 = SplineBuilder::new(ridges.clone(), amplifier.clone())
        .add_value(-1f32, continental, 0f32)
        .add_spline(-0.4f32, SplineValue::Spline(spline6.clone()), 0f32)
        .add_value(0f32, g + 0.07f32, 0f32)
        .build();
    let spline9 = meth_42048(
        ridges.clone(),
        -0.02f32,
        j,
        j,
        f,
        g,
        0f32,
        amplifier.clone(),
    );

    let mut builder = SplineBuilder::new(erosion, amplifier);
    let builder = builder
        .add_spline(-0.85f32, SplineValue::Spline(spline), 0f32)
        .add_spline(-0.7f32, SplineValue::Spline(spline2), 0f32)
        .add_spline(-0.4f32, SplineValue::Spline(spline3), 0f32)
        .add_spline(-0.35f32, SplineValue::Spline(spline4), 0f32)
        .add_spline(-0.1f32, SplineValue::Spline(spline5), 0f32)
        .add_spline(0.2f32, SplineValue::Spline(spline6), 0f32);

    let builder = if bl {
        builder
            .add_spline(0.4f32, SplineValue::Spline(spline7.clone()), 0f32)
            .add_spline(0.45f32, SplineValue::Spline(spline8.clone()), 0f32)
            .add_spline(0.55f32, SplineValue::Spline(spline8), 0f32)
            .add_spline(0.58f32, SplineValue::Spline(spline7), 0f32)
    } else {
        builder
    };

    builder
        .add_spline(0.7f32, SplineValue::Spline(spline9), 0f32)
        .build()
}

pub fn create_offset_spline<'a>(
    contentents: Arc<DensityFunction<'a>>,
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    amplified: bool,
) -> Spline<'a> {
    let amplification = if amplified {
        FloatAmplifier::Amplifier
    } else {
        FloatAmplifier::Identity
    };

    let spline = create_continental_offset_spline(
        erosion.clone(),
        ridges.clone(),
        -0.15f32,
        0f32,
        0f32,
        0.132,
        0f32,
        -0.03f32,
        false,
        false,
        amplification.clone(),
    );
    let spline2 = create_continental_offset_spline(
        erosion.clone(),
        ridges.clone(),
        -0.1f32,
        0.03f32,
        0.1f32,
        0.1f32,
        0.01f32,
        -0.03f32,
        false,
        false,
        amplification.clone(),
    );
    let spline3 = create_continental_offset_spline(
        erosion.clone(),
        ridges.clone(),
        -0.1f32,
        0.03f32,
        0.1f32,
        0.7f32,
        0.01f32,
        -0.03f32,
        true,
        true,
        amplification.clone(),
    );
    let spline4 = create_continental_offset_spline(
        erosion.clone(),
        ridges.clone(),
        -0.05f32,
        0.03f32,
        0.1f32,
        1f32,
        0.01f32,
        0.01f32,
        true,
        true,
        amplification.clone(),
    );

    SplineBuilder::new(contentents.clone(), amplification.clone())
        .add_value(-1.1f32, 0.044f32, 0f32)
        .add_value(-1.02f32, -0.2222f32, 0f32)
        .add_value(-0.51f32, -0.2222f32, 0f32)
        .add_value(-0.44f32, -0.12f32, 0f32)
        .add_value(-0.18f32, -0.12f32, 0f32)
        .add_spline(-0.16f32, SplineValue::Spline(spline.clone()), 0f32)
        .add_spline(-0.15f32, SplineValue::Spline(spline), 0f32)
        .add_spline(-0.1f32, SplineValue::Spline(spline2), 0f32)
        .add_spline(0.25f32, SplineValue::Spline(spline3), 0f32)
        .add_spline(1f32, SplineValue::Spline(spline4), 0f32)
        .build()
}

pub fn create_factor_spline<'a>(
    continents: Arc<DensityFunction<'a>>,
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    ridges_folded: Arc<DensityFunction<'a>>,
    amplified: bool,
) -> Spline<'a> {
    let amplification = if amplified {
        FloatAmplifier::Amplifier
    } else {
        FloatAmplifier::Identity
    };

    SplineBuilder::new(continents, FloatAmplifier::Identity)
        .add_value(-0.19f32, 3.95f32, 0f32)
        .add_spline(
            -0.15f32,
            SplineValue::Spline(method_42054(
                erosion.clone(),
                ridges.clone(),
                ridges_folded.clone(),
                6.25f32,
                true,
                FloatAmplifier::Identity,
            )),
            0f32,
        )
        .add_spline(
            -0.1f32,
            SplineValue::Spline(method_42054(
                erosion.clone(),
                ridges.clone(),
                ridges_folded.clone(),
                5.47f32,
                true,
                amplification.clone(),
            )),
            0f32,
        )
        .add_spline(
            0.03f32,
            SplineValue::Spline(method_42054(
                erosion.clone(),
                ridges.clone(),
                ridges_folded.clone(),
                5.08f32,
                true,
                amplification.clone(),
            )),
            0f32,
        )
        .add_spline(
            0.06f32,
            SplineValue::Spline(method_42054(
                erosion,
                ridges,
                ridges_folded,
                4.69f32,
                false,
                amplification,
            )),
            0f32,
        )
        .build()
}

pub fn create_jaggedness_spline<'a>(
    continents: Arc<DensityFunction<'a>>,
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    ridges_folded: Arc<DensityFunction<'a>>,
    amplified: bool,
) -> Spline<'a> {
    let amplification = if amplified {
        FloatAmplifier::Amplifier
    } else {
        FloatAmplifier::Identity
    };

    SplineBuilder::new(continents.clone(), amplification.clone())
        .add_value(-0.11f32, 0f32, 0f32)
        .add_spline(
            0.03f32,
            SplineValue::Spline(method_42053(
                erosion.clone(),
                ridges.clone(),
                ridges_folded.clone(),
                1f32,
                0.5f32,
                0f32,
                0f32,
                amplification.clone(),
            )),
            0f32,
        )
        .add_spline(
            0.65f32,
            SplineValue::Spline(method_42053(
                erosion,
                ridges,
                ridges_folded,
                1f32,
                1f32,
                1f32,
                0f32,
                amplification,
            )),
            0f32,
        )
        .build()
}
