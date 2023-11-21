//! Widgets for displaying progress indicators.

use std::ops::RangeInclusive;
use std::time::Duration;

use kludgine::figures::Ranged;

use crate::animation::easings::EaseInOutSine;
use crate::animation::{
    AnimationHandle, AnimationTarget, IntoAnimate, PercentBetween, Spawn, ZeroToOne,
};
use crate::value::{Dynamic, IntoDynamic, IntoValue, MapEach, Value};
use crate::widget::{MakeWidget, WidgetInstance};
use crate::widgets::slider::Slidable;
use crate::widgets::Data;

/// A bar-shaped progress indicator.
#[derive(Debug)]
pub struct ProgressBar {
    progress: Value<Progress>,
}

impl ProgressBar {
    /// Returns an indeterminant progress bar.
    #[must_use]
    pub const fn indeterminant() -> Self {
        Self {
            progress: Value::Constant(Progress::Indeterminant),
        }
    }

    /// Returns a new progress bar that displays `progress`.
    #[must_use]
    pub fn new(progress: impl IntoDynamic<Progress>) -> Self {
        Self {
            progress: Value::Dynamic(progress.into_dynamic()),
        }
    }
}

/// A measurement of progress for an indicator widget like [`ProgressBar`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Progress {
    /// The task has an indeterminant length.
    Indeterminant,
    /// The task is a specified percent complete.
    Percent(ZeroToOne),
}

impl MakeWidget for ProgressBar {
    fn make_widget(self) -> WidgetInstance {
        let start = Dynamic::new(ZeroToOne::ZERO);
        let end = Dynamic::new(ZeroToOne::ZERO);
        let value = (&start, &end).map_each(|(start, end)| *start..=*end);

        let mut indeterminant_animation = None;
        update_progress_bar(
            self.progress.get(),
            &mut indeterminant_animation,
            &start,
            &end,
        );

        let slider = value.slider().knobless().non_interactive();
        match self.progress {
            Value::Dynamic(progress) => {
                progress.for_each(move |progress| {
                    update_progress_bar(*progress, &mut indeterminant_animation, &start, &end);
                });
                Data::new_wrapping(progress, slider).make_widget()
            }
            Value::Constant(_) => Data::new_wrapping(indeterminant_animation, slider).make_widget(),
        }
    }
}

fn update_progress_bar(
    progress: Progress,
    indeterminant_animation: &mut Option<AnimationHandle>,
    start: &Dynamic<ZeroToOne>,
    end: &Dynamic<ZeroToOne>,
) {
    match progress {
        Progress::Indeterminant => {
            *indeterminant_animation = Some(
                end.transition_to(ZeroToOne::new(0.66))
                    .over(Duration::from_millis(500))
                    .with_easing(EaseInOutSine)
                    .and_then(
                        start
                            .transition_to(ZeroToOne::new(0.33))
                            .over(Duration::from_millis(500))
                            .with_easing(EaseInOutSine),
                    )
                    .and_then(
                        end.transition_to(ZeroToOne::ONE)
                            .over(Duration::from_millis(500))
                            .with_easing(EaseInOutSine),
                    )
                    .and_then(
                        start
                            .transition_to(ZeroToOne::ONE)
                            .over(Duration::from_millis(500))
                            .with_easing(EaseInOutSine),
                    )
                    .and_then(
                        (
                            start.transition_to(ZeroToOne::ZERO),
                            end.transition_to(ZeroToOne::ZERO),
                        )
                            .over(Duration::ZERO),
                    )
                    .cycle()
                    .spawn(),
            );
        }
        Progress::Percent(value) => {
            let _stopped_animation = indeterminant_animation.take();
            start.update(ZeroToOne::ZERO);
            end.update(value);
        }
    }
}

/// A value that can be used in a progress indicator.
pub trait Progressable<T>: IntoDynamic<T> + Sized {
    /// Returns a new progress bar that displays progress from `T::MIN` to
    /// `T::MAX`.
    fn progress_bar(self) -> ProgressBar
    where
        T: Ranged + PercentBetween,
    {
        ProgressBar::new(
            self.into_dynamic()
                .map_each(|t| Progress::Percent(t.percent_between(&T::MIN, &T::MAX))),
        )
    }

    /// Returns a new progress bar that displays progress from `T::MIN` to
    /// `max`. The maximum value can be either a `T` or an `Option<T>`. If
    /// `None` is the maximum value, an indeterminant progress bar will be
    /// displayed.
    fn progress_bar_to(self, max: impl IntoValue<Option<T>>) -> ProgressBar
    where
        T: Ranged + PercentBetween + Clone + Send + Sync + 'static,
    {
        let max = max.into_value();
        match max {
            Value::Constant(max) => self.progress_bar_between(max.map(|max| T::MIN..=max)),
            Value::Dynamic(max) => {
                self.progress_bar_between(max.map_each(|max| max.clone().map(|max| T::MIN..=max)))
            }
        }
    }

    /// Returns a new progress bar that displays progress over the specified
    /// `range` of `T`. The range can be either a `T..=T` or an `Option<T>`. If
    /// `None` is specified as the range, an indeterminant progress bar will be
    /// displayed.
    fn progress_bar_between<Range>(self, range: Range) -> ProgressBar
    where
        T: PercentBetween + Clone + Send + Sync + 'static,
        Range: IntoValue<Option<RangeInclusive<T>>>,
    {
        let value = self.into_dynamic();
        let range = range.into_value();
        match range {
            Value::Constant(range) => ProgressBar::new(value.map_each(move |value| {
                range
                    .as_ref()
                    .map(|range| value.percent_between(range.start(), range.end()))
                    .map_or(Progress::Indeterminant, Progress::Percent)
            })),
            Value::Dynamic(range) => {
                ProgressBar::new((&range, &value).map_each(|(range, value)| {
                    range.clone().map_or(Progress::Indeterminant, |range| {
                        Progress::Percent(value.percent_between(range.start(), range.end()))
                    })
                }))
            }
        }
    }
}

impl<U, T> Progressable<U> for T where T: IntoDynamic<U> {}
