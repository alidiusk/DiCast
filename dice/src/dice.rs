use rand::{
    distributions::uniform::SampleUniform,
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
    Rng,
};

use std::ops::{Bound, RangeBounds, RangeInclusive};

pub type StdDice = Dice<RangeInclusive<i64>>;

pub trait ToUniform<T>
where
    T: SampleUniform,
{
    fn to_uniform(&self) -> Uniform<T>;
}

/// Wrapper to convert Ranges to Uniforms.
///
/// Note that if either endpoint is not specified in the range,
/// it defaults to an inclusive bound of containing the type
/// default, which for most numerics is zero.
///
/// The start point is always included.
impl<T, R> ToUniform<T> for R
where
    T: SampleUniform + std::fmt::Debug + Default + Copy,
    R: RangeBounds<T>,
{
    fn to_uniform(&self) -> Uniform<T> {
        let start_bound = match self.start_bound() {
            Bound::Unbounded => T::default(),
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x,
        };

        let (end_include, end_bound) = match self.end_bound() {
            Bound::Unbounded => (true, T::default()),
            Bound::Included(x) => (true, *x),
            Bound::Excluded(x) => (false, *x),
        };

        if end_include {
            Uniform::from(start_bound..=end_bound)
        } else {
            Uniform::from(start_bound..end_bound)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiceRoller<T: Rng> {
    /// The kind of Rng generator to use for rolling the dice.
    rng: T,
}

impl DiceRoller<ThreadRng> {
    pub fn new() -> Self {
        DiceRoller {
            rng: rand::thread_rng(),
        }
    }

    pub fn roll_dice<T: ToUniform<i64>>(&mut self, dice: &Dice<T>) -> i64 {
        dice.roll_with_rng(&mut self.rng)
    }

    pub fn roll<T: ToUniform<i64>>(
        &mut self,
        count: i64,
        range: T,
        multiplier: i64,
        modifier: i64,
        drop: i64,
    ) -> i64 {
        let dice = Dice::new(count, range, multiplier, modifier, drop);
        self.roll_dice(&dice)
    }

    pub fn roll_dice_times<T: ToUniform<i64>>(&mut self, dice: &Dice<T>, times: i64) -> Vec<i64> {
        let mut rolls = vec![];

        for _ in 0..times {
            rolls.push(self.roll_dice(dice));
        }

        rolls
    }

    pub fn roll_times<T: ToUniform<i64>>(
        &mut self,
        count: i64,
        range: T,
        multiplier: i64,
        modifier: i64,
        drop: i64,
        times: i64,
    ) -> Vec<i64> {
        let dice = Dice::new(count, range, multiplier, modifier, drop);

        self.roll_dice_times(&dice, times)
    }
}

impl Default for DiceRoller<ThreadRng> {
    fn default() -> DiceRoller<ThreadRng> {
        DiceRoller {
            rng: rand::thread_rng(),
        }
    }
}

impl<T: Rng> From<T> for DiceRoller<T> {
    fn from(rng: T) -> DiceRoller<T> {
        DiceRoller { rng }
    }
}

#[derive(Debug, Clone)]
pub struct Dice<T: ToUniform<i64>> {
    /// The number of equivalently sided dice being rolled.
    pub(crate) count: i64,
    /// The uniform that represents the sides of the dice.
    pub(crate) range: T,
    /// The modifier that is added onto the dice roll.
    pub(crate) multiplier: i64,
    /// The modifier that is added onto the dice roll.
    pub(crate) modifier: i64,
    /// The number of lowest dice rolls to drop.
    pub(crate) drop: i64,
}

impl<T: ToUniform<i64>> Dice<T> {
    /// If the number of dice to drop exceeds the number of dice being rolled, all rolls will be zero.
    pub fn new(count: i64, range: T, multiplier: i64, modifier: i64, mut drop: i64) -> Self {
        if drop > count {
            drop = count;
        }

        Dice {
            count,
            range,
            multiplier,
            modifier,
            drop,
        }
    }

    pub fn roll_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> i64 {
        let uniform = self.range.to_uniform();

        let mut rolls = vec![];
        for _ in 0..self.count {
            rolls.push(uniform.sample(rng));
        }

        rolls.sort();
        rolls.drain(..self.drop as usize);

        self.multiplier * rolls.iter().sum::<i64>() + self.modifier
    }
}

impl Default for Dice<RangeInclusive<i64>> {
    fn default() -> Dice<RangeInclusive<i64>> {
        Dice {
            count: 1,
            range: 1..=6,
            multiplier: 1,
            modifier: 0,
            drop: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_new() {
        let dice = Dice::new(2, 1..=20, 1, 0, 0);

        assert_eq!(2, dice.count);
        assert_eq!(1, dice.multiplier);
        assert_eq!(0, dice.modifier);
        assert_eq!(0, dice.drop);
    }

    #[test]
    fn dice_default() {
        let dice_0 = Dice::default();

        assert_eq!(1, dice_0.count);
        assert_eq!(1..=6, dice_0.range);
        assert_eq!(1, dice_0.multiplier);
        assert_eq!(0, dice_0.modifier);
        assert_eq!(0, dice_0.drop);

        let dice_1 = Dice {
            count: 3,
            range: 1..=20,
            multiplier: 2,
            modifier: 1,
            ..Default::default()
        };

        assert_eq!(3, dice_1.count);
        assert_eq!(1..=20, dice_1.range);
        assert_eq!(2, dice_1.multiplier);
        assert_eq!(1, dice_1.modifier);
        assert_eq!(0, dice_1.drop);
    }

    #[test]
    fn dice_roll_with_rng() {
        let dice = Dice {
            count: 2,
            range: 1..=20,
            modifier: 2,
            ..Default::default()
        };
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            assert!(4 <= dice.roll_with_rng(&mut rng));
            assert!(42 >= dice.roll_with_rng(&mut rng));
        }
    }

    #[test]
    fn dice_roller_from_rng() {
        let _dice_roller_0 = DiceRoller::from(rand::rngs::OsRng);
        let _dice_roller_1 = DiceRoller::from(rand::thread_rng());
    }

    #[test]
    fn dice_roller_default() {
        let _dice_roller = DiceRoller::default();
    }

    #[test]
    fn dice_roller_roll_dice() {
        let mut dice_roller = DiceRoller::default();
        let dice = Dice {
            count: 3,
            range: 1..=6,
            modifier: 4,
            ..Default::default()
        };

        for _ in 0..100 {
            assert!(7 <= dice_roller.roll_dice(&dice));
            assert!(22 >= dice_roller.roll_dice(&dice));
        }
    }

    #[test]
    fn dice_drop_exceeds_count() {
        let mut dice_roller = DiceRoller::default();
        let dice = Dice {
            count: 3,
            range: 1..=6,
            modifier: 4,
            ..Default::default()
        };

        for _ in 0..100 {
            assert!(7 <= dice_roller.roll_dice(&dice));
            assert!(22 >= dice_roller.roll_dice(&dice));
        }
    }

    #[test]
    fn dice_roller_roll_dice_times() {
        let mut dice_roller = DiceRoller::default();
        let dice = Dice {
            count: 3,
            range: 1..=6,
            modifier: 4,
            ..Default::default()
        };

        let rolls = dice_roller.roll_dice_times(&dice, 10);
        assert_eq!(10, rolls.len());
        for roll in rolls {
            dbg!(roll);
            assert!(5 <= roll);
            assert!(11 >= roll);
        }
    }
}
