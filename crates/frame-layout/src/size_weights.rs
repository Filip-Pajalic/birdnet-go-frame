//! How much bigger heavier birds are drawn.

use alloc::vec::Vec;

/// Display size grows as body mass to this power. 0 draws every bird the same size, 1 makes
/// size proportional to mass. 0.14, as in fugleramme, draws the heaviest garden birds about
/// two and a half times the size of the lightest, so a wren stays visible beside a pigeon.
pub const SIZE_EXPONENT: f32 = 0.14;

/// Returns a size weight per bird from its body mass in grams: 1.0 for a bird of the set's
/// geometric-mean mass, more for heavier birds, less for lighter ones. Masses that are not
/// positive count as the mean.
#[must_use]
pub fn size_weights(masses_grams: &[f32]) -> Vec<f32> {
    let logs: Vec<Option<f32>> = masses_grams
        .iter()
        .map(|&mass| (mass > 0.0 && mass.is_finite()).then(|| libm::logf(mass)))
        .collect();
    let known: Vec<f32> = logs.iter().flatten().copied().collect();
    let mean: f32 = if known.is_empty() {
        0.0
    } else {
        known.iter().sum::<f32>() / known.len() as f32
    };
    logs.iter()
        .map(|log| libm::expf(SIZE_EXPONENT * (log.unwrap_or(mean) - mean)))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn heavier_birds_are_bigger_but_not_proportionally() {
        // Goldcrest, great tit, wood pigeon.
        let weights: Vec<f32> = size_weights(&[5.5, 16.3, 490.0]);
        assert!(weights[0] < weights[1] && weights[1] < weights[2]);
        let ratio: f32 = weights[2] / weights[0];
        assert!((1.8..2.0).contains(&ratio), "{ratio}");
        let product: f32 = weights.iter().product();
        assert!(
            (product - 1.0).abs() < 1e-4,
            "centred on the geometric mean: {product}"
        );
    }

    #[test]
    fn unknown_masses_count_as_the_mean() {
        assert_eq!(size_weights(&[0.0, f32::NAN]), [1.0, 1.0]);
        assert!(size_weights(&[]).is_empty());
    }
}
