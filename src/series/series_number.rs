use std::vec;

use crate::{chart::*, coord::*, utils::cal_step::*, TAU};

#[derive(Debug, Clone, Default)]
/// A series of numbers represented on a chart
pub struct SNumber {
    series: Vec<f64>,
    is_float: bool,
    stick: usize,
    origin: f64,
    // For range of axis
    range: Option<(f64, f64)>,
}

impl SNumber {
    pub fn new(series: Vec<f64>) -> Self {
        SNumber {
            series,
            is_float: true,
            stick: 0,
            origin: 0.,
            range: None,
        }
    }

    pub fn set_stick(&self, stick: usize) -> Self {
        Self {
            series: self.series.clone(),
            is_float: self.is_float,
            stick: stick,
            origin: self.origin,
            range: self.range,
        }
    }

    pub fn series(&self) -> Vec<f64> {
        self.series.clone()
    }

    pub fn merge(&self, other: SNumber) -> Self {
        let mut series =  self.series.clone();
        series.extend(other.series);
        Self {
            series,
            is_float: self.is_float,
            stick: self.stick,
            origin: self.origin,
            range: self.range,
        }
    }

    pub fn set_range(&self, min: f64, max: f64) -> Self {
        Self {
            series: self.series.clone(),
            is_float: self.is_float,
            stick: self.stick,
            origin: self.origin,
            range: Some((min, max)),
        }
    }
}

impl From<Vec<i64>> for SNumber {
    fn from(value: Vec<i64>) -> Self {
        let mut series: Vec<f64> = vec![];
        for i in value {
            series.push(i as f64)
        }
        Self {
            series,
            is_float: false,
            stick: 0,
            origin: 0.,
            range: None,
        }
    }
}

impl From<Vec<u64>> for SNumber {
    fn from(value: Vec<u64>) -> Self {
        let mut series: Vec<f64> = vec![];
        for i in value {
            series.push(i as f64)
        }

        Self {
            series,
            is_float: false,
            stick: 0,
            origin: 0.,
            range: None,
        }
    }
}

impl ScaleNumber for SNumber {
    fn domain(&self) -> (f64, f64) {
        let mut all = self.series();
        if let Some(range) = self.range {
            all.push(range.0);
            all.push(range.1);
        } else {
            all.push(self.origin);
        }

        min_max_vec(&all)
    }

    fn count_distance_step(&self) -> (f64, f64, f64) {
        let (min, max) = self.domain();
        let count_distance = if self.stick == 0 {
            10.
        } else {
            self.stick as f64 - 1.
        };
        let (distance_up, step, distance_down) = if min >= 0. && max >= 0. {
            let mut step = max / count_distance;
            step = CalStep::new(step).cal_scale();
            (max / step, step, 0.)
        } else if min < 0. && max < 0. {
            let mut step = min / count_distance;
            step = CalStep::new(step).cal_scale();
            (0., step, min.abs() / step)
        } else {
            let mut step = (max - min) / count_distance;
            step = CalStep::new(step).cal_scale();
            (max / step, step, min.abs() / step)
        };
        (distance_up.ceil(), step, distance_down.ceil())
    }

    fn to_percent(&self) -> Vec<f64> {
        let total: f64 = self.series.iter().sum();
        self.series.clone().into_iter().map(|f| f / total).collect()
    }

    fn to_percent_radar(&self) -> Vec<f64> {
        let total = 100.;
        self.series.clone().into_iter().map(|f| f / total).collect()
    }

    fn gen_pie(&self) -> Vec<Arc> {
        let series = self.series.clone();
        let total: f64 = series.iter().sum();
        let percent: Vec<f64> = series.clone().into_iter().map(|f| f / total).collect();
        let mut vector_begin = Vector::new(0., -1.);
        let mut vec_arc: Vec<Arc> = vec![];
        for p in percent {
            let arc = Arc::new_polar(Point::default(), vector_begin.clone(), p * TAU);
            vector_begin = arc.end.clone();
            vec_arc.push(arc);
        }
        vec_arc
    }

    fn gen_radar_grid(&self, count: usize) -> Vec<Vector> {
        let mut vector_begin = Vector::new(0., -1.);
        let mut vectors: Vec<Vector> = vec![];
        vectors.push(vector_begin);
        for index in 1..count {
            vector_begin = vectors[index - 1].clone();
            vectors.push(vector_begin.az_rotate_tau(TAU / count as f64));
        }
        vectors
    }

    fn gen_axes(&self) -> Axes {
        let (distance_up, step, distance_down) = self.count_distance_step();
        let (_, precision) = count_precision(step.clone(), 0);
        let mut vec_value: Vec<f64> = vec![];
        let mut vec_stick: Vec<Stick> = vec![];
        for index in 1..(distance_down as i64 + 1) {
            vec_value.push(-index as f64 * step);
        }

        for index in 0..(distance_up as i64 + 1) {
            vec_value.push(index as f64 * step);
        }

        vec_value.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for index in 0..(vec_value.len()) {
            let value = vec_value[index];
            let label = format!("{:.prec$}", value, prec = precision);
            let stick = Stick::new(label, self.scale(value));
            vec_stick.push(stick);
        }
        let sticks = vec_stick
            .into_iter()
            .filter(|stick| stick.value >= -0.0000001 && stick.value <= 1.0000001)
            .collect::<Vec<_>>();

        Axes {
            sticks: sticks,
            step: step,
        }
    }

    fn scale(&self, value: f64) -> f64 {
        let (min, max) = self.domain();
        let range = max - min;

        let diff = value - min;
        diff / range
    }

    fn to_stick(&self) -> Vec<Stick> {
        let mut vec_stick: Vec<Stick> = vec![];
        let len = self.series().len();
        for index in 0..len {
            let stick = Stick::new(format!("{}", self.series()[index]), self.series()[index]);
            vec_stick.push(stick);
        }
        vec_stick
    }
}

fn count_precision(mut number: f64, mut count: usize) -> (f64, usize) {
    let floor = number - number.floor();
    if floor == 0. {
        return (number, count);
    } else {
        number *= 10.;
        count += 1;
        count_precision(number, count)
    }
}
