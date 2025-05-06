use std::cmp::min;

// free from https://github.com/mattsse/str-distance/blob/master/src/levenshtein.rs
// by https://github.com/mattsse

#[derive(Debug, Clone, Default)]
pub struct Levenshtein {
    /// The maximum edit distance of interest.
    ///
    /// Used to short circuit the exact evaluation of the distance, if the exact
    /// value is guaranteed to exceed the configured maximum.
    max_distance: Option<usize>,
}

impl Levenshtein {
    pub fn with_max_distance(max_distance: usize) -> Self {
        Self {
            max_distance: Some(max_distance),
        }
    }
}

impl Levenshtein {
    pub fn str_distance(&self, a: &str, b: &str) -> Option<usize> {
        // make sure we use the shortest str for the outer loop
        if a.len() < b.len() {
            self.distance(a.chars(), b.chars())
        } else {
            self.distance(b.chars(), a.chars())
        }
    }

    pub fn distance<S, T>(&self, a: S, b: T) -> Option<usize>
    where
        S: IntoIterator,
        T: IntoIterator,
        <S as IntoIterator>::IntoIter: Clone,
        <T as IntoIterator>::IntoIter: Clone,
        <S as IntoIterator>::Item: PartialEq + PartialEq<<T as IntoIterator>::Item>,
        <T as IntoIterator>::Item: PartialEq,
    {
        // exclude matching prefix and suffix
        let delim = DelimDistinct::new_skip_take(a.into_iter(), b.into_iter());

        if delim.remaining_s1() == 0 {
            // the longer str starts or ends completely with the shorter str
            return Some(delim.remaining_s2());
        }

        if let Some(max_dist) = self.max_distance {
            if delim.remaining_s2() - delim.remaining_s1() > max_dist {
                return None;
            }
        }

        let max_dist = self.max_distance.unwrap_or_else(|| delim.remaining_s2());

        let mut cache: Vec<usize> = (1..=delim.remaining_s2()).collect();

        let mut result = 0;

        for (c1_idx, c1) in delim.distinct_s1.enumerate() {
            result = c1_idx + 1;
            let mut dist_c2 = c1_idx;
            let mut min_dist = if c1_idx == 0 { 0 } else { c1_idx - 1 };

            for (c2_idx, c2) in delim.distinct_s2.clone().enumerate() {
                let cost = if c1 == c2 { 0usize } else { 1usize };
                let dist_c1 = dist_c2 + cost;
                dist_c2 = cache[c2_idx];
                result = min(result + 1, min(dist_c1, dist_c2 + 1));
                min_dist = min(min_dist, dist_c2);
                cache[c2_idx] = result;
            }
            if min_dist > max_dist {
                return None;
            }
        }

        Some(result)
    }
}

struct DelimDistinct<S, T>
where
    S: Iterator + Clone,
    T: Iterator + Clone,
    <S as Iterator>::Item: PartialEq<<T as Iterator>::Item>,
{
    /// The amount of items both iter share at their beginning.
    pub prefix_len: usize,
    /// Iterator over the distinct items of s1
    pub distinct_s1: S,
    /// The amount of distinct items left in iter 1
    pub s1_len: usize,
    /// Iterator over the distinct items of s2
    pub distinct_s2: T,
    /// The amount of distinct items left in iter 2
    pub s2_len: usize,
    /// The amount of items both iters share at their end.
    pub suffix_len: usize,
}

#[allow(unused)]
impl<S, T> DelimDistinct<S, T>
where
    S: Iterator + Clone,
    T: Iterator + Clone,
    <S as Iterator>::Item: PartialEq<<T as Iterator>::Item>,
{
    /// Amount of chars both str share at their tails.
    #[inline]
    pub fn common(&self) -> usize {
        self.prefix_len + self.suffix_len
    }

    /// The number of distinct chars for each str
    #[inline]
    pub fn remaining(&self) -> (usize, usize) {
        (self.s1_len, self.s2_len)
    }

    /// Whether both str are identical.
    #[inline]
    pub fn is_eq(&self) -> bool {
        self.remaining() == (0, 0)
    }

    #[inline]
    pub fn remaining_s2(&self) -> usize {
        self.s2_len
    }

    #[inline]
    pub fn remaining_s1(&self) -> usize {
        self.s1_len
    }

    /// Return the len of common prefix and suffix items, and the distinct left
    /// elements in between.
    #[inline]
    pub(crate) fn new_skip_take(
        a: S,
        b: T,
    ) -> DelimDistinct<std::iter::Skip<std::iter::Take<S>>, std::iter::Skip<std::iter::Take<T>>>
    {
        // collecting is a little tedious here, but we can't rely on the iters also
        // being DoubleEnded
        let a_vec: Vec<_> = a.clone().collect();
        let b_vec: Vec<_> = b.clone().collect();

        let a_len = a_vec.len();
        let b_len = b_vec.len();

        let suffix_len = count_eq(a_vec.into_iter().rev(), b_vec.into_iter().rev());

        let a_iter = a.take(a_len - suffix_len);
        let b_iter = b.take(b_len - suffix_len);

        let prefix_len = count_eq(a_iter.clone(), b_iter.clone());

        let common_len = prefix_len + suffix_len;
        DelimDistinct {
            suffix_len,
            prefix_len,
            s1_len: a_len - common_len,
            s2_len: b_len - common_len,
            distinct_s1: a_iter.skip(prefix_len),
            distinct_s2: b_iter.skip(prefix_len),
        }
    }
}

#[inline]
pub(crate) fn count_eq<S, T>(mut s1_iter: S, mut s2_iter: T) -> usize
where
    S: Iterator,
    T: Iterator,
    <S as Iterator>::Item: PartialEq<<T as Iterator>::Item>,
{
    let mut match_ctn = 0;
    loop {
        let c1 = match s1_iter.next() {
            None => {
                // s2 ends with completely with s1
                break;
            }
            Some(val) => val,
        };

        let c2 = match s2_iter.next() {
            None => {
                // s1 ends completely with s2
                break;
            }
            Some(val) => val,
        };
        if c1 == c2 {
            match_ctn += 1;
        } else {
            break;
        }
    }
    match_ctn
}